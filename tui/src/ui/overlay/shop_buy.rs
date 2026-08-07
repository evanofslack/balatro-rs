use crate::app::{AppState, ShopSlot, WidgetId};
use crate::ui::overlay::centered_rect;
use crate::ui::overlay::inspect::{consumable_lines, joker_lines, pack_lines};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &mut AppState, area: Rect, slot: ShopSlot) {
    let w: u16 = 44;
    let h: u16 = 18;
    let rect = centered_rect(w, h, area);
    f.render_widget(Clear, rect);

    let Some((title, cost, mut lines)) = item_info(app, slot, w) else {
        app.overlay = None;
        return;
    };

    let can_afford = app.game.money >= cost;
    let buy_selected = app.overlay_cursor == 0;
    let buy_style = if !can_afford {
        Style::default().fg(Color::DarkGray)
    } else if buy_selected {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let cancel_style = if !buy_selected {
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let buy_text = format!("  [ Buy (${}) ]", cost);
    let gap = "   ";
    let cancel_text = "[ Cancel ]";

    // Row within `lines` (0-indexed) that the buy/cancel line will occupy —
    // the description wraps to a variable number of lines depending on its
    // text, so this can't be a fixed offset from the bottom of the box.
    let buy_row = lines.len() as u16;
    lines.push(Line::from(vec![
        Span::styled(buy_text.clone(), buy_style),
        Span::raw(gap),
        Span::styled(cancel_text, cancel_style),
    ]));

    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    let para = Paragraph::new(Text::from(lines)).block(block);
    f.render_widget(para, rect);

    // +1 for the block's top border row.
    let y = rect.y + 1 + buy_row;
    let buy_x = rect.x + 1;
    let cancel_x = buy_x + buy_text.chars().count() as u16 + gap.chars().count() as u16;
    app.widget_rects.insert(
        WidgetId::OverlayButton(0),
        Rect {
            x: buy_x,
            y,
            width: buy_text.chars().count() as u16,
            height: 1,
        },
    );
    app.widget_rects.insert(
        WidgetId::OverlayButton(1),
        Rect {
            x: cancel_x,
            y,
            width: cancel_text.chars().count() as u16,
            height: 1,
        },
    );
}

fn item_info(app: &AppState, slot: ShopSlot, w: u16) -> Option<(String, usize, Vec<Line<'static>>)> {
    match slot {
        ShopSlot::Joker(idx) => {
            let joker = app.game.shop.jokers.get(idx)?;
            Some((
                format!(" {} ", joker.name()),
                joker.cost(),
                joker_lines(joker, w),
            ))
        }
        ShopSlot::Consumable(idx) => {
            let c = app.game.shop.consumables.get(idx)?;
            Some((
                format!(" {} ", c.name()),
                c.cost(),
                consumable_lines(c, w),
            ))
        }
        ShopSlot::Pack(idx) => {
            let pack = app.game.shop.packs.get(idx)?;
            Some((format!(" {} ", pack.name()), pack.cost(), pack_lines(pack, w)))
        }
    }
}
