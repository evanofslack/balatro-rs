use crate::app::{AppState, WidgetId};
use crate::ui::overlay::centered_rect;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &mut AppState, area: Rect) {
    let w: u16 = 58;
    let h: u16 = 22;
    let rect = centered_rect(w, h, area);
    f.render_widget(Clear, rect);

    let block = Block::default()
        .title(Span::styled(
            " Controls ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));
    let inner = block.inner(rect);
    f.render_widget(block, rect);

    let close_label = "  [ Close ]";
    let lines = vec![
        Line::from(""),
        bind("?", "Show controls"),
        bind("r", "Run info (Deck, Poker Hands)"),
        bind("e", "Export game"),
        bind("q", "Quit"),
        bind("i", "Inspect item"),
        bind("Tab / Shift+Tab", "Next / prev zone"),
        bind("←/→", "Move cursor"),
        bind("Enter / Space", "Select / confirm"),
        bind("Esc / Enter", "Close overlay"),
        Line::from(""),
        bind("p", "Play hand  (blind)"),
        bind("s", "Sort hand rank/suit  (blind)"),
        bind("d", "Discard    (blind)"),
        bind("n", "Next round (shop)"),
        Line::from(""),
        Line::from(Span::styled(
            close_label,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
    ];
    let close_row = lines.len() as u16 - 1;

    let para = Paragraph::new(lines);
    f.render_widget(para, inner);

    app.widget_rects.insert(
        WidgetId::OverlayButton(0),
        Rect {
            x: inner.x,
            y: inner.y + close_row,
            width: close_label.chars().count() as u16,
            height: 1,
        },
    );
}

fn bind(key: &'static str, action: &'static str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {:<18}", key), Style::default().fg(Color::Green)),
        Span::styled(action, Style::default().fg(Color::White)),
    ])
}
