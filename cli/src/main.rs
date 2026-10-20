use balatro_rs::action::Action;
use balatro_rs::game::Game;
use balatro_rs::replay_log::{
    loggable_move, result_for_game, ReplayResult, ReplaySource, ReplayWriter,
};
use clap::Parser;
use std::fmt::Write as FmtWrite;
use std::fs::{self, File};
use std::io::{self, BufRead, BufWriter};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
struct Args {
    #[arg(long, value_name = "FILE")]
    load: Option<String>,

    /// Record this run to a new replay log. With no PATH, writes under the replays directory.
    /// The menu number is not the gym index stored in the file.
    #[arg(
        long,
        value_name = "PATH",
        num_args = 0..=1,
        default_missing_value = "",
        conflicts_with = "load"
    )]
    record: Option<String>,

    /// Directory for default `--record` paths.
    #[arg(long, value_name = "DIR")]
    data_dir: Option<PathBuf>,
}

fn default_record_path(replays_dir: &PathBuf, seed: u64) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    replays_dir.join(format!("{seed}_{nanos}.log"))
}

fn save_game(game: &Game, data_dir: &PathBuf) {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let path = data_dir.join("saves").join(format!("game_{}.json", ts));
    if let Err(e) = fs::create_dir_all(data_dir.join("saves")) {
        println!("Save failed: {e}");
        return;
    }
    match game.to_json() {
        Ok(json) => match fs::write(&path, json) {
            Ok(_) => println!("Saved to {}", path.display()),
            Err(e) => println!("Save failed: {}", e),
        },
        Err(e) => println!("Serialize failed: {}", e),
    }
}

enum Input {
    Action(usize),
    Save,
    Quit,
    Invalid,
}

fn read_input(max: usize) -> Input {
    let stdin = io::stdin();
    let Some(Ok(line)) = stdin.lock().lines().next() else {
        return Input::Quit;
    };
    let trimmed = line.trim();
    if trimmed == "s" {
        return Input::Save;
    }
    if trimmed == "q" {
        return Input::Quit;
    }
    match trimmed.parse::<usize>() {
        Ok(i) if i <= max => Input::Action(i),
        _ => Input::Invalid,
    }
}

fn game_loop(
    game: &mut Game,
    mut recorder: Option<ReplayWriter<BufWriter<File>>>,
    data_dir: &PathBuf,
) -> bool {
    loop {
        if game.is_over() {
            if let Some(writer) = recorder.as_mut() {
                writer
                    .record_outcome(game, result_for_game(game))
                    .expect("finish replay");
            }
            return true;
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
                Input::Action(i) => break i,
                Input::Save => {
                    save_game(game, data_dir);
                    println!("Select action:");
                }
                Input::Invalid => println!("Enter an action number, s to save, or q to quit."),
                Input::Quit => {
                    if let Some(writer) = recorder.as_mut() {
                        writer
                            .record_outcome(game, ReplayResult::Abandoned)
                            .expect("finish replay");
                    }
                    return false;
                }
            }
        };

        if index == 0 {
            println!("\n{}", game);
            continue;
        }
        let action = actions[index - 1].clone();
        let logged = loggable_move(game, &action);
        game.handle_action(action).expect("handle action");
        if let Some(recorder) = recorder.as_mut() {
            if let Some(step) = logged {
                recorder.record_move(step).expect("write replay move");
                if game.is_over() {
                    recorder
                        .record_outcome(game, result_for_game(game))
                        .expect("finish replay");
                }
            } else {
                eprintln!("replay log: action succeeded but has no log encoding");
            }
        }
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

    let data_dir = args.data_dir.unwrap_or_else(|| PathBuf::from("data"));
    let recorder = args.record.map(|record| {
        let replays_dir = data_dir.join("replays");
        let path = if record.is_empty() {
            fs::create_dir_all(&replays_dir).expect("create replays dir");
            default_record_path(&replays_dir, game.seed)
        } else {
            let path = PathBuf::from(record);
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    fs::create_dir_all(parent).expect("create record parent");
                }
            }
            path
        };
        println!("recording to {}", path.display());
        ReplayWriter::new(
            BufWriter::new(
                File::options()
                    .write(true)
                    .create_new(true)
                    .open(&path)
                    .expect("create replay"),
            ),
            &game,
            ReplaySource::Player,
            None,
        )
        .expect("write replay header")
    });

    println!("Starting game...");
    if game_loop(&mut game, recorder, &data_dir) {
        println!("Game over!");
    } else {
        println!("Run abandoned.");
    }
}
