mod agent;
mod app;
mod input;
mod library_input;
mod replay;
mod replay_transport;
mod ui;

use agent::{autoplay_step, finish_pending_action, replay_step, AgentSidecar};
use anyhow::{bail, Context, Result};
use app::{AppMode, AppState, REPLAY_COMPLETE_DELAY_MS};
use balatro_rs::{config::Config, game::Game};
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use library_input::{new_library_state, return_to_library};
use ratatui::{backend::CrosstermBackend, Terminal};
use replay::{default_replays_dir, load_replay};
use replay_transport::{attach_replay, replay_delay_ms, tick_rewind};
use std::{fs, io, path::PathBuf, time::Duration};

#[derive(Parser)]
#[command(name = "balatro-spectator")]
#[command(about = "Balatro spectator — replay library, recorded episodes, or live agent")]
struct Args {
    #[arg(long, value_name = "FILE")]
    load: Option<String>,

    #[arg(long, value_name = "SEED")]
    seed: Option<String>,

    #[arg(long, conflicts_with = "seed")]
    random: bool,

    #[arg(long, value_name = "N")]
    ante_end: Option<usize>,

    #[arg(long, value_name = "CMD")]
    agent: Option<String>,

    #[arg(long, value_name = "PATH")]
    checkpoint: Option<PathBuf>,

    #[arg(long)]
    expert: bool,

    #[arg(long, default_value_t = 400)]
    speed_ms: u64,

    #[arg(long)]
    paused: bool,

    #[arg(long, value_name = "FILE", conflicts_with = "library")]
    replay: Option<PathBuf>,

    #[arg(long, conflicts_with = "replay")]
    library: bool,

    #[arg(long, value_name = "DIR")]
    replays_dir: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let replays_dir = args.replays_dir.clone().unwrap_or_else(default_replays_dir);

    if args.library {
        return run_library(replays_dir, args.speed_ms);
    }

    if let Some(path) = &args.replay {
        return run_replay_file(path, replays_dir, args.speed_ms);
    }

    run_interactive(args)
}

fn run_library(replays_dir: PathBuf, speed_ms: u64) -> Result<()> {
    let mut app = AppState::new(Game::new(Config::default()));
    app.mode = AppMode::Library;
    app.replays_dir = replays_dir.clone();
    app.library = Some(new_library_state(replays_dir));
    app.speed_ms = speed_ms;
    eprintln!("replay library: j/k=move  [/]=page  Enter=watch  R=rename  r=refresh  d=delete  q=quit");

    run_tui_loop(&mut app, None)
}

fn run_replay_file(path: &PathBuf, replays_dir: PathBuf, speed_ms: u64) -> Result<()> {
    let replay = load_replay(path)?;
    let mut app = AppState::new(Game::new(Config::default()));
    app.replays_dir = replays_dir;
    app.speed_ms = speed_ms;
    attach_replay(&mut app, path.clone(), replay);

    eprintln!(
        "replay: {}  seed={}  transport bar at bottom",
        path.display(),
        app.game.seed
    );

    run_tui_loop(&mut app, None)
}

fn run_interactive(args: Args) -> Result<()> {
    let game = match &args.load {
        Some(path) => {
            let contents = fs::read_to_string(path)
                .with_context(|| format!("failed to read save file {path}"))?;
            Game::from_json(&contents).context("failed to parse save JSON")?
        }
        None => {
            let mut config = Config::default();
            let ante_end = args
                .ante_end
                .unwrap_or(if args.agent.is_some() { 1 } else { 8 });
            config.ante_end = ante_end;

            if let Some(s) = &args.seed {
                match s.parse::<u64>() {
                    Ok(u) => config.seed = Some(u),
                    Err(_) => config.seed_str = Some(s.clone()),
                }
            }

            let mut g = Game::new(config);
            g.start();
            g
        }
    };

    match &game.seed_str {
        Some(s) => eprintln!("seed={s} ({})", game.seed),
        None => eprintln!("seed={}", game.seed),
    }
    if args.agent.is_some() {
        eprintln!(
            "spectator: a=pause/resume  .=step while paused  q=quit  (speed={}ms)",
            args.speed_ms
        );
    }

    let mut agent = match &args.agent {
        Some(cmd) => {
            let expert = args.expert || cmd.contains("--expert");
            if expert {
                let full_cmd = if cmd.contains("--expert") {
                    cmd.clone()
                } else {
                    format!("{cmd} --expert")
                };
                eprintln!("agent={} (expert / flush heuristic)", full_cmd);
                Some(AgentSidecar::spawn_cmd(&full_cmd)?)
            } else {
                let checkpoint = resolve_checkpoint(args.checkpoint.as_ref())?;
                eprintln!("agent={} --checkpoint {}", cmd, checkpoint.display());
                Some(AgentSidecar::spawn(cmd, &checkpoint.to_string_lossy())?)
            }
        }
        None => None,
    };

    let mut app = AppState::new(game);
    if agent.is_some() {
        app.mode = AppMode::Spectating;
    } else {
        app.mode = AppMode::Manual;
    }
    app.replays_dir = args.replays_dir.clone().unwrap_or_else(default_replays_dir);
    app.agent_mode = agent.is_some();
    app.autoplay_paused = args.paused || agent.is_none();
    app.speed_ms = args.speed_ms;
    app.sync_focus_to_stage();

    run_tui_loop(&mut app, agent.as_mut())
}

