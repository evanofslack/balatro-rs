use crate::game::Game;
use crate::hand::MadeHand;
use crate::joker::{Joker, Jokers};
use std::sync::{Arc, Mutex};

type GameHandFn = Arc<Mutex<dyn Fn(&mut Game, MadeHand) + Send + 'static>>;
type GameFn = Arc<Mutex<dyn Fn(&mut Game) + Send + 'static>>;
type GameModifyFn = Arc<Mutex<dyn Fn(&mut Game, &mut MadeHand) + Send + 'static>>;

#[derive(Debug, Clone)]
pub struct EffectRegistry {
    pub on_play: Vec<Effects>,
    pub on_discard: Vec<Effects>,
    pub on_score: Vec<Effects>,
    pub on_handrank: Vec<Effects>,
    pub on_modify_hand: Vec<Effects>,
}

impl EffectRegistry {
    pub fn new() -> Self {
        Self {
            on_play: Vec::new(),
            on_discard: Vec::new(),
            on_score: Vec::new(),
            on_handrank: Vec::new(),
            on_modify_hand: Vec::new(),
        }
    }
}

impl Default for EffectRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectRegistry {
    pub fn register_jokers(&mut self, jokers: Vec<Jokers>, game: &Game) {
        self.on_play.clear();
        self.on_discard.clear();
        self.on_score.clear();
        self.on_handrank.clear();
        self.on_modify_hand.clear();
        for j in jokers {
            for e in j.effects(game) {
                match e {
                    Effects::OnPlay(_) => self.on_play.push(e),
                    Effects::OnDiscard(_) => self.on_discard.push(e),
                    Effects::OnScore(_) => self.on_score.push(e),
                    Effects::OnHandRank(_) => self.on_handrank.push(e),
                    Effects::OnModifyHand(_) => self.on_modify_hand.push(e),
                }
            }
        }
    }
}

#[derive(Clone)]
// signature of these callbacks are more complicated so they
// can be used by pyo3 as part of python class.
pub enum Effects {
    OnPlay(GameHandFn),
    OnDiscard(GameHandFn),
    OnScore(GameHandFn),
    OnHandRank(GameFn),
    OnModifyHand(GameModifyFn),
}

impl std::fmt::Debug for Effects {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::OnPlay(_) => write!(f, "OnPlay"),
            Self::OnDiscard(_) => write!(f, "OnDiscard"),
            Self::OnScore(_) => write!(f, "OnScore"),
            Self::OnHandRank(_) => write!(f, "OnHandRank"),
            Self::OnModifyHand(_) => write!(f, "OnModifyHand"),
        }
    }
}
