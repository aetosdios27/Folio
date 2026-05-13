use crate::app::{AppState, Screen};
use anyhow::Result;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};

pub fn render<B: Backend>(terminal: &mut Terminal<B>, state: &AppState) -> Result<()> {
    terminal.draw(|frame| {
        let area = frame.size();
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Body
                Constraint::Length(3), // Footer
            ])
            .split(area);

        render_header(frame, layout[0], state);

        match state.screen {
            Screen::Start => render_start(frame, layout[1], state),
            Screen::Results => render_results(frame, layout[1], state),
            Screen::Imported => render_imported(frame, layout[1], state),
        }

        render_footer(frame, layout[2], state);

        if state.loading {
            render_loading(frame, centered_rect(38, 5, area), state);
        }
    })?;

    Ok(())
}

fn render_header(frame: &mut ratatui::Frame<'_>, area: Rect, state: &AppState) {
    let title = match state.screen {
        Screen::Start => " Librarian ",
        Screen::Results => " Librarian Results ",
        Screen::Imported => " Import Complete ",
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(&state.topic, Style::default().fg(Color::Yellow)),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));

    frame.render_widget(header, area);
}

fn render_start(frame: &mut ratatui::Frame<'_>, area: Rect, state: &AppState) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Left Column: Topics
    let items: Vec<ListItem> = state
        .config
        .topics
        .iter()
        .map(|topic| {
            ListItem::new(Line::from(vec![
                Span::styled("• ", Style::default().fg(Color::DarkGray)),
                Span::raw(&topic.name),
            ]))
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(state.topic_cursor));

    let topics_list = List::new(items)
        .block(Block::default().title(" Topics ").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(topics_list, columns[0], &mut list_state);

    // Right Column: Search Details
    let selected_topic = state.config.topics.get(state.topic_cursor);
    let topic_name = selected_topic.map(|t| t.name.as_str()).unwrap_or("Custom");

    let query_style = if state.editing_query {
        Style::default().fg(Color::Black).bg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let details = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            topic_name,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Search Query",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!(" {} ", display_query(state)),
            query_style,
        )),
        Line::from(""),
        Line::from("Controls:"),
        Line::from("  Enter fetches papers"),
        Line::from("  /     edits the search query"),
        Line::from("  q     quits the application"),
    ])
    .wrap(Wrap { trim: true })
    .block(Block::default().title(" Start ").borders(Borders::ALL));

    frame.render_widget(details, columns[1]);
}

