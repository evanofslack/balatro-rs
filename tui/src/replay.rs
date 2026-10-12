//! Tail a replay log. A growing file and a finished file use the same reader.
//! Forward consumes the next complete line. A partial line stays buffered.
//! Back recomputes from the header through the moves already applied.

use anyhow::{anyhow, bail, Context, Result};
use balatro_rs::game::Game;
use balatro_rs::replay_log::{
    apply_move, game_from_header, parse_line, ReplayHeader, ReplayMove, ReplayRecord, ReplayResult,
    ReplaySource,
};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub const DEFAULT_SPEED_MS: u64 = 400;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayStep {
    Waiting,
    Applied,
    Finished,
}

pub struct ReplayReader {
    file: File,
    path: PathBuf,
    pending: Vec<u8>,
    header: ReplayHeader,
    applied: Vec<ReplayMove>,
    cursor: usize,
    outcome_seen: bool,
    result: Option<ReplayResult>,
}

impl ReplayReader {
    pub fn open(path: &Path) -> Result<(Self, Game)> {
        let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
        let mut reader = Self {
            file,
            path: path.to_path_buf(),
            pending: Vec::new(),
            header: placeholder_header(),
            applied: Vec::new(),
            cursor: 0,
            outcome_seen: false,
            result: None,
        };
        match reader.read_record()? {
            Some(ReplayRecord::Header(header)) => {
                let game = game_from_header(&header);
                reader.header = *header;
                Ok((reader, game))
            }
            Some(_) => bail!("replay file must start with a header line"),
            None => bail!("replay header is not available yet"),
        }
    }

    pub fn applied_len(&self) -> usize {
        self.cursor
    }

    pub fn is_finished(&self) -> bool {
        self.outcome_seen && self.cursor == self.applied.len()
    }

    pub fn result(&self) -> Option<ReplayResult> {
        self.result
    }

    pub fn source(&self) -> ReplaySource {
        self.header.source()
    }

    pub fn at_start(&self) -> bool {
        self.cursor == 0
    }

    /// Apply the next move already read, or the next complete line in the file.
    /// End of file with no newline-terminated record is [`ReplayStep::Waiting`].
    /// An outcome line, or a later forward after one, is [`ReplayStep::Finished`].
    pub fn forward(&mut self, game: &mut Game) -> Result<ReplayStep> {
        if self.cursor < self.applied.len() {
            let step = self.applied[self.cursor];
            apply_logged(game, step)?;
            self.cursor += 1;
            return Ok(ReplayStep::Applied);
        }
        if self.outcome_seen {
            return Ok(ReplayStep::Finished);
        }
        match self.read_record()? {
            None if self.path.exists() => Ok(ReplayStep::Waiting),
            None => {
                self.outcome_seen = true;
                Ok(ReplayStep::Finished)
            }
            Some(ReplayRecord::Move(step)) => {
                apply_logged(game, step)?;
                self.applied.push(step);
                self.cursor += 1;
                Ok(ReplayStep::Applied)
            }
            Some(ReplayRecord::Outcome(outcome)) => {
                self.outcome_seen = true;
                self.result = outcome.result();
                Ok(ReplayStep::Finished)
            }
            Some(ReplayRecord::Header(_)) => bail!("unexpected header in replay"),
        }
    }

    /// Rebuild the game one applied move earlier. The outcome line stays consumed,
    /// so a later forward can finish again without re-reading it.
    pub fn back(&mut self) -> Result<Game> {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
        let mut game = game_from_header(&self.header);
        for &step in &self.applied[..self.cursor] {
            apply_logged(&mut game, step)?;
        }
        Ok(game)
    }

    fn read_record(&mut self) -> Result<Option<ReplayRecord>> {
        loop {
            if let Some(pos) = self.pending.iter().position(|byte| *byte == b'\n') {
                let raw: Vec<u8> = self.pending.drain(..=pos).collect();
                let text = String::from_utf8_lossy(&raw);
                let text = text.trim();
                if text.is_empty() {
                    continue;
                }
                return Ok(Some(parse_line(text)?));
            }
            let mut buf = [0u8; 4096];
            let n = self.file.read(&mut buf)?;
            if n == 0 {
                return Ok(None);
            }
            self.pending.extend_from_slice(&buf[..n]);
        }
    }
}

fn apply_logged(game: &mut Game, step: ReplayMove) -> Result<()> {
    apply_move(game, step).map_err(|err| match step {
        ReplayMove::Action(index) => anyhow!("action {index}: {err:?}"),
        ReplayMove::Deselect(slot) => anyhow!("deselect {slot}: {err:?}"),
    })
}

fn placeholder_header() -> ReplayHeader {
    ReplayHeader {
        format_version: 0,
        seed: 0,
        seed_str: None,
        config: balatro_rs::config::Config::default(),
        agent: None,
        source: None,
    }
}

pub const REPLAY_COMPLETE_DELAY_MS: u64 = 1500;

pub struct ReplaySession {
    pub reader: ReplayReader,
    pub paused: bool,
    pub speed_ms: u64,
    pub waiting: bool,
    pub last_step: Instant,
    pub from_library: bool,
    pub finished_at: Option<Instant>,
}

impl ReplaySession {
    pub fn new(reader: ReplayReader) -> Self {
        Self {
            reader,
            paused: false,
            speed_ms: DEFAULT_SPEED_MS,
            waiting: true,
            last_step: Instant::now(),
            from_library: false,
            finished_at: None,
        }
    }

