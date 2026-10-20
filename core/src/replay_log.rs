//! Line-delimited replay log and shared writer.
//!
//! Gym action indices cover the RL space. `DeselectCard` is legal in the TUI/CLI
//! move generator but not in that space, so it is logged as `{"deselect": slot}`.

use crate::action::Action;
use crate::config::Config;
use crate::error::GameError;
use crate::game::Game;
use crate::stage::End;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, Write};
use thiserror::Error;

pub const FORMAT_VERSION: u32 = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayHeader {
    pub format_version: u32,
    pub seed: ReplaySeed,
    pub config: Config,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    pub source: ReplaySource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ReplaySeed {
    Number(u64),
    Text(String),
}

impl std::fmt::Display for ReplaySeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{n}"),
            Self::Text(s) => f.write_str(s),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplaySource {
    Player,
    Model,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayOutcome {
    pub result: ReplayResult,
    pub score: usize,
    pub steps: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplayResult {
    Won,
    Lost,
    Abandoned,
    Error,
}

pub fn result_for_game(game: &Game) -> ReplayResult {
    match game.result() {
        Some(End::Win) => ReplayResult::Won,
        Some(_) => ReplayResult::Lost,
        None => ReplayResult::Abandoned,
    }
}

/// One state-changing step in the log: a gym index, or a hand-slot deselect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayMove {
    Action(usize),
    /// Index into `game.available.cards()` at the time of the deselect.
    Deselect(usize),
}

#[derive(Debug, Clone)]
pub enum ReplayRecord {
    Header(Box<ReplayHeader>),
    Move(ReplayMove),
    Outcome(ReplayOutcome),
}

#[derive(Debug, Error)]
pub enum ReplayLogError {
    #[error("invalid replay json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unrecognized replay line")]
    UnknownLine,
    #[error("unsupported replay format_version {0}")]
    BadVersion(u32),
}

pub fn header_line(
    config: &Config,
    seed: u64,
    seed_str: Option<&str>,
    source: ReplaySource,
    agent: Option<&str>,
) -> Result<String, serde_json::Error> {
    let mut config = config.clone();
    // The actual seed lives in the header alone, including for random starts.
    config.seed = None;
    config.seed_str = None;
    let header = ReplayHeader {
        format_version: FORMAT_VERSION,
        seed: seed_str
            .map(|s| ReplaySeed::Text(s.to_owned()))
            .unwrap_or(ReplaySeed::Number(seed)),
        config,
        agent: agent.map(str::to_string),
        source,
    };
    serde_json::to_string(&header)
}

pub fn action_line(index: usize) -> String {
    move_line(ReplayMove::Action(index))
}

pub fn deselect_line(slot: usize) -> String {
    move_line(ReplayMove::Deselect(slot))
}

pub fn move_line(step: ReplayMove) -> String {
    match step {
        ReplayMove::Action(index) => serde_json::json!({ "action": index }).to_string(),
        ReplayMove::Deselect(slot) => serde_json::json!({ "deselect": slot }).to_string(),
    }
}

pub fn outcome_line(game: &Game, steps: usize) -> Result<String, serde_json::Error> {
    terminal_line(game, steps, result_for_game(game))
}

pub fn terminal_line(
    game: &Game,
    steps: usize,
    result: ReplayResult,
) -> Result<String, serde_json::Error> {
    let outcome = ReplayOutcome {
        result,
        score: game.score,
        steps,
    };
    serde_json::to_string(&serde_json::json!({ "outcome": outcome }))
}

/// Gym index for `action`, or a hand-slot deselect when the action is not in
/// the fixed action space (TUI toggle-off).
pub fn loggable_move(game: &Game, action: &Action) -> Option<ReplayMove> {
    match action {
        Action::DeselectCard(card) => game
            .available
            .cards()
            .iter()
            .position(|c| c.id == card.id)
            .map(ReplayMove::Deselect),
        other => game
            .gen_action_space()
            .index_of(other, game)
            .map(ReplayMove::Action),
    }
}

pub fn apply_move(game: &mut Game, step: ReplayMove) -> Result<(), GameError> {
    match step {
        ReplayMove::Action(index) => game.handle_action_index(index),
        ReplayMove::Deselect(slot) => {
            let card = game
                .available
                .cards()
                .get(slot)
                .copied()
                .ok_or(GameError::InvalidAction)?;
            game.handle_action(Action::DeselectCard(card))
        }
    }
}

pub fn parse_line(line: &str) -> Result<ReplayRecord, ReplayLogError> {
    let value: Value = serde_json::from_str(line.trim())?;
    if let Some(version) = value.get("format_version") {
        let version = version.as_u64().ok_or(ReplayLogError::UnknownLine)? as u32;
        if version != FORMAT_VERSION {
            return Err(ReplayLogError::BadVersion(version));
        }
        let header: ReplayHeader = serde_json::from_value(value)?;
        return Ok(ReplayRecord::Header(Box::new(header)));
    }
    if let Some(action) = value.get("action") {
        let index = action.as_u64().ok_or(ReplayLogError::UnknownLine)? as usize;
        return Ok(ReplayRecord::Move(ReplayMove::Action(index)));
    }
    if let Some(deselect) = value.get("deselect") {
        let slot = deselect.as_u64().ok_or(ReplayLogError::UnknownLine)? as usize;
        return Ok(ReplayRecord::Move(ReplayMove::Deselect(slot)));
    }
    if let Some(outcome) = value.get("outcome") {
        let outcome: ReplayOutcome = serde_json::from_value(outcome.clone())?;
        return Ok(ReplayRecord::Outcome(outcome));
    }
    Err(ReplayLogError::UnknownLine)
}

/// Config that reproduces `header`. A run started with no seed still records
/// the seed `Game::new` actually used, and replay injects that seed.
pub fn config_for_replay(header: &ReplayHeader) -> Config {
    let mut config = header.config.clone();
    match &header.seed {
        ReplaySeed::Number(seed) => {
            config.seed = Some(*seed);
            config.seed_str = None;
        }
        ReplaySeed::Text(seed) => {
            config.seed = None;
            config.seed_str = Some(seed.clone());
        }
    }
    config
}

/// Flushes each record so another process can tail a live run.
pub struct ReplayWriter<W: Write> {
    writer: W,
    steps: usize,
    finished: bool,
}

impl<W: Write> ReplayWriter<W> {
    pub fn new(
        mut writer: W,
        game: &Game,
        source: ReplaySource,
        agent: Option<&str>,
    ) -> io::Result<Self> {
        writeln!(
            writer,
            "{}",
            header_line(
                &game.config,
                game.seed,
                game.seed_str.as_deref(),
                source,
                agent
            )?
        )?;
        writer.flush()?;
        Ok(Self {
            writer,
            steps: 0,
            finished: false,
        })
    }

    pub fn record_move(&mut self, step: ReplayMove) -> io::Result<()> {
        if self.finished {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "replay already finished",
            ));
        }
        writeln!(self.writer, "{}", move_line(step))?;
        self.writer.flush()?;
        self.steps += 1;
        Ok(())
    }

    pub fn record_outcome(&mut self, game: &Game, result: ReplayResult) -> io::Result<()> {
        if self.finished {
            return Ok(());
        }
        writeln!(self.writer, "{}", terminal_line(game, self.steps, result)?)?;
        self.writer.flush()?;
        self.finished = true;
        Ok(())
    }
}

