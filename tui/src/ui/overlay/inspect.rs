use crate::app::{AppState, InspectTarget, WidgetId};
use crate::ui::cards::{rank_str, suit_char, suit_color};
use crate::ui::overlay::centered_rect;
use balatro_rs::joker::Joker;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &mut AppState, area: Rect, target: InspectTarget) {
    let w: u16 = 44;
    let h: u16 = 18;
    let rect = centered_rect(w, h, area);

    f.render_widget(Clear, rect);

    let (title, lines) = match target {
        InspectTarget::Card(card) => {
            let suit_style = Style::default()
                .fg(suit_color(card.suit))
                .add_modifier(Modifier::BOLD);
            let title = format!(" {} of {}s ", rank_str(card.value), suit_char(card.suit));
            let lines = vec![
                Line::from(""),
                Line::from(vec![
                    Span::raw("  Rank:        "),
                    Span::styled(
                        rank_str(card.value),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("  Suit:        "),
                    Span::styled(suit_char(card.suit).to_string(), suit_style),
                ]),
                Line::from(vec![
                    Span::raw("  Enhancement: "),
                    Span::styled(
                        card.enhancement
                            .map_or_else(|| "none".to_string(), |e| format!("{:?}", e)),
                        Style::default().fg(Color::Cyan),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("  Edition:     "),
                    Span::styled(
                        format!("{:?}", card.edition),
                        Style::default().fg(Color::Cyan),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("  Seal:        "),
                    Span::styled(
                        card.seal
                            .map_or_else(|| "none".to_string(), |e| format!("{:?}", e)),
                        Style::default().fg(Color::Cyan),
                    ),
                ]),
                Line::from(""),
                close_line(),
            ];
            (title, lines)
        }
        InspectTarget::Joker(joker) => {
            let title = format!(" {} ", joker.name());
            let desc = joker.desc();
            let mut lines = vec![
                Line::from(""),
                Line::from(vec![
                    Span::raw("  Rarity: "),
                    Span::styled(
                        joker.rarity().to_string(),
                        Style::default().fg(Color::Magenta),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("  Cost:   "),
                    Span::styled(
                        format!("${}", joker.cost()),
                        Style::default().fg(Color::Yellow),
                    ),
                ]),
                Line::from(""),
            ];
            for word_line in wrap(&desc, w as usize - 4) {
                lines.push(Line::from(Span::styled(
                    format!("  {}", word_line),
                    Style::default().fg(Color::White),
                )));
            }
            lines.push(Line::from(""));
            lines.push(close_line());
            (title, lines)
        }
        InspectTarget::Consumable(c) => {
            let title = format!(" {} ", c.name());
            let desc = c.description();
            let mut lines = vec![
                Line::from(""),
                Line::from(vec![
                    Span::raw("  Type:  "),
                    Span::styled(c.type_label().to_string(), Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![
                    Span::raw("  Cost:  "),
                    Span::styled(format!("${}", c.cost()), Style::default().fg(Color::Yellow)),
                ]),
                Line::from(""),
            ];
            for word_line in wrap(&desc, w as usize - 4) {
                lines.push(Line::from(Span::styled(
                    format!("  {}", word_line),
                    Style::default().fg(Color::White),
                )));
            }
            lines.push(Line::from(""));
            lines.push(close_line());
            (title, lines)
        }
    };

    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    let para = Paragraph::new(Text::from(lines)).block(block);
    f.render_widget(para, rect);

    // Close button rect
    app.widget_rects.insert(
        WidgetId::OverlayButton(0),
        Rect {
            x: rect.x + w / 2 - 5,
            y: rect.y + h - 2,
            width: 10,
            height: 1,
        },
    );
}

fn close_line() -> Line<'static> {
    Line::from(Span::styled(
        "       [ Close ]",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ))
}

fn wrap(s: &str, width: usize) -> Vec<String> {
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
