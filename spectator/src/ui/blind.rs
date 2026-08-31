use crate::app::{AppState, FocusZone, WidgetId};
use crate::ui::{cards, joker_strip, sidebar};
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
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(joker_strip::STRIP_H),
            Constraint::Length(cards::SLOT_H + 1),
            Constraint::Length(5),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .split(inner);

    joker_strip::render(f, app, chunks[0]);
    render_cards(f, app, chunks[1]);
    render_buttons(f, app, chunks[2]);
    render_hints(f, chunks[3]);
    render_deck_count(f, app, chunks[2]);
}

fn render_cards(f: &mut Frame, app: &mut AppState, area: Rect) {
    let all_cards = app.game.available.cards();
    let selected_ids: std::collections::HashSet<usize> =
        app.game.available.selected().iter().map(|c| c.id).collect();
    let cards_and_selected: Vec<(balatro_rs::card::Card, bool)> = all_cards
        .iter()
        .map(|c| (*c, selected_ids.contains(&c.id)))
        .collect();
    let cursor = if app.focus == FocusZone::Cards {
        app.cursor
    } else {
        usize::MAX
    };

    // Center the cards in the available area
    let total_cards = cards_and_selected.len();
    let cards_width = total_cards as u16 * cards::SLOT_W;
    let x_offset = area.x + area.width.saturating_sub(cards_width) / 2;

    let card_area = Rect {
        x: x_offset,
        y: area.y,
        width: cards_width.min(area.width),
        height: area.height,
    };

    cards::render_hand(
        f,
        app,
        card_area,
        &cards_and_selected,
        cursor,
        WidgetId::Card,
    );
}

fn render_buttons(f: &mut Frame, app: &mut AppState, area: Rect) {
    let focused_btns = app.focus == FocusZone::ActionButtons;
    let btn_height: u16 = 3;
    let play_w: u16 = 16;
    let sort_w: u16 = 22;
    let discard_w: u16 = 16;
    let gap: u16 = 2;
    let total_width = play_w + gap + sort_w + gap + discard_w;
    let x_start = area.x + area.width.saturating_sub(total_width) / 2;
    let y = area.y + area.height.saturating_sub(btn_height) / 2;

    // Play Hand (cursor 0)
    {
        let is_focused = focused_btns && app.cursor == 0;
        let rect = Rect {
            x: x_start,
            y,
            width: play_w,
            height: btn_height,
        };
        let style = if is_focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(if is_focused {
                BorderType::Double
            } else {
                BorderType::Plain
            })
            .border_style(if is_focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            });
        let para = Paragraph::new(Line::from(Span::styled("Play Hand", style)))
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(para, rect);
        app.widget_rects.insert(WidgetId::ActionButton(0), rect);
    }

    // Sort Hand box (cursors 1=Rank, 2=Suit)
    {
        let sort_focused = focused_btns && (app.cursor == 1 || app.cursor == 2);
        let rect = Rect {
            x: x_start + play_w + gap,
            y,
            width: sort_w,
            height: btn_height,
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(if sort_focused {
                BorderType::Double
            } else {
                BorderType::Plain
            })
            .border_style(if sort_focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            })
            .title("Sort Hand")
            .title_alignment(Alignment::Center);
        let rank_style = if focused_btns && app.cursor == 1 {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let suit_style = if focused_btns && app.cursor == 2 {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let inner = block.inner(rect);
        f.render_widget(block, rect);
        let line = Line::from(vec![
            Span::styled("Rank", rank_style),
            Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Suit", suit_style),
        ]);
        f.render_widget(Paragraph::new(line).alignment(Alignment::Center), inner);
        let half = rect.width / 2;
        app.widget_rects.insert(
            WidgetId::ActionButton(1),
            Rect {
                x: rect.x,
                y: rect.y,
                width: half,
                height: rect.height,
            },
        );
        app.widget_rects.insert(
            WidgetId::ActionButton(2),
            Rect {
                x: rect.x + half,
                y: rect.y,
                width: rect.width - half,
                height: rect.height,
            },
        );
    }

    // Discard (cursor 3)
    {
        let is_focused = focused_btns && app.cursor == 3;
        let rect = Rect {
            x: x_start + play_w + gap + sort_w + gap,
            y,
            width: discard_w,
            height: btn_height,
        };
        let style = if is_focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(if is_focused {
                BorderType::Double
            } else {
                BorderType::Plain
            })
            .border_style(if is_focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            });
        let para = Paragraph::new(Line::from(Span::styled("Discard", style)))
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(para, rect);
        app.widget_rects.insert(WidgetId::ActionButton(3), rect);
    }
}

fn render_deck_count(f: &mut Frame, app: &AppState, area: Rect) {
    let deck_count = app.game.deck.cards().len();
    let total = deck_count + app.game.available.cards().len() + app.game.discarded.len();
    let label = format!("{}/{}", deck_count, total);
    let para = Paragraph::new(Span::styled(label, Style::default().fg(Color::DarkGray)))
        .alignment(Alignment::Right);
    let count_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 1,
    };
    f.render_widget(para, count_area);
}

fn render_hints(f: &mut Frame, area: Rect) {
    let hints = "? controls";
    let para = Paragraph::new(Span::styled(hints, Style::default().fg(Color::DarkGray)))
        .alignment(Alignment::Center);
    f.render_widget(para, area);
}
