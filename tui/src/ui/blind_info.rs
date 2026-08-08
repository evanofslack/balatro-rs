use balatro_rs::game::Game;
use balatro_rs::stage::Blind;

/// The score a given blind will require - unlike `Game::required_score()`,
/// doesn't depend on `self.blind` already being set to this blind, so it
/// can be computed for all three (including the as-yet-unselected Boss) at
/// any point in the run, not just while it's the active blind.
pub fn required_score(game: &Game, blind: Blind) -> usize {
    let base = game.ante_current.base();
    match blind {
        Blind::Small => base,
        Blind::Big => (base as f32 * 1.5) as usize,
        Blind::Boss => game.boss_required_score(),
    }
}

/// Whether `blind` has already been beaten this ante, based on `game.blind`
/// (the last blind resolved) and `Blind`'s ordering (Small < Big < Boss).
pub fn is_cleared(game: &Game, blind: Blind) -> bool {
    match game.blind {
        Some(last) => blind <= last,
        None => false,
    }
}
