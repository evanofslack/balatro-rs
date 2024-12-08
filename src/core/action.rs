use crate::core::card::Card;
use crate::core::hand::SelectHand;
use crate::core::joker::Jokers;
use crate::core::stage::Blind;
use std::fmt;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Action {
    Play(SelectHand),
    Discard(SelectHand),
    MoveCard(MoveDirection, Card),
    CashOut(usize),
    BuyJoker(Jokers),
    NextRound(),
    SelectBlind(Blind),
    // SkipBlind(Blind),
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Play(cards) => {
                write!(f, "Play: {:?}", cards)
            }
            Self::Discard(cards) => {
                write!(f, "Discard: {:?}", cards)
            }
            Self::MoveCard(dir, card) => {
                write!(f, "MoveCard: {:?} - {:}", card, dir)
            }
            Self::CashOut(reward) => {
                write!(f, "CashOut: {:}", reward)
            }
            Self::BuyJoker(joker) => {
                write!(f, "BuyJoker: {:?}", joker)
            }
            Self::NextRound() => {
                write!(f, "NextRound")
            }
            Self::SelectBlind(blind) => {
                write!(f, "SelectBlind: {:?}", blind)
            }
        }
    }
}
