use crate::core::effect::Effects;
use crate::core::game::Game;
use std::sync::Arc;
use strum::{EnumIter, IntoEnumIterator};

pub trait Joker: std::fmt::Debug + Clone {
    fn name(&self) -> String;
    fn desc(&self) -> String;
    fn rarity(&self) -> Rarity;
    fn categories(&self) -> Vec<Categories>;
    fn effects(&self, game: &Game) -> Vec<Effects>;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Categories {
    MultPlus,
    MultMult,
    Chips,
    Economy,
    Retrigger,
    Effect,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, EnumIter, Eq, PartialEq, Hash)]
pub enum Jokers {
    TheJoker(TheJoker),
    LustyJoker(LustyJoker),
}

impl Jokers {
    pub fn by_rarity(rarirty: Rarity) -> Vec<Self> {
        return Self::iter().filter(|j| j.rarity() == rarirty).collect();
    }
}

impl Joker for Jokers {
    fn name(&self) -> String {
        match self {
            Self::TheJoker(j) => j.name(),
            Self::LustyJoker(j) => j.name(),
        }
    }
    fn desc(&self) -> String {
        match self {
            Self::TheJoker(j) => j.desc(),
            Self::LustyJoker(j) => j.desc(),
        }
    }
    fn rarity(&self) -> Rarity {
        match self {
            Self::TheJoker(j) => j.rarity(),
            Self::LustyJoker(j) => j.rarity(),
        }
    }
    fn categories(&self) -> Vec<Categories> {
        match self {
            Self::TheJoker(j) => j.categories(),
            Self::LustyJoker(j) => j.categories(),
        }
    }
    fn effects(&self, game: &Game) -> Vec<Effects> {
        match self {
            Self::TheJoker(j) => j.effects(game),
            Self::LustyJoker(j) => j.effects(game),
        }
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TheJoker {}

impl Joker for TheJoker {
    fn name(&self) -> String {
        "Joker".to_string()
    }
    fn desc(&self) -> String {
        "+4 Mult".to_string()
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::MultPlus]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game) {
            g.mult += 4;
        }
        vec![Effects::OnScore(Arc::new(apply))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub struct LustyJoker {}

impl Joker for LustyJoker {
    fn name(&self) -> String {
        "LustyJoker".to_string()
    }
    fn desc(&self) -> String {
        "+4 Mult for each heart".to_string()
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::MultPlus]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game) {
            g.mult += 4;
        }
        vec![Effects::OnScore(Arc::new(apply))]
    }
}