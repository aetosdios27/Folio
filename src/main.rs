use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event as CrosstermEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};
use tokio::{
    sync::mpsc,
    time::{interval, MissedTickBehavior},
};

// Declare our modules
mod app;
mod config;
mod fetch;
mod obsidian;
mod templates;
mod ui;

use app::{reduce, AppEvent, AppState};
use config::load_or_init_config;

/// Folio — search ArXiv, import papers into Obsidian
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Start directly with a custom search query
    #[arg(short, long)]
    query: Option<String>,

    /// Re-run the vault setup wizard (resets your config)
    #[arg(long)]
    reconfigure: bool,
}

fn install_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Guarantee we give the user their terminal back before printing the panic
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original(info);
    }));
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> Result<Self> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Parse CLI arguments (gives us --help and --version for free)
    let cli = Cli::parse();

    // --reconfigure: delete the config file so onboarding runs fresh
    if cli.reconfigure {
        let proj_dirs = config::get_project_dirs()?;
        let config_path = proj_dirs.config_dir().join("config.toml");
        if config_path.exists() {
            std::fs::remove_file(&config_path)?;
            println!("Config reset. Re-running setup...\n");
        }
    }

    // 2. Load Config (will run interactive onboarding if first boot)
    let config = load_or_init_config()?;

    // 3. Set up terminal safety nets
    install_panic_hook();
    let _guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    // 4. Initialize application state
    let mut state = AppState::new(config);

    // If user passed `folio --query "LLMs"`, override the default start state
    if let Some(q) = cli.query {
        state.query = q;
        state.topic = "Custom CLI Search".into();
    }

    // 5. Set up async channels for the event loop
    let (tx, mut rx) = mpsc::channel::<AppEvent>(512);

    // Input Spawner (Listens for keyboard events)
    let input_tx = tx.clone();
    tokio::spawn(async move {
        loop {
            if let Ok(true) = event::poll(Duration::from_millis(16)) {
                if let Ok(CrosstermEvent::Key(key)) = event::read() {
                    let _ = input_tx.send(AppEvent::Input(key)).await;
                }
            }
        }
    });

    // Tick Spawner (Drives UI animations like the loading spinner)
    let tick_tx = tx.clone();
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_millis(120));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            ticker.tick().await;
            if tick_tx.send(AppEvent::Tick).await.is_err() {
                break; // Channel closed
            }
        }
    });

    // 6. Main Event Loop
    //
    // Redraw gating: we only call ui::render when state has actually changed.
    // Tick events advance the spinner frame but only trigger a redraw while
    // loading is active, so idle screens burn zero CPU on repaints.
    let mut should_redraw = true;

    loop {
        if state.wants_quit {
            break;
        }

        if should_redraw {
            ui::render(&mut terminal, &state)?;
            should_redraw = false;
        }

        if let Some(event) = rx.recv().await {
            match event {
                AppEvent::Tick => {
                    // Only pay for a redraw when the spinner is actually visible.
                    for effect in reduce(&mut state, AppEvent::Tick) {
                        let tx_clone = tx.clone();
                        tokio::spawn(app::handle_effect(effect, tx_clone));
                    }
                    if state.loading {
                        should_redraw = true;
                    }
                }
                other => {
                    // Every real event (input, fetch result, import result) triggers a redraw.
                    for effect in reduce(&mut state, other) {
                        let tx_clone = tx.clone();
                        tokio::spawn(app::handle_effect(effect, tx_clone));
                    }
                    should_redraw = true;
                }
            }
        }
    }

    Ok(())
}
