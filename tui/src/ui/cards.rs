use crate::app::{AppState, WidgetId};
use balatro_rs::card::{Card, Edition, Enhancement, Seal, Suit, Value};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub fn suit_color(suit: Suit) -> Color {
    match suit {
        Suit::Heart => Color::Red,
        Suit::Diamond => Color::Blue,
        Suit::Club => Color::Green,
        Suit::Spade => Color::White,
    }
}

pub fn rank_str(value: Value) -> &'static str {
    match value {
        Value::Ace => "A",
        Value::King => "K",
        Value::Queen => "Q",
        Value::Jack => "J",
        Value::Ten => "T",
        Value::Nine => "9",
        Value::Eight => "8",
        Value::Seven => "7",
        Value::Six => "6",
        Value::Five => "5",
        Value::Four => "4",
        Value::Three => "3",
        Value::Two => "2",
    }
}

pub fn suit_char(suit: Suit) -> char {
    match suit {
        Suit::Heart => '♥',
        Suit::Diamond => '♦',
        Suit::Club => '♣',
        Suit::Spade => '♠',
    }
}

fn edition_color(edition: Edition) -> Color {
    match edition {
        Edition::Base => Color::Gray,
        Edition::Foil => Color::Cyan,
        Edition::Holographic => Color::Magenta,
        Edition::Polychrome => Color::LightGreen,
        Edition::Negative => Color::White,
    }
}

fn enhancement_indicator(enh: Enhancement) -> (&'static str, Color) {
    match enh {
        Enhancement::Bonus => ("Bn", Color::Blue),
        Enhancement::Mult => ("Mx", Color::Red),
        Enhancement::Wild => ("? ", Color::Magenta),
        Enhancement::Glass => ("Gl", Color::LightBlue),
        Enhancement::Steel => ("St", Color::Cyan),
        Enhancement::Stone => ("●●", Color::DarkGray),
        Enhancement::Gold => ("Au", Color::Yellow),
        Enhancement::Lucky => ("Lk", Color::Green),
    }
}

// Card dimensions: 8 wide × 6 tall (including borders)
// Each card slot: 9 wide (8 + 1 gap), 7 tall (6 card + 1 shift buffer)
pub const CARD_W: u16 = 8;
pub const CARD_H: u16 = 6;
pub const SLOT_W: u16 = 9; // CARD_W + 1 gap
pub const SLOT_H: u16 = 7; // CARD_H + 1 shift row

fn card_inner_text(card: Card) -> Text<'static> {
    let is_wild = card.enhancement == Some(Enhancement::Wild);
    let is_stone = card.enhancement == Some(Enhancement::Stone);

    let (rank_color, suit_display_color, suit_display_char) = if is_wild {
        (Color::Magenta, Color::Magenta, '?')
    } else {
        (suit_color(card.suit), suit_color(card.suit), suit_char(card.suit))
    };

    let rank_style = Style::default().fg(rank_color).add_modifier(Modifier::BOLD);
    let suit_style = Style::default().fg(suit_display_color).add_modifier(Modifier::BOLD);

    // Interior: 6 wide, 4 tall.
    // Row 0: seal indicator (top-right) or blank
    // Row 1: rank + suit centered
    // Row 2: blank
    // Row 3: enhancement indicator (bottom-left) or blank

    let seal_span: Option<Span<'static>> = card.seal.map(|s| match s {
        Seal::Gold => Span::styled("◆ ", Style::default().fg(Color::Yellow)),
        Seal::Red => Span::styled("◆ ", Style::default().fg(Color::Red)),
        Seal::Blue => Span::styled("◆ ", Style::default().fg(Color::Blue)),
        Seal::Purple => Span::styled("◆ ", Style::default().fg(Color::Magenta)),
    });

    let row0 = match seal_span {
        Some(s) => Line::from(vec![Span::raw("    "), s]),
        None => Line::from(""),
    };

    let row1 = if is_stone {
        Line::from(vec![
            Span::raw("  "),
            Span::styled("●●●", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
        ])
    } else {
        Line::from(vec![
            Span::raw("  "),
            Span::styled(rank_str(card.value), rank_style),
            Span::styled(suit_display_char.to_string(), suit_style),
            Span::raw("   "),
        ])
    };

    let row3 = match card.enhancement {
        Some(enh) if !is_stone => {
            let (label, color) = enhancement_indicator(enh);
            Line::from(vec![
                Span::raw(" "),
                Span::styled(label, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::raw("   "),
            ])
        }
        _ => Line::from(""),
    };

    Text::from(vec![row0, row1, Line::from(""), row3])
}

fn card_block(card: Card, is_cursor: bool) -> Block<'static> {
    let border_type = if is_cursor {
        BorderType::Double
    } else {
        BorderType::Plain
    };
    let border_color = if is_cursor {
        Color::Yellow
    } else {
        edition_color(card.edition)
    };
    Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(Style::default().fg(border_color))
}

pub fn render_hand(
    f: &mut Frame,
    app: &mut AppState,
    area: Rect,
    cards_and_selected: &[(Card, bool)],
    cursor_idx: usize,
    widget_id_fn: impl Fn(usize) -> WidgetId,
) {
    for (i, (card, selected)) in cards_and_selected.iter().enumerate() {
        let x = area.x + i as u16 * SLOT_W;
        if x + CARD_W > area.x + area.width {
            break;
        }

        // Shift selected cards up by 1 row
        let y_offset: u16 = if *selected { 0 } else { 1 };
        let card_rect = Rect {
            x,
            y: area.y + y_offset,
            width: CARD_W,
            height: CARD_H,
        };

        let is_cursor = i == cursor_idx;
        let block = card_block(*card, is_cursor);
        let text = card_inner_text(*card);
        let para = Paragraph::new(text).block(block);
        f.render_widget(para, card_rect);

        app.widget_rects.insert(widget_id_fn(i), card_rect);
    }
}
