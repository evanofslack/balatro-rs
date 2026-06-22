use crate::app::{AppState, WidgetId};
use balatro_rs::stage::{End, Stage};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};

pub fn render(f: &mut Frame, app: &mut AppState, area: Rect) {
    let end = match &app.game.stage {
        Stage::End(e) => e.clone(),
        _ => return,
    };

    let (color, msg) = match end {
        End::Win => (Color::Green, "You Win!"),
        End::Lose => (Color::Red, "Game Over"),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines = vec![
        Line::from(""),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            msg,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press q to quit",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "[ Quit ]",
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )),
    ];

    let para = Paragraph::new(Text::from(lines))
        .alignment(Alignment::Center);
    f.render_widget(para, inner);

    // Register quit button rect (approximate center)
    let btn_rect = Rect {
        x: inner.x + inner.width / 2 - 5,
        y: inner.y + 8,
        width: 10,
        height: 1,
    };
    app.widget_rects.insert(WidgetId::CashOutButton, btn_rect);
}
