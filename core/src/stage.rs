#[cfg(feature = "python")]
use pyo3::{pyclass, pymethods};
use std::fmt;

/// Types of blinds
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Copy)]
pub enum Blind {
    Small,
    Big,
    Boss,
}

impl Blind {
    /// reward is money earned for beating the blind
    pub fn reward(&self) -> usize {
        match self {
            Self::Small => 3,
            Self::Big => 4,
            Self::Boss => 5,
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Small => Self::Big,
            Self::Big => Self::Boss,
            Self::Boss => Self::Small,
        }
    }
}

impl fmt::Display for Blind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Small => write!(f, "Small Blind"),
            Self::Big => write!(f, "Big Blind"),
            Self::Boss => write!(f, "Boss Blind"),
        }
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
}

impl Stage {
    pub(crate) fn is_blind(&self) -> bool {
        matches!(self, Stage::Blind(_))
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
        }
    }
}
