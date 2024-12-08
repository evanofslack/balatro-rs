use crate::core::game::Game;
use crate::core::joker::{Joker, Jokers};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct EffectRegistry {
    pub on_play: Vec<Effects>,
    pub on_discard: Vec<Effects>,
    pub on_score: Vec<Effects>,
    pub on_handrank: Vec<Effects>,
}

impl EffectRegistry {
    pub fn new() -> Self {
        return Self {
            on_play: Vec::new(),
            on_discard: Vec::new(),
            on_score: Vec::new(),
            on_handrank: Vec::new(),
        };
    }
    pub fn register_jokers(&mut self, jokers: Vec<Jokers>, game: &Game) {
        for j in jokers.clone() {
            for e in j.effects(game) {
                match e {
                    Effects::OnPlay(_) => self.on_play.push(e),
                    Effects::OnDiscard(_) => self.on_discard.push(e),
                    Effects::OnScore(_) => self.on_score.push(e),
                    Effects::OnHandRank(_) => self.on_handrank.push(e),
                }
            }
        }
    }
}

#[derive(Clone)]
pub enum Effects {
    OnPlay(Arc<dyn Fn(&mut Game)>),
    OnDiscard(Arc<dyn Fn(&mut Game)>),
    OnScore(Arc<dyn Fn(&mut Game)>),
    OnHandRank(Arc<dyn Fn(&mut Game)>),
}

impl std::fmt::Debug for Effects {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::OnPlay(_) => write!(f, "OnPlay"),
            Self::OnDiscard(_) => write!(f, "OnDiscard"),
            Self::OnScore(_) => write!(f, "OnScore"),
            Self::OnHandRank(_) => write!(f, "OnHandRank"),
        }
    }
}
