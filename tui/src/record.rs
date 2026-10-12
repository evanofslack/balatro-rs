//! Append-only replay log. The file handle stays in the TUI; core only formats lines.

use anyhow::{Context, Result};
use balatro_rs::game::Game;
use balatro_rs::replay_log::{header_line, move_line, outcome_line, ReplayMove};
use std::fs::{self, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

pub struct ReplayWriter {
    path: PathBuf,
    file: BufWriter<std::fs::File>,
    steps: usize,
    wrote_outcome: bool,
}

impl ReplayWriter {
    pub fn create(path: &Path, game: &Game) -> Result<Self> {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .with_context(|| format!("create new replay log {}", path.display()))?;
        let mut file = BufWriter::new(file);
        let line = header_line(&game.config, game.seed, game.seed_str.as_deref(), None)
            .context("encode replay header")?;
        writeln!(file, "{line}").context("write replay header")?;
        file.flush().context("flush replay header")?;
        Ok(Self {
            path: path.to_path_buf(),
            file,
            steps: 0,
            wrote_outcome: false,
        })
    }

    pub fn record_move(&mut self, step: ReplayMove) -> Result<()> {
        writeln!(self.file, "{}", move_line(step))?;
        self.file.flush()?;
        self.steps += 1;
        Ok(())
    }

    pub fn record_outcome(&mut self, game: &Game) -> Result<()> {
        if self.wrote_outcome {
            return Ok(());
        }
        let line = outcome_line(game, self.steps).context("encode replay outcome")?;
        writeln!(self.file, "{line}")?;
        self.file.flush()?;
        self.wrote_outcome = true;
        Ok(())
    }

    pub fn discard(self) -> Result<()> {
        let path = self.path.clone();
        drop(self);
        match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => {
                Err(err).with_context(|| format!("delete abandoned replay {}", path.display()))
            }
        }
    }
}
