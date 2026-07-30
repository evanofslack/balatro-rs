use crate::app::{AppState, FocusZone, WidgetId};
use crate::ui::cards::{rank_str, suit_char, suit_color, CARD_H, CARD_W, SLOT_W};
use crate::ui::{joker_strip, sidebar};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
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
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(inner);

    joker_strip::render(f, app, chunks[0]);
    render_for_sale(f, app, chunks[1]);
    render_reroll(f, app, chunks[2]);
    render_next_round(f, app, chunks[3]);
    render_hints(f, chunks[4]);
}

fn render_for_sale(f: &mut Frame, app: &mut AppState, area: Rect) {
    let row_h = CARD_H + 1;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(row_h),
            Constraint::Length(row_h),
            Constraint::Min(0),
        ])
        .split(area);

    render_jokers_for_sale(f, app, chunks[0]);
    render_packs_for_sale(f, app, chunks[1]);
}

fn render_jokers_for_sale(f: &mut Frame, app: &mut AppState, area: Rect) {
    let label = Paragraph::new(Span::styled(
        "Cards for Sale",
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
    let consumables = app.game.shop.consumables.clone();
    let cards = app.game.shop.cards.clone();
    let focused = app.focus == FocusZone::ShopJokers;
    let inner_w = (CARD_W as usize).saturating_sub(2);

    for (i, joker) in jokers.iter().enumerate() {
        let x = area.x + 1 + i as u16 * SLOT_W;
        if x + CARD_W > area.x + area.width {
            break;
        }
        let item_rect = Rect {
            x,
            y: area.y + 1,
            width: CARD_W,
            height: CARD_H,
        };

        let is_cursor = focused && app.cursor == i;
        let price = app.game.price(joker.cost());
        let can_afford = app.game.money >= price;

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
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(border_color));

        let name = joker.name().to_string();
        let (line1, line2) = wrap_name(&name, inner_w);
        let mut lines = vec![
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
        while lines.len() < (CARD_H as usize).saturating_sub(2) {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(vec![
            Span::styled(
                joker.rarity().to_string(),
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw(" "),
            Span::styled(
                format!("${}", price),
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

    let joker_count = jokers.len();
    for (ci, consumable) in consumables.iter().enumerate() {
        let slot = joker_count + ci;
        let x = area.x + 1 + slot as u16 * SLOT_W;
        if x + CARD_W > area.x + area.width {
            break;
        }
        let item_rect = Rect {
            x,
            y: area.y + 1,
            width: CARD_W,
            height: CARD_H,
        };

        let is_cursor = focused && app.cursor == slot;
        let price = app.game.price(consumable.cost());
        let can_afford = app.game.money >= price;
        let fg = consumable_color(consumable);

        let border_type = if is_cursor {
            BorderType::Double
        } else {
            BorderType::Plain
        };
        let border_color = if is_cursor {
            Color::Yellow
        } else if can_afford {
            fg
        } else {
            Color::DarkGray
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(border_color));

        let name = consumable.name().to_string();
        let (line1, line2) = wrap_name(&name, inner_w);
        let mut lines = vec![
            Line::from(Span::styled(
                line1,
                Style::default().fg(fg).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                line2,
                Style::default().fg(fg).add_modifier(Modifier::BOLD),
            )),
        ];
        while lines.len() < (CARD_H as usize).saturating_sub(2) {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(Span::styled(
            format!("${}", price),
            Style::default()
                .fg(if can_afford {
                    Color::Yellow
                } else {
                    Color::DarkGray
                })
                .add_modifier(Modifier::BOLD),
        )));

        let para = Paragraph::new(Text::from(lines)).block(block);
        f.render_widget(para, item_rect);
        app.widget_rects
            .insert(WidgetId::ShopConsumable(ci), item_rect);
    }

    // Playing cards only ever appear here with the Magic Trick voucher.
    let before_cards = joker_count + consumables.len();
    for (di, card) in cards.iter().enumerate() {
        let slot = before_cards + di;
        let x = area.x + 1 + slot as u16 * SLOT_W;
        if x + CARD_W > area.x + area.width {
            break;
        }
        let item_rect = Rect {
            x,
            y: area.y + 1,
            width: CARD_W,
            height: CARD_H,
        };

        let is_cursor = focused && app.cursor == slot;
        let price = app.game.price(card.shop_cost());
        let can_afford = app.game.money >= price;
        let fg = suit_color(card.suit);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(if is_cursor {
                BorderType::Double
            } else {
                BorderType::Plain
            })
            .border_style(Style::default().fg(if is_cursor {
                Color::Yellow
            } else if can_afford {
                fg
            } else {
                Color::DarkGray
            }));

        let mut lines = vec![
            Line::from(Span::styled(
                format!("{}{}", rank_str(card.value), suit_char(card.suit)),
                Style::default().fg(fg).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                card.enhancement
                    .map(|e| format!("{:?}", e))
                    .unwrap_or_default(),
                Style::default().fg(Color::Cyan),
            )),
        ];
        while lines.len() < (CARD_H as usize).saturating_sub(2) {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(Span::styled(
            format!("${}", price),
            Style::default()
                .fg(if can_afford {
                    Color::Yellow
                } else {
                    Color::DarkGray
                })
                .add_modifier(Modifier::BOLD),
        )));

        let para = Paragraph::new(Text::from(lines)).block(block);
        f.render_widget(para, item_rect);
        app.widget_rects.insert(WidgetId::ShopCard(di), item_rect);
    }
}

fn render_packs_for_sale(f: &mut Frame, app: &mut AppState, area: Rect) {
    let label = Paragraph::new(Span::styled(
        "Booster Packs",
        Style::default().fg(Color::DarkGray),
    ));
    let label_area = Rect {
        x: area.x + 1,
        y: area.y,
        width: 15,
        height: 1,
    };
    f.render_widget(label, label_area);

    let packs = app.game.shop.packs.clone();
    let focused = app.focus == FocusZone::ShopPacks;
    let inner_w = (CARD_W as usize).saturating_sub(2);

    for (i, pack) in packs.iter().enumerate() {
        let x = area.x + 1 + i as u16 * SLOT_W;
        if x + CARD_W > area.x + area.width {
            break;
        }
        let item_rect = Rect {
            x,
            y: area.y + 1,
            width: CARD_W,
            height: CARD_H,
        };

        let is_cursor = focused && app.cursor == i;
        let price = app.game.price(pack.cost());
        let can_afford = app.game.money >= price;
        let category_color = pack_category_color(&pack.category);

        let border_type = if is_cursor {
            BorderType::Double
        } else {
            BorderType::Plain
        };
        let border_color = if is_cursor {
            Color::Yellow
        } else if can_afford {
            category_color
        } else {
            Color::DarkGray
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(border_color));

        let name = pack.name();
        let (line1, line2) = wrap_name(&name, inner_w);
        let mut lines = vec![
            Line::from(Span::styled(
                line1,
                Style::default()
                    .fg(category_color)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                line2,
                Style::default()
                    .fg(category_color)
                    .add_modifier(Modifier::BOLD),
            )),
        ];
        while lines.len() < (CARD_H as usize).saturating_sub(2) {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(Span::styled(
            format!("${}", price),
            Style::default()
                .fg(if can_afford {
                    Color::Yellow
                } else {
                    Color::DarkGray
                })
                .add_modifier(Modifier::BOLD),
        )));

        let para = Paragraph::new(Text::from(lines)).block(block);
        f.render_widget(para, item_rect);
        app.widget_rects.insert(WidgetId::ShopPack(i), item_rect);
    }

    render_voucher_for_sale(f, app, area, packs.len());
}

/// The ante's voucher offer, sharing the booster-pack row. It sits one
/// slot past the packs and is selected as the last cursor position of the
/// ShopPacks zone.
fn render_voucher_for_sale(f: &mut Frame, app: &mut AppState, area: Rect, slot: usize) {
    let Some(voucher) = app.game.shop.voucher else {
        return;
    };
    let x = area.x + 2 + (slot + 1) as u16 * SLOT_W;
    if x + CARD_W > area.x + area.width {
        return;
    }

    let label = Paragraph::new(Span::styled(
        "Voucher",
        Style::default().fg(Color::DarkGray),
    ));
    f.render_widget(
        label,
        Rect {
            x,
            y: area.y,
            width: 10,
            height: 1,
        },
    );

    let item_rect = Rect {
        x,
        y: area.y + 1,
        width: CARD_W,
        height: CARD_H,
    };
    let is_cursor = app.focus == FocusZone::ShopPacks && app.cursor == slot;
    // Vouchers are exempt from Clearance Sale / Liquidation.
    let can_afford = app.game.money >= voucher.cost();
    let inner_w = (CARD_W as usize).saturating_sub(2);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(if is_cursor {
            BorderType::Double
        } else {
            BorderType::Plain
        })
        .border_style(Style::default().fg(if is_cursor {
            Color::Yellow
        } else if can_afford {
            Color::LightGreen
        } else {
            Color::DarkGray
        }));

    let (line1, line2) = wrap_name(voucher.name(), inner_w);
    let mut lines = vec![
        Line::from(Span::styled(
            line1,
            Style::default()
                .fg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            line2,
            Style::default()
                .fg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )),
    ];
    while lines.len() < (CARD_H as usize).saturating_sub(2) {
        lines.push(Line::from(""));
    }
    lines.push(Line::from(Span::styled(
        format!("${}", voucher.cost()),
        Style::default()
            .fg(if can_afford {
                Color::Yellow
            } else {
                Color::DarkGray
            })
            .add_modifier(Modifier::BOLD),
    )));

    let para = Paragraph::new(Text::from(lines)).block(block);
    f.render_widget(para, item_rect);
    app.widget_rects.insert(WidgetId::ShopVoucher, item_rect);
}

fn consumable_color(consumable: &balatro_rs::consumable::Consumable) -> Color {
    use balatro_rs::consumable::Consumable;
    match consumable {
        Consumable::Tarot(_) => Color::Cyan,
        Consumable::Planet(_) => Color::Blue,
        Consumable::Spectral(_) => Color::Magenta,
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

fn wrap_name(name: &str, max_w: usize) -> (String, String) {
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
