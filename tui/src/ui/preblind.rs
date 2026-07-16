use crate::app::{AppState, FocusZone, WidgetId};
use crate::ui::sidebar;
use balatro_rs::action::Action;
use balatro_rs::stage::{Blind, BlindExt, blind_display};

fn blind_state(game: &balatro_rs::game::Game, blind: &Blind) -> BlindState {
    let valid = game
        .gen_actions()
        .any(|a| matches!(a, Action::SelectBlind(b) if &b == blind));
    if valid {
        return BlindState::Available;
    }
    let cleared = match game.blind {
        Some(ref last) => blind <= last,
        None => false,
    };
    if cleared {
        BlindState::Cleared
    } else {
        BlindState::NotYet
    }
}

#[derive(PartialEq)]
enum BlindState {
    Available,
    Cleared,
    NotYet,
}
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &mut AppState, area: Rect) {
    let (sidebar_area, main_area) = sidebar::split_sidebar_main(area);
    sidebar::render(f, app, sidebar_area);
    render_main(f, app, main_area);
}

fn render_main(f: &mut Frame, app: &mut AppState, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Header
    let header_area = Rect {
        x: inner.x,
        y: inner.y + 2,
        width: inner.width,
        height: 2,
    };
    let header = Paragraph::new(Line::from(Span::styled(
        "Select your blind",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center);
    f.render_widget(header, header_area);

    // Three blind cards side by side
    let blinds = [Blind::Small, Blind::Big, Blind::Boss];
    let card_w: u16 = 30;
    let card_h: u16 = 12;
    let gap: u16 = 4;
    let total_w = blinds.len() as u16 * card_w + (blinds.len() as u16 - 1) * gap;
    let x_start = inner.x + inner.width.saturating_sub(total_w) / 2;
    let y_start = inner.y + (inner.height.saturating_sub(card_h)) / 2;

    let focused = app.focus == FocusZone::BlindSelect;

    for (i, blind) in blinds.iter().enumerate() {
        let x = x_start + i as u16 * (card_w + gap);
        let blind_rect = Rect {
            x,
            y: y_start,
            width: card_w,
            height: card_h,
        };

        let state = blind_state(&app.game, blind);
        let is_cursor = focused && app.cursor == i;

        let base_color = match blind {
            Blind::Small => Color::Cyan,
            Blind::Big => Color::Yellow,
            Blind::Boss => Color::Red,
        };
        let (title_color, border_color) = match state {
            BlindState::Available => (base_color, base_color),
            BlindState::Cleared | BlindState::NotYet => (Color::DarkGray, Color::DarkGray),
        };

        let border_type = if is_cursor && state == BlindState::Available {
            BorderType::Double
        } else {
            BorderType::Plain
        };
        let border_style = if is_cursor && state == BlindState::Available {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(border_color)
        };

        let block = Block::default()
            .title(Span::styled(
                blind_display(blind),
                Style::default()
                    .fg(title_color)
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(border_style);

        let mut lines = vec![Line::from("")];

        match state {
            BlindState::Available => {
                lines.push(Line::from(vec![
                    Span::styled("  Reward: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("${}", blind.reward()),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]));
                if is_cursor {
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled(
                        "  [ SELECT ]",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )));
                }
            }
            BlindState::Cleared => {
                lines.push(Line::from(Span::styled(
                    "  Cleared",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )));
            }
            BlindState::NotYet => {
                lines.push(Line::from(vec![
                    Span::styled("  Reward: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("${}", blind.reward()),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "  (not yet selectable)",
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }

        let para = Paragraph::new(Text::from(lines)).block(block);
        f.render_widget(para, blind_rect);

        app.widget_rects
            .insert(WidgetId::BlindOption(i), blind_rect);
    }

    // Key hint
    let hint_area = Rect {
        x: inner.x,
        y: inner.y + inner.height.saturating_sub(2),
        width: inner.width,
        height: 1,
    };
    let hints = Paragraph::new(Span::styled(
        "? controls",
        Style::default().fg(Color::DarkGray),
    ))
    .alignment(Alignment::Center);
    f.render_widget(hints, hint_area);
}
