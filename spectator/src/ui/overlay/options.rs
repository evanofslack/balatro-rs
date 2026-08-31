use crate::app::{AppState, WidgetId};
use crate::ui::overlay::centered_rect;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Minimal Options menu — for now its only content is Save Game, the same
/// action as the `e` keybind (`AppState::open_save`). Room to grow later.
pub fn render(f: &mut Frame, app: &mut AppState, area: Rect) {
    let w: u16 = 30;
    let h: u16 = 8;
    let rect = centered_rect(w, h, area);
    f.render_widget(Clear, rect);

    let button_text = "  [ Save Game ]";
    let button_row = 2u16;
    let lines = vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            button_text,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Esc to close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let block = Block::default()
        .title(Span::styled(
            " Options ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    let para = Paragraph::new(Text::from(lines)).block(block);
    f.render_widget(para, rect);

    app.widget_rects.insert(
        WidgetId::OverlayButton(0),
        Rect {
            x: rect.x + 1,
            y: rect.y + 1 + button_row,
            width: button_text.chars().count() as u16,
            height: 1,
        },
    );
}
