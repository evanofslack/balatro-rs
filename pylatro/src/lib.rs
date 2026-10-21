use balatro_rs::action::Action;
use balatro_rs::card::Card;
use balatro_rs::config::Config;
use balatro_rs::game::Game;
use balatro_rs::joker::Jokers;
use balatro_rs::replay_log::{self, ReplayMove, ReplaySource, ReplayWriter};
use balatro_rs::stage::{End, Stage};
use balatro_rs::voucher::Voucher;
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use std::fs::{File, OpenOptions};
use std::io::BufWriter;

#[pyclass]
struct GameEngine {
    game: Game,
    recorder: Option<ReplayWriter<BufWriter<File>>>,
}

#[pymethods]
impl GameEngine {
    #[new]
    #[pyo3(signature = (config=None))]
    fn new(config: Option<Config>) -> Self {
        GameEngine {
            game: Game::new(config.unwrap_or_default()),
            recorder: None,
        }
    }

    fn replay_header_line(&self) -> PyResult<String> {
        replay_log::header_line(
            &self.game.config,
            self.game.seed,
            self.game.seed_str.as_deref(),
            ReplaySource::Model,
            Some("pylatro"),
        )
        .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    fn replay_outcome_line(&self, steps: usize) -> PyResult<String> {
        if !self.game.is_over() {
            return Err(PyValueError::new_err("game has no outcome yet"));
        }
        replay_log::outcome_line(&self.game, steps).map_err(|e| PyIOError::new_err(e.to_string()))
    }

    fn start_recording(&mut self, path: &str) -> PyResult<()> {
        self.finish_recording()?;
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| PyIOError::new_err(e.to_string()))?;
        self.recorder = Some(
            ReplayWriter::new(
                BufWriter::new(file),
                &self.game,
                ReplaySource::Model,
                Some("pylatro"),
            )
            .map_err(|e| PyIOError::new_err(e.to_string()))?,
        );
        Ok(())
    }

    fn finish_recording(&mut self) -> PyResult<()> {
        if let Some(mut recorder) = self.recorder.take() {
            recorder
                .record_outcome(&self.game, replay_log::result_for_game(&self.game))
                .map_err(|e| PyIOError::new_err(e.to_string()))?;
        }
        Ok(())
    }

    fn gen_actions(&self) -> Vec<Action> {
        self.game.gen_actions().collect()
    }

    fn gen_action_space(&self) -> Vec<usize> {
        self.game.gen_action_space().to_vec()
    }

    fn handle_action(&mut self, action: Action) -> PyResult<()> {
        let step = if self.recorder.is_some() {
            Some(
                replay_log::loggable_move(&self.game, &action)
                    .ok_or_else(|| PyIOError::new_err("action has no replay encoding"))?,
            )
        } else {
            None
        };
        self.game.handle_action(action)?;
        if let (Some(recorder), Some(step)) = (self.recorder.as_mut(), step) {
            recorder
                .record_move(step)
                .map_err(|e| PyIOError::new_err(e.to_string()))?;
        }
        Ok(())
    }

    fn handle_action_index(&mut self, index: usize) -> PyResult<()> {
        self.game.handle_action_index(index)?;
        if let Some(recorder) = self.recorder.as_mut() {
            recorder
                .record_move(ReplayMove::Action(index))
                .map_err(|e| PyIOError::new_err(e.to_string()))?;
        }
        Ok(())
    }

    #[getter]
    fn state(&self) -> GameState {
        GameState {
            game: self.game.clone(),
        }
    }
    #[getter]
    fn is_over(&self) -> bool {
        self.game.is_over()
    }
    #[getter]
    fn is_win(&self) -> bool {
        self.game.result() == Some(End::Win)
    }
}

#[pyclass]
struct GameState {
    game: Game,
}

#[pymethods]
impl GameState {
    #[getter]
    fn stage(&self) -> Stage {
        self.game.stage
    }
    #[getter]
    fn round(&self) -> usize {
        self.game.round
    }
    #[getter]
    fn action_history(&self) -> Vec<Action> {
        self.game.action_history.clone()
    }
    #[getter]
    fn deck(&self) -> Vec<Card> {
        self.game.deck.cards()
    }
    #[getter]
    fn selected(&self) -> Vec<Card> {
        self.game.available.selected()
    }
    #[getter]
    fn available(&self) -> Vec<Card> {
        self.game.available.cards()
    }
    #[getter]
    fn discarded(&self) -> Vec<Card> {
        self.game.discarded.clone()
    }
    #[getter]
    fn plays(&self) -> usize {
        self.game.plays
    }
    #[getter]
    fn discards(&self) -> usize {
        self.game.discards()
    }

    #[getter]
    fn score(&self) -> usize {
        self.game.score
    }
    #[getter]
    fn required_score(&self) -> usize {
        self.game.required_score()
    }
    #[getter]
    fn jokers(&self) -> Vec<Jokers> {
        self.game.jokers.clone()
    }
    #[getter]
    fn money(&self) -> usize {
        self.game.money
    }
    /// Vouchers redeemed this run, in purchase order.
    #[getter]
    fn vouchers(&self) -> Vec<Voucher> {
        self.game.vouchers.owned().to_vec()
    }
    /// The voucher this ante's shop is offering, if it's unsold.
    #[getter]
    fn shop_voucher(&self) -> Option<Voucher> {
        self.game.shop.voucher
    }

    #[getter]
    fn seed(&self) -> u64 {
        self.game.seed
    }

    #[getter]
    fn seed_str(&self) -> Option<String> {
        self.game.seed_str.clone()
    }

    fn __repr__(&self) -> String {
        format!("GameState:\n{}", self.game)
    }
}

#[pyfunction]
fn seed_from_str(s: &str) -> u64 {
    balatro_rs::seed_from_str(s)
}

#[pymodule]
fn pylatro(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Config>()?;
    m.add_class::<GameEngine>()?;
    m.add_class::<GameState>()?;
    m.add_class::<Stage>()?;
    m.add_class::<Voucher>()?;
    m.add_function(wrap_pyfunction!(seed_from_str, m)?)?;
    Ok(())
}
