mod app;
mod input;
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
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{fs, io, time::Duration};

#[derive(Parser)]
#[command(name = "balatro-tui")]
struct Args {
    #[arg(long, value_name = "FILE")]
    load: Option<String>,
    #[arg(long, value_name = "SEED")]
    seed: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let game = match args.load {
        Some(path) => {
            let contents = fs::read_to_string(&path)?;
            Game::from_json(&contents)?
        }
        None => {
            let mut config = Config::default();
            config.seed_str = args.seed;
            let mut g = Game::new(config);
            g.start();
            g
        }
    };

    // The game starts in PreBlind after start(), sync focus accordingly
    let mut app = AppState::new(game);
    app.sync_focus_to_stage();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut AppState) -> Result<()> {
    loop {
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
