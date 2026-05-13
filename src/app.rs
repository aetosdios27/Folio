use crate::config::Config;
use crate::fetch::Paper;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Start,
    Results,
    Imported,
}

#[derive(Debug, Clone)]
pub enum AppError {
    Network(String),
    EmptyQuery,
    NothingSelected,
    Import(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(e) => write!(f, "Network error: {e}"),
            Self::EmptyQuery => write!(f, "Type a search query before fetching papers"),
            Self::NothingSelected => {
                write!(f, "Select a paper first, or highlight one and press Enter")
            }
            Self::Import(e) => write!(f, "Import error: {e}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImportReport {
    pub written: usize,
    pub skipped: usize,
    pub folder: std::path::PathBuf,
    pub files: Vec<std::path::PathBuf>,
}

pub struct AppState {
    pub config: Config,
    pub screen: Screen,
    pub papers: Vec<Paper>,
    pub selected: HashSet<String>,
    pub cursor: usize,
    pub topic_cursor: usize,
    pub loading: bool,
    pub loading_frame: usize,
    pub error: Option<AppError>,
    pub query: String,
    pub topic: String,
    pub editing_query: bool,
    pub status: String,
    pub import_report: Option<ImportReport>,
    pub wants_quit: bool,
    pub preview_scroll: u16,
    pub offset: usize,                                    // Used for pagination
    pub tera: tera::Tera, // Cached template engine — parsed once at startup
    pub page_cache: HashMap<(String, usize), Vec<Paper>>, // keyed by (query, offset)
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let default_topic =
            config
                .topics
                .first()
                .cloned()
                .unwrap_or_else(|| crate::config::Topic {
                    name: "Custom Search".into(),
                    query: "".into(),
                });

        Self {
            config,
            screen: Screen::Start,
            papers: vec![],
            selected: HashSet::new(),
            cursor: 0,
            topic_cursor: 0,
            loading: false,
            loading_frame: 0,
            error: None,
            query: default_topic.query,
            topic: default_topic.name,
            editing_query: false,
            status: "Choose a topic or edit the search, then press Enter.".into(),
            import_report: None,
            wants_quit: false,
            preview_scroll: 0,
            offset: 0,
            tera: crate::templates::build_tera().unwrap_or_else(|_| tera::Tera::default()),
            page_cache: HashMap::new(),
        }
    }
}

pub enum AppEvent {
    Input(KeyEvent),
    FetchSucceeded(Vec<Paper>),
    FetchFailed(AppError),
    ImportSucceeded(ImportReport),
    ImportFailed(AppError),
    Tick,
}

pub enum Effect {
    Fetch {
        query: String,
        offset: usize,
    },
    Import {
        papers: Vec<Paper>,
        topic: String,
        rendered: Vec<String>, // Pre-rendered note content from the cached Tera engine
    },
}

pub fn reduce(state: &mut AppState, event: AppEvent) -> Vec<Effect> {
    match event {
        AppEvent::Input(key) => handle_input(state, key),
        AppEvent::FetchSucceeded(papers) => {
            state.loading = false;
            state.error = None;
            state.cursor = 0;
            state.preview_scroll = 0;
            state.screen = Screen::Results;
            state.import_report = None;

            // Populate the cache for this (query, offset) pair.
            let cache_key = (state.query.clone(), state.offset);
            state.page_cache.insert(cache_key, papers.clone());
            state.papers = papers;

            state.status = if state.papers.is_empty() {
                "No papers found. Try a broader query.".into()
            } else {
                format!(
                    "Page {} · {} papers for '{}'",
                    (state.offset / 20) + 1,
                    state.papers.len(),
                    state.query,
                )
            };
            vec![]
        }
        AppEvent::FetchFailed(err) => {
            state.loading = false;
            state.error = Some(err.clone());

            let err_msg = err.to_string();
            if err_msg.contains("429") || err_msg.contains("Too Many Requests") {
                state.status = "API Rate Limit Hit. Obtain a free key at https://www.semanticscholar.org/product/api and add it to ~/.config/folio/config.toml".into();
            } else {
                state.status = format!("Fetch failed: {}", err_msg);
            }
            vec![]
        }
        AppEvent::ImportSucceeded(report) => {
            state.loading = false;
            state.error = None;
            state.selected.clear();
            state.status = if report.skipped > 0 {
                format!(
                    "Imported {} papers ({} skipped) to {}",
                    report.written,
                    report.skipped,
                    report.folder.display()
                )
            } else {
                format!(
                    "Imported {} papers to {}",
                    report.written,
                    report.folder.display()
                )
            };
            state.import_report = Some(report);
            state.screen = Screen::Imported;
            vec![]
        }
        AppEvent::ImportFailed(err) => {
            state.loading = false;
            state.error = Some(err);
            state.status = "Import failed".into();
            vec![]
        }
        AppEvent::Tick => {
            state.loading_frame = (state.loading_frame + 1) % 4;
            vec![]
        }
    }
}

fn handle_input(state: &mut AppState, key: KeyEvent) -> Vec<Effect> {
    if state.editing_query {
        return handle_query_input(state, key);
    }

    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.wants_quit = true;
            vec![]
        }
        KeyCode::Char('q') => {
            state.wants_quit = true;
            vec![]
        }
        KeyCode::Char('/') => {
            state.editing_query = true;
            state.status = "Editing search. Enter fetches, Esc cancels editing.".into();
            vec![]
        }
        KeyCode::Char('j') | KeyCode::Down => {
            move_cursor(state, 1);
            vec![]
        }
        KeyCode::Char('k') | KeyCode::Up => {
            move_cursor(state, -1);
            vec![]
        }
        KeyCode::Char('J') | KeyCode::PageDown => {
            state.preview_scroll = state.preview_scroll.saturating_add(1);
            vec![]
        }
        KeyCode::Char('K') | KeyCode::PageUp => {
            state.preview_scroll = state.preview_scroll.saturating_sub(1);
            vec![]
        }

