use crate::ui::overlay::centered_rect;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

pub fn render(f: &mut Frame, _app: &mut crate::app::AppState, area: Rect) {
    let w: u16 = 58;
    let h: u16 = 22;
    let rect = centered_rect(w, h, area);
    f.render_widget(Clear, rect);

    let block = Block::default()
        .title(Span::styled(
            " Controls ",
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));
    let inner = block.inner(rect);
    f.render_widget(block, rect);

    let lines = vec![
        Line::from(""),
        bind("?", "Show controls"),
        bind("r", "Run info (Deck, Poker Hands)"),
        bind("s", "Save"),
        bind("q", "Quit"),
        bind("Tab / Shift+Tab", "Next / prev zone"),
        bind("←/→", "Move cursor"),
        bind("Enter / Space", "Select / confirm"),
        bind("i", "Inspect item"),
        bind("Esc / Enter", "Close overlay"),
        Line::from(""),
        bind("p", "Play hand  (blind)"),
        bind("d", "Discard    (blind)"),
        bind("n", "Next round (shop)"),
    ];

    let para = Paragraph::new(lines);
    f.render_widget(para, inner);
}

fn bind(key: &'static str, action: &'static str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {:<18}", key),
            Style::default().fg(Color::Green),
        ),
        Span::styled(action, Style::default().fg(Color::White)),
    ])
}
