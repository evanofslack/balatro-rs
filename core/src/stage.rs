use crate::tarot::Tarot;
pub use balatro_types::Blind;
#[cfg(feature = "python")]
use pyo3::{pyclass, pymethods};

/// Engine behavior for `Blind`
pub trait BlindExt {
    /// Money earned for beating the blind.
    fn reward(&self) -> usize;
    // Ring around the rosie.
    fn next(&self) -> Self;
}

impl BlindExt for Blind {
    fn reward(&self) -> usize {
        match self {
            Self::Small => 3,
            Self::Big => 4,
            Self::Boss => 5,
        }
    }

    fn next(&self) -> Self {
        match self {
            Self::Small => Self::Big,
            Self::Big => Self::Boss,
            Self::Boss => Self::Small,
        }
    }
}

pub fn blind_display(blind: &Blind) -> &'static str {
    match blind {
        Blind::Small => "Small Blind",
        Blind::Big => "Big Blind",
        Blind::Boss => "Boss Blind",
    }
}

/// Game ending
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Copy)]
pub enum End {
    Win,
    Lose,
}

/// Stages of playing.
// Playing through an ante looks like:
// Pre -> Small -> Post -> Shop -> Pre -> Big -> Post -> Shop -> Boss -> Post -> Shop
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Copy)]
pub enum Stage {
    // See blind conditions, choose blind (or skip blind)
    PreBlind(),
    // Play blind
    Blind(Blind),
    // Collect payout, optionally play consumables
    PostBlind(),
    // Buy jokers, consumables
    Shop(),
    // Game ending
    End(End),
    // (Temp stage) Target a tarot, hand drawn for card selection, pending tarot applied on ApplyTarot
    TarotHand(Tarot),
    // (Temp stage) A booster pack has been purchased and is open; player chooses contents
    PackOpen(),
}

impl Stage {
    pub(crate) fn is_blind(&self) -> bool {
        matches!(self, Stage::Blind(_))
    }

    pub(crate) fn is_pack_open(&self) -> bool {
        matches!(self, Stage::PackOpen())
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl Stage {
    fn int(&self) -> usize {
        match self {
            Self::PreBlind() => 0,
            Self::Blind(blind) => match blind {
                Blind::Small => 1,
                Blind::Big => 2,
                Blind::Boss => 3,
            },
            Self::PostBlind() => 4,
            Self::Shop() => 5,
            Self::End(end) => match end {
                End::Win => 6,
                End::Lose => 7,
            },
            Self::TarotHand(_) => 8,
            Self::PackOpen() => 9,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blind_reward() {
        assert_eq!(Blind::Small.reward(), 3);
        assert_eq!(Blind::Big.reward(), 4);
        assert_eq!(Blind::Boss.reward(), 5);
    }

    #[test]
    fn test_blind_next_cycles() {
        assert_eq!(Blind::Small.next(), Blind::Big);
        assert_eq!(Blind::Big.next(), Blind::Boss);
        assert_eq!(Blind::Boss.next(), Blind::Small);
    }

    #[test]
    fn test_blind_display() {
        assert_eq!(blind_display(&Blind::Small), "Small Blind");
        assert_eq!(blind_display(&Blind::Big), "Big Blind");
        assert_eq!(blind_display(&Blind::Boss), "Boss Blind");
    }

    #[test]
    fn test_stage_is_blind_and_is_pack_open() {
        assert!(Stage::Blind(Blind::Small).is_blind());
        assert!(!Stage::Shop().is_blind());
        assert!(Stage::PackOpen().is_pack_open());
        assert!(!Stage::Shop().is_pack_open());
    }
}