        KeyCode::Char('n') if state.screen == Screen::Results => {
            state.offset += 20; // Pagination: Next Page
            fetch_current(state)
        }
        KeyCode::Char('p') if state.screen == Screen::Results => {
            state.offset = state.offset.saturating_sub(20); // Pagination: Prev Page
            fetch_current(state)
        }
        KeyCode::Char('r') => fetch_current(state),
        KeyCode::Char('b') => {
            state.screen = match state.screen {
                Screen::Imported => Screen::Results,
                _ => Screen::Start,
            };
            state.error = None;
            state.status = match state.screen {
                Screen::Start => "Choose a topic or edit the search, then press Enter.".into(),
                Screen::Results => "Continue browsing. Enter imports selected papers.".into(),
                Screen::Imported => state.status.clone(),
            };
            vec![]
        }
        KeyCode::Char('t') if state.screen == Screen::Imported => {
            state.screen = Screen::Start;
            state.error = None;
            state.status = "Choose a topic or edit the search, then press Enter.".into();
            vec![]
        }
        KeyCode::Char(' ') if state.screen == Screen::Results => {
            if let Some(paper) = state.papers.get(state.cursor) {
                if !state.selected.remove(&paper.id) {
                    state.selected.insert(paper.id.clone());
                }
            }
            vec![]
        }
        KeyCode::Enter if state.screen == Screen::Results => import_selected(state),
        KeyCode::Enter if state.screen == Screen::Imported => {
            state.screen = Screen::Results;
            state.status = "Continue browsing. Enter imports selected papers.".into();
            vec![]
        }
        KeyCode::Enter => fetch_current(state),
        _ => vec![],
    }
}

fn handle_query_input(state: &mut AppState, key: KeyEvent) -> Vec<Effect> {
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.wants_quit = true;
            vec![]
        }
        KeyCode::Esc => {
            state.editing_query = false;
            state.status = "Search editing cancelled.".into();
            vec![]
        }
        KeyCode::Enter => {
            state.editing_query = false;
            state.offset = 0;
            state.page_cache.clear(); // New search — discard cached pages
            fetch_current(state)
        }
        KeyCode::Backspace => {
            state.query.pop();
            vec![]
        }
        KeyCode::Char(c) => {
            if !c.is_control() {
                state.query.push(c);
                state.topic = "Custom Search".into();
            }
            vec![]
        }
        _ => vec![],
    }
}

