use crate::ui::overlay::centered_rect;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

pub fn render(f: &mut Frame, _app: &mut crate::app::AppState, area: Rect) {
    let w: u16 = 52;
    let h: u16 = 30;
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
        section("Universal"),
        Line::from(""),
        bind("?", "Show controls"),
        bind("r", "Run info (deck, poker hands)"),
        bind("s", "Save game"),
        bind("q / Ctrl+C", "Quit"),
        Line::from(""),
        section("Navigation"),
        Line::from(""),
        bind("Tab / Shift+Tab", "Cycle zones"),
        bind("i", "Inspect focused item"),
        bind("Esc / Enter", "Close overlay"),
        Line::from(""),
        section("Blind"),
        Line::from(""),
        bind("Enter", "Select / deselect card"),
        bind("p", "Play hand"),
        bind("d", "Discard"),
        bind("←/→", "Move cursor"),
        Line::from(""),
        section("Pre-Blind"),
        Line::from(""),
        bind("←/→", "Navigate blinds"),
        bind("Enter", "Select blind"),
        Line::from(""),
        section("Shop"),
        Line::from(""),
        bind("Enter", "Buy item"),
        bind("n", "Next round"),
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

fn section(title: &'static str) -> Line<'static> {
    Line::from(Span::styled(
        format!("  {} ", title),
        Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD),
    ))
}
