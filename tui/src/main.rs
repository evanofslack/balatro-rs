mod app;
mod input;
mod library;
mod library_input;
mod paths;
mod record;
mod replay;
mod ui;

use anyhow::Result;
use app::AppState;
use balatro_rs::{config::Config, game::Game};
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use paths::{default_data_dir, default_record_path};
use ratatui::{backend::CrosstermBackend, Terminal};
use record::create as create_record;
use std::{fs, io, path::PathBuf, time::Duration};

#[derive(Parser)]
#[command(name = "balatro-tui")]
struct Args {
    #[arg(long, value_name = "FILE")]
    load: Option<String>,

    #[arg(long, value_name = "SEED")]
    seed: Option<String>,

    /// Watch a replay log. The same file works while it is still growing and after it ends.
    #[arg(long, value_name = "FILE", conflicts_with_all = ["load", "record", "seed", "library"])]
    replay: Option<PathBuf>,

    /// Record this run to a new replay log. With no PATH, writes under the replays directory.
    #[arg(
        long,
        value_name = "PATH",
        num_args = 0..=1,
        default_missing_value = "",
        conflicts_with_all = ["load", "replay", "library"]
    )]
    record: Option<String>,

    /// Open the replay library.
    #[arg(long, conflicts_with_all = ["load", "seed", "replay", "record"])]
    library: bool,

    /// Directory containing saves/ and replays/.
    #[arg(long, value_name = "DIR")]
    data_dir: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let data_dir = args.data_dir.clone().unwrap_or_else(default_data_dir);
    let replays_dir = data_dir.join("replays");

    let mut app = if let Some(path) = args.replay {
        let mut app = AppState::new(Game::new(Config::default()));
        app.replays_dir = replays_dir;
        app.data_dir = data_dir.clone();
        app.attach_replay(path, false)?;
        app
    } else if args.library {
        let mut app = AppState::new(Game::new(Config::default()));
        app.replays_dir = replays_dir;
        app.data_dir = data_dir.clone();
        app.open_library();
        app
    } else {
        let game = match &args.load {
            Some(path) => {
                let contents = fs::read_to_string(path)?;
                Game::from_json(&contents)?
            }
            None => {
                let mut config = Config::default();
                match &args.seed {
                    None => {}
                    Some(s) => match s.parse::<u64>() {
                        Ok(u) => config.seed = Some(u),
                        Err(_) => config.seed_str = Some(s.clone()),
                    },
                }
                let mut g = Game::new(config);
                g.start();
                g
            }
        };
        let mut app = AppState::new(game);
        app.replays_dir = replays_dir.clone();
        app.data_dir = data_dir.clone();
        if let Some(record) = args.record {
            let path = if record.is_empty() {
                fs::create_dir_all(&replays_dir)?;
                default_record_path(&replays_dir, app.game.seed)
            } else {
                PathBuf::from(record)
            };
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    fs::create_dir_all(parent)?;
                }
            }
            app.recorder = Some(create_record(&path, &app.game)?);
        }
        app
    };

    if app.screen == app::Screen::Playing {
        app.sync_focus_to_stage();
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal, &mut app);
    let finish_result = app.finish_recording();

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result?;
    finish_result
}

fn run<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut AppState) -> Result<()> {
    loop {
        if app.replay_autoplay_due() {
            app.replay_forward();
        }
        app.tick_replay_library_return();

        if app.clear_screen {
            terminal.clear()?;
            app.clear_screen = false;
        }

        terminal.draw(|f| ui::render(f, app))?;

        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(k) => input::handle_key(app, k),
                Event::Mouse(m) => input::handle_mouse(app, m),
                Event::Resize(_, _) => {}
                _ => {}
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
