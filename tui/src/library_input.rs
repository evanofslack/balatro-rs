use crate::app::{AppState, ReturnTarget, TitleButton, WidgetId};
use crate::library::{delete_replay, rename_replay};
use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};

pub fn handle_title_key(app: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Left | KeyCode::Char('h') => {
            let i = app.title_focus.index();
            app.title_focus = TitleButton::from_index(i.saturating_sub(1));
        }
        KeyCode::Right | KeyCode::Char('l') => {
            let i = (app.title_focus.index() + 1).min(1);
            app.title_focus = TitleButton::from_index(i);
        }
        KeyCode::Char('r') => app.toggle_recording(),
        KeyCode::Enter | KeyCode::Char(' ') => app.activate_title_button(),
        _ => {}
    }
}

pub fn handle_library_key(app: &mut AppState, key: KeyEvent) {
    let Some(library) = app.library.as_mut() else {
        return;
    };

    if let Some(input) = library.rename_input.as_mut() {
        match key.code {
            KeyCode::Esc => {
                library.rename_input = None;
                library.status = "rename cancelled".into();
            }
            KeyCode::Enter => {
                let name = input.clone();
                let path = library.selected().map(|e| e.path.clone());
                library.rename_input = None;
                if let Some(path) = path {
                    match rename_replay(&path, &name) {
                        Ok(_) => {
                            library.refresh();
                            library.status = format!("renamed to {name}");
                        }
                        Err(err) => library.status = format!("rename failed: {err:#}"),
                    }
                }
            }
            KeyCode::Backspace => {
                input.pop();
            }
            KeyCode::Char(c) => {
                if !c.is_control() {
                    input.push(c);
                }
            }
            _ => {}
        }
        return;
    }

    if library.confirm_delete {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                let path = library.selected().map(|e| e.path.clone());
                library.confirm_delete = false;
                if let Some(path) = path {
                    match delete_replay(&path) {
                        Ok(()) => {
                            library.refresh();
                            library.status = "deleted".into();
                        }
                        Err(err) => library.status = format!("delete failed: {err:#}"),
                    }
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                library.confirm_delete = false;
                library.status = "delete cancelled".into();
            }
            _ => {}
        }
        return;
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            if app.library_is_root || app.return_target == ReturnTarget::Quit {
                app.should_quit = true;
            } else {
                app.open_title();
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if !library.entries.is_empty() {
                library.cursor = (library.cursor + 1).min(library.entries.len() - 1);
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            library.cursor = library.cursor.saturating_sub(1);
        }
        KeyCode::Char('r') => library.refresh(),
        KeyCode::Char('R') => {
            if let Some(entry) = library.selected() {
                library.rename_input = Some(entry.display_name());
            }
        }
        KeyCode::Char('d') => {
            if library.selected().is_some() {
                library.confirm_delete = true;
            }
        }
        KeyCode::Enter => {
            let path = library.selected().map(|e| e.path.clone());
            if let Some(path) = path {
                if let Err(err) = app.attach_replay(path, true) {
                    if let Some(library) = app.library.as_mut() {
                        library.status = format!("open failed: {err:#}");
                    }
                }
            }
        }
        _ => {}
    }
}

pub fn handle_title_mouse(app: &mut AppState, event: MouseEvent) {
    if event.kind != MouseEventKind::Down(crossterm::event::MouseButton::Left) {
        return;
    }
    let hit = app.widget_rects.iter().find(|(_, rect)| {
        event.column >= rect.x
            && event.column < rect.x + rect.width
            && event.row >= rect.y
            && event.row < rect.y + rect.height
    });
    if let Some((WidgetId::TitleButton(i), _)) = hit {
        app.title_focus = TitleButton::from_index(*i);
        app.activate_title_button();
    }
}

pub fn handle_library_mouse(app: &mut AppState, event: MouseEvent) {
    if event.kind != MouseEventKind::Down(crossterm::event::MouseButton::Left) {
        return;
    }
    if app
        .library
        .as_ref()
        .is_some_and(|l| l.rename_input.is_some() || l.confirm_delete)
    {
        return;
    }
    let hit = app.widget_rects.iter().find_map(|(id, rect)| {
        if event.column >= rect.x
            && event.column < rect.x + rect.width
            && event.row >= rect.y
            && event.row < rect.y + rect.height
        {
            Some(id.clone())
        } else {
            None
        }
    });
    if let Some(WidgetId::LibraryRow(i)) = hit {
        if let Some(library) = app.library.as_mut() {
            if i < library.entries.len() {
                library.cursor = i;
            }
        }
        let path = app
            .library
            .as_ref()
            .and_then(|l| l.selected().map(|e| e.path.clone()));
        if let Some(path) = path {
            let _ = app.attach_replay(path, true);
        }
    }
}