fn render_results(frame: &mut ratatui::Frame<'_>, area: Rect, state: &AppState) {
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Left Column: Papers List
    let items: Vec<ListItem> = state
        .papers
        .iter()
        .map(|paper| {
            let selected = state.selected.contains(&paper.id);
            let marker = if selected { "[x]" } else { "[ ]" };

            let mut title = paper.title.clone();
            if title.chars().count() > 55 {
                title = title.chars().take(52).collect::<String>();
                title.push_str("...");
            }

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{marker} "),
                    Style::default().fg(if selected {
                        Color::Green
                    } else {
                        Color::DarkGray
                    }),
                ),
                Span::styled(title, Style::default().add_modifier(Modifier::BOLD)),
            ]))
        })
        .collect();

    let mut list_state = ListState::default();
    if !state.papers.is_empty() {
        list_state.select(Some(state.cursor));
    }

    let list = List::new(items)
        .block(
            Block::default()
                .title(format!(" Papers (Page {}) ", (state.offset / 20) + 1))
                .borders(Borders::ALL),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, body[0], &mut list_state);

    // Right Column: Preview
    let preview = if let Some(paper) = state.papers.get(state.cursor) {
        let authors = if paper.authors.is_empty() {
            "Unknown".into()
        } else {
            paper.authors.join(", ")
        };

        Paragraph::new(vec![
            Line::from(Span::styled(
                &paper.title,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Published: ", Style::default().fg(Color::Cyan)),
                Span::raw(&paper.published),
                Span::raw("    "),
                Span::styled("Citations: ", Style::default().fg(Color::Yellow)),
                Span::raw(paper.citations.to_string()),
            ]),
            Line::from(vec![
                Span::styled("Authors: ", Style::default().fg(Color::Green)),
                Span::raw(authors),
            ]),
            Line::from(vec![
                Span::styled("URL: ", Style::default().fg(Color::Cyan)),
                Span::raw(&paper.abs_url),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Abstract",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::raw(&paper.abstract_text),
        ])
    } else {
        Paragraph::new(vec![
            Line::from("No papers loaded."),
            Line::from("Press 'b' to go back and try another search."),
        ])
    }
    .wrap(Wrap { trim: true })
    .scroll((state.preview_scroll, 0)) // FIX: Apply scrolling to long abstracts
    .block(Block::default().title(" Preview ").borders(Borders::ALL));

    frame.render_widget(preview, body[1]);
}

fn render_imported(frame: &mut ratatui::Frame<'_>, area: Rect, state: &AppState) {
    let Some(report) = &state.import_report else {
        return;
    };

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Imported to Obsidian!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Folder:  ", Style::default().fg(Color::Cyan)),
            Span::raw(report.folder.display().to_string()),
        ]),
        Line::from(vec![
            Span::styled("Written: ", Style::default().fg(Color::Yellow)),
            Span::raw(report.written.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Skipped: ", Style::default().fg(Color::DarkGray)),
            Span::raw(format!("{} (Already existed)", report.skipped)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Files",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )),
    ];

    for file in report.files.iter().take(10) {
        lines.push(Line::from(format!("- {}", file.display())));
    }
    if report.files.len() > 10 {
        lines.push(Line::from(format!(
            "- ... and {} more",
            report.files.len() - 10
        )));
    }

    let panel = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .block(Block::default().title(" Done ").borders(Borders::ALL));

    frame.render_widget(panel, area);
}

fn render_footer(frame: &mut ratatui::Frame<'_>, area: Rect, state: &AppState) {
    let footer_text = if state.loading {
        format!("{} Working...", spinner(state.loading_frame))
    } else if let Some(err) = &state.error {
        format!("Error: {err}")
    } else {
        state.status.clone()
    };

    let controls = match state.screen {
        Screen::Start => "Enter fetch  / edit  j/k move  q quit",
        Screen::Results => {
            "j/k move  J/K scroll  space select  Enter import  n/p page  b back  q quit"
        }
        Screen::Imported => "Enter continue  t topics  q quit",
    };

    let footer = Paragraph::new(vec![
        Line::from(Span::styled(
            footer_text,
            Style::default().fg(Color::Yellow),
        )),
        Line::from(Span::styled(controls, Style::default().fg(Color::DarkGray))),
    ])
    .block(Block::default().borders(Borders::TOP));

    frame.render_widget(footer, area);
}

fn render_loading(frame: &mut ratatui::Frame<'_>, area: Rect, state: &AppState) {
    frame.render_widget(Clear, area); // Clear background to make popup readable
    let popup = Paragraph::new(Line::from(vec![
        Span::styled(
            spinner(state.loading_frame),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw(" Fetching papers... "),
    ]))
    .block(Block::default().title(" Network ").borders(Borders::ALL));

    frame.render_widget(popup, area);
}

// --- Helpers ---

fn spinner(frame: usize) -> &'static str {
    match frame {
        0 => "⠋",
        1 => "⠙",
        2 => "⠹",
        3 => "⠸",
        4 => "⠼",
        5 => "⠴",
        6 => "⠦",
        7 => "⠧",
        8 => "⠇",
        _ => "⠏",
    }
}

fn display_query(state: &AppState) -> String {
    if state.query.is_empty() {
        "type a search query".into()
    } else {
        state.query.clone()
    }
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(x, y, width, height)
}
