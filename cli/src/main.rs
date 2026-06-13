use balatro_rs::action::Action;
use balatro_rs::game::Game;
use std::fmt::Write as FmtWrite;
use text_io::read;

fn input_loop(max: usize) -> usize {
    loop {
        let i: usize = read!();
        if i <= max {
            return i;
        } else {
            println!("Input must be between 0 and {}", max)
        }
    }
}

fn game_loop(game: &mut Game) {
    loop {
        if game.is_over() {
            return;
        }
        let actions: Vec<Action> = game.gen_actions().collect();
        println!("Select action:");
        println!("[0] Show game state");
        for (i, action) in actions.clone().iter().enumerate() {
            let label = match action {
                Action::Play() | Action::Discard() => {
                    let selected = game.available.selected();
                    let cards: String = selected.iter().enumerate().fold(String::new(), |mut s, (j, c)| {
                        if j > 0 { s.push(' '); }
                        let _ = write!(s, "{}", c);
                        s
                    });
                    let verb = if matches!(action, Action::Play()) { "Play" } else { "Discard" };
                    format!("{}: [{}]", verb, cards)
                }
                _ => format!("{}", action),
            };
            println!("[{}] {}", i + 1, label);
        }
        let index = input_loop(actions.len());
        if index == 0 {
            println!("\n{}", game);
            continue;
        }
        let action = actions[index - 1].clone();
        game.handle_action(action).expect("handle action");
    }
}

fn main() {
    let mut game = Game::default();
    game.start();
    println!("Starting game...");
    game_loop(&mut game);
    println!("Game over!");
}
