use crate::app::{AppState, WidgetId};
use crate::ui::overlay::centered_rect;
use balatro_rs::consumable::Consumable;
use balatro_rs::stage::Stage;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &mut AppState, area: Rect, idx: usize) {
    let w: u16 = 36;
    let h: u16 = 12;
    let rect = centered_rect(w, h, area);
    f.render_widget(Clear, rect);

    let Some(c) = app.game.consumables.get(idx).cloned() else {
        app.overlay = None;
        return;
    };
    let type_color = crate::ui::consumable_type_color(&c);

    // Check if this consumable needs card targets and we're in blind stage
    let selection_info = consumable_selection_info(app, &c);

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Type:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(c.type_label().to_string(), Style::default().fg(type_color)),
        ]),
        Line::from(""),
    ];

    if let Some((needed, selected, valid)) = selection_info {
        let sel_color = if valid { Color::Green } else { Color::Yellow };
        lines.push(Line::from(vec![Span::styled(
            format!("  Select {} card(s)  ({} selected)", needed, selected),
            Style::default().fg(sel_color),
        )]));
        lines.push(Line::from(""));
    } else {
        lines.push(Line::from(Span::styled(
            "  What would you like to do?",
            Style::default().fg(Color::White),
        )));
        lines.push(Line::from(""));
    }

    let use_unavailable = matches!(selection_info, Some((_, _, false)));
    let use_selected = app.overlay_cursor == 0;
    let sell_selected = app.overlay_cursor == 1;

    let use_color = if use_unavailable {
        Color::DarkGray
    } else if use_selected {
        Color::Green
    } else {
        Color::DarkGray
    };
    let use_style = if use_selected && !use_unavailable {
        Style::default().fg(use_color).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(use_color)
    };

    let sell_color = if sell_selected {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    let sell_style = if sell_selected {
        Style::default().fg(sell_color).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(sell_color)
    };

    let sell_value = c.sell_value();
    let use_text = "  [ Use ]";
    let gap = "   ";
    let sell_text = format!("[ Sell (${}) ]", sell_value);
    let button_row = lines.len() as u16;
    lines.push(Line::from(vec![
        Span::styled(use_text, use_style),
        Span::raw(gap),
        Span::styled(sell_text.clone(), sell_style),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Esc to cancel",
        Style::default().fg(Color::DarkGray),
    )));

    let block = Block::default()
        .title(Span::styled(
            format!(" {} ", c.name()),
            Style::default()
                .fg(type_color)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(type_color));

    let para = Paragraph::new(Text::from(lines)).block(block);
    f.render_widget(para, rect);

    // +1 for the block's top border row.
    let y = rect.y + 1 + button_row;
    let use_x = rect.x + 1;
    let sell_x = use_x + use_text.chars().count() as u16 + gap.chars().count() as u16;
    app.widget_rects.insert(
        WidgetId::OverlayButton(0),
        Rect {
            x: use_x,
            y,
            width: use_text.chars().count() as u16,
            height: 1,
        },
    );
    app.widget_rects.insert(
        WidgetId::OverlayButton(1),
        Rect {
            x: sell_x,
            y,
            width: sell_text.chars().count() as u16,
            height: 1,
        },
    );
}

fn consumable_selection_info(app: &AppState, c: &Consumable) -> Option<(usize, usize, bool)> {
    let (needed, max) = match c {
        Consumable::Tarot(t) if t.requires_targets() => (t.min_targets(), t.max_targets()),
        Consumable::Spectral(s) if s.requires_targets() => (s.min_targets(), s.max_targets()),
        _ => return None,
    };
    if !matches!(app.game.stage, Stage::Blind(_)) {
        return None;
    }
    let selected = app.game.available.selected().len();
    let valid = selected >= needed && selected <= max;
    Some((needed, selected, valid))
}
