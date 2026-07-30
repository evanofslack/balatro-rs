//! Engine telemetry overlay (`m`) — what a step costs and what an agent
//! would see at it. Everything shown is measured in the TUI, so the
//! numbers are the real cost of driving `core`, not a synthetic benchmark.

use crate::app::AppState;
use crate::metrics::{fmt_count, fmt_ns, Timing};
use crate::ui::overlay::centered_rect;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

const W: u16 = 78;

pub fn render(f: &mut Frame, app: &mut AppState, area: Rect) {
    // Grows to fill the terminal: with every action kind timed and a full
    // score trace this runs well past a fixed 30 rows, so the panel takes
    // what it can get and scrolls for the rest.
    let rect = centered_rect(W.min(area.width), area.height.saturating_sub(2), area);
    f.render_widget(Clear, rect);

    let block = Block::default()
        .title(Span::styled(
            " Engine Metrics ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));
    let inner = block.inner(rect);
    f.render_widget(block, rect);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    let mut lines: Vec<Line> = Vec::new();
    push_last_step(&mut lines, app);
    lines.push(Line::from(""));
    push_timings(&mut lines, app);
    lines.push(Line::from(""));
    push_throughput(&mut lines, app);
    push_score_trace(&mut lines, app, inner.width as usize);

    // `overlay_cursor` doubles as this panel's scroll offset. Clamping
    // happens here because only render knows the body height.
    let body_h = chunks[0].height as usize;
    let max_scroll = lines.len().saturating_sub(body_h);
    app.overlay_cursor = app.overlay_cursor.min(max_scroll);
    let scroll = app.overlay_cursor;

    f.render_widget(Paragraph::new(lines).scroll((scroll as u16, 0)), chunks[0]);

    let footer = if max_scroll > 0 {
        format!("  ↑/↓ scroll ({}/{})   Esc / m close", scroll, max_scroll)
    } else {
        "  Esc / m close".to_string()
    };
    f.render_widget(
        Paragraph::new(Span::styled(footer, Style::default().fg(Color::DarkGray))),
        chunks[1],
    );
}

fn heading(text: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!(" {text}"),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ))
}

fn field(name: &str, value: String, color: Color) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("   {name:<16}"),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(value, Style::default().fg(color)),
    ])
}

/// The most recent step: what it cost, and the decision the agent faced.
fn push_last_step(lines: &mut Vec<Line>, app: &AppState) {
    lines.push(heading("Last step"));
    let Some(last) = &app.metrics.last_action else {
        lines.push(field(
            "(none yet)",
            "play a move to populate".into(),
            Color::DarkGray,
        ));
        return;
    };

    lines.push(field("action", last.label.clone(), Color::White));
    lines.push(field("stage", last.stage.clone(), Color::Cyan));
    // Show this step against its own kind's history, not the blended mean
    // across every action type — that comparison is the useful one.
    let vs_kind = app
        .metrics
        .by_kind
        .get(last.kind)
        .filter(|t| t.count > 1)
        .map(|t| format!("   ({} mean {})", last.kind, fmt_ns(t.mean_ns())))
        .unwrap_or_default();
    lines.push(field(
        "handle_action",
        format!("{}{vs_kind}", fmt_ns(last.ns)),
        if last.ok { Color::Green } else { Color::Red },
    ));
    if let Some(err) = &last.error {
        lines.push(field("error", err.clone(), Color::Red));
    }
    lines.push(field(
        "legal actions",
        format!(
            "{}  (gen_actions {})",
            last.legal_actions,
            fmt_ns(app.metrics.gen_actions.last_ns)
        ),
        Color::White,
    ));
    lines.push(field(
        "action mask",
        format!(
            "{}/{} unmasked  ({:.1}% dense)",
            last.unmasked,
            last.mask_size,
            last.mask_density() * 100.0
        ),
        Color::White,
    ));
    lines.push(field(
        "deltas",
        format!(
            "score {:+}   money {:+}",
            last.score_delta, last.money_delta
        ),
        Color::LightBlue,
    ));
}

