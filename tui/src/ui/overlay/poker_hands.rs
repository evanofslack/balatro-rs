use crate::app::AppState;
use balatro_rs::rank::HandRank;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

const BASE_RANKS: &[HandRank] = &[
    HandRank::StraightFlush,
    HandRank::FourOfAKind,
    HandRank::FullHouse,
    HandRank::Flush,
    HandRank::Straight,
    HandRank::ThreeOfAKind,
    HandRank::TwoPair,
    HandRank::OnePair,
    HandRank::HighCard,
];

const SECRET_RANKS: &[HandRank] = &[
    HandRank::FlushFive,
    HandRank::FlushHouse,
    HandRank::FiveOfAKind,
    HandRank::RoyalFlush,
];

fn rank_name(rank: HandRank) -> &'static str {
    match rank {
        HandRank::HighCard => "High Card",
        HandRank::OnePair => "One Pair",
        HandRank::TwoPair => "Two Pair",
        HandRank::ThreeOfAKind => "Three of a Kind",
        HandRank::Straight => "Straight",
        HandRank::Flush => "Flush",
        HandRank::FullHouse => "Full House",
        HandRank::FourOfAKind => "Four of a Kind",
        HandRank::StraightFlush => "Straight Flush",
        HandRank::RoyalFlush => "Royal Flush",
        HandRank::FiveOfAKind => "Five of a Kind",
        HandRank::FlushHouse => "Flush House",
        HandRank::FlushFive => "Flush Five",
    }
}

fn level_color(level: usize) -> Color {
    match level {
        1 => Color::Gray,
        2..=4 => Color::Cyan,
        5..=9 => Color::Yellow,
        _ => Color::Magenta,
    }
}

pub fn render_body(f: &mut Frame, app: &AppState, area: Rect) {
    let secret_visible: Vec<HandRank> = SECRET_RANKS
        .iter()
        .copied()
        .filter(|&r| app.game.planetarium.level(r).plays > 0)
        .collect();

    let inner_w = area.width as usize;

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));

    for &rank in BASE_RANKS {
        lines.push(rank_row(app, rank, inner_w));
    }

    if !secret_visible.is_empty() {
        lines.push(Line::from(Span::styled(
            "─".repeat(inner_w),
            Style::default().fg(Color::DarkGray),
        )));
        for rank in &secret_visible {
            lines.push(rank_row(app, *rank, inner_w));
        }
    }

    let para = Paragraph::new(lines).alignment(Alignment::Left);
    f.render_widget(para, area);
}

fn rank_row(app: &AppState, rank: HandRank, inner_w: usize) -> Line<'static> {
    let lvl = app.game.planetarium.level(rank);
    let lc = level_color(lvl.level);

    let badge = format!(" lvl.{:<2} ", lvl.level);
    let name  = format!("  {:<18}", rank_name(rank));
    let chips = format!("{:>4}", lvl.chips);
    let mult  = format!("{:>2}", lvl.mult);
    let plays = format!("{:>3}", lvl.plays);
    let fixed = badge.len() + name.len() + chips.len() + 3 + mult.len() + 4 + plays.len();
    let gap   = " ".repeat(inner_w.saturating_sub(fixed));

    Line::from(vec![
        Span::styled(badge, Style::default().fg(lc).add_modifier(Modifier::BOLD)),
        Span::styled(name,  Style::default().fg(Color::White)),
        Span::raw(gap),
        Span::styled(chips, Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
        Span::styled(" x ", Style::default().fg(Color::White)),
        Span::styled(mult,  Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::styled("  # ", Style::default().fg(Color::White)),
        Span::styled(plays, Style::default().fg(Color::Yellow)),
    ])
}
