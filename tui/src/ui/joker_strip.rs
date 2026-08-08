use crate::app::{AppState, FocusZone, WidgetId};
use crate::ui::cards::{self, CARD_H, CARD_W, SLOT_W};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
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

fn render_jokers(f: &mut Frame, app: &mut AppState, area: Rect) {
    let jokers = &app.game.jokers;
    let slots = app.game.config.joker_slots;
    let focused = app.focus == FocusZone::JokerStrip;
    let inner_w = (CARD_W as usize).saturating_sub(2);

    let mut x = area.x + 1;

    // Negative-edition jokers each add +1 slot, but can also just push the
    // owned count past the base cap outright — either way, keep drawing
    // real joker boxes past the 5th rather than stopping at `slots` and
    // hiding them. The "x/x" label (below) can then read e.g. "7/5".
    for i in 0..jokers.len().max(slots) {
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
            let name = joker.name();
            let (line1, line2) = cards::wrap_two_lines(&name, inner_w);
            let text_style = Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD);
            let lines = vec![
                Line::from(Span::styled(line1, text_style)),
                Line::from(Span::styled(line2, text_style)),
            ];
            cards::render_item_box(f, slot_rect, is_cursor, Color::Magenta, None, lines, None);
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
            let color = super::consumable_type_color(c);
            let name = c.name();
            let (line1, line2) = cards::wrap_two_lines(&name, inner_w);
            let text_style = Style::default().fg(color).add_modifier(Modifier::BOLD);
            let lines = vec![
                Line::from(Span::styled(line1, text_style)),
                Line::from(Span::styled(line2, text_style)),
            ];
            let footer = Line::from(Span::styled(
                c.type_label().to_string(),
                Style::default().fg(Color::DarkGray),
            ));
            cards::render_item_box(f, slot_rect, is_cursor, color, None, lines, Some(footer));
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
