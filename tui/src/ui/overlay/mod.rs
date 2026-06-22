pub mod consumable;
pub mod controls;
pub mod deck;
pub mod inspect;
pub mod poker_hands;
pub mod run_info;
pub mod save;

use crate::app::{AppState, Overlay};
use ratatui::{Frame, layout::Rect};

pub fn render(f: &mut Frame, app: &mut AppState, area: Rect, overlay: &Overlay) {
    match overlay {
        Overlay::Inspect(target) => inspect::render(f, app, area, target.clone()),
        Overlay::RunInfo => run_info::render(f, app, area),
        Overlay::Controls => controls::render(f, app, area),
        Overlay::Save => save::render(f, app, area),
        Overlay::Consumable(idx) => consumable::render(f, app, area, *idx),
    }
}

pub fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect {
        x,
        y,
        width: width.min(area.width),
        height: height.min(area.height),
    }
}
