use crate::app::AppState;
use crate::app::WidgetId;
use crate::ui::overlay::centered_rect;
use crate::ui::overlay::inspect::{card_lines, consumable_lines, joker_lines};
use balatro_rs::consumable::Consumable;
use balatro_rs::pack::PackContent;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Green, matching the real game's "SELECT" pill on pack-open choices
/// (distinct from the shop's orange Buy — the reference screenshots use
/// different colors for the two, so this doesn't try to force one palette).
const SELECT_GREEN: Color = Color::Green;

pub fn render(f: &mut Frame, app: &mut AppState, area: Rect, idx: usize) {
    let w: u16 = 44;
    let h: u16 = 18;
    let rect = centered_rect(w, h, area);
    f.render_widget(Clear, rect);

    let Some(content) = app
        .game
        .open_pack
        .as_ref()
        .and_then(|s| s.contents.get(idx))
        .cloned()
    else {
        app.overlay = None;
        return;
    };

    let (title, mut lines) = item_info(&content, w);

    let select_selected = app.overlay_cursor == 0;
    let select_style = {
        let base = Style::default()
            .fg(Color::White)
            .bg(SELECT_GREEN)
            .add_modifier(Modifier::BOLD);
        if select_selected {
            base.add_modifier(Modifier::UNDERLINED)
        } else {
            base
        }
    };
    let cancel_style = if !select_selected {
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let indent = "  ";
    let select_text = " Select ";
    let gap = "   ";
    let cancel_text = "[ Cancel ]";

    // Row within `lines` (0-indexed) that the select/cancel line will
    // occupy — description length varies by content type, so this can't be
    // a fixed offset from the bottom of the box.
    let select_row = lines.len() as u16;
    lines.push(Line::from(vec![
        Span::raw(indent),
        Span::styled(select_text, select_style),
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
    let y = rect.y + 1 + select_row;
    let select_x = rect.x + 1 + indent.chars().count() as u16;
    let cancel_x = select_x + select_text.chars().count() as u16 + gap.chars().count() as u16;
    app.widget_rects.insert(
        WidgetId::OverlayButton(0),
        Rect {
            x: select_x,
            y,
            width: select_text.chars().count() as u16,
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

fn item_info(content: &PackContent, w: u16) -> (String, Vec<Line<'static>>) {
    match content {
        PackContent::Joker(j) => (format!(" {} ", j.name()), joker_lines(j, w)),
        PackContent::Tarot(t) => {
            let c = Consumable::Tarot(*t);
            (format!(" {} ", c.name()), consumable_lines(&c, w))
        }
        PackContent::Planet(p) => {
            let c = Consumable::Planet(*p);
            (format!(" {} ", c.name()), consumable_lines(&c, w))
        }
        PackContent::Spectral(s) => {
            let c = Consumable::Spectral(*s);
            (format!(" {} ", c.name()), consumable_lines(&c, w))
        }
        PackContent::PlayingCard(card) => (
            format!(
                " {} of {}s ",
                crate::ui::cards::rank_str(card.value),
                crate::ui::cards::suit_char(card.suit)
            ),
            card_lines(card),
        ),
    }
}
