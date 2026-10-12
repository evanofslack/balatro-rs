use balatro_rs::action::Action;
use balatro_rs::game::Game;
use balatro_rs::replay_log::{header_line, loggable_move, move_line, outcome_line, ReplayMove};
use clap::Parser;
use std::fmt::Write as FmtWrite;
use std::fs::{self, File};
use std::io::{self, BufRead, BufWriter, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
struct Args {
    #[arg(long, value_name = "FILE")]
    load: Option<String>,

    /// Append this run to a replay log. With no PATH, writes under the replays directory.
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
    replays_dir: Option<PathBuf>,
}

fn default_replays_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("BALATRO_REPLAYS_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(cwd) = std::env::current_dir() {
        let mut dir = cwd.as_path();
        loop {
            let candidate = dir.join("replays");
            if candidate.is_dir() {
                return candidate;
            }
            dir = match dir.parent() {
                Some(p) => p,
                None => break,
            };
        }
        return cwd.join("replays");
    }
    PathBuf::from("replays")
}

fn default_record_path(replays_dir: &PathBuf, seed: u64) -> PathBuf {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    replays_dir.join(format!("{seed}_{secs}.log"))
}

struct ReplayWriter {
    file: BufWriter<File>,
    steps: usize,
    wrote_outcome: bool,
}

impl ReplayWriter {
    fn create(path: &PathBuf, game: &Game) -> Self {
        let file = File::create(path).unwrap_or_else(|err| {
            panic!("create {}: {err}", path.display());
        });
        let mut file = BufWriter::new(file);
        let line = header_line(&game.config, game.seed, game.seed_str.as_deref(), None)
            .expect("encode replay header");
        writeln!(file, "{line}").expect("write replay header");
        file.flush().expect("flush replay header");
        Self {
            file,
            steps: 0,
            wrote_outcome: false,
        }
    }

    fn record_move(&mut self, step: ReplayMove) {
        writeln!(self.file, "{}", move_line(step)).expect("write replay move");
        self.file.flush().expect("flush replay move");
        self.steps += 1;
    }

    fn record_outcome(&mut self, game: &Game) {
        if self.wrote_outcome {
            return;
        }
        let line = outcome_line(game, self.steps).expect("encode replay outcome");
        writeln!(self.file, "{line}").expect("write replay outcome");
        self.file.flush().expect("flush replay outcome");
        self.wrote_outcome = true;
    }
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

fn game_loop(game: &mut Game, mut recorder: Option<ReplayWriter>) {
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
        let logged = loggable_move(game, &action);
        game.handle_action(action).expect("handle action");
        if let Some(recorder) = recorder.as_mut() {
            if let Some(step) = logged {
                recorder.record_move(step);
                if game.is_over() {
                    recorder.record_outcome(game);
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

    let recorder = args.record.map(|record| {
        let replays_dir = args.replays_dir.clone().unwrap_or_else(default_replays_dir);
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
        ReplayWriter::create(&path, &game)
    });

    println!("Starting game...");
    game_loop(&mut game, recorder);
    println!("Game over!");
}
