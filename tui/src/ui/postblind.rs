use crate::app::{AppState, FocusZone, WidgetId};
use crate::ui::{joker_strip, sidebar};
use balatro_rs::stage::BlindExt;
use ratatui::{
    layout::{Alignment, Rect},
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
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = outer_block.inner(area);
    f.render_widget(outer_block, area);

    // Joker strip at top
    let strip_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: joker_strip::STRIP_H,
    };
    joker_strip::render(f, app, strip_area);

    // Cashout panel centered in remaining area
    let remaining = Rect {
        x: inner.x,
        y: inner.y + joker_strip::STRIP_H,
        width: inner.width,
        height: inner.height.saturating_sub(joker_strip::STRIP_H),
    };

    let panel_w: u16 = 50;
    let panel_h: u16 = 11;
    let btn_w: u16 = 20;
    let btn_h: u16 = 3;
    let gap: u16 = 1;
    let combined_h = panel_h + gap + btn_h;
    let panel_x = remaining.x + remaining.width.saturating_sub(panel_w) / 2;
    let panel_y = remaining.y + remaining.height.saturating_sub(combined_h) / 2;
    let panel_rect = Rect {
        x: panel_x,
        y: panel_y,
        width: panel_w,
        height: panel_h,
    };

    // game.reward = blind_base + interest + hand_bonus (computed by engine on blind clear)
    let total = app.game.reward;
    let blind_base = app.game.blind.map(|b| b.reward()).unwrap_or(0);
    let hands_bonus = app.game.plays * app.game.config.money_per_hand;
    let interest = ((app.game.money as f32 * app.game.config.interest_rate) as usize)
        .min(app.game.config.interest_max);

    let block = Block::default()
        .title(Span::styled(
            format!(" Cash Out: ${} ", total),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let sep = "─".repeat(panel_w as usize - 2);

    let score_label = format!("  Round score: ◆ {}", app.game.last_score);
    let blind_label = "  Beat the blind".to_string();
    let hands_label = format!(
        "  {} hand(s) remaining x ${}",
        app.game.plays, app.game.config.money_per_hand
    );
    let interest_per = (1.0 / app.game.config.interest_rate).round() as usize;
    let interest_label = format!(
        "  Interest ($1 per ${}, max {})",
        interest_per, app.game.config.interest_max
    );

    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            score_label.clone(),
            Style::default().fg(Color::LightBlue),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(blind_label.clone(), Style::default().fg(Color::White)),
            Span::raw(format!(
                "{:>width$}",
                format!("${}", blind_base),
                width = (panel_w as usize).saturating_sub(blind_label.len() + 2)
            )),
        ]),
        Line::from(vec![
            Span::styled(hands_label.clone(), Style::default().fg(Color::White)),
            Span::styled(
                format!(
                    "{:>width$}",
                    format!("${}", hands_bonus),
                    width = (panel_w as usize).saturating_sub(hands_label.len() + 2)
                ),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::styled(interest_label.clone(), Style::default().fg(Color::White)),
            Span::styled(
                format!(
                    "{:>width$}",
                    format!("${}", interest),
                    width = (panel_w as usize).saturating_sub(interest_label.len() + 2)
                ),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Joker bonuses", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:>width$}", "(coming soon)", width = panel_w as usize - 17),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(Span::styled(sep, Style::default().fg(Color::DarkGray))),
        Line::from(vec![
            Span::styled(
                "  Total",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    "{:>width$}",
                    format!("${}", total),
                    width = panel_w as usize - 9
                ),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let para = Paragraph::new(Text::from(lines)).block(block);
    f.render_widget(para, panel_rect);

    // Cash Out button, as its own bordered widget below the stats panel
    let btn_rect = Rect {
        x: remaining.x + remaining.width.saturating_sub(btn_w) / 2,
        y: panel_y + panel_h + gap,
        width: btn_w,
        height: btn_h,
    };
    let focused = app.focus == FocusZone::CashOutButton;
    let border_type = if focused {
        BorderType::Double
    } else {
        BorderType::Plain
    };
    let btn_block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(Style::default().fg(if focused { Color::Yellow } else { Color::DarkGray }));
    let btn_para = Paragraph::new(Line::from(Span::styled(
        "Cash Out",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )))
    .block(btn_block)
    .alignment(Alignment::Center);
    f.render_widget(btn_para, btn_rect);
    app.widget_rects.insert(WidgetId::CashOutButton, btn_rect);
}
