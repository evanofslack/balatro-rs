use crate::app::{AppState, FocusZone, WidgetId};
use crate::ui::cards::{self, CARD_H, CARD_W, SLOT_W};
use crate::ui::{joker_strip, sidebar};
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

    let row_h = CARD_H + 1;
    let for_sale_h = row_h * 2;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(joker_strip::STRIP_H),
            Constraint::Length(for_sale_h),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(inner);

    joker_strip::render(f, app, chunks[0]);
    render_for_sale(f, app, chunks[1]);
    render_reroll(f, app, chunks[3]);
    render_next_round(f, app, chunks[4]);
    render_hints(f, chunks[6]);
}

fn render_for_sale(f: &mut Frame, app: &mut AppState, area: Rect) {
    let row_h = CARD_H + 1;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(row_h), Constraint::Length(row_h)])
        .split(area);

    render_jokers_for_sale(f, app, chunks[0]);
    render_packs_for_sale(f, app, chunks[1]);
}

fn render_price_tag(f: &mut Frame, x: u16, y: u16, cost: usize, can_afford: bool) {
    let rect = Rect {
        x,
        y,
        width: CARD_W,
        height: 1,
    };
    let color = if can_afford { Color::Yellow } else { Color::DarkGray };
    let para = Paragraph::new(Span::styled(
        format!("${cost}"),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    ))
    .alignment(Alignment::Center);
    f.render_widget(para, rect);
}

fn render_jokers_for_sale(f: &mut Frame, app: &mut AppState, area: Rect) {
    let jokers = app.game.shop.jokers.clone();
    let consumables = app.game.shop.consumables.clone();
    let focused = app.focus == FocusZone::ShopJokers;
    let inner_w = (CARD_W as usize).saturating_sub(2);

    for (i, joker) in jokers.iter().enumerate() {
        let x = area.x + 1 + i as u16 * SLOT_W;
        if x + CARD_W > area.x + area.width {
            break;
        }
        let is_cursor = focused && app.cursor == i;
        let can_afford = app.game.money >= joker.cost();
        render_price_tag(f, x, area.y, joker.cost(), can_afford);

        let item_rect = Rect {
            x,
            y: area.y + 1,
            width: CARD_W,
            height: CARD_H,
        };

        let border_color = if can_afford { Color::Magenta } else { Color::DarkGray };
        let name = joker.name().to_string();
        let (line1, line2) = cards::wrap_two_lines(&name, inner_w);
        let text_style = Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD);
        let lines = vec![
            Line::from(Span::styled(line1, text_style)),
            Line::from(Span::styled(line2, text_style)),
        ];
        let footer = Line::from(Span::styled(
            joker.rarity().to_string(),
            Style::default().fg(Color::DarkGray),
        ));

        cards::render_item_box(f, item_rect, is_cursor, border_color, None, lines, Some(footer));
        app.widget_rects.insert(WidgetId::ShopJoker(i), item_rect);
    }

    let joker_count = jokers.len();
    for (ci, consumable) in consumables.iter().enumerate() {
        let slot = joker_count + ci;
        let x = area.x + 1 + slot as u16 * SLOT_W;
        if x + CARD_W > area.x + area.width {
            break;
        }
        let is_cursor = focused && app.cursor == slot;
        let can_afford = app.game.money >= consumable.cost();
        render_price_tag(f, x, area.y, consumable.cost(), can_afford);
        let fg = super::consumable_type_color(consumable);

        let item_rect = Rect {
            x,
            y: area.y + 1,
            width: CARD_W,
            height: CARD_H,
        };

        let border_color = if can_afford { fg } else { Color::DarkGray };
        let name = consumable.name().to_string();
        let (line1, line2) = cards::wrap_two_lines(&name, inner_w);
        let text_style = Style::default().fg(fg).add_modifier(Modifier::BOLD);
        let lines = vec![
            Line::from(Span::styled(line1, text_style)),
            Line::from(Span::styled(line2, text_style)),
        ];

        cards::render_item_box(f, item_rect, is_cursor, border_color, None, lines, None);
        app.widget_rects
            .insert(WidgetId::ShopConsumable(ci), item_rect);
    }
}

fn render_packs_for_sale(f: &mut Frame, app: &mut AppState, area: Rect) {
    let packs = app.game.shop.packs.clone();
    let focused = app.focus == FocusZone::ShopPacks;
    let inner_w = (CARD_W as usize).saturating_sub(2);

    for (i, pack) in packs.iter().enumerate() {
        let x = area.x + 1 + i as u16 * SLOT_W;
        if x + CARD_W > area.x + area.width {
            break;
        }
        let is_cursor = focused && app.cursor == i;
        let can_afford = app.game.money >= pack.cost();
        render_price_tag(f, x, area.y, pack.cost(), can_afford);
        let category_color = pack_category_color(&pack.category);

        let item_rect = Rect {
            x,
            y: area.y + 1,
            width: CARD_W,
            height: CARD_H,
        };

        let border_color = if can_afford { category_color } else { Color::DarkGray };
        let name = pack.name();
        let (line1, line2) = cards::wrap_two_lines(&name, inner_w);
        let text_style = Style::default()
            .fg(category_color)
            .add_modifier(Modifier::BOLD);
        let lines = vec![
            Line::from(Span::styled(line1, text_style)),
            Line::from(Span::styled(line2, text_style)),
        ];

        cards::render_item_box(f, item_rect, is_cursor, border_color, None, lines, None);
        app.widget_rects.insert(WidgetId::ShopPack(i), item_rect);
    }
}

fn render_reroll(f: &mut Frame, app: &mut AppState, area: Rect) {
    let cost = app.game.reroll_cost;
    let can_afford = app.game.money >= cost;
    let focused = app.focus == FocusZone::ShopReroll;

    let btn_w: u16 = 24;
    let btn_rect = Rect {
        x: area.x + area.width.saturating_sub(btn_w) / 2,
        y: area.y,
        width: btn_w,
        height: 3,
    };

    let border_type = if focused {
        BorderType::Double
    } else {
        BorderType::Plain
    };
    let color = if focused {
        Color::Yellow
    } else if can_afford {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(Style::default().fg(color));

    let label = format!("Reroll (${cost})");
    let para = Paragraph::new(Line::from(Span::styled(
        label,
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )))
    .block(block)
    .alignment(Alignment::Center);
    f.render_widget(para, btn_rect);
    app.widget_rects.insert(WidgetId::RerollButton, btn_rect);
}

fn pack_category_color(category: &balatro_rs::pack::PackCategory) -> Color {
    use balatro_rs::pack::PackCategory;
    match category {
        PackCategory::Arcana => Color::Cyan,
        PackCategory::Celestial => Color::Blue,
        PackCategory::Buffoon => Color::Magenta,
        PackCategory::Standard => Color::White,
        PackCategory::Spectral => Color::LightGreen,
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
