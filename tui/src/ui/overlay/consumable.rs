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

    // Check if this tarot needs card targets and we're in blind stage
    let selection_info = tarot_selection_info(app, &c);

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Type:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(c.type_label().to_string(), Style::default().fg(Color::Cyan)),
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
        Style::default()
            .fg(use_color)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(use_color)
    };

    let sell_color = if sell_selected {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    let sell_style = if sell_selected {
        Style::default()
            .fg(sell_color)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(sell_color)
    };

    let sell_value = c.sell_value();
    lines.push(Line::from(vec![
        Span::styled("  [ Use ]", use_style),
        Span::raw("   "),
        Span::styled(format!("[ Sell (${}) ]", sell_value), sell_style),
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
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let para = Paragraph::new(Text::from(lines)).block(block);
    f.render_widget(para, rect);

    app.widget_rects.insert(
        WidgetId::OverlayButton(0),
        Rect {
            x: rect.x + 2,
            y: rect.y + 7,
            width: 7,
            height: 1,
        },
    );
    app.widget_rects.insert(
        WidgetId::OverlayButton(1),
        Rect {
            x: rect.x + 12,
            y: rect.y + 7,
            width: 14,
            height: 1,
        },
    );
}

fn tarot_selection_info(app: &AppState, c: &Consumable) -> Option<(usize, usize, bool)> {
    if let Consumable::Tarot(t) = c {
        if t.requires_targets() && matches!(app.game.stage, Stage::Blind(_)) {
            let needed = t.min_targets();
            let selected = app.game.available.selected().len();
            let valid = selected >= needed && selected <= t.max_targets();
            return Some((needed, selected, valid));
        }
    }
    None
}
