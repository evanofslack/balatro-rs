use crate::app::{AppState, DeckTab, WidgetId};
use crate::ui::cards::{rank_str, suit_char, suit_color};
use balatro_rs::card::{Suit, Value};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
    Frame,
};

const SUITS: [Suit; 4] = [Suit::Spade, Suit::Heart, Suit::Diamond, Suit::Club];
const VALUES: [Value; 13] = [
    Value::Ace,
    Value::King,
    Value::Queen,
    Value::Jack,
    Value::Ten,
    Value::Nine,
    Value::Eight,
    Value::Seven,
    Value::Six,
    Value::Five,
    Value::Four,
    Value::Three,
    Value::Two,
];

pub fn render_body(f: &mut Frame, app: &mut AppState, area: Rect) {
    let in_deck = app.game.deck.cards();
    let in_hand = app.game.available.cards();
    let discarded = &app.game.discarded;

    let active_pool: Vec<balatro_rs::card::Card> = match app.deck_tab {
        DeckTab::InDeck => in_deck.clone(),
        DeckTab::InHand => in_hand.clone(),
        DeckTab::Discarded => discarded.clone(),
    };

    let tabs = [
        (DeckTab::InDeck, format!("In Deck: {}", in_deck.len())),
        (DeckTab::InHand, format!("In Hand: {}", in_hand.len())),
        (
            DeckTab::Discarded,
            format!("Discarded: {}", discarded.len()),
        ),
    ];

    let mut lines: Vec<Line> = Vec::new();

    let tab_spans: Vec<Span> = tabs
        .iter()
        .enumerate()
        .flat_map(|(i, (tab, label))| {
            let active = &app.deck_tab == tab;
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
    lines.push(Line::from(tab_spans));
    lines.push(Line::from(Span::styled(
        "─".repeat(area.width as usize),
        Style::default().fg(Color::DarkGray),
    )));

    for suit in &SUITS {
        let suit_style = Style::default()
            .fg(suit_color(*suit))
            .add_modifier(Modifier::BOLD);
        let mut row_spans = vec![
            Span::styled(suit_char(*suit).to_string(), suit_style),
            Span::raw("  "),
        ];
        for value in &VALUES {
            let present = active_pool
                .iter()
                .any(|c| c.suit == *suit && c.value == *value);
            if present {
                row_spans.push(Span::styled(
                    format!("{:<3}", rank_str(*value)),
                    Style::default().fg(suit_color(*suit)),
                ));
            } else {
                row_spans.push(Span::styled(
                    format!("{:<3}", "·"),
                    Style::default().fg(Color::DarkGray),
                ));
            }
        }
        lines.push(Line::from(row_spans));
        lines.push(Line::from(""));
    }

    let para = Paragraph::new(Text::from(lines));
    f.render_widget(para, area);

    let mut x = area.x;
    for (i, (_, label)) in tabs.iter().enumerate() {
        let tab_w = label.len() as u16 + 4;
        app.widget_rects.insert(
            WidgetId::DeckTab(i),
            Rect {
                x,
                y: area.y,
                width: tab_w,
                height: 1,
            },
        );
        x += tab_w + 2;
    }
}
