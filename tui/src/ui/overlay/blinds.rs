use crate::app::AppState;
use crate::ui::{blind_info, wrap};
use balatro_rs::stage::{blind_display, Blind, BlindExt, Stage};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

#[derive(PartialEq)]
enum Status {
    Current,
    Cleared,
    Upcoming,
}

fn status(app: &AppState, blind: Blind) -> Status {
    if matches!(app.game.stage, Stage::Blind(b) if b == blind) {
        return Status::Current;
    }
    if blind_info::is_cleared(&app.game, blind) {
        Status::Cleared
    } else {
        Status::Upcoming
    }
}

/// Read-only view of the ante's three blinds — same underlying data as the
/// interactive blind-select cards (`preblind.rs`), but no cursor/selection
/// since this is reachable from any stage via Run Info, not just PreBlind.
pub fn render_body(f: &mut Frame, app: &AppState, area: Rect) {
    let blinds = [Blind::Small, Blind::Big, Blind::Boss];
    let card_w: u16 = 22;
    let gap: u16 = 2;
    let total_w = blinds.len() as u16 * card_w + (blinds.len() as u16 - 1) * gap;
    let x_start = area.x + area.width.saturating_sub(total_w) / 2;
    let card_h = area.height.min(12);

    for (i, blind) in blinds.iter().copied().enumerate() {
        let x = x_start + i as u16 * (card_w + gap);
        let rect = Rect {
            x,
            y: area.y,
            width: card_w,
            height: card_h,
        };

        let state = status(app, blind);
        let base_color = match blind {
            Blind::Small => Color::Cyan,
            Blind::Big => Color::Yellow,
            Blind::Boss => Color::Red,
        };
        let (title_color, border_color, status_text, status_color) = match state {
            Status::Current => (base_color, Color::Yellow, "In Progress", Color::Yellow),
            Status::Cleared => (Color::DarkGray, Color::DarkGray, "Defeated", Color::DarkGray),
            Status::Upcoming => (base_color, base_color, "Upcoming", Color::DarkGray),
        };

        let title_text = match blind {
            Blind::Boss => app
                .game
                .current_boss
                .map(|boss| boss.name().to_string())
                .unwrap_or_else(|| blind_display(&blind).to_string()),
            _ => blind_display(&blind).to_string(),
        };
        let block = Block::default()
            .title(Span::styled(
                title_text,
                Style::default()
                    .fg(title_color)
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let mut lines = vec![
            Line::from(Span::styled(
                format!("  {status_text}"),
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];

        if blind == Blind::Boss && state != Status::Cleared {
            if let Some(boss) = app.game.current_boss {
                for word_line in wrap(boss.description(), (card_w as usize).saturating_sub(3)) {
                    lines.push(Line::from(Span::styled(
                        format!("  {word_line}"),
                        Style::default().fg(Color::White),
                    )));
                }
                lines.push(Line::from(""));
            }
        }

        let score_style = if state == Status::Cleared {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
                .fg(Color::LightBlue)
                .add_modifier(Modifier::BOLD)
        };
        lines.push(Line::from(vec![
            Span::styled("  Score: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", blind_info::required_score(&app.game, blind)),
                score_style,
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Reward: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("${}", blind.reward()),
                Style::default()
                    .fg(if state == Status::Cleared {
                        Color::DarkGray
                    } else {
                        Color::Yellow
                    })
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        let para = Paragraph::new(Text::from(lines)).block(block);
        f.render_widget(para, rect);
    }
}
