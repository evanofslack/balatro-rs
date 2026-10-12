use crate::app::{AppState, TitleButton, WidgetId};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Shared button fill; focus changes brightness and border.
const BTN_BG: Color = Color::Rgb(40, 90, 180);
const BTN_BG_FOCUSED: Color = Color::Rgb(70, 140, 255);
const BTN_BORDER: Color = Color::Rgb(80, 120, 180);
const BTN_BORDER_FOCUSED: Color = Color::White;

/// Big FIGlet-style banner for the title screen (`BALATRO - RS`, one baseline).
const TITLE_ART: &[&str] = &[
    r" ____          _            _______ _____   ____             _____   _____ ",
    r"|  _ \   /\   | |        /\|__   __|  __ \ / __ \           |  __ \ / ____|",
    r"| |_) | /  \  | |       /  \  | |  | |__) | |  | |  ______  | |__) | (___  ",
    r"|  _ < / /\ \ | |      / /\ \ | |  |  _  /| |  | | |______| |  _  / \___ \ ",
    r"| |_) / ____ \| |____ / ____ \| |  | | \ \| |__| |          | | \ \ ____) |",
    r"|____/_/    \_\______/_/    \_\_|  |_|  \_\\____/           |_|  \_\_____/ ",
];

pub fn render(f: &mut Frame, app: &mut AppState, area: Rect) {
    f.render_widget(Clear, area);

    let art_h = TITLE_ART.len() as u16;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(18),
            Constraint::Length(art_h),
            Constraint::Min(2),
            Constraint::Length(7),
            Constraint::Length(2),
        ])
        .split(area);

    let style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let lines: Vec<Line> = TITLE_ART
        .iter()
        .map(|row| Line::from(Span::styled(*row, style)))
        .collect();
    f.render_widget(
        Paragraph::new(lines).alignment(Alignment::Center),
        chunks[1],
    );

    render_buttons(f, app, chunks[3]);

    let record = if app.record_runs { "on" } else { "off" };
    let hint = Paragraph::new(vec![
        Line::from(format!(
            "←/→ move   Enter select   r record runs: {record}   q quit"
        )),
        Line::from(app.title_status.as_str()),
    ])
    .alignment(Alignment::Center)
    .style(Style::default().fg(Color::DarkGray));
    f.render_widget(hint, chunks[4]);
}

fn render_buttons(f: &mut Frame, app: &mut AppState, area: Rect) {
    let panel = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .style(Style::default().bg(Color::Rgb(24, 24, 30)));
    let inner = panel.inner(area);
    f.render_widget(panel, area);

    // Two equal columns with padding around the buttons.
    let row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(1),
            Constraint::Ratio(1, 2),
            Constraint::Length(1),
            Constraint::Ratio(1, 2),
            Constraint::Length(1),
        ])
        .split(inner);

    let button_areas = [row[1], row[3]];
    let buttons = [TitleButton::Play, TitleButton::Library];

    for (button, btn_area) in buttons.into_iter().zip(button_areas) {
        let focused = app.title_focus == button;
        let (bg, border) = if focused {
            (BTN_BG_FOCUSED, BTN_BORDER_FOCUSED)
        } else {
            (BTN_BG, BTN_BORDER)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border).add_modifier(if focused {
                Modifier::BOLD
            } else {
                Modifier::empty()
            }))
            .style(Style::default().bg(bg));
        let label_area = block.inner(btn_area);
        f.render_widget(block, btn_area);

        let label = if focused {
            format!("► {} ◄", button.label())
        } else {
            button.label().to_string()
        };

        app.widget_rects
            .insert(WidgetId::TitleButton(button.index()), btn_area);

        // Vertically center the label in the button body.
        let label_row = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Length(1),
                Constraint::Percentage(50),
            ])
            .split(label_area);

        f.render_widget(
            Paragraph::new(label).alignment(Alignment::Center).style(
                Style::default()
                    .fg(Color::White)
                    .bg(bg)
                    .add_modifier(Modifier::BOLD),
            ),
            label_row[1],
        );
    }
}
