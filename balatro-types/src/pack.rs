use crate::card::Card;
use crate::joker::Jokers;
use crate::planet::Planets;
use crate::spectral::Spectral;
use crate::tarot::Tarot;
#[cfg(feature = "python")]
use pyo3::pyclass;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum PackCategory {
    Arcana,
    Buffoon,
    Celestial,
    Standard,
    Spectral,
}

impl PackCategory {
    /// Save-file id fragment for this pack category (the `"arcana"` in
    /// `"p_arcana_jumbo_1"`).
    pub fn id(&self) -> &'static str {
        match self {
            Self::Arcana => "arcana",
            Self::Buffoon => "buffoon",
            Self::Celestial => "celestial",
            Self::Standard => "standard",
            Self::Spectral => "spectral",
        }
    }

    pub fn from_id(s: &str) -> Option<Self> {
        match s {
            "arcana" => Some(Self::Arcana),
            "buffoon" => Some(Self::Buffoon),
            "celestial" => Some(Self::Celestial),
            "standard" => Some(Self::Standard),
            "spectral" => Some(Self::Spectral),
            _ => None,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum PackSize {
    Normal,
    Jumbo,
    Mega,
}

impl PackSize {
    /// Save-file id fragment for this pack size (the `"jumbo"` in
    /// `"p_arcana_jumbo_1"`).
    pub fn id(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Jumbo => "jumbo",
            Self::Mega => "mega",
        }
    }

    pub fn from_id(s: &str) -> Option<Self> {
        match s {
            "normal" => Some(Self::Normal),
            "jumbo" => Some(Self::Jumbo),
            "mega" => Some(Self::Mega),
            _ => None,
        }
    }
}

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
            Self::Planet(p) => p.name(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{Suit, Value};

    #[test]
    fn test_pack_category_size_id_round_trip() {
        let categories = [
            PackCategory::Arcana,
            PackCategory::Buffoon,
            PackCategory::Celestial,
            PackCategory::Standard,
            PackCategory::Spectral,
        ];
        for c in categories {
            assert_eq!(PackCategory::from_id(c.id()), Some(c));
        }
        let sizes = [PackSize::Normal, PackSize::Jumbo, PackSize::Mega];
        for s in sizes {
            assert_eq!(PackSize::from_id(s.id()), Some(s));
        }
        assert_eq!(PackCategory::Arcana.id(), "arcana");
        assert_eq!(PackSize::Jumbo.id(), "jumbo");
    }

    #[test]
    fn test_pack_cost_by_size() {
        let mut pack = Pack {
            category: PackCategory::Arcana,
            size: PackSize::Normal,
            contents: vec![],
        };
        assert_eq!(pack.cost(), 4);
        pack.size = PackSize::Jumbo;
        assert_eq!(pack.cost(), 6);
        pack.size = PackSize::Mega;
        assert_eq!(pack.cost(), 8);
    }

    #[test]
    fn test_pack_content_playing_card_name() {
        let content = PackContent::PlayingCard(Card::new(Value::Ace, Suit::Spade));
        assert_eq!(content.name(), "Ace of Spades");
        assert_eq!(content.type_label(), "Card");
    }

    #[test]
    fn test_pack_content_spectral() {
        let content = PackContent::Spectral(Spectral::Familiar);
        assert_eq!(content.type_label(), "Spectral");
    }
}
