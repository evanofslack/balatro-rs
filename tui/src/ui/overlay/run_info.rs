use crate::app::{AppState, RunInfoTab};
use crate::ui::overlay::{centered_rect, deck, poker_hands};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

pub fn render(f: &mut Frame, app: &mut AppState, area: Rect) {
    let w: u16 = 72;
    let h: u16 = 22;
    let rect = centered_rect(w, h, area);
    f.render_widget(Clear, rect);

    let block = Block::default()
        .title(Span::styled(
            " Run Info ",
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
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
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
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

    f.render_widget(
        Paragraph::new(Span::styled(
            "  Tab next  ←/→ deck view  Esc / r close",
            Style::default().fg(Color::DarkGray),
        )),
        chunks[3],
    );
}
