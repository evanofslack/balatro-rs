use crate::app::{AppState, RunInfoTab, WidgetId};
use crate::ui::overlay::{centered_rect, deck, poker_hands};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &mut AppState, area: Rect) {
    let w: u16 = 72;
    let h: u16 = 22;
    let rect = centered_rect(w, h, area);
    f.render_widget(Clear, rect);

    let block = Block::default()
        .title(Span::styled(
            " Run Info ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));
    let inner = block.inner(rect);
    f.render_widget(block, rect);

    let tabs = [
        (RunInfoTab::Deck, "Deck"),
        (RunInfoTab::PokerHands, "Poker Hands"),
        (RunInfoTab::Vouchers, "Vouchers"),
    ];

    let tab_spans: Vec<Span> = tabs
        .iter()
        .enumerate()
        .flat_map(|(i, (tab, label))| {
            let active = &app.run_info_tab == tab;
            let style = if active {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let mut spans = vec![Span::styled(format!("[ {} ]", label), style)];
            if i < tabs.len() - 1 {
                spans.push(Span::raw("  "));
            }
            spans
        })
        .collect();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(inner);

    f.render_widget(Paragraph::new(Line::from(tab_spans)), chunks[0]);

    // Register mouse rects for tabs
    let mut x = inner.x;
    for (i, (_, label)) in tabs.iter().enumerate() {
        let tab_w = label.len() as u16 + 4;
        app.widget_rects.insert(
            WidgetId::RunInfoTab(i),
            Rect {
                x,
                y: chunks[0].y,
                width: tab_w,
                height: 1,
            },
        );
        x += tab_w + 2;
    }

    f.render_widget(
        Paragraph::new(Span::styled(
            "─".repeat(inner.width as usize),
            Style::default().fg(Color::DarkGray),
        )),
        chunks[1],
    );

    match app.run_info_tab {
        RunInfoTab::Deck => deck::render_body(f, app, chunks[2]),
        RunInfoTab::PokerHands => poker_hands::render_body(f, app, chunks[2]),
        RunInfoTab::Vouchers => {
            f.render_widget(
                Paragraph::new(Span::styled(
                    "  Coming soon",
                    Style::default().fg(Color::DarkGray),
                )),
                chunks[2],
            );
        }
    }

    let seed_label = app
        .game
        .seed_str
        .as_deref()
        .map(|s| s.to_string())
        .unwrap_or_else(|| app.game.seed.to_string());
    let seed_text = format!("Seed: {}  ", seed_label);
    let seed_w = seed_text.len() as u16;
    let footer = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(seed_w)])
        .split(chunks[3]);
    let prefix = "  Tab next  ←/→ deck view  ";
    let close_label = "[ Close ]";
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(prefix, Style::default().fg(Color::DarkGray)),
            Span::styled(
                close_label,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ])),
        footer[0],
    );
    app.widget_rects.insert(
        WidgetId::OverlayButton(0),
        Rect {
            x: footer[0].x + prefix.chars().count() as u16,
            y: footer[0].y,
            width: close_label.len() as u16,
            height: 1,
        },
    );
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Seed: ", Style::default().fg(Color::DarkGray)),
            Span::styled(seed_label, Style::default().fg(Color::White)),
            Span::raw("  "),
        ])),
        footer[1],
    );
}
