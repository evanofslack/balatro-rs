#[cfg(feature = "python")]
use pyo3::prelude::*;

const DEFAULT_ROUND_START: usize = 0;
const DEFAULT_PLAYS: usize = 4;
const DEFAULT_DISCARDS: usize = 4;
const DEFAULT_MONEY_START: usize = 0;
const DEFAULT_MONEY_MAX: usize = 500;
const DEFAULT_REWARD_BASE: usize = 0;
const DEFAULT_MONEY_PER_HAND: usize = 1;
const DEFAULT_INTEREST_RATE: f32 = 0.2;
const DEFAULT_INTEREST_MAX: usize = 5;
const DEFAULT_BASE_MULT: usize = 0;
const DEFAULT_BASE_CHIPS: usize = 0;
const DEFAULT_BASE_SCORE: usize = 0;
const DEFAULT_ANTE_START: usize = 1;
const DEFAULT_ANTE_END: usize = 8;
const DEFAULT_JOKER_SLOTS: usize = 5;
const DEFAULT_JOKER_SLOTS_MAX: usize = 10;
const DEFAULT_AVAILABLE: usize = 8;
const DEFAULT_AVAILABLE_MAX: usize = 24; // arbitrary
const DEFAULT_STORE_CONSUMABLE_SLOTS_MAX: usize = 4;
const DEFAULT_DECK_MAX: usize = 100;
const DEFAULT_DISCARDED_MAX: usize = 100;
const DEFAULT_SELECTED_MAX: usize = 5;
const DEFAULT_CONSUMABLE_SLOTS: usize = 2;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass)]
#[derive(Debug, Clone)]
pub struct Config {
    pub round_start: usize,
    pub plays: usize,
    pub discards: usize,
    pub money_start: usize,
    pub money_max: usize,
    pub reward_base: usize,
    pub money_per_hand: usize,
    pub interest_rate: f32,
    pub interest_max: usize,
    pub base_mult: usize,
    pub base_chips: usize,
    pub base_score: usize,
    pub ante_start: usize,
    pub ante_end: usize,
    pub joker_slots: usize,
    pub joker_slots_max: usize,
    pub selected_max: usize,
    pub available: usize,
    pub available_max: usize,
    pub store_consumable_slots_max: usize,
    pub consumable_slots: usize,
    pub deck_max: usize,
    pub discarded_max: usize,
    pub seed: Option<u64>,
    pub seed_str: Option<String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            round_start: DEFAULT_ROUND_START,
            plays: DEFAULT_PLAYS,
            discards: DEFAULT_DISCARDS,
            money_start: DEFAULT_MONEY_START,
            money_max: DEFAULT_MONEY_MAX,
            reward_base: DEFAULT_REWARD_BASE,
            money_per_hand: DEFAULT_MONEY_PER_HAND,
            interest_rate: DEFAULT_INTEREST_RATE,
            interest_max: DEFAULT_INTEREST_MAX,
            base_mult: DEFAULT_BASE_MULT,
            base_chips: DEFAULT_BASE_CHIPS,
            base_score: DEFAULT_BASE_SCORE,
            ante_start: DEFAULT_ANTE_START,
            ante_end: DEFAULT_ANTE_END,
            joker_slots: DEFAULT_JOKER_SLOTS,
            joker_slots_max: DEFAULT_JOKER_SLOTS_MAX,
            selected_max: DEFAULT_SELECTED_MAX,
            available: DEFAULT_AVAILABLE,
            available_max: DEFAULT_AVAILABLE_MAX,
            store_consumable_slots_max: DEFAULT_STORE_CONSUMABLE_SLOTS_MAX,
            consumable_slots: DEFAULT_CONSUMABLE_SLOTS,
            deck_max: DEFAULT_DECK_MAX,
            discarded_max: DEFAULT_DISCARDED_MAX,
            seed: None,
            seed_str: None,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl Config {
    #[new]
    fn py_new() -> Self {
        Self::default()
    }

    #[getter]
    fn get_ante_end(&mut self) -> usize {
        self.ante_end
    }

    #[setter]
    fn set_ante_end(&mut self, i: usize) {
        self.ante_end = i;
    }

    #[getter]
    fn get_plays(&mut self) -> usize {
        self.plays
    }

    #[setter]
    fn set_plays(&mut self, i: usize) {
        self.plays = i;
    }

    #[getter]
    fn get_discards(&mut self) -> usize {
        self.discards
    }

    #[setter]
    fn set_discards(&mut self, i: usize) {
        self.discards = i;
    }

    #[getter]
    fn get_selected_max(&mut self) -> usize {
        self.selected_max
    }

    #[setter]
    fn set_selected_max(&mut self, i: usize) {
        self.selected_max = i;
    }

    #[getter]
    fn get_deck_max(&mut self) -> usize {
        self.deck_max
    }

    #[setter]
    fn set_deck_max(&mut self, i: usize) {
        self.deck_max = i;
    }

    #[getter]
    fn get_discarded_max(&mut self) -> usize {
        self.discarded_max
    }

    #[setter]
    fn set_discarded_max(&mut self, i: usize) {
        self.discarded_max = i;
    }

    #[getter]
    fn get_available_max(&mut self) -> usize {
        self.available_max
    }

    #[setter]
    fn set_available_max(&mut self, i: usize) {
        self.available_max = i;
    }

    #[getter]
    fn get_joker_slots(&mut self) -> usize {
        self.joker_slots
    }

    #[setter]
    fn set_joker_slots(&mut self, i: usize) {
        self.joker_slots = i;
    }

    #[getter]
    fn get_joker_slots_max(&mut self) -> usize {
        self.joker_slots_max
    }

    #[setter]
    fn set_joker_slots_max(&mut self, i: usize) {
        self.joker_slots_max = i;
    }

    #[getter]
    fn get_money_max(&mut self) -> usize {
        self.money_max
    }

    #[setter]
    fn set_money_max(&mut self, i: usize) {
        self.money_max = i;
    }
    #[getter]
    fn get_consumable_slots(&mut self) -> usize {
        self.consumable_slots
    }

    #[setter]
    fn set_consumable_slots(&mut self, i: usize) {
        self.consumable_slots = i;
    }

    #[getter]
    fn get_stage_max(&self) -> usize {
        9
    }

    #[getter]
    fn get_seed(&self) -> Option<u64> {
        self.seed
    }

    #[setter]
    fn set_seed(&mut self, v: Option<u64>) {
        self.seed = v;
    }

    #[getter]
    fn get_seed_str(&self) -> Option<String> {
        self.seed_str.clone()
    }

    #[setter]
    fn set_seed_str(&mut self, v: Option<String>) {
        self.seed_str = v;
    }
}
