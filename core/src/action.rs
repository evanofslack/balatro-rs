use crate::card::Card;
use crate::consumable::Consumable;
use crate::joker::{joker_display, Jokers};
use crate::pack::{Pack, PackContent};
use crate::stage::{blind_display, Blind};
#[cfg(feature = "python")]
use pyo3::prelude::*;
use std::fmt;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum MoveDirection {
    Left,
    Right,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum SortBy {
    Rank,
    Suit,
}

/// A bitmask over hand-position indices into `Available`.
/// Positions not encoded, retain from move actions.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct CardMask(pub u32);

impl CardMask {
    pub fn from_positions(positions: &[usize]) -> Self {
        let mut mask = 0u32;
        for &p in positions {
            if p < 32 {
                mask |= 1 << p;
            }
        }
        CardMask(mask)
    }

    pub fn contains(&self, i: usize) -> bool {
        i < 32 && (self.0 >> i) & 1 == 1
    }

    pub fn count(&self) -> u32 {
        self.0.count_ones()
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl CardMask {
    #[staticmethod]
    #[pyo3(name = "from_positions")]
    fn py_from_positions(positions: Vec<usize>) -> Self {
        Self::from_positions(&positions)
    }
}

impl fmt::Display for MoveDirection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Left => {
                write!(f, "left")
            }
            Self::Right => {
                write!(f, "right")
            }
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Action {
    SelectCard(Card),
    DeselectCard(Card),
    MoveCard(MoveDirection, Card),
    Play(),
    Discard(),
    PlayHand(CardMask),
    DiscardHand(CardMask),
    CashOut(usize),
    BuyJoker(Jokers),
    BuyConsumable(Consumable),
    UseConsumable(Consumable),
    NextRound(),
    SelectBlind(Blind),
    SkipBlind(Blind),
    ApplyTarot(),
    SellJoker(usize),
    SellConsumable(usize),
    BuyPack(Pack),
    PickPackCard(PackContent),
    SkipPack(),
    SortHand(SortBy),
    Reroll(),
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SelectCard(card) => {
                write!(f, "SelectCard: {}", card)
            }
            Self::DeselectCard(card) => {
                write!(f, "DeselectCard: {}", card)
            }
            Self::Play() => {
                write!(f, "Play")
            }
            Self::Discard() => {
                write!(f, "Discard")
            }
            Self::PlayHand(mask) => {
                write!(f, "PlayHand: {:#010x}", mask.0)
            }
            Self::DiscardHand(mask) => {
                write!(f, "DiscardHand: {:#010x}", mask.0)
            }
            Self::MoveCard(dir, card) => {
                write!(f, "MoveCard: {} - {}", card, dir)
            }
            Self::CashOut(reward) => {
                write!(f, "CashOut: {}", reward)
            }
            Self::BuyJoker(joker) => {
                write!(f, "BuyJoker: {}", joker_display(joker))
            }
            Self::BuyConsumable(consumable) => {
                write!(f, "BuyConsumable: {}", consumable.name())
            }
            Self::UseConsumable(consumable) => {
                write!(f, "UseConsumable: {}", consumable.name())
            }
            Self::NextRound() => {
                write!(f, "NextRound")
            }
            Self::SelectBlind(blind) => {
                write!(f, "SelectBlind: {}", blind_display(blind))
            }
            Self::SkipBlind(blind) => {
                write!(f, "SkipBlind: {}", blind_display(blind))
            }
            Self::ApplyTarot() => write!(f, "ApplyTarot"),
            Self::SellJoker(idx) => write!(f, "SellJoker: {}", idx),
            Self::SellConsumable(idx) => write!(f, "SellConsumable: {}", idx),
            Self::BuyPack(pack) => write!(f, "BuyPack: {}", pack.name()),
            Self::PickPackCard(content) => write!(f, "PickPackCard: {}", content.name()),
            Self::SkipPack() => write!(f, "SkipPack"),
            Self::SortHand(sort_by) => write!(f, "SortHand: {}", sort_by),
            Self::Reroll() => write!(f, "Reroll"),
        }
    }
}

impl fmt::Display for SortBy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Rank => write!(f, "rank"),
            Self::Suit => write!(f, "suit"),
        }
    }
}

#[cfg(feature = "python")]
impl Action {
    fn __repr__(&self) -> String {
        format!("Action: {}", self)
    }
}
