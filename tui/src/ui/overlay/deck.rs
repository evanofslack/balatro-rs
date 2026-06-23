use crate::app::{AppState, DeckTab, WidgetId};
use crate::ui::cards::{rank_str, suit_color};
use balatro_rs::card::{Card, Edition, Suit, Value};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

const SUITS: [Suit; 4] = [Suit::Spade, Suit::Heart, Suit::Club, Suit::Diamond];
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

fn edition_color(edition: Edition) -> Option<Color> {
    match edition {
        Edition::Base => None,
        Edition::Foil => Some(Color::Cyan),
        Edition::Holographic => Some(Color::Magenta),
        Edition::Polychrome => Some(Color::LightGreen),
        Edition::Negative => Some(Color::White),
    }
}

pub fn render_body(f: &mut Frame, app: &mut AppState, area: Rect) {
    let in_deck = app.game.deck.cards();
    let in_hand = app.game.available.cards();
    let discarded = app.game.discarded.clone();

    let (active_pool, ghost_pool): (Vec<Card>, Vec<Card>) = match app.deck_tab {
        DeckTab::Remaining => {
            let mut ghosts = in_hand.to_vec();
            ghosts.extend(discarded.iter().copied());
            (in_deck.clone(), ghosts)
        }
        DeckTab::Full => {
            let mut all = in_deck.clone();
            all.extend(in_hand.iter().copied());
            all.extend(discarded.iter().copied());
            (all, vec![])
        }
    };

    let tabs = [
        (DeckTab::Remaining, format!("Remaining: {}", in_deck.len())),
        (
            DeckTab::Full,
            format!("Full: {}", in_deck.len() + in_hand.len() + discarded.len()),
        ),
    ];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(area);

    // Sub-tabs
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
    f.render_widget(Paragraph::new(Line::from(tab_spans)), chunks[0]);
    f.render_widget(
        Paragraph::new(Span::styled(
            "─".repeat(area.width as usize),
            Style::default().fg(Color::DarkGray),
        )),
        chunks[1],
    );

    // Horizontal split: stats | cards
    let body = chunks[2];
    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(18),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(body);

    render_stats(f, &active_pool, split[0]);

    // Vertical divider
    let div_lines: Vec<Line> = (0..split[1].height).map(|_| Line::from("│")).collect();
    f.render_widget(
        Paragraph::new(div_lines).style(Style::default().fg(Color::DarkGray)),
        split[1],
    );

    render_cards(f, app, &active_pool, &ghost_pool, split[2]);

    // Register mouse rects for sub-tabs
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

fn render_stats(f: &mut Frame, pool: &[Card], area: Rect) {
    let aces = pool.iter().filter(|c| c.value == Value::Ace).count();
    let face = pool
        .iter()
        .filter(|c| matches!(c.value, Value::King | Value::Queen | Value::Jack))
        .count();
    let num = pool.len().saturating_sub(aces + face);

    let suit_count = |s: Suit| pool.iter().filter(|c| c.suit == s).count();
    let rank_count = |v: Value| pool.iter().filter(|c| c.value == v).count();

    let dim = Style::default().fg(Color::DarkGray);
    let white = Style::default().fg(Color::White);

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled(" Aces ", dim),
            Span::styled(aces.to_string(), white),
            Span::styled("  Face ", dim),
            Span::styled(face.to_string(), white),
        ]),
        Line::from(vec![
            Span::styled(" Num  ", dim),
            Span::styled(num.to_string(), white),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" ♠ ", Style::default().fg(suit_color(Suit::Spade))),
            Span::styled(format!("{:<3}", suit_count(Suit::Spade)), white),
            Span::styled(" ♥ ", Style::default().fg(suit_color(Suit::Heart))),
            Span::styled(suit_count(Suit::Heart).to_string(), white),
        ]),
        Line::from(vec![
            Span::styled(" ♣ ", Style::default().fg(suit_color(Suit::Club))),
            Span::styled(format!("{:<3}", suit_count(Suit::Club)), white),
            Span::styled(" ♦ ", Style::default().fg(suit_color(Suit::Diamond))),
            Span::styled(suit_count(Suit::Diamond).to_string(), white),
        ]),
        Line::from(""),
    ];

    let rank_pairs: [&[Value; 2]; 6] = [
        &[Value::Ace, Value::Eight],
        &[Value::King, Value::Seven],
        &[Value::Queen, Value::Six],
        &[Value::Jack, Value::Five],
        &[Value::Ten, Value::Four],
        &[Value::Nine, Value::Three],
    ];
    for pair in &rank_pairs {
        lines.push(Line::from(vec![
            Span::styled(format!(" {:<2}", rank_str(pair[0])), dim),
            Span::styled(format!(" {:<4}", rank_count(pair[0])), white),
            Span::styled(format!(" {:<2}", rank_str(pair[1])), dim),
            Span::styled(rank_count(pair[1]).to_string(), white),
        ]));
    }
    lines.push(Line::from(vec![
        Span::styled("         2 ", dim),
        Span::styled(rank_count(Value::Two).to_string(), white),
    ]));

    f.render_widget(Paragraph::new(lines), area);
}

fn render_cards(f: &mut Frame, _app: &mut AppState, pool: &[Card], ghosts: &[Card], area: Rect) {
    let card_w: u16 = 3;
    let card_h: u16 = 3;
    let gap_h: u16 = 1;

    for (si, suit) in SUITS.iter().enumerate() {
        let row_y = area.y + si as u16 * (card_h + gap_h);
        if row_y + card_h > area.y + area.height {
            break;
        }

        // Collect cards in VALUES order: (value, is_ghost)
        let row: Vec<(Value, bool)> = VALUES
            .iter()
            .filter_map(|v| {
                if pool.iter().any(|c| c.suit == *suit && c.value == *v) {
                    Some((*v, false))
                } else if ghosts.iter().any(|c| c.suit == *suit && c.value == *v) {
                    Some((*v, true))
                } else {
                    None
                }
            })
            .collect();

        for (ci, (value, is_ghost)) in row.iter().enumerate() {
            let card_x = area.x + ci as u16 * card_w;
            if card_x + card_w > area.x + area.width {
                break;
            }

            let rect = Rect {
                x: card_x,
                y: row_y,
                width: card_w,
                height: card_h,
            };

            if *is_ghost {
                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Plain)
                    .border_style(Style::default().fg(Color::DarkGray));
                let inner = block.inner(rect);
                f.render_widget(block, rect);
                f.render_widget(
                    Paragraph::new(Span::styled("·", Style::default().fg(Color::DarkGray))),
                    inner,
                );
            } else {
                let card = pool
                    .iter()
                    .find(|c| c.suit == *suit && c.value == *value)
                    .unwrap();
                let border_color = edition_color(card.edition).unwrap_or_else(|| suit_color(*suit));
                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Plain)
                    .border_style(Style::default().fg(border_color));
                let inner = block.inner(rect);
                f.render_widget(block, rect);
                f.render_widget(
                    Paragraph::new(Span::styled(
                        rank_str(*value),
                        Style::default()
                            .fg(suit_color(*suit))
                            .add_modifier(Modifier::BOLD),
                    )),
                    inner,
                );
            }
        }
    }
}
