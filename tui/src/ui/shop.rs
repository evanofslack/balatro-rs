use crate::app::{AppState, FocusZone, WidgetId};
use crate::ui::{joker_strip, sidebar};
use balatro_rs::joker::Joker;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

const ITEM_W: u16 = 24;
const ITEM_H: u16 = 10;

pub fn render(f: &mut Frame, app: &mut AppState, area: Rect) {
    let (sidebar_area, main_area) = sidebar::split_sidebar_main(area);
    sidebar::render(f, app, sidebar_area);
    render_main(f, app, main_area);
}

fn render_main(f: &mut Frame, app: &mut AppState, area: Rect) {
    let block = Block::default()
        .title(Span::styled(
            " SHOP — Improve your run! ",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(joker_strip::STRIP_H),
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(inner);

    joker_strip::render(f, app, chunks[0]);
    render_for_sale(f, app, chunks[1]);
    render_next_round(f, app, chunks[2]);
    render_hints(f, chunks[3]);
}

fn render_for_sale(f: &mut Frame, app: &mut AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(30)])
        .split(area);

    render_jokers_for_sale(f, app, chunks[0]);
    render_consumables_for_sale(f, app, chunks[1]);
}

fn render_jokers_for_sale(f: &mut Frame, app: &mut AppState, area: Rect) {
    let label = Paragraph::new(Span::styled(
        "Jokers for Sale",
        Style::default().fg(Color::DarkGray),
    ));
    let label_area = Rect {
        x: area.x + 1,
        y: area.y,
        width: 20,
        height: 1,
    };
    f.render_widget(label, label_area);

    let jokers = app.game.shop.jokers.clone();
    let focused = app.focus == FocusZone::ShopJokers;

    for (i, joker) in jokers.iter().enumerate() {
        let x = area.x + 1 + i as u16 * (ITEM_W + 2);
        if x + ITEM_W > area.x + area.width {
            break;
        }
        let item_rect = Rect {
            x,
            y: area.y + 1,
            width: ITEM_W,
            height: ITEM_H,
        };

        let is_cursor = focused && app.cursor == i;
        let can_afford = app.game.money >= joker.cost();

        let border_type = if is_cursor {
            BorderType::Double
        } else {
            BorderType::Plain
        };
        let border_color = if is_cursor {
            Color::Yellow
        } else if can_afford {
            Color::Magenta
        } else {
            Color::DarkGray
        };

        let block = Block::default()
            .title(Span::styled(
                format!(" {} ", joker.name()),
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(border_color));

        let desc = joker.desc();
        let mut lines = vec![Line::from("")];
        for word_line in wrap(&desc, ITEM_W as usize - 2) {
            lines.push(Line::from(Span::styled(
                word_line,
                Style::default().fg(Color::White),
            )));
        }
        while lines.len() < ITEM_H as usize - 3 {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(vec![
            Span::styled(
                joker.rarity().to_string(),
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw("  "),
            Span::styled(
                format!("${}", joker.cost()),
                Style::default()
                    .fg(if can_afford {
                        Color::Yellow
                    } else {
                        Color::DarkGray
                    })
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        let para = Paragraph::new(Text::from(lines)).block(block);
        f.render_widget(para, item_rect);
        app.widget_rects.insert(WidgetId::ShopJoker(i), item_rect);
    }
}

fn render_consumables_for_sale(f: &mut Frame, app: &mut AppState, area: Rect) {
    let label = Paragraph::new(Span::styled(
        "Consumables",
        Style::default().fg(Color::DarkGray),
    ));
    let label_area = Rect {
        x: area.x + 1,
        y: area.y,
        width: 15,
        height: 1,
    };
    f.render_widget(label, label_area);

    let consumables = app.game.shop.consumables.clone();
    let focused = app.focus == FocusZone::ShopConsumables;

    for (i, c) in consumables.iter().enumerate() {
        let x = area.x + 1;
        let y = area.y + 1 + i as u16 * (ITEM_H / 2 + 1);
        if y + ITEM_H / 2 > area.y + area.height {
            break;
        }
        let item_rect = Rect {
            x,
            y,
            width: area.width.saturating_sub(2),
            height: ITEM_H / 2,
        };

        let is_cursor = focused && app.cursor == i;
        let can_afford = app.game.money >= c.cost();

        let border_type = if is_cursor {
            BorderType::Double
        } else {
            BorderType::Plain
        };
        let border_color = if is_cursor {
            Color::Yellow
        } else if can_afford {
            Color::Cyan
        } else {
            Color::DarkGray
        };

        let block = Block::default()
            .title(Span::styled(
                format!(" {} ", c.name()),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(border_color));

        let lines = vec![Line::from(vec![
            Span::styled(
                c.type_label().to_string(),
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw("  "),
            Span::styled(
                format!("${}", c.cost()),
                Style::default()
                    .fg(if can_afford {
                        Color::Yellow
                    } else {
                        Color::DarkGray
                    })
                    .add_modifier(Modifier::BOLD),
            ),
        ])];
        let para = Paragraph::new(Text::from(lines)).block(block);
        f.render_widget(para, item_rect);
        app.widget_rects
            .insert(WidgetId::ShopConsumable(i), item_rect);
    }
}

fn render_next_round(f: &mut Frame, app: &mut AppState, area: Rect) {
    let btn_w: u16 = 20;
    let btn_rect = Rect {
        x: area.x + area.width.saturating_sub(btn_w) / 2,
        y: area.y,
        width: btn_w,
        height: 3,
    };
    let focused = app.focus == FocusZone::ShopNextRound;
    let border_type = if focused {
        BorderType::Double
    } else {
        BorderType::Plain
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(Style::default().fg(if focused { Color::Yellow } else { Color::Green }));
    let para = Paragraph::new(Line::from(Span::styled(
        "Next Round →",
        Style::default()
            .fg(if focused { Color::Yellow } else { Color::Green })
            .add_modifier(Modifier::BOLD),
    )))
    .block(block)
    .alignment(Alignment::Center);
    f.render_widget(para, btn_rect);
    app.widget_rects.insert(WidgetId::NextRoundButton, btn_rect);
}

fn render_hints(f: &mut Frame, area: Rect) {
    let hints = "? controls";
    let para = Paragraph::new(Span::styled(hints, Style::default().fg(Color::DarkGray)))
        .alignment(Alignment::Center);
    f.render_widget(para, area);
}

fn wrap(s: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in s.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.len() + 1 + word.len() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current.clone());
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}