pub fn game_from_header(header: &ReplayHeader) -> Game {
    let mut game = Game::new(config_for_replay(header));
    game.start();
    game
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stage::End;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    #[test]
    fn replay_log_recomputes_seeded_game() {
        let config = Config {
            seed: Some(10037),
            ante_end: 1,
            ..Config::default()
        };
        let mut game = Game::new(config.clone());
        let seed = game.seed;
        let seed_str = game.seed_str.clone();
        game.start();

        let mut lines = vec![header_line(
            &config,
            seed,
            seed_str.as_deref(),
            ReplaySource::Player,
            None,
        )
        .unwrap()];
        let mut rng = StdRng::seed_from_u64(1);
        let mut steps = 0usize;
        while !game.is_over() && steps < 20_000 {
            let space = game.gen_action_space();
            let mask = space.to_vec();
            let legal: Vec<usize> = mask
                .iter()
                .enumerate()
                .filter_map(|(index, bit)| (*bit == 1).then_some(index))
                .collect();
            assert!(!legal.is_empty(), "no legal action at step {steps}");
            let index = legal[rng.gen_range(0..legal.len())];
            let action = space.to_action(index, &game).expect("index maps to action");
            assert_eq!(space.index_of(&action, &game), Some(index));
            game.handle_action_index(index).expect("apply action");
            lines.push(action_line(index));
            steps += 1;
        }
        assert!(
            game.is_over(),
            "seeded game did not finish after {steps} steps ({:?})",
            game.stage
        );
        lines.push(outcome_line(&game, steps).unwrap());

        let mut records = lines.iter().map(|line| parse_line(line).unwrap());
        let ReplayRecord::Header(header) = records.next().unwrap() else {
            panic!("first line must be the header");
        };
        let mut replay = game_from_header(&header);
        let mut replay_steps = 0usize;
        let mut saw_outcome = false;
        for record in records {
            match record {
                ReplayRecord::Move(step) => {
                    apply_move(&mut replay, step).expect("replay move");
                    replay_steps += 1;
                }
                ReplayRecord::Outcome(outcome) => {
                    assert_eq!(
                        outcome.result,
                        if matches!(game.result(), Some(End::Win)) {
                            ReplayResult::Won
                        } else {
                            ReplayResult::Lost
                        }
                    );
                    assert_eq!(outcome.score, game.score);
                    assert_eq!(outcome.steps, steps);
                    saw_outcome = true;
                }
                ReplayRecord::Header(_) => panic!("unexpected extra header"),
            }
        }
        assert!(saw_outcome);
        assert_eq!(replay_steps, steps);
        assert_eq!(replay.score, game.score);
        assert_eq!(replay.is_over(), game.is_over());
        assert_eq!(replay.result(), game.result());
    }

    #[test]
    fn deselect_is_logged_and_replays() {
        let config = Config {
            seed: Some(10037),
            ante_end: 1,
            ..Config::default()
        };
        let mut game = Game::new(config.clone());
        game.start();
        game.handle_action_index(78).expect("select blind");

        let space = game.gen_action_space();
        let select = space
            .to_vec()
            .iter()
            .position(|&bit| bit == 1)
            .expect("select a card");
        let select_action = space.to_action(select, &game).unwrap();
        let Action::SelectCard(card) = select_action.clone() else {
            panic!("expected SelectCard");
        };
        assert_eq!(
            loggable_move(&game, &select_action),
            Some(ReplayMove::Action(select))
        );
        game.handle_action(select_action).unwrap();

        let deselect = Action::DeselectCard(card);
        let logged = loggable_move(&game, &deselect).expect("deselect is loggable");
        assert_eq!(logged, ReplayMove::Deselect(select));
        game.handle_action(deselect).unwrap();

        let line = move_line(logged);
        match parse_line(&line).unwrap() {
            ReplayRecord::Move(step) => assert_eq!(step, logged),
            other => panic!("expected Move, got {other:?}"),
        }

        let mut replay = Game::new(config);
        replay.start();
        apply_move(&mut replay, ReplayMove::Action(78)).unwrap();
        apply_move(&mut replay, ReplayMove::Action(select)).unwrap();
        apply_move(&mut replay, logged).unwrap();
        assert!(replay.available.selected().is_empty());
    }

    #[test]
    fn parse_line_rejects_unknown_version() {
        let line = r#"{"format_version":1,"seed":1,"config":{}}"#;
        let err = parse_line(line).unwrap_err();
        assert!(matches!(err, ReplayLogError::BadVersion(1)));
    }

    #[test]
    fn abandoned_line_is_readable() {
        let mut game = Game::new(Config::default());
        game.start();
        let line = terminal_line(&game, 0, ReplayResult::Abandoned).unwrap();
        let ReplayRecord::Outcome(outcome) = parse_line(&line).unwrap() else {
            panic!("expected outcome")
        };
        assert_eq!(outcome.result, ReplayResult::Abandoned);
    }

    #[test]
    fn headers_mark_source_explicitly() {
        let config = Config::default();
        for (agent, expected) in [
            (None, ReplaySource::Player),
            (Some("policy"), ReplaySource::Model),
            (Some("human label"), ReplaySource::Player),
        ] {
            let line = header_line(&config, 42, None, expected, agent).unwrap();
            let ReplayRecord::Header(header) = parse_line(&line).unwrap() else {
                panic!("expected header")
            };
            assert_eq!(header.source, expected);
            assert_eq!(header.config.seed, None);
        }
    }

    #[test]
    fn text_seed_replays_from_single_header_field() {
        let config = Config {
            seed_str: Some("TEST".into()),
            ..Config::default()
        };
        let mut original = Game::new(config);
        original.start();
        let line = header_line(
            &original.config,
            original.seed,
            original.seed_str.as_deref(),
            ReplaySource::Player,
            None,
        )
        .unwrap();
        let ReplayRecord::Header(header) = parse_line(&line).unwrap() else {
            panic!("header")
        };
        assert_eq!(header.seed, ReplaySeed::Text("TEST".into()));
        assert!(header.config.seed.is_none());
        assert!(header.config.seed_str.is_none());
        let replay = game_from_header(&header);
        assert_eq!(original.seed, replay.seed);
        assert_eq!(original.seed_str, replay.seed_str);
    }

    #[test]
    fn writer_flushes_a_live_run_and_finishes_once() {
        let mut game = Game::new(Config {
            seed: Some(42),
            ..Config::default()
        });
        game.start();
        let mut writer =
            ReplayWriter::new(Vec::new(), &game, ReplaySource::Model, Some("gym")).unwrap();
        writer.record_move(ReplayMove::Action(78)).unwrap();
        game.handle_action_index(78).unwrap();
        writer
            .record_outcome(&game, ReplayResult::Abandoned)
            .unwrap();
        writer
            .record_outcome(&game, ReplayResult::Abandoned)
            .unwrap();
        assert!(writer.record_move(ReplayMove::Action(78)).is_err());
        let text = String::from_utf8(writer.writer).unwrap();
        let lines: Vec<_> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        let ReplayRecord::Header(header) = parse_line(lines[0]).unwrap() else {
            panic!("header")
        };
        assert_eq!(header.source, ReplaySource::Model);
        assert_eq!(header.config.seed, None);
        let ReplayRecord::Outcome(outcome) = parse_line(lines[2]).unwrap() else {
            panic!("outcome")
        };
        assert_eq!(outcome.result, ReplayResult::Abandoned);
        assert_eq!(outcome.steps, 1);
    }
}
