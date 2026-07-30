use crate::app::{AppState, RunInfoTab};
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
        RunInfoTab::Vouchers => render_vouchers(f, app, chunks[2]),
    }

    render_footer(f, app, chunks[3]);
}

/// Vouchers redeemed this run, in purchase order, plus whatever this
/// ante still has on offer.
fn render_vouchers(f: &mut Frame, app: &AppState, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    if app.game.vouchers.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No vouchers redeemed yet",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for voucher in app.game.vouchers.owned() {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:<16}", voucher.name()),
                    Style::default()
                        .fg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    voucher.description().to_string(),
                    Style::default().fg(Color::White),
                ),
            ]));
        }
    }

    if let Some(offered) = app.game.shop.voucher {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  In shop: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                offered.name().to_string(),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                format!(" (${})", offered.cost()),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }

    f.render_widget(Paragraph::new(lines), area);
}

fn render_footer(f: &mut Frame, app: &AppState, area: Rect) {
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
        .split(area);
    f.render_widget(
        Paragraph::new(Span::styled(
            "  Tab next  ←/→ deck view  Esc / r close",
            Style::default().fg(Color::DarkGray),
        )),
        footer[0],
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