fn run_tui_loop(app: &mut AppState, agent: Option<&mut AgentSidecar>) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, app, agent);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    result
}

fn resolve_checkpoint(explicit: Option<&PathBuf>) -> Result<PathBuf> {
    if let Some(p) = explicit {
        if !p.is_file() {
            bail!("checkpoint not found: {}", p.display());
        }
        return Ok(p.clone());
    }
    for candidate in [
        PathBuf::from("pylatro/gym/checkpoints/dagger_policy_028.pt"),
        PathBuf::from("pylatro/gym/checkpoints/dagger_policy_011.pt"),
    ] {
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    bail!("no --checkpoint given and no default dagger_policy checkpoint found");
}

fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut AppState,
    mut agent: Option<&mut AgentSidecar>,
) -> Result<()> {
    loop {
        if app.mode != AppMode::Library {
            if let Some(until) = app.highlight_until {
                if std::time::Instant::now() >= until {
                    finish_pending_action(app)?;
                }
            }

            if app.is_json_replay() {
                tick_rewind(app);

                let replay_done = app
                    .replay_driver
                    .as_ref()
                    .is_some_and(|d| d.finished())
                    && app.game.is_over();

                if replay_done {
                    if app.replay_complete_at.is_none() {
                        app.replay_complete_at = Some(std::time::Instant::now());
                        app.autoplay_paused = true;
                    } else if app
                        .replay_complete_at
                        .is_some_and(|t| t.elapsed().as_millis() >= REPLAY_COMPLETE_DELAY_MS)
                    {
                        return_to_library(app);
                        continue;
                    }
                }
            }
        }

        if app.clear_screen {
            terminal.clear()?;
            app.clear_screen = false;
        }

        terminal.draw(|f| ui::render(f, app))?;

        let speed_ms = if app.is_json_replay() {
            replay_delay_ms(app)
        } else {
            app.speed_ms
        };

        let poll_ms = if app.mode == AppMode::Library {
            16u64
        } else if app.pending_ai_action.is_some() {
            16u64
        } else if app.agent_mode && !app.autoplay_paused {
            speed_ms.min(50)
        } else {
            16
        };

        if event::poll(Duration::from_millis(poll_ms))? {
            match event::read()? {
                Event::Key(k) => {
                    if app.mode == AppMode::Library {
                        library_input::handle_library_key(app, k);
                    } else {
                        input::handle_key(app, k);
                    }
                }
                Event::Mouse(m) => {
                    if app.mode == AppMode::Library {
                        library_input::handle_library_mouse(app, m);
                    } else {
                        input::handle_mouse(app, m);
                    }
                }
                Event::Resize(_, _) => {
                    app.pending_ai_action = None;
                    app.ai_action = None;
                    app.highlight_until = None;
                }
                _ => {}
            }
        }

        if app.should_quit {
            break;
        }

        if app.mode == AppMode::Library {
            continue;
        }

        let want_step = app.agent_mode
            && app.pending_ai_action.is_none()
            && !app.game.is_over()
            && (!app.autoplay_paused || app.step_once)
            && !app
                .replay_chrome
                .as_ref()
                .is_some_and(|c| c.rewinding);

        if want_step {
            app.step_once = false;
            if app.replay_driver.is_some() {
                let mut driver = app.replay_driver.take().unwrap();
                let result = replay_step(app, &mut driver, speed_ms);
                app.replay_driver = Some(driver);
                if let Err(e) = result {
                    eprintln!("replay error: {e:#}");
                    app.autoplay_paused = true;
                }
            } else if let Some(sidecar) = agent.as_deref_mut() {
                if let Err(e) = autoplay_step(app, sidecar, speed_ms) {
                    eprintln!("autoplay error: {e:#}");
                    app.autoplay_paused = true;
                }
            }
        }
    }
    Ok(())
}
