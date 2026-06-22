pub mod blind;
pub mod cards;
pub mod end;
pub mod joker_strip;
pub mod overlay;
pub mod postblind;
pub mod preblind;
pub mod shop;
pub mod sidebar;
pub mod size_check;
pub mod tarot;

use crate::app::AppState;
use balatro_rs::stage::Stage;
use ratatui::Frame;

const MIN_WIDTH: u16 = 80;
const MIN_HEIGHT: u16 = 20;

pub fn render(f: &mut Frame, app: &mut AppState) {
    let area = f.area();

    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        size_check::render(f, area, area.width, area.height, MIN_WIDTH, MIN_HEIGHT);
        return;
    }

    match &app.game.stage.clone() {
        Stage::PreBlind() => preblind::render(f, app, area),
        Stage::Blind(_) => blind::render(f, app, area),
        Stage::PostBlind() => postblind::render(f, app, area),
        Stage::Shop() => shop::render(f, app, area),
        Stage::TarotHand(_) => tarot::render(f, app, area),
        Stage::End(_) => end::render(f, app, area),
    }

    if let Some(ref ov) = app.overlay.clone() {
        overlay::render(f, app, area, ov);
    }
}
