use crate::app::{AppState, FocusZone, WidgetId};
use balatro_rs::joker::Joker;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub const STRIP_H: u16 = 5;
const JOKER_W: u16 = 16;
const CONSUMABLE_W: u16 = 12;

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

    // Split inner into joker section and consumable section
    let consumable_total = app.game.config.consumable_slots as u16 * (CONSUMABLE_W + 1) + 6;
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(consumable_total),
        ])
        .split(inner);

    render_jokers(f, app, chunks[0]);
    render_consumables(f, app, chunks[1]);
}

fn render_jokers(f: &mut Frame, app: &mut AppState, area: Rect) {
    let jokers = &app.game.jokers;
    let slots = app.game.config.joker_slots;
    let focused = app.focus == FocusZone::JokerStrip;

    let mut x = area.x + 1;

    for i in 0..slots {
        if x + JOKER_W > area.x + area.width {
            break;
        }
        let slot_rect = Rect {
            x,
            y: area.y,
            width: JOKER_W,
            height: STRIP_H - 1,
        };

        if let Some(joker) = jokers.get(i) {
            let is_cursor = focused && app.cursor == i;
            let border_type = if is_cursor { BorderType::Double } else { BorderType::Plain };
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
            let truncated = if name.len() > (JOKER_W as usize).saturating_sub(2) {
                format!("{}…", &name[..(JOKER_W as usize).saturating_sub(3)])
            } else {
                name.clone()
            };
            let lines = vec![
                Line::from(Span::styled(truncated, Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))),
                Line::from(Span::styled(joker.rarity().to_string(), Style::default().fg(Color::DarkGray))),
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

        x += JOKER_W + 1;
    }

    // Slot count label
    let label = format!("{}/{}", jokers.len(), slots);
    let label_rect = Rect {
        x: area.x + area.width.saturating_sub(label.len() as u16 + 1),
        y: area.y,
        width: label.len() as u16 + 1,
        height: 1,
    };
    let para = Paragraph::new(Span::styled(label, Style::default().fg(Color::DarkGray)));
    f.render_widget(para, label_rect);
}

fn render_consumables(f: &mut Frame, app: &mut AppState, area: Rect) {
    let consumables = &app.game.consumables;
    let slots = app.game.config.consumable_slots;
    let focused = app.focus == FocusZone::ConsumableStrip;

    let mut x = area.x + 1;

    for i in 0..slots {
        if x + CONSUMABLE_W > area.x + area.width {
            break;
        }
        let slot_rect = Rect {
            x,
            y: area.y,
            width: CONSUMABLE_W,
            height: STRIP_H - 1,
        };

        if let Some(c) = consumables.get(i) {
            let is_cursor = focused && app.cursor == i;
            let border_type = if is_cursor { BorderType::Double } else { BorderType::Plain };
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
            let truncated = if name.len() > (CONSUMABLE_W as usize).saturating_sub(2) {
                format!("{}…", &name[..(CONSUMABLE_W as usize).saturating_sub(3)])
            } else {
                name
            };
            let lines = vec![
                Line::from(Span::styled(truncated, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
                Line::from(Span::styled(c.type_label().to_string(), Style::default().fg(Color::DarkGray))),
            ];
            let para = Paragraph::new(lines).block(block);
            f.render_widget(para, slot_rect);
            app.widget_rects.insert(WidgetId::ConsumableSlot(i), slot_rect);
        } else {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray));
            f.render_widget(block, slot_rect);
        }

        x += CONSUMABLE_W + 1;
    }

    // Count label
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
