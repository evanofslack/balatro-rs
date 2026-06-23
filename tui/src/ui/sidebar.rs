use super::{hand_rank_name, level_color, wrap};
use crate::app::AppState;
use balatro_rs::stage::Stage;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub const SIDEBAR_W: u16 = 24;

fn label(s: &str) -> Span<'static> {
    Span::styled(s.to_string(), Style::default().fg(Color::DarkGray))
}

fn value(s: String, color: Color) -> Span<'static> {
    Span::styled(s, Style::default().fg(color).add_modifier(Modifier::BOLD))
}

pub fn render(f: &mut Frame, app: &AppState, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let game = &app.game;

    let mut lines: Vec<Line> = Vec::new();

    // Stage header
    let stage_line = match &game.stage {
        Stage::PreBlind() => Line::from(Span::styled(
            "Select Blind",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Stage::Blind(b) => Line::from(Span::styled(
            b.to_string(),
            Style::default()
                .fg(match b {
                    balatro_rs::stage::Blind::Small => Color::Cyan,
                    balatro_rs::stage::Blind::Big => Color::Yellow,
                    balatro_rs::stage::Blind::Boss => Color::Red,
                })
                .add_modifier(Modifier::BOLD),
        )),
        Stage::PostBlind() => Line::from(Span::styled(
            "Cash Out",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Stage::Shop() => Line::from(Span::styled(
            "SHOP",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Stage::TarotHand(t) => Line::from(Span::styled(
            t.name(),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )),
        Stage::PackOpen() => Line::from(Span::styled(
            "Pack Open",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Stage::End(_) => Line::from(Span::styled(
            "Game Over",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
    };
    lines.push(stage_line);
    lines.push(Line::from(""));

    // Target score (during blind)
    if let Stage::Blind(_) = &game.stage {
        lines.push(Line::from(vec![
            label("Target "),
            value(format!("◆ {}", game.required_score()), Color::LightBlue),
        ]));
        lines.push(Line::from(vec![
            label("Reward "),
            value(
                format!("${}", game.blind.map(|b| b.reward()).unwrap_or(0)),
                Color::Yellow,
            ),
        ]));
        lines.push(Line::from(""));
    }

    // TarotHand description
    if let Stage::TarotHand(t) = &game.stage {
        let desc = t.description();
        for word_line in wrap(desc, (inner.width as usize).saturating_sub(1)) {
            lines.push(Line::from(Span::raw(word_line)));
        }
        lines.push(Line::from(""));
    }

    // Round score
    lines.push(Line::from(label("Round Score")));
    lines.push(Line::from(vec![
        Span::raw("  "),
        value(format!("◆ {}", game.score), Color::LightBlue),
    ]));
    lines.push(Line::from(""));

    // Live hand evaluation — always reserves space so layout doesn't shift.
    // In blind with selection: shows detected hand name + level + live chips/mult.
    // Otherwise: blank name line + 0×0 boxes.
    let eval = if let Stage::Blind(_) = &game.stage {
        let selected = game.available.selected();
        if !selected.is_empty() {
            balatro_rs::hand::SelectHand::new(selected.clone())
                .best_hand()
                .ok()
        } else {
            None
        }
    } else {
        None
    };

    let (hand_name_line, disp_chips, disp_mult) = match &eval {
        Some(made) => {
            let lvl = game.planetarium.level(made.rank);
            let line = Line::from(vec![
                Span::styled(
                    hand_rank_name(made.rank),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" lvl.{}", lvl.level),
                    Style::default().fg(level_color(lvl.level)),
                ),
            ]);
            (line, lvl.chips, lvl.mult)
        }
        None => (Line::from(""), 0, 0),
    };
    lines.push(hand_name_line);
    lines.push(Line::from(vec![
        Span::styled(
            format!(" {} ", disp_chips),
            Style::default()
                .fg(Color::White)
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" × "),
        Span::styled(
            format!(" {} ", disp_mult),
            Style::default()
                .fg(Color::White)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(""));

    // Stats
    lines.push(Line::from(vec![
        label("Hands    "),
        value(game.plays.to_string(), Color::Blue),
    ]));
    lines.push(Line::from(vec![
        label("Discards "),
        value(game.discards.to_string(), Color::Red),
    ]));
    lines.push(Line::from(vec![
        label("Money    "),
        value(format!("${}", game.money), Color::Yellow),
    ]));
    lines.push(Line::from(""));

    // Ante / Round
    lines.push(Line::from(vec![
        label("Ante  "),
        value(
            format!(
                "{}/{}",
                ante_num(game.ante_current),
                ante_num(game.ante_end)
            ),
            Color::White,
        ),
    ]));
    lines.push(Line::from(vec![
        label("Round "),
        value(game.round.to_string(), Color::White),
    ]));

    let para = Paragraph::new(Text::from(lines));
    f.render_widget(para, inner);
}

fn ante_num(ante: balatro_rs::ante::Ante) -> usize {
    use balatro_rs::ante::Ante;
    match ante {
        Ante::Zero => 0,
        Ante::One => 1,
        Ante::Two => 2,
        Ante::Three => 3,
        Ante::Four => 4,
        Ante::Five => 5,
        Ante::Six => 6,
        Ante::Seven => 7,
        Ante::Eight => 8,
    }
}

pub fn split_sidebar_main(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(SIDEBAR_W), Constraint::Min(0)])
        .split(area);
    (chunks[0], chunks[1])
}