    pub fn from_library(reader: ReplayReader) -> Self {
        let mut session = Self::new(reader);
        session.from_library = true;
        session
    }

    pub fn autoplay_due(&self) -> bool {
        if self.paused || self.reader.is_finished() {
            return false;
        }
        self.waiting || self.last_step.elapsed().as_millis() as u64 >= self.speed_ms
    }

    pub fn library_return_due(&self) -> bool {
        self.from_library
            && self
                .finished_at
                .is_some_and(|t| t.elapsed().as_millis() as u64 >= REPLAY_COMPLETE_DELAY_MS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use balatro_rs::action::Action;
    use balatro_rs::config::Config;
    use balatro_rs::replay_log::{deselect_line, header_line, move_line, outcome_line, ReplayMove};
    use std::io::Write;

    #[test]
    fn tail_waits_applies_then_stops_on_outcome() {
        let path = std::env::temp_dir().join(format!(
            "balatro-replay-{}-{}.log",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        let config = Config {
            seed: Some(10037),
            ante_end: 1,
            ..Config::default()
        };
        let mut recorded = Game::new(config);
        recorded.start();
        let space = recorded.gen_action_space();
        let index = space
            .to_vec()
            .iter()
            .position(|&bit| bit == 1)
            .expect("legal action");

        let mut file = File::create(&path).unwrap();
        writeln!(
            file,
            "{}",
            header_line(
                &recorded.config,
                recorded.seed,
                recorded.seed_str.as_deref(),
                None
            )
            .unwrap()
        )
        .unwrap();
        file.flush().unwrap();

        let (mut reader, mut game) = ReplayReader::open(&path).unwrap();
        assert_eq!(reader.forward(&mut game).unwrap(), ReplayStep::Waiting);

        write!(file, "{{\"action\":").unwrap();
        file.flush().unwrap();
        assert_eq!(
            reader.forward(&mut game).unwrap(),
            ReplayStep::Waiting,
            "a line without a newline is not consumed"
        );

        writeln!(file, "{index}}}").unwrap();
        file.flush().unwrap();
        assert_eq!(reader.forward(&mut game).unwrap(), ReplayStep::Applied);
        assert_eq!(reader.applied_len(), 1);

        recorded.handle_action_index(index).unwrap();
        writeln!(file, "{}", outcome_line(&recorded, 1).unwrap()).unwrap();
        file.flush().unwrap();
        assert_eq!(reader.forward(&mut game).unwrap(), ReplayStep::Finished);
        assert!(reader.is_finished());
        assert_eq!(reader.forward(&mut game).unwrap(), ReplayStep::Finished);
        assert_eq!(reader.applied_len(), 1);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn deselect_between_selects_replays() {
        let path = std::env::temp_dir().join(format!(
            "balatro-deselect-{}-{}.log",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        let config = Config {
            seed: Some(10037),
            ante_end: 1,
            ..Config::default()
        };
        let mut recorded = Game::new(config.clone());
        recorded.start();
        recorded.handle_action_index(78).unwrap();
        let space = recorded.gen_action_space();
        let select = space
            .to_vec()
            .iter()
            .position(|&bit| bit == 1)
            .expect("select");
        let Action::SelectCard(card) = space.to_action(select, &recorded).unwrap() else {
            panic!("expected SelectCard");
        };
        recorded.handle_action(Action::SelectCard(card)).unwrap();
        recorded.handle_action(Action::DeselectCard(card)).unwrap();
        recorded.handle_action(Action::SelectCard(card)).unwrap();

        let mut file = File::create(&path).unwrap();
        writeln!(
            file,
            "{}",
            header_line(
                &recorded.config,
                recorded.seed,
                recorded.seed_str.as_deref(),
                None
            )
            .unwrap()
        )
        .unwrap();
        writeln!(file, "{}", move_line(ReplayMove::Action(78))).unwrap();
        writeln!(file, "{}", move_line(ReplayMove::Action(select))).unwrap();
        writeln!(file, "{}", deselect_line(select)).unwrap();
        writeln!(file, "{}", move_line(ReplayMove::Action(select))).unwrap();
        file.flush().unwrap();

        let (mut reader, mut game) = ReplayReader::open(&path).unwrap();
        assert_eq!(reader.forward(&mut game).unwrap(), ReplayStep::Applied);
        assert_eq!(reader.forward(&mut game).unwrap(), ReplayStep::Applied);
        assert_eq!(reader.forward(&mut game).unwrap(), ReplayStep::Applied);
        assert!(game.available.selected().is_empty());
        assert_eq!(reader.forward(&mut game).unwrap(), ReplayStep::Applied);
        assert_eq!(game.available.selected().len(), 1);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn deleted_growing_log_finishes_for_existing_viewer() {
        let path = std::env::temp_dir().join(format!(
            "balatro-deleted-{}-{}.log",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut game = Game::new(Config {
            seed: Some(42),
            ..Config::default()
        });
        game.start();
        std::fs::write(
            &path,
            format!(
                "{}\n",
                header_line(&game.config, game.seed, None, None).unwrap()
            ),
        )
        .unwrap();
        let (mut reader, mut replay_game) = ReplayReader::open(&path).unwrap();
        assert_eq!(
            reader.forward(&mut replay_game).unwrap(),
            ReplayStep::Waiting
        );
        std::fs::remove_file(&path).unwrap();
        assert_eq!(
            reader.forward(&mut replay_game).unwrap(),
            ReplayStep::Finished
        );
        assert!(reader.is_finished());
    }
}
