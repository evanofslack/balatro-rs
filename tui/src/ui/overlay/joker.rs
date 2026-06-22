use crate::app::{AppState, WidgetId};
use crate::ui::overlay::centered_rect;
use balatro_rs::joker::Joker;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &mut AppState, area: Rect, idx: usize) {
    let w: u16 = 36;
    let h: u16 = 12;
    let rect = centered_rect(w, h, area);
    f.render_widget(Clear, rect);

    let Some(joker) = app.game.jokers.get(idx).cloned() else {
        app.overlay = None;
        return;
    };

    let sell_value = joker.sell_value();

    let sell_selected = app.overlay_cursor == 0;
    let sell_style = if sell_selected {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let close_style = if !sell_selected {
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Rarity: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:?}", joker.rarity()),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", joker.desc()),
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Sell value: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("${}", sell_value),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("  [ Sell (${}) ]", sell_value),
                sell_style,
            ),
            Span::raw("   "),
            Span::styled("[ Close ]", close_style),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Esc to cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let block = Block::default()
        .title(Span::styled(
            format!(" {} ", joker.name()),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let para = Paragraph::new(Text::from(lines)).block(block);
    f.render_widget(para, rect);

    app.widget_rects.insert(
        WidgetId::OverlayButton(0),
        Rect {
            x: rect.x + 2,
            y: rect.y + 8,
            width: 14,
            height: 1,
        },
    );
    app.widget_rects.insert(
        WidgetId::OverlayButton(1),
        Rect {
            x: rect.x + 19,
            y: rect.y + 8,
            width: 9,
            height: 1,
        },
    );
}