fn move_cursor(state: &mut AppState, direction: isize) {
    match state.screen {
        Screen::Start => {
            let last = state.config.topics.len().saturating_sub(1);
            state.topic_cursor = if direction.is_positive() {
                (state.topic_cursor + 1).min(last)
            } else {
                state.topic_cursor.saturating_sub(1)
            };

            if let Some(topic) = state.config.topics.get(state.topic_cursor) {
                state.topic = topic.name.clone();
                state.query = topic.query.clone();
                state.status = if topic.query.is_empty() {
                    "Press / to type a custom search.".into()
                } else {
                    "Press Enter to fetch this topic.".into()
                };
            }
        }
        Screen::Results => {
            if state.papers.is_empty() {
                return;
            }

            state.cursor = if direction.is_positive() {
                (state.cursor + 1).min(state.papers.len() - 1)
            } else {
                state.cursor.saturating_sub(1)
            };
            state.preview_scroll = 0;
        }
        Screen::Imported => {}
    }
}

fn fetch_current(state: &mut AppState) -> Vec<Effect> {
    if state.loading {
        return vec![];
    }

    let query = state.query.trim();

    if query.is_empty() {
        state.error = Some(AppError::EmptyQuery);
        state.status = "Type a search query with / before fetching.".into();
        return vec![];
    }

    // Check the cache first — no network call needed if we've seen this page.
    let cache_key = (query.to_string(), state.offset);
    if let Some(cached) = state.page_cache.get(&cache_key) {
        state.papers = cached.clone();
        state.cursor = 0;
        state.preview_scroll = 0;
        state.screen = Screen::Results;
        state.error = None;
        state.status = format!(
            "Page {} · {} papers for '{}' (cached)",
            (state.offset / 20) + 1,
            state.papers.len(),
            query,
        );
        return vec![];
    }

    state.loading = true;
    state.error = None;
    state.status = format!("Fetching papers for '{}'...", query);

    vec![Effect::Fetch {
        query: query.to_string(),
        offset: state.offset,
    }]
}

fn import_selected(state: &mut AppState) -> Vec<Effect> {
    if state.loading {
        return vec![];
    }

    let papers = selected_papers(state);

    if papers.is_empty() {
        state.error = Some(AppError::NothingSelected);
        state.status = "Select with space, or highlight a paper and press Enter.".into();
        return vec![];
    }

    state.loading = true;
    state.error = None;
    state.status = format!("Importing {} papers to Obsidian...", papers.len());

    // Render all notes now using the cached Tera engine, before handing off to
    // the blocking thread. Any template errors surface immediately on the async
    // side rather than mid-write inside spawn_blocking.
    let topic = state.topic.clone();
    let mut rendered = Vec::with_capacity(papers.len());
    for paper in &papers {
        match crate::templates::render_note(&state.tera, paper, &topic) {
            Ok(content) => rendered.push(content),
            Err(e) => {
                state.loading = false;
                state.error = Some(AppError::Import(e.to_string()));
                state.status = format!("Template error: {e}");
                return vec![];
            }
        }
    }

    vec![Effect::Import {
        papers,
        topic,
        rendered,
    }]
}

fn selected_papers(state: &AppState) -> Vec<Paper> {
    let selected = state
        .papers
        .iter()
        .filter(|paper| state.selected.contains(&paper.id))
        .cloned()
        .collect::<Vec<_>>();

    if selected.is_empty() {
        state
            .papers
            .get(state.cursor)
            .cloned()
            .into_iter()
            .collect()
    } else {
        selected
    }
}

// Effect handler to be triggered from main.rs loop
pub async fn handle_effect(effect: Effect, tx: tokio::sync::mpsc::Sender<AppEvent>) {
    match effect {
        Effect::Fetch { query, offset } => {
            // We will hook this up to fetch.rs next
            let result = crate::fetch::fetch_papers(&query, offset).await;
            match result {
                Ok(papers) => {
                    let _ = tx.send(AppEvent::FetchSucceeded(papers)).await;
                }
                Err(e) => {
                    let _ = tx
                        .send(AppEvent::FetchFailed(AppError::Network(e.to_string())))
                        .await;
                }
            }
        }
        Effect::Import {
            papers,
            topic,
            rendered,
        } => {
            let result = tokio::task::spawn_blocking(move || {
                crate::obsidian::import_to_obsidian(&papers, &topic, &rendered)
            })
            .await
            .unwrap_or_else(|e| Err(anyhow::anyhow!("Task panicked: {}", e)));

            match result {
                Ok(report) => {
                    let _ = tx.send(AppEvent::ImportSucceeded(report)).await;
                }
                Err(e) => {
                    let _ = tx
                        .send(AppEvent::ImportFailed(AppError::Import(e.to_string())))
                        .await;
                }
            }
        }
    }
}