/// Per-operation timing table. Split per action kind because a `Play`
/// costs orders of magnitude more than a `SelectCard`, and one blended
/// mean hides exactly the thing you'd tune.
fn push_timings(lines: &mut Vec<Line>, app: &AppState) {
    lines.push(heading(
        "Timings                n     last     mean      p50      p99      max",
    ));

    let mut row = |name: &str, t: &Timing, color: Color| {
        if t.is_empty() {
            return;
        }
        lines.push(Line::from(vec![
            Span::styled(format!("   {name:<16}"), Style::default().fg(color)),
            Span::styled(
                format!(
                    "{:>6} {:>8} {:>8} {:>8} {:>8} {:>8}",
                    t.count,
                    fmt_ns(t.last_ns),
                    fmt_ns(t.mean_ns()),
                    fmt_ns(t.percentile_ns(50)),
                    fmt_ns(t.percentile_ns(99)),
                    fmt_ns(t.max_ns),
                ),
                Style::default().fg(Color::White),
            ),
        ]));
    };

    row("handle_action", &app.metrics.handle_action, Color::Green);
    row("gen_actions", &app.metrics.gen_actions, Color::Green);
    row(
        "gen_action_space",
        &app.metrics.gen_action_space,
        Color::Green,
    );
    row("calc_score", &app.metrics.score, Color::Green);
    row("render frame", &app.metrics.render, Color::Magenta);

    for (kind, timing) in &app.metrics.by_kind {
        row(&format!("· {kind}"), timing, Color::DarkGray);
    }
}

/// The numbers to extrapolate a training run from.
fn push_throughput(lines: &mut Vec<Line>, app: &AppState) {
    lines.push(heading("Throughput"));
    let m = &app.metrics;
    lines.push(field(
        "steps",
        format!("{} ok, {} rejected", m.actions_ok, m.actions_err),
        Color::White,
    ));
    lines.push(field(
        "session",
        format!(
            "{:.1}s wall, {:.2}% in engine",
            m.elapsed().as_secs_f64(),
            m.engine_time_fraction() * 100.0
        ),
        Color::White,
    ));
    lines.push(field(
        "engine ceiling",
        format!(
            "{} steps/s single-threaded",
            fmt_count(m.engine_steps_per_sec())
        ),
        Color::LightGreen,
    ));
    lines.push(field(
        "observed rate",
        format!(
            "{} steps/s (incl. think time)",
            fmt_count(m.actions_per_sec())
        ),
        Color::DarkGray,
    ));
}

/// Where the last hand's score actually came from, step by step. This is
/// the reward signal decomposed — usually the first thing you want when a
/// policy is learning something strange.
fn push_score_trace(lines: &mut Vec<Line>, app: &AppState, width: usize) {
    let Some(hand) = &app.metrics.last_hand else {
        return;
    };
    lines.push(Line::from(""));
    lines.push(heading(&format!(
        "Last hand — {} for {} in {}",
        hand.rank,
        hand.score,
        fmt_ns(hand.score_ns)
    )));

    if hand.trace.0.is_empty() {
        lines.push(field("(no steps)", String::new(), Color::DarkGray));
        return;
    }

    // Generous now that the panel scrolls; still bounded so a pathological
    // retrigger stack can't make the list unusable.
    const MAX_ROWS: usize = 32;
    for step in hand.trace.0.iter().take(MAX_ROWS) {
        let desc = step.describe();
        let desc = if desc.len() > width.saturating_sub(20) {
            format!("{}…", &desc[..width.saturating_sub(21)])
        } else {
            desc
        };
        lines.push(Line::from(vec![
            Span::styled("   ", Style::default()),
            Span::styled(
                format!("{:>5} × {:<4}  ", step.chips_after, step.mult_after),
                Style::default().fg(Color::LightBlue),
            ),
            Span::styled(desc, Style::default().fg(Color::White)),
        ]));
    }
    if hand.trace.0.len() > MAX_ROWS {
        lines.push(field(
            "",
            format!("… {} more steps", hand.trace.0.len() - MAX_ROWS),
            Color::DarkGray,
        ));
    }
}

#[cfg(test)]
mod tests {
    use crate::app::{AppState, Overlay};
    use balatro_rs::action::Action;
    use balatro_rs::game::Game;
    use balatro_rs::stage::Blind;
    use ratatui::{backend::TestBackend, Terminal};

    /// A game that has selected a blind and played one hand, so every
    /// timing bucket and the score trace are populated.
    fn app_after_a_hand() -> AppState {
        let mut game = Game::default();
        game.start();
        let mut app = AppState::new(game);
        app.sync_focus_to_stage();
        app.act(Action::SelectBlind(Blind::Small))
            .expect("select blind");
        for card in app.game.available.cards().iter().take(5) {
            app.act(Action::SelectCard(*card)).expect("select card");
        }
        app.act(Action::Play()).expect("play");
        app
    }

