pub mod blind;
pub mod cards;
pub mod end;
pub mod joker_strip;
pub mod overlay;
pub mod pack;
pub mod postblind;
pub mod preblind;
pub mod shop;
pub mod sidebar;
pub mod size_check;
pub mod tarot;

use crate::app::AppState;
use balatro_rs::rank::HandRank;
use balatro_rs::stage::Stage;
use ratatui::style::Color;
use ratatui::Frame;

const MIN_WIDTH: u16 = 80;
const MIN_HEIGHT: u16 = 30;

pub fn render(f: &mut Frame, app: &mut AppState) {
    app.widget_rects.clear();

    let area = f.area();

    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        size_check::render(f, area, area.width, area.height, MIN_WIDTH, MIN_HEIGHT);
        return;
    }

    match &app.game.stage {
        Stage::PreBlind() => preblind::render(f, app, area),
        Stage::Blind(_) => blind::render(f, app, area),
        Stage::PostBlind() => postblind::render(f, app, area),
        Stage::Shop() => shop::render(f, app, area),
        Stage::TarotHand(_) => tarot::render(f, app, area),
        Stage::PackOpen() => pack::render(f, app, area),
        Stage::End(_) => end::render(f, app, area),
    }

    if let Some(ov) = app.overlay.clone() {
        overlay::render(f, app, area, &ov);
    }
}

pub(super) fn wrap(s: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in s.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.len() + 1 + word.len() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current.clone());
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

pub(super) fn hand_rank_name(rank: HandRank) -> &'static str {
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

pub(super) fn level_color(level: usize) -> Color {
    match level {
        1 => Color::Gray,
        2..=4 => Color::Cyan,
        5..=9 => Color::Yellow,
        _ => Color::Magenta,
    }
}
