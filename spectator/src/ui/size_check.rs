use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, w: u16, h: u16, min_w: u16, min_h: u16) {
    let lines = vec![
        Line::from(""),
        Line::from("Terminal too small").style(Style::default().fg(Color::Red)),
        Line::from(""),
        Line::from(format!("Current:  {} × {}", w, h)),
        Line::from(format!("Minimum: {} × {}", min_w, min_h)),
        Line::from(""),
        Line::from("Resize your terminal to continue.").style(Style::default().fg(Color::DarkGray)),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::DarkGray));

    let para = Paragraph::new(Text::from(lines))
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(para, area);
}
