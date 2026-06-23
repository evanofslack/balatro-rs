use crate::card::Card;
use crate::consumable::Consumable;
use crate::joker::Jokers;
use crate::pack::{Pack, PackContent};
use crate::stage::Blind;
#[cfg(feature = "python")]
use pyo3::pyclass;
use std::fmt;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum MoveDirection {
    Left,
    Right,
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
    CashOut(usize),
    BuyJoker(Jokers),
    BuyConsumable(Consumable),
    UseConsumable(Consumable),
    NextRound(),
    SelectBlind(Blind),
    // SkipBlind(Blind),
    ApplyTarot(),
    SellJoker(usize),
    SellConsumable(usize),
    BuyPack(Pack),
    PickPackCard(PackContent),
    SkipPack(),
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
            Self::MoveCard(dir, card) => {
                write!(f, "MoveCard: {} - {}", card, dir)
            }
            Self::CashOut(reward) => {
                write!(f, "CashOut: {}", reward)
            }
            Self::BuyJoker(joker) => {
                write!(f, "BuyJoker: {}", joker)
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
                write!(f, "SelectBlind: {}", blind)
            }
            Self::ApplyTarot() => write!(f, "ApplyTarot"),
            Self::SellJoker(idx) => write!(f, "SellJoker: {}", idx),
            Self::SellConsumable(idx) => write!(f, "SellConsumable: {}", idx),
            Self::BuyPack(pack) => write!(f, "BuyPack: {}", pack.name()),
            Self::PickPackCard(content) => write!(f, "PickPackCard: {}", content.name()),
            Self::SkipPack() => write!(f, "SkipPack"),
        }
    }
}

#[cfg(feature = "python")]
impl Action {
    fn __repr__(&self) -> String {
        format!("Action: {}", self)
    }
}