    fn render_to_string(app: &mut AppState, w: u16, h: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(w, h)).expect("terminal");
        terminal
            .draw(|f| crate::ui::render(f, app))
            .expect("draw frame");
        let buf = terminal.backend().buffer();
        let mut out = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                out.push_str(buf[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }

    #[test]
    fn test_act_records_timings_and_step_context() {
        let app = app_after_a_hand();
        let m = &app.metrics;

        assert_eq!(m.actions_ok, 7);
        assert_eq!(m.actions_err, 0);
        assert_eq!(m.handle_action.count, 7);
        // One gen_actions + one gen_action_space sampled per step.
        assert_eq!(m.gen_actions.count, 7);
        assert_eq!(m.gen_action_space.count, 7);
        assert!(m.by_kind.contains_key("SelectCard"));
        assert_eq!(m.by_kind["SelectCard"].count, 5);
        assert_eq!(m.by_kind["Play"].count, 1);

        let last = m.last_action.as_ref().expect("last action recorded");
        assert_eq!(last.kind, "Play");
        assert!(last.ok);
        assert!(last.ns > 0, "handle_action should take measurable time");
        assert!(last.legal_actions > 0);
        assert_eq!(last.mask_size, 108, "fixed action space width");
        assert!(last.unmasked > 0 && last.unmasked <= last.mask_size);
        assert!(last.score_delta > 0, "playing a hand scores");

        // The Play path captures a score trace off a clone.
        let hand = m.last_hand.as_ref().expect("score trace captured");
        assert!(hand.score > 0);
        assert!(!hand.trace.0.is_empty());
    }

    #[test]
    fn test_probe_clone_does_not_disturb_the_real_game() {
        // capture_score_trace scores on a clone; if it ever scored the real
        // game the hand would be counted twice in the planetarium.
        let mut game = Game::default();
        game.start();
        let mut app = AppState::new(game);
        app.sync_focus_to_stage();
        app.act(Action::SelectBlind(Blind::Small)).expect("blind");
        for card in app.game.available.cards().iter().take(2) {
            app.act(Action::SelectCard(*card)).expect("select");
        }
        let rank = balatro_rs::hand::SelectHand::new(app.game.available.selected())
            .best_hand()
            .expect("made hand")
            .rank;
        let plays_before = app.game.planetarium.level(rank).plays;

        app.act(Action::Play()).expect("play");

        assert_eq!(
            app.game.planetarium.level(rank).plays,
            plays_before + 1,
            "the traced probe must not double-count the hand"
        );
    }

    #[test]
    fn test_overlay_renders_every_section() {
        let mut app = app_after_a_hand();
        app.overlay = Some(Overlay::Metrics);
        let out = render_to_string(&mut app, 100, 40);

        for expected in [
            "Engine Metrics",
            "Last step",
            "handle_action",
            "action mask",
            "Timings",
            "gen_action_space",
            "Throughput",
            "engine ceiling",
            "Last hand",
        ] {
            assert!(out.contains(expected), "overlay missing {expected:?}");
        }
        // Per-kind breakdown is present, not just the blended total.
        assert!(out.contains("· SelectCard"));
    }

    #[test]
    fn test_overlay_scroll_is_clamped_to_content() {
        let mut app = app_after_a_hand();
        app.overlay = Some(Overlay::Metrics);
        app.overlay_cursor = 9999;

        // A tall terminal fits everything, so scrolling is pinned at 0.
        render_to_string(&mut app, 100, 40);
        assert_eq!(app.overlay_cursor, 0, "no scroll when content fits");

        // A short one has to scroll, but never past the end.
        app.overlay_cursor = 9999;
        render_to_string(&mut app, 100, 30);
        assert!(
            app.overlay_cursor > 0 && app.overlay_cursor < 100,
            "clamped to a sane offset, got {}",
            app.overlay_cursor
        );
    }

    #[test]
    fn test_sidebar_shows_the_last_step_cost() {
        let mut app = app_after_a_hand();
        let out = render_to_string(&mut app, 100, 40);
        assert!(out.contains("⏱"), "sidebar timing line missing");
        assert!(out.contains("legal"), "sidebar legal-action count missing");
    }
}
