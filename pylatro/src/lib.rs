use balatro_rs::action::{Action, CardMask};
use balatro_rs::card::Card;
use balatro_rs::config::{Config, RngMode};
use balatro_rs::error::GameError;
use balatro_rs::game::Game;
use balatro_rs::joker::{JokerState, Jokers, Rarity, SelectorValue};
use balatro_rs::stage::{End, Stage};
use pyo3::prelude::*;

#[pyclass]
struct GameEngine {
    game: Game,
}

#[pymethods]
impl GameEngine {
    #[new]
    #[pyo3(signature = (config=None))]
    fn new(config: Option<Config>) -> Self {
        GameEngine {
            game: Game::new(config.unwrap_or_default()),
        }
    }

    fn gen_actions(&self) -> Vec<Action> {
        self.game.gen_actions().collect()
    }

    fn gen_action_space(&self) -> Vec<usize> {
        self.game.gen_action_space().to_vec()
    }

    fn handle_action(&mut self, action: Action) -> Result<(), GameError> {
        self.game.handle_action(action)
    }

    fn handle_action_index(&mut self, index: usize) -> Result<(), GameError> {
        self.game.handle_action_index(index)
    }

    fn clone(&self) -> Self {
        GameEngine {
            game: self.game.clone(),
        }
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
        self.game.discards
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
    fn score_log10(&self) -> f64 {
        self.game.score_log10()
    }
    #[getter]
    fn jokers(&self) -> Vec<Jokers> {
        self.game.jokers.clone()
    }
    #[getter]
    fn money(&self) -> usize {
        self.game.money
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
    m.add_class::<RngMode>()?;
    m.add_class::<CardMask>()?;
    m.add_class::<Rarity>()?;
    m.add_class::<JokerState>()?;
    m.add_class::<SelectorValue>()?;
    m.add_class::<Card>()?;
    m.add_class::<Jokers>()?;
    m.add_class::<Action>()?;
    m.add_function(wrap_pyfunction!(seed_from_str, m)?)?;
    Ok(())
}
