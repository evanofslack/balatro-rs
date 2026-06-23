use crate::app::{AppState, WidgetId};
use crate::ui::overlay::centered_rect;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &mut AppState, area: Rect) {
    let w: u16 = 50;
    let h: u16 = 12;
    let rect = centered_rect(w, h, area);
    f.render_widget(Clear, rect);

    let filename = &app.save_input;

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Filename:",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(vec![
            Span::raw("  ┌"),
            Span::raw("─".repeat(w as usize - 6)),
            Span::raw("┐"),
        ]),
        Line::from(vec![
            Span::raw("  │ "),
            Span::styled(
                format!("{:<width$}", filename, width = w as usize - 7),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw("│"),
        ]),
        Line::from(vec![
            Span::raw("  └"),
            Span::raw("─".repeat(w as usize - 6)),
            Span::raw("┘"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Type to edit, Enter to export, Esc to cancel",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "        [ Export ]",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("   "),
            Span::styled("[ Cancel ]", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let block = Block::default()
        .title(Span::styled(
            " Export Game ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    let para = Paragraph::new(Text::from(lines)).block(block);
    f.render_widget(para, rect);

    // Button rects
    app.widget_rects.insert(
        WidgetId::OverlayButton(0),
        Rect {
            x: rect.x + 9,
            y: rect.y + h - 3,
            width: 8,
            height: 1,
        },
    );
    app.widget_rects.insert(
        WidgetId::OverlayButton(1),
        Rect {
            x: rect.x + 20,
            y: rect.y + h - 3,
            width: 10,
            height: 1,
        },
    );
}
