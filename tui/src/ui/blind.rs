use crate::app::{AppState, FocusZone, WidgetId};
use crate::ui::{cards, joker_strip, sidebar};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
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

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(joker_strip::STRIP_H),
            Constraint::Length(cards::SLOT_H + 1),
            Constraint::Length(5),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .split(inner);

    joker_strip::render(f, app, chunks[0]);
    render_cards(f, app, chunks[1]);
    render_buttons(f, app, chunks[2]);
    render_hints(f, chunks[3]);
    render_deck_count(f, app, chunks[2]);
}

fn render_cards(f: &mut Frame, app: &mut AppState, area: Rect) {
    let all_cards = app.game.available.cards();
    let selected_ids: std::collections::HashSet<usize> =
        app.game.available.selected().iter().map(|c| c.id).collect();
    let cards_and_selected: Vec<(balatro_rs::card::Card, bool)> = all_cards
        .iter()
        .map(|c| (*c, selected_ids.contains(&c.id)))
        .collect();
    let cursor = if app.focus == FocusZone::Cards {
        app.cursor
    } else {
        usize::MAX
    };

    // Center the cards in the available area
    let total_cards = cards_and_selected.len();
    let cards_width = total_cards as u16 * cards::SLOT_W;
    let x_offset = area.x + area.width.saturating_sub(cards_width) / 2;

    let card_area = Rect {
        x: x_offset,
        y: area.y,
        width: cards_width.min(area.width),
        height: area.height,
    };

    cards::render_hand(
        f,
        app,
        card_area,
        &cards_and_selected,
        cursor,
        WidgetId::Card,
    );
}

fn render_buttons(f: &mut Frame, app: &mut AppState, area: Rect) {
    let buttons = [
        ("Play Hand", FocusZone::ActionButtons, 0),
        ("Sort Hand", FocusZone::ActionButtons, 1),
        ("Discard", FocusZone::ActionButtons, 2),
    ];

    let btn_width: u16 = 16;
    let btn_height: u16 = 3;
    let total_width = buttons.len() as u16 * (btn_width + 2);
    let x_start = area.x + area.width.saturating_sub(total_width) / 2;

    let focused_btns = app.focus == FocusZone::ActionButtons;

    for (i, (label, _, btn_idx)) in buttons.iter().enumerate() {
        let x = x_start + i as u16 * (btn_width + 2);
        let btn_rect = Rect {
            x,
            y: area.y + area.height.saturating_sub(btn_height) / 2,
            width: btn_width,
            height: btn_height,
        };

        let is_sort = *btn_idx == 1;
        let is_focused = focused_btns && app.cursor == *btn_idx;

        let style = if is_sort {
            Style::default().fg(Color::DarkGray)
        } else if is_focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let border_type = if is_focused {
            BorderType::Double
        } else {
            BorderType::Plain
        };
        let border_style = if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(border_style);

        let para = Paragraph::new(Line::from(Span::styled(label.to_string(), style)))
            .block(block)
            .alignment(Alignment::Center);

        f.render_widget(para, btn_rect);
        app.widget_rects
            .insert(WidgetId::ActionButton(*btn_idx), btn_rect);
    }
}

fn render_deck_count(f: &mut Frame, app: &AppState, area: Rect) {
    let deck_count = app.game.deck.cards().len();
    let total = deck_count + app.game.available.cards().len() + app.game.discarded.len();
    let label = format!("{}/{}", deck_count, total);
    let para = Paragraph::new(Span::styled(label, Style::default().fg(Color::DarkGray)))
        .alignment(Alignment::Right);
    let count_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 1,
    };
    f.render_widget(para, count_area);
}

fn render_hints(f: &mut Frame, area: Rect) {
    let hints = "? controls";
    let para = Paragraph::new(Span::styled(hints, Style::default().fg(Color::DarkGray)))
        .alignment(Alignment::Center);
    f.render_widget(para, area);
}
