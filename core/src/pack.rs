use crate::card::Card;
use crate::joker::Jokers;
use crate::planet::Planets;
use crate::tarot::Tarot;
use balatro_types::Spectral;
#[cfg(feature = "python")]
use pyo3::pyclass;

pub use balatro_types::{PackCategory, PackSize};

// `PackContent`/`Pack` stay defined locally because `PlayingCard` needs to embed
// this crate's own `Card` (bc dedup `id`), not `balatro_types::Card`
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum PackContent {
    Tarot(Tarot),
    Joker(Jokers),
    Planet(Planets),
    PlayingCard(Card),
    Spectral(Spectral),
}

impl PackContent {
    pub fn name(&self) -> String {
        match self {
            Self::Tarot(t) => t.name().to_string(),
            Self::Joker(j) => j.name().to_string(),
            Self::Planet(p) => p.name().to_string(),
            Self::PlayingCard(c) => c.to_string(),
            Self::Spectral(s) => s.name().to_string(),
        }
    }

    pub fn type_label(&self) -> &str {
        match self {
            Self::Tarot(_) => "Tarot",
            Self::Joker(_) => "Joker",
            Self::Planet(_) => "Planet",
            Self::PlayingCard(_) => "Card",
            Self::Spectral(_) => "Spectral",
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Pack {
    pub category: PackCategory,
    pub size: PackSize,
    pub contents: Vec<PackContent>,
}

impl Pack {
    pub fn cost(&self) -> usize {
        match self.size {
            PackSize::Normal => 4,
            PackSize::Jumbo => 6,
            PackSize::Mega => 8,
        }
    }

    pub fn picks_allowed(&self) -> usize {
        match self.size {
            PackSize::Normal | PackSize::Jumbo => 1,
            PackSize::Mega => 2,
        }
    }

    pub fn name(&self) -> String {
        let cat = match self.category {
            PackCategory::Arcana => "Arcana",
            PackCategory::Buffoon => "Buffoon",
            PackCategory::Celestial => "Celestial",
            PackCategory::Standard => "Standard",
            PackCategory::Spectral => "Spectral",
        };
        let sz = match self.size {
            PackSize::Normal => "",
            PackSize::Jumbo => " Jumbo",
            PackSize::Mega => " Mega",
        };
        format!("{}{} Pack", cat, sz)
    }

    pub fn description(&self) -> String {
        let picks = self.picks_allowed();
        let count = match (&self.category, &self.size) {
            (PackCategory::Buffoon, PackSize::Normal) => 2,
            (PackCategory::Buffoon, _) => 4,
            (_, PackSize::Normal) => 3,
            _ => 5,
        };
        let item = match self.category {
            PackCategory::Arcana => "Tarot cards to use immediately",
            PackCategory::Celestial => "Planet cards to use immediately",
            PackCategory::Buffoon => "Jokers",
            PackCategory::Standard => "Playing Cards to add to your deck",
            PackCategory::Spectral => "Spectral cards to use immediately",
        };
        format!("Choose {} of {} {}", picks, count, item)
    }

    pub fn category_color_hint(&self) -> &str {
        match self.category {
            PackCategory::Arcana => "cyan",
            PackCategory::Buffoon => "magenta",
            PackCategory::Celestial => "blue",
            PackCategory::Standard => "white",
            PackCategory::Spectral => "yellow",
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct OpenPackState {
    pub contents: Vec<PackContent>,
    pub picks_remaining: usize,
    pub description: String,
}
