use balatro_rs::action::Action;
use balatro_rs::game::Game;
use clap::Parser;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::io::{self, BufRead};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
struct Args {
    #[arg(long, value_name = "FILE")]
    load: Option<String>,
}

fn save_game(game: &Game) {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let path = format!("game_{}.json", ts);
    match game.to_json() {
        Ok(json) => match fs::write(&path, json) {
            Ok(_) => println!("Saved to {}", path),
            Err(e) => println!("Save failed: {}", e),
        },
        Err(e) => println!("Serialize failed: {}", e),
    }
}

fn read_input(max: usize) -> Option<usize> {
    let stdin = io::stdin();
    let line = stdin.lock().lines().next()?.ok()?;
    let trimmed = line.trim();
    if trimmed == "s" {
        return None;
    }
    trimmed.parse::<usize>().ok().filter(|&i| i <= max)
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
                    let cards: String =
                        selected
                            .iter()
                            .enumerate()
                            .fold(String::new(), |mut s, (j, c)| {
                                if j > 0 {
                                    s.push(' ');
                                }
                                let _ = write!(s, "{}", c);
                                s
                            });
                    let verb = if matches!(action, Action::Play()) {
                        "Play"
                    } else {
                        "Discard"
                    };
                    format!("{}: [{}]", verb, cards)
                }
                _ => format!("{}", action),
            };
            println!("[{}] {}", i + 1, label);
        }

        let index = loop {
            match read_input(actions.len()) {
                Some(i) => break i,
                None => {
                    save_game(game);
                    println!("Select action:");
                }
            }
        };

        if index == 0 {
            println!("\n{}", game);
            continue;
        }
        let action = actions[index - 1].clone();
        game.handle_action(action).expect("handle action");
    }
}

fn main() {
    let args = Args::parse();

    let mut game = match args.load {
        Some(path) => {
            let contents = fs::read_to_string(&path).expect("failed to read file");
            Game::from_json(&contents).expect("failed to parse game state")
        }
        None => {
            let mut g = Game::default();
            g.start();
            g
        }
    };

    println!("Starting game...");
    game_loop(&mut game);
    println!("Game over!");
}
