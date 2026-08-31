use crate::agent::{apply_gym_action, focus_for_action};
use crate::app::AppState;
use crate::app::TransportButton;
use crate::library_input::return_to_library;
use crate::replay::{reset_game_to_step, REPLAY_BASE_MS};
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};

pub fn handle_transport_key(app: &mut AppState, key: KeyEvent) -> bool {
    if !app.is_json_replay() {
        return false;
    }
    match key.code {
        KeyCode::Char(' ') | KeyCode::Char('a') => {
            toggle_pause(app);
            true
        }
        KeyCode::Char('b') | KeyCode::Esc => {
            exit_replay(app);
            true
        }
        KeyCode::Left => {
            step_back(app);
            true
        }
        KeyCode::Right => {
            step_forward(app);
            true
        }
        _ => false,
    }
}

pub fn handle_transport_mouse(app: &mut AppState, event: MouseEvent) -> bool {
    if !app.is_json_replay() || event.kind != MouseEventKind::Down(MouseButton::Left) {
        return false;
    }
    let Some(chrome) = app.replay_chrome.as_ref() else {
        return false;
    };
    let col = event.column;
    let row = event.row;
    for (btn, rect) in &chrome.button_rects {
        if col >= rect.x
            && col < rect.x + rect.width
            && row >= rect.y
            && row < rect.y + rect.height
        {
            match btn {
                TransportButton::Pause => toggle_pause(app),
                TransportButton::RewindFast => {
                    if let Some(c) = app.replay_chrome.as_mut() {
                        c.cycle_rewind_mult();
                    }
                    app.last_rewind_at = None;
                }
                TransportButton::RewindStep => step_back(app),
                TransportButton::ForwardStep => step_forward(app),
                TransportButton::ForwardFast => {
                    if let Some(c) = app.replay_chrome.as_mut() {
                        c.cycle_forward_mult();
                    }
                }
                TransportButton::Exit => exit_replay(app),
            }
            return true;
        }
    }
    false
}

pub fn toggle_pause(app: &mut AppState) {
    app.autoplay_paused = !app.autoplay_paused;
    app.step_once = false;
    if let Some(c) = app.replay_chrome.as_mut() {
        c.rewinding = false;
    }
}

pub fn exit_replay(app: &mut AppState) {
    app.replay_complete_at = None;
    return_to_library(app);
}

pub fn clear_pending(app: &mut AppState) {
    app.pending_ai_action = None;
    app.ai_action = None;
    app.highlight_until = None;
}

pub fn step_back(app: &mut AppState) {
    let Some(session) = app.replay_session.clone() else {
        return;
    };
    let target = app
        .replay_driver
        .as_ref()
        .filter(|d| d.can_step_back())
        .map(|d| d.cursor().saturating_sub(1));
    let Some(target) = target else {
        if let Some(c) = app.replay_chrome.as_mut() {
            c.rewinding = false;
        }
        return;
    };
    clear_pending(app);
    app.autoplay_paused = true;
    app.replay_complete_at = None;
    if let Some(driver) = app.replay_driver.as_mut() {
        driver.seek(target);
    }
    match reset_game_to_step(&session.file, target) {
        Ok(game) => {
            app.game = game;
            app.sync_focus_to_stage();
        }
        Err(e) => eprintln!("rewind error: {e:#}"),
    }
    app.last_rewind_at = Some(std::time::Instant::now());
}

pub fn step_forward(app: &mut AppState) {
    let action = app.replay_driver.as_ref().and_then(|d| d.peek_next());
    let Some(action) = action else {
        return;
    };
    clear_pending(app);
    app.autoplay_paused = true;
    app.replay_complete_at = None;
    if let Some(driver) = app.replay_driver.as_mut() {
        driver.next_action();
    }
    if let Err(e) = apply_gym_action(&mut app.game, action) {
        eprintln!("forward step error: {e:#}");
        return;
    }
    focus_for_action(app, action);
    app.sync_focus_to_stage();
}

pub fn replay_delay_ms(app: &AppState) -> u64 {
    let mult = app
        .replay_chrome
        .as_ref()
        .map(|c| c.forward_mult)
        .unwrap_or(1)
        .max(1) as u64;
    REPLAY_BASE_MS / mult
}

pub fn rewind_delay_ms(app: &AppState) -> u64 {
    let mult = app
        .replay_chrome
        .as_ref()
        .map(|c| c.rewind_mult)
        .unwrap_or(1)
        .max(1) as u64;
    REPLAY_BASE_MS / mult
}

pub fn tick_rewind(app: &mut AppState) {
    let rewinding = app
        .replay_chrome
        .as_ref()
        .is_some_and(|c| c.rewinding);
    if !rewinding || app.pending_ai_action.is_some() {
        return;
    }
    let delay = rewind_delay_ms(app);
    let due = app
        .last_rewind_at
        .map(|t| t.elapsed().as_millis() >= delay as u128)
        .unwrap_or(true);
    if due {
        step_back(app);
    }
}

pub fn attach_replay(app: &mut AppState, path: std::path::PathBuf, replay: crate::replay::ReplayFile) {
    use crate::app::{AppMode, ReplayChrome, ReplaySession};
    use crate::replay::{init_game_from_replay, ReplayDriver};

    let actions = replay.actions.clone();
    app.game = init_game_from_replay(&replay);
    app.replay_session = Some(ReplaySession {
        file: replay,
        source_path: path,
    });
    app.replay_driver = Some(ReplayDriver::new(actions));
    app.replay_chrome = Some(ReplayChrome::new());
    app.replay_complete_at = None;
    app.last_rewind_at = None;
    app.library = None;
    app.mode = AppMode::Spectating;
    app.agent_mode = true;
    app.autoplay_paused = false;
    app.step_once = false;
    clear_pending(app);
    app.sync_focus_to_stage();
}
