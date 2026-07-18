//! Score a hand from an existing game state, with a step-by-step trace of
//! how the number was reached.
//!
//! Usage: `calc score STATE.json`
//!
//! STATE.json is whatever `Game::to_json()` produces. Which cards are being
//! played comes entirely from the file's own selected-card state
//! (`Available`'s per-card selected flag) — this tool never chooses or
//! rearranges cards itself, it just scores what's already there.

use balatro_rs::game::Game;
use balatro_rs::hand::SelectHand;

fn usage() -> ! {
    eprintln!("usage: calc score STATE.json");
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 2 || args[0] != "score" {
        usage();
    }
    let path = &args[1];

    let contents = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("failed to read {path}: {e}");
        std::process::exit(1);
    });

    let mut game = Game::from_json(&contents).unwrap_or_else(|e| {
        eprintln!("failed to parse {path} as a game state: {e}");
        std::process::exit(1);
    });

    let selected = game.available.selected();
    if selected.is_empty() {
        eprintln!("no cards selected in {path} — nothing to score");
        std::process::exit(1);
    }

    let made = SelectHand::new(selected).best_hand().unwrap_or_else(|e| {
        eprintln!("could not score selected cards: {e}");
        std::process::exit(1);
    });

    let (score, trace) = game.calc_score_traced(made);

    for step in &trace.0 {
        println!(
            "{}  (chips {} mult {})",
            step.describe(),
            step.chips_after,
            step.mult_after
        );
    }
    println!();

    let final_chips = trace.0.last().map(|s| s.chips_after).unwrap_or(0);
    let final_mult = trace.0.last().map(|s| s.mult_after).unwrap_or(0);
    println!("{final_chips} x {final_mult} = {score}");
}
