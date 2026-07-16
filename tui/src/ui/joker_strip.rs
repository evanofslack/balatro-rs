use crate::app::{AppState, FocusZone, WidgetId};
use crate::ui::cards::{CARD_H, CARD_W, SLOT_W};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

// STRIP_H = CARD_H + 1 separator line, so held cards match playing card height
pub const STRIP_H: u16 = CARD_H + 1;

pub fn render(f: &mut Frame, app: &mut AppState, area: Rect) {
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));
    f.render_widget(block, area);

    let inner = Rect {
        x: area.x + 1,
        y: area.y,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(1),
    };

    let consumable_total = app.game.config.consumable_slots as u16 * SLOT_W + 2;
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(consumable_total)])
        .split(inner);

    render_jokers(f, app, chunks[0]);
    render_consumables(f, app, chunks[1]);
}

fn wrap_joker_name(name: &str, max_w: usize) -> (String, String) {
    if name.len() <= max_w {
        return (name.to_string(), String::new());
    }
    let split = name[..max_w].rfind(' ').unwrap_or(max_w);
    let line1 = name[..split].to_string();
    let rest = name[split..].trim_start();
    let line2 = if rest.len() > max_w {
        format!("{}…", &rest[..max_w.saturating_sub(1)])
    } else {
        rest.to_string()
    };
    (line1, line2)
}

fn render_jokers(f: &mut Frame, app: &mut AppState, area: Rect) {
    let jokers = &app.game.jokers;
    let slots = app.game.config.joker_slots;
    let focused = app.focus == FocusZone::JokerStrip;
    let inner_w = (CARD_W as usize).saturating_sub(2);

    let mut x = area.x + 1;

    for i in 0..slots {
        if x + CARD_W > area.x + area.width {
            break;
        }
        let slot_rect = Rect {
            x,
            y: area.y,
            width: CARD_W,
            height: CARD_H,
        };

        if let Some(joker) = jokers.get(i) {
            let is_cursor = focused && app.cursor == i;
            let border_type = if is_cursor {
                BorderType::Double
            } else {
                BorderType::Plain
            };
            let border_style = if is_cursor {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Magenta)
            };
            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(border_type)
                .border_style(border_style);
            let name = joker.name();
            let (line1, line2) = wrap_joker_name(&name, inner_w);
            let lines = vec![
                Line::from(Span::styled(
                    line1,
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    line2,
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                )),
            ];
            let para = Paragraph::new(lines).block(block);
            f.render_widget(para, slot_rect);
            app.widget_rects.insert(WidgetId::JokerSlot(i), slot_rect);
        } else {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray));
            f.render_widget(block, slot_rect);
        }

        x += SLOT_W;
    }

    let label = format!("{}/{}", jokers.len(), slots);
    let label_x = area.x + 1 + jokers.len() as u16 * SLOT_W + 1;
    if label_x + label.len() as u16 <= area.x + area.width {
        let label_rect = Rect {
            x: label_x,
            y: area.y + CARD_H / 2,
            width: label.len() as u16,
            height: 1,
        };
        let para = Paragraph::new(Span::styled(label, Style::default().fg(Color::DarkGray)));
        f.render_widget(para, label_rect);
    }
}

fn render_consumables(f: &mut Frame, app: &mut AppState, area: Rect) {
    let consumables = &app.game.consumables;
    let slots = app.game.config.consumable_slots;
    let focused = app.focus == FocusZone::ConsumableStrip;
    let inner_w = (CARD_W as usize).saturating_sub(2);

    let mut x = area.x + 1;

    for i in 0..slots {
        if x + CARD_W > area.x + area.width {
            break;
        }
        let slot_rect = Rect {
            x,
            y: area.y,
            width: CARD_W,
            height: CARD_H,
        };

        if let Some(c) = consumables.get(i) {
            let is_cursor = focused && app.cursor == i;
            let border_type = if is_cursor {
                BorderType::Double
            } else {
                BorderType::Plain
            };
            let border_style = if is_cursor {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Cyan)
            };
            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(border_type)
                .border_style(border_style);
            let name = c.name();
            let truncated = if name.len() > inner_w {
                format!("{}…", &name[..inner_w.saturating_sub(1)])
            } else {
                name
            };
            let lines = vec![
                Line::from(Span::styled(
                    truncated,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    c.type_label().to_string(),
                    Style::default().fg(Color::DarkGray),
                )),
            ];
            let para = Paragraph::new(lines).block(block);
            f.render_widget(para, slot_rect);
            app.widget_rects
                .insert(WidgetId::ConsumableSlot(i), slot_rect);
        } else {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray));
            f.render_widget(block, slot_rect);
        }

        x += SLOT_W;
    }

    let label = format!("{}/{}", consumables.len(), slots);
    let label_rect = Rect {
        x: area.x + area.width.saturating_sub(label.len() as u16 + 1),
        y: area.y,
        width: label.len() as u16 + 1,
        height: 1,
    };
    let para = Paragraph::new(Span::styled(label, Style::default().fg(Color::DarkGray)));
    f.render_widget(para, label_rect);
}
