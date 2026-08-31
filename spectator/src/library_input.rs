use std::fs;
use std::path::PathBuf;

use crate::app::{AppMode, AppState, LibraryState};
use crate::replay::{clear_replay_name, load_replay, scan_replays, set_replay_name, ReplayEntry};
use crate::replay_transport::attach_replay;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

pub fn new_library_state(replays_dir: PathBuf) -> LibraryState {
    let entries = scan_replays(&replays_dir).unwrap_or_default();
    LibraryState {
        replays_dir,
        entries,
        selected: 0,
        confirm_delete: false,
        rename_active: false,
        rename_input: String::new(),
        rename_error: None,
        row_rects: Vec::new(),
    }
}

pub fn refresh_library(library: &mut LibraryState) {
    library.entries = scan_replays(&library.replays_dir).unwrap_or_default();
    if library.selected >= library.entries.len() {
        library.selected = library.entries.len().saturating_sub(1);
    }
    library.confirm_delete = false;
    library.rename_active = false;
    library.rename_input.clear();
    library.rename_error = None;
}

pub fn handle_library_key(app: &mut AppState, key: KeyEvent) -> bool {
    let Some(library) = app.library.as_mut() else {
        return false;
    };

    if library.rename_active {
        handle_rename_key(library, key);
        return false;
    }

    if library.confirm_delete {
        match key.code {
            KeyCode::Enter => {
                if let Some(entry) = library.entries.get(library.selected) {
                    let path = entry.path.clone();
                    if let Err(e) = fs::remove_file(&path) {
                        eprintln!("delete failed: {e:#}");
                    }
                }
                refresh_library(library);
            }
            KeyCode::Esc => {
                library.confirm_delete = false;
            }
            _ => {}
        }
        return false;
    }

    match key.code {
        KeyCode::Char('q') => {
            app.should_quit = true;
            return true;
        }
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::SHIFT) => start_rename(library),
        KeyCode::Char('R') => start_rename(library),
        KeyCode::Char('r') => refresh_library(library),
        KeyCode::Char('d') => {
            if !library.entries.is_empty() {
                library.confirm_delete = true;
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            library.selected = library.selected.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if !library.entries.is_empty() {
                library.selected = (library.selected + 1).min(library.entries.len() - 1);
            }
        }
        KeyCode::Char('[') | KeyCode::PageUp => library.prev_page(),
        KeyCode::Char(']') | KeyCode::PageDown => library.next_page(),
        KeyCode::Enter => {
            if let Some(entry) = library.entries.get(library.selected).cloned() {
                return start_replay_from_path(app, entry.path);
            }
        }
        _ => {}
    }
    false
}

fn start_rename(library: &mut LibraryState) {
    if library.entries.is_empty() {
        return;
    }
    library.confirm_delete = false;
    library.rename_active = true;
    library.rename_error = None;
    library.rename_input = library.entries[library.selected].display_name();
}

fn handle_rename_key(library: &mut LibraryState, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => cancel_rename(library),
        KeyCode::Enter => commit_rename(library),
        KeyCode::Backspace => {
            library.rename_input.pop();
            library.rename_error = None;
        }
        KeyCode::Char(c) => {
            if library.rename_input.len() < 48 {
                library.rename_input.push(c);
                library.rename_error = None;
            }
        }
        _ => {}
    }
}

fn cancel_rename(library: &mut LibraryState) {
    library.rename_active = false;
    library.rename_input.clear();
    library.rename_error = None;
}

fn commit_rename(library: &mut LibraryState) {
    let Some(entry) = library.entries.get(library.selected).cloned() else {
        cancel_rename(library);
        return;
    };
    let input = library.rename_input.trim().to_string();
    let default_name = ReplayEntry::default_name(entry.seed, &entry.agent_label, entry.ante_end);
    let result = if input == default_name {
        clear_replay_name(&entry.path)
    } else {
        set_replay_name(&entry.path, &input)
    };
    match result {
        Ok(()) => {
            refresh_library(library);
        }
        Err(e) => {
            library.rename_error = Some(e.to_string());
        }
    }
}

pub fn handle_library_mouse(app: &mut AppState, event: MouseEvent) -> bool {
    if event.kind != MouseEventKind::Down(crossterm::event::MouseButton::Left) {
        return false;
    }
    let Some(library) = app.library.as_mut() else {
        return false;
    };
    if library.rename_active {
        return false;
    }
    library.confirm_delete = false;
    for (row_i, rect) in library.row_rects.iter().enumerate() {
        let idx = library.page_start() + row_i;
        if event.column >= rect.x
            && event.column < rect.x + rect.width
            && event.row >= rect.y
            && event.row < rect.y + rect.height
        {
            library.selected = idx;
            if let Some(entry) = library.entries.get(idx).cloned() {
                return start_replay_from_path(app, entry.path);
            }
            break;
        }
    }
    false
}

fn start_replay_from_path(app: &mut AppState, path: PathBuf) -> bool {
    let replay = match load_replay(&path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("failed to load replay: {e:#}");
            return false;
        }
    };
    attach_replay(app, path.clone(), replay);
    true
}

pub fn return_to_library(app: &mut AppState) {
    app.mode = AppMode::Library;
    app.agent_mode = false;
    app.autoplay_paused = true;
    app.replay_driver = None;
    app.replay_session = None;
    app.replay_chrome = None;
    app.replay_complete_at = None;
    app.last_rewind_at = None;
    app.ai_action = None;
    app.pending_ai_action = None;
    app.highlight_until = None;
    app.overlay = None;
    app.clear_screen = true;
    let replays_dir = app.replays_dir.clone();
    app.library = Some(new_library_state(replays_dir));
    let game = balatro_rs::game::Game::new(balatro_rs::config::Config::default());
    app.game = game;
}
