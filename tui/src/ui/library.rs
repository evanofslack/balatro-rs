use crate::app::{AppState, WidgetId};
use balatro_rs::replay_log::{ReplayResult, ReplaySource};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &mut AppState, area: Rect) {
    f.render_widget(Clear, area);
    let Some(library) = app.library.as_ref() else {
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(area);

    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            " Replay Library ",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {}", library.status),
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let items: Vec<ListItem> = if library.entries.is_empty() {
        vec![ListItem::new(Span::styled(
            "  (no runs yet — use --record to create one)",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        library
            .entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let outcome = match entry.result {
                    Some(ReplayResult::Won) => "WIN",
                    Some(ReplayResult::Lost) => "LOSS",
                    Some(ReplayResult::Abandoned) => "LEFT",
                    Some(ReplayResult::Error) => "ERROR",
                    None => "LIVE",
                };
                let score = entry
                    .score
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "-".into());
                let steps = entry
                    .steps
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "-".into());
                let agent = entry.agent.as_deref().unwrap_or("-");
                let source = match entry.source {
                    ReplaySource::Player => "PLAYER",
                    ReplaySource::Model => "MODEL",
                };
                let line = format!(
                    " {:<24}  {:<6}  seed={}  {:>4}  score={:<6}  steps={:<5}  {}",
                    entry.display_name(),
                    source,
                    entry.seed,
                    outcome,
                    score,
                    steps,
                    agent
                );
                let style = if i == library.cursor {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Span::styled(line, style))
            })
            .collect()
    };

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(" runs "));
    f.render_widget(list, chunks[1]);

    // Store row hit targets roughly as full-width lines inside the list.
    if let Some(library) = app.library.as_ref() {
        let list_inner = Block::default().borders(Borders::ALL).inner(chunks[1]);
        for i in 0..library.entries.len() {
            let y = list_inner.y.saturating_add(i as u16);
            if y < list_inner.y + list_inner.height {
                app.widget_rects.insert(
                    WidgetId::LibraryRow(i),
                    Rect {
                        x: list_inner.x,
                        y,
                        width: list_inner.width,
                        height: 1,
                    },
                );
            }
        }
    }

    let footer = if let Some(library) = app.library.as_ref() {
        if let Some(input) = &library.rename_input {
            format!("Rename: {input}_    Enter save   Esc cancel")
        } else if library.confirm_delete {
            "Delete selected?  y confirm   n/Esc cancel".to_string()
        } else {
            "j/k move   Enter watch   R rename   d delete   r refresh   Esc/q quit".to_string()
        }
    } else {
        String::new()
    };
    f.render_widget(
        Paragraph::new(footer)
            .alignment(Alignment::Left)
            .block(Block::default().borders(Borders::ALL)),
        chunks[2],
    );
}
