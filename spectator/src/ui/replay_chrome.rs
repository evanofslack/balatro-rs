use crate::app::{AppState, TransportButton};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

const COMPACT_WIDTH: u16 = 40;

pub fn render_top_banner(f: &mut Frame, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    f.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta))
        .style(Style::default().bg(Color::Rgb(20, 20, 40)));
    let inner = block.inner(area);
    f.render_widget(block, area);
    let para = Paragraph::new(Line::from(Span::styled(
        "Model Replay",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center);
    f.render_widget(para, inner);
}

pub fn render_bottom_chrome(f: &mut Frame, app: &mut AppState, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let Some(chrome) = app.replay_chrome.as_mut() else {
        return;
    };
    chrome.button_rects.clear();

    f.render_widget(Clear, area);
    f.render_widget(
        Paragraph::new("").style(Style::default().bg(Color::Rgb(16, 16, 16))),
        area,
    );

    let speed_row_h = 1u16.min(area.height);
    let bar_row_h = area.height.saturating_sub(speed_row_h);
    let speed_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: speed_row_h,
    };
    let bar_area = Rect {
        x: area.x,
        y: area.y + speed_row_h,
        width: area.width,
        height: bar_row_h.max(1),
    };

    let speed_label = format_speed_label(chrome.rewinding, chrome.rewind_mult, chrome.forward_mult);
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            speed_label,
            Style::default().fg(Color::Cyan),
        )))
        .alignment(Alignment::Center),
        speed_area,
    );

    if bar_area.height == 0 || bar_area.width < 8 {
        return;
    }

    let compact = bar_area.width < COMPACT_WIDTH;
    let buttons: &[(&str, TransportButton)] = if compact {
        &[
            ("◀◀", TransportButton::RewindFast),
            ("❚❚", TransportButton::Pause),
            ("▶▶", TransportButton::ForwardFast),
            (" ✕ ", TransportButton::Exit),
        ]
    } else {
        &[
            ("◀◀", TransportButton::RewindFast),
            (" ◀ ", TransportButton::RewindStep),
            (
                if app.autoplay_paused { " ▶ " } else { "❚❚" },
                TransportButton::Pause,
            ),
            (" ▶ ", TransportButton::ForwardStep),
            ("▶▶", TransportButton::ForwardFast),
            (" ✕ ", TransportButton::Exit),
        ]
    };

    let btn_w = (bar_area.width / buttons.len().max(1) as u16).max(3);
    let mut x = bar_area.x;
    for (label, id) in buttons {
        let w = btn_w.min(bar_area.x + bar_area.width - x);
        if w == 0 {
            break;
        }
        let rect = Rect {
            x,
            y: bar_area.y,
            width: w,
            height: bar_area.height.max(1),
        };
        let style = if *id == TransportButton::Exit {
            Style::default()
                .fg(Color::White)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let borders = button_borders(rect.height);
        let block = Block::default()
            .borders(borders)
            .border_style(Style::default().fg(Color::DarkGray))
            .style(style);
        let inner_btn = block.inner(rect);
        f.render_widget(block, rect);
        if inner_btn.width > 0 && inner_btn.height > 0 {
            f.render_widget(
                Paragraph::new(Line::from(Span::styled(*label, style)))
                    .alignment(Alignment::Center),
                inner_btn,
            );
        } else {
            f.render_widget(
                Paragraph::new(Line::from(Span::styled(*label, style)))
                    .alignment(Alignment::Center),
                rect,
            );
        }
        chrome.button_rects.insert(*id, rect);
        x = x.saturating_add(w);
    }
}

fn format_speed_label(rewinding: bool, rewind_mult: u8, forward_mult: u8) -> String {
    if rewinding {
        match rewind_mult {
            2 => ".5".to_string(),
            4 => ".25".to_string(),
            _ => "1".to_string(),
        }
    } else {
        format!("{forward_mult}x")
    }
}

fn button_borders(height: u16) -> Borders {
    if height >= 3 {
        Borders::ALL
    } else if height >= 2 {
        Borders::LEFT | Borders::RIGHT
    } else {
        Borders::NONE
    }
}
