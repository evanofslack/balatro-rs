//! Run a policy outside the TUI and append its actions to the shared replay log.
use anyhow::{anyhow, bail, Context, Result};
use balatro_rs::config::Config;
use balatro_rs::game::Game;
use balatro_rs::replay_log::{action_line, header_line, outcome_line, terminal_line, ReplayResult};
use clap::Parser;
use serde_json::Value;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
#[command(about = "Run an external Balatro policy and write a live replay log")]
struct Args {
    /// Policy executable (for a Python script, use python3 and --agent-arg script.py).
    #[arg(long)]
    agent: PathBuf,

    /// Argument passed to the policy executable; repeat for multiple arguments.
    #[arg(long, value_name = "ARG", allow_hyphen_values = true)]
    agent_arg: Vec<String>,

    /// Existing checkpoint path, passed to the policy as --checkpoint PATH.
    #[arg(long)]
    checkpoint: Option<PathBuf>,

    /// Label shown in the replay library.
    #[arg(long)]
    agent_label: Option<String>,

    /// Numeric seed or Balatro seed string.
    #[arg(long)]
    seed: Option<String>,

    /// Final ante for this run.
    #[arg(long, default_value_t = 8)]
    ante_end: usize,

    /// Log path; defaults to a new file in the replays directory.
    #[arg(long)]
    record: Option<PathBuf>,

    /// Directory for a default log path (or BALATRO_REPLAYS_DIR).
    #[arg(long)]
    replays_dir: Option<PathBuf>,
}

struct Policy {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Policy {
    fn start(args: &Args) -> Result<Self> {
        if let Some(path) = &args.checkpoint {
            if !path.is_file() {
                bail!("checkpoint not found: {}", path.display());
            }
        }
        let mut command = Command::new(&args.agent);
        command.args(&args.agent_arg);
        if let Some(path) = &args.checkpoint {
            command.arg("--checkpoint").arg(path);
        }
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| format!("start policy {}", args.agent.display()))?;
        Ok(Self {
            stdin: child.stdin.take().context("policy stdin unavailable")?,
            stdout: BufReader::new(child.stdout.take().context("policy stdout unavailable")?),
            child,
        })
    }

    fn action(&mut self, game: &Game, mask: &[usize]) -> Result<usize> {
        // Keep the old sidecar's game JSON shape and add a legal-action mask.
        let mut state: Value = serde_json::from_str(&game.to_json().context("serialize game")?)?;
        state
            .as_object_mut()
            .context("game JSON is not an object")?
            .insert("action_mask".into(), serde_json::to_value(mask)?);
        writeln!(self.stdin, "{state}").context("send game to policy")?;
        self.stdin.flush().context("flush game to policy")?;

        let mut line = String::new();
        if self
            .stdout
            .read_line(&mut line)
            .context("read policy reply")?
            == 0
        {
            bail!("policy exited before replying");
        }
        let reply: Value = serde_json::from_str(&line).context("policy reply is not JSON")?;
        if let Some(error) = reply.get("error") {
            bail!("policy error: {error}");
        }
        let index = reply
            .get("action")
            .and_then(Value::as_u64)
            .and_then(|n| usize::try_from(n).ok())
            .ok_or_else(|| anyhow!("policy reply must contain a nonnegative integer action"))?;
        if mask.get(index) != Some(&1) {
            bail!("policy chose illegal action {index}");
        }
        Ok(index)
    }
}

impl Drop for Policy {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
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
            match dir.parent() {
                Some(parent) => dir = parent,
                None => break,
            }
        }
        return cwd.join("replays");
    }
    PathBuf::from("replays")
}

fn open_log(path: &Path) -> Result<BufWriter<File>> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
    }
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|| format!("create new replay log {}", path.display()))?;
    Ok(BufWriter::new(file))
}

fn run(args: Args) -> Result<PathBuf> {
    let mut config = Config {
        ante_end: args.ante_end,
        ..Config::default()
    };
    if let Some(seed) = &args.seed {
        match seed.parse::<u64>() {
            Ok(value) => config.seed = Some(value),
            Err(_) => config.seed_str = Some(seed.clone()),
        }
    }
    let mut game = Game::new(config);
    game.start();
    let mut policy = Policy::start(&args)?;
    let dir = args.replays_dir.clone().unwrap_or_else(default_replays_dir);
    let path = match args.record {
        Some(path) => path,
        None => {
            let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
            dir.join(format!("{}_agent_{nanos}.log", game.seed))
        }
    };
    let mut log = open_log(&path)?;
    let label = args.agent_label.as_deref().or_else(|| args.agent.to_str());
    writeln!(
        log,
        "{}",
        header_line(&game.config, game.seed, game.seed_str.as_deref(), label)?
    )?;
    log.flush().context("flush replay header")?;
    eprintln!("replay log: {}", path.display());

    let mut steps = 0;
    let episode = (|| -> Result<()> {
        while !game.is_over() {
            let mask = game.gen_action_space().to_vec();
            if !mask.contains(&1) {
                bail!("game has no legal actions at step {steps}");
            }
            let index = policy
                .action(&game, &mask)
                .with_context(|| format!("step {steps}"))?;
            game.handle_action_index(index)
                .map_err(|err| anyhow!("apply action {index} at step {steps}: {err:?}"))?;
            writeln!(log, "{}", action_line(index))?;
            log.flush().context("flush replay action")?;
            steps += 1;
        }
        Ok(())
    })();
    if let Err(err) = episode {
        writeln!(log, "{}", terminal_line(&game, steps, ReplayResult::Error)?)?;
        log.flush().context("flush replay error")?;
        return Err(err);
    }
    writeln!(log, "{}", outcome_line(&game, steps)?)?;
    log.flush().context("flush replay outcome")?;
    Ok(path)
}

fn main() -> Result<()> {
    let path = run(Args::parse())?;
    eprintln!("finished: {}", path.display());
    Ok(())
}
