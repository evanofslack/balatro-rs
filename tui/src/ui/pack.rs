use crate::app::{AppState, FocusZone, WidgetId};
use crate::ui::{cards, joker_strip, sidebar, wrap};
use balatro_rs::joker::Joker;
use balatro_rs::pack::PackContent;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

const CONTENT_W: u16 = 16;
const CONTENT_H: u16 = 6;

pub fn render(f: &mut Frame, app: &mut AppState, area: Rect) {
    let (sidebar_area, main_area) = sidebar::split_sidebar_main(area);
    sidebar::render(f, app, sidebar_area);
    render_main(f, app, main_area);
}

fn render_main(f: &mut Frame, app: &mut AppState, area: Rect) {
    let Some(state) = app.game.open_pack.clone() else {
        return;
    };

    let title = format!(" {} pick(s) remaining ", state.picks_remaining);

    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let is_arcana = !app.game.available.cards().is_empty();

    let constraints = if is_arcana {
        vec![
            Constraint::Length(joker_strip::STRIP_H),
            Constraint::Length(2),
            Constraint::Length(CONTENT_H + 1),
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Length(cards::SLOT_H + 1),
            Constraint::Min(0),
        ]
    } else {
        vec![
            Constraint::Length(joker_strip::STRIP_H),
            Constraint::Length(2),
            Constraint::Length(CONTENT_H + 1),
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Min(0),
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    joker_strip::render(f, app, chunks[0]);
    render_description(f, &state.description, chunks[1]);
    render_contents(f, app, &state.contents, chunks[2]);
    render_skip_button(f, app, chunks[3]);
    render_content_info(f, app, &state.contents, chunks[4]);

    if is_arcana && chunks.len() > 5 {
        render_drawn_hand(f, app, chunks[5]);
    }
}

fn render_description(f: &mut Frame, description: &str, area: Rect) {
    let para = Paragraph::new(Span::styled(
        description,
        Style::default().fg(Color::DarkGray),
    ))
    .alignment(Alignment::Center);
    f.render_widget(para, area);
}

fn render_contents(
    f: &mut Frame,
    app: &mut AppState,
    contents: &[PackContent],
    area: Rect,
) {
    let focused = app.focus == FocusZone::PackContents;
    let total_w = contents.len() as u16 * (CONTENT_W + 1);
    let x_offset = area.x + area.width.saturating_sub(total_w) / 2;

    for (i, content) in contents.iter().enumerate() {
        let x = x_offset + i as u16 * (CONTENT_W + 1);
        if x + CONTENT_W > area.x + area.width {
            break;
        }
        let item_rect = Rect {
            x,
            y: area.y,
            width: CONTENT_W,
            height: CONTENT_H,
        };

        let is_cursor = focused && app.cursor == i;
        let color = content_color(content);
        let border_type = if is_cursor {
            BorderType::Double
        } else {
            BorderType::Plain
        };
        let border_color = if is_cursor { Color::Yellow } else { color };

        let block = Block::default()
            .title(Span::styled(
                format!(" {} ", content.type_label()),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(border_color));

        let name = content.name();
        let lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                name,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )),
        ];
        let para = Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(para, item_rect);
        app.widget_rects.insert(WidgetId::PackContent(i), item_rect);
    }
}

fn render_skip_button(f: &mut Frame, app: &mut AppState, area: Rect) {
    let btn_w: u16 = 12;
    let btn_rect = Rect {
        x: area.x + area.width.saturating_sub(btn_w) / 2,
        y: area.y,
        width: btn_w,
        height: 3,
    };
    let focused = app.focus == FocusZone::PackSkip;
    let border_type = if focused {
        BorderType::Double
    } else {
        BorderType::Plain
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(Style::default().fg(if focused { Color::Yellow } else { Color::DarkGray }));
    let para = Paragraph::new(Line::from(Span::styled(
        "Skip",
        Style::default()
            .fg(if focused { Color::Yellow } else { Color::DarkGray })
            .add_modifier(Modifier::BOLD),
    )))
    .block(block)
    .alignment(Alignment::Center);
    f.render_widget(para, btn_rect);
    app.widget_rects.insert(WidgetId::SkipPackButton, btn_rect);
}

fn render_content_info(
    f: &mut Frame,
    app: &AppState,
    contents: &[PackContent],
    area: Rect,
) {
    if app.focus != FocusZone::PackContents {
        return;
    }
    let Some(content) = contents.get(app.cursor) else {
        return;
    };

    let (label, desc, color) = match content {
        PackContent::Tarot(t) => (
            t.name().to_string(),
            t.description().to_string(),
            Color::Magenta,
        ),
        PackContent::Planet(p) => (
            p.name(),
            p.desc(),
            Color::Blue,
        ),
        PackContent::Joker(j) => (
            j.name().to_string(),
            j.desc(),
            Color::Yellow,
        ),
        PackContent::PlayingCard(c) => (
            c.to_string(),
            String::new(),
            Color::White,
        ),
    };

    let width = area.width.saturating_sub(4) as usize;
    let mut lines = vec![Line::from(Span::styled(
        label,
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    ))];
    if !desc.is_empty() {
        for word_line in wrap(&desc, width) {
            lines.push(Line::from(Span::styled(
                word_line,
                Style::default().fg(Color::White),
            )));
        }
    }

    let para = Paragraph::new(lines).alignment(Alignment::Center);
    f.render_widget(para, area);
}

fn render_drawn_hand(f: &mut Frame, app: &mut AppState, area: Rect) {
    let all_cards = app.game.available.cards();
    let selected_ids: std::collections::HashSet<usize> =
        app.game.available.selected().iter().map(|c| c.id).collect();
    let cards_and_selected: Vec<(balatro_rs::card::Card, bool)> = all_cards
        .iter()
        .map(|c| (*c, selected_ids.contains(&c.id)))
        .collect();

    if cards_and_selected.is_empty() {
        return;
    }

    let label = Paragraph::new(Span::styled(
        "Hand (for tarot targeting):",
        Style::default().fg(Color::DarkGray),
    ));
    let label_area = Rect {
        x: area.x + 1,
        y: area.y,
        width: area.width.saturating_sub(2),
        height: 1,
    };
    f.render_widget(label, label_area);

    let cards_area = Rect {
        x: area.x,
        y: area.y + 1,
        width: area.width,
        height: area.height.saturating_sub(1),
    };

    let total_cards = cards_and_selected.len();
    let cards_width = total_cards as u16 * cards::SLOT_W;
    let x_offset = cards_area.x + cards_area.width.saturating_sub(cards_width) / 2;
    let card_area = Rect {
        x: x_offset,
        y: cards_area.y,
        width: cards_width.min(cards_area.width),
        height: cards_area.height,
    };

    cards::render_hand(f, app, card_area, &cards_and_selected, usize::MAX, WidgetId::Card);
}

fn content_color(content: &PackContent) -> Color {
    match content {
        PackContent::Tarot(_) => Color::Magenta,
        PackContent::Planet(_) => Color::Blue,
        PackContent::Joker(_) => Color::Yellow,
        PackContent::PlayingCard(_) => Color::White,
    }
}

