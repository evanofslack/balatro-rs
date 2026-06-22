use crate::app::{AppState, FocusZone, WidgetId};
use crate::ui::{cards, joker_strip, sidebar};
use balatro_rs::stage::Stage;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
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

    let tarot = match &app.game.stage {
        Stage::TarotHand(t) => t.clone(),
        _ => return,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(joker_strip::STRIP_H),
            Constraint::Length(cards::SLOT_H + 1),
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(0),
        ])
        .split(inner);

    joker_strip::render(f, app, chunks[0]);
    render_cards(f, app, chunks[1]);
    render_apply_button(f, app, chunks[2]);
    render_tarot_info(f, &tarot, app, chunks[3]);

    // Key hints
    let hints = Paragraph::new(Span::styled(
        "? controls",
        Style::default().fg(Color::DarkGray),
    ))
    .alignment(Alignment::Center);
    if chunks.len() > 4 {
        let hint_area = Rect {
            x: chunks[4].x,
            y: chunks[4].y,
            width: chunks[4].width,
            height: 1,
        };
        f.render_widget(hints, hint_area);
    }
}

fn render_cards(f: &mut Frame, app: &mut AppState, area: Rect) {
    let all_cards = app.game.available.cards();
    let selected_ids: std::collections::HashSet<usize> = app
        .game
        .available
        .selected()
        .iter()
        .map(|c| c.id)
        .collect();
    let cards_and_selected: Vec<(balatro_rs::card::Card, bool)> = all_cards
        .iter()
        .map(|c| (*c, selected_ids.contains(&c.id)))
        .collect();

    let cursor = if matches!(app.focus, FocusZone::TarotCards) {
        app.cursor
    } else {
        usize::MAX
    };

    let total_cards = cards_and_selected.len();
    let cards_width = total_cards as u16 * cards::SLOT_W;
    let x_offset = area.x + area.width.saturating_sub(cards_width) / 2;

    let card_area = Rect {
        x: x_offset,
        y: area.y,
        width: cards_width.min(area.width),
        height: area.height,
    };

    cards::render_hand(f, app, card_area, &cards_and_selected, cursor, WidgetId::Card);
}

fn render_apply_button(f: &mut Frame, app: &mut AppState, area: Rect) {
    let selected_count = app.game.available.selected().len();
    let needed = match &app.game.stage {
        Stage::TarotHand(t) => t.min_targets(),
        _ => 0,
    };
    let ready = selected_count >= needed;

    let btn_w: u16 = 16;
    let x_start = area.x + area.width.saturating_sub(btn_w) / 2;
    let focused = matches!(app.focus, FocusZone::TarotButtons);

    let btn_rect = Rect {
        x: x_start,
        y: area.y,
        width: btn_w,
        height: 3,
    };
    let border_type = if focused { BorderType::Double } else { BorderType::Plain };
    let color = if ready { Color::Green } else { Color::DarkGray };
    let border_color = if focused { Color::Yellow } else { color };
    let text_style = if focused {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(color)
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(Style::default().fg(border_color));
    let para = Paragraph::new(Line::from(Span::styled("Apply Tarot", text_style)))
        .block(block)
        .alignment(Alignment::Center);
    f.render_widget(para, btn_rect);
    app.widget_rects.insert(WidgetId::TarotButton(0), btn_rect);
}

fn render_tarot_info(f: &mut Frame, tarot: &balatro_rs::tarot::Tarot, app: &AppState, area: Rect) {
    let selected_count = app.game.available.selected().len();
    let needed = tarot.min_targets();

    let lines = vec![
        Line::from(vec![
            Span::styled("  Tarot: ", Style::default().fg(Color::DarkGray)),
            Span::styled(tarot.name(), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(tarot.description(), Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            format!(
                "  Select {} card(s) ({} selected)",
                needed, selected_count
            ),
            Style::default().fg(if selected_count >= needed { Color::Green } else { Color::Yellow }),
        )),
    ];

    let para = Paragraph::new(lines);
    f.render_widget(para, area);
}
