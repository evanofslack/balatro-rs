#[cfg(feature = "python")]
use pyo3::pyclass;
use strum::EnumIter;

/// Spectral cards. Every variant appears with equal odds in Spectral packs
/// except `Soul` and `BlackHole`, which are additionally restricted (see
/// `is_rare`).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, EnumIter)]
pub enum Spectral {
    Familiar,
    Grim,
    Incantation,
    Talisman,
    Aura,
    Wraith,
    Sigil,
    Ouija,
    Ectoplasm,
    Immolate,
    Ankh,
    DejaVu,
    Hex,
    Trance,
    Medium,
    Cryptid,
    Soul,
    BlackHole,
}

impl Spectral {
    pub fn name(&self) -> &str {
        match self {
            Self::Familiar => "Familiar",
            Self::Grim => "Grim",
            Self::Incantation => "Incantation",
            Self::Talisman => "Talisman",
            Self::Aura => "Aura",
            Self::Wraith => "Wraith",
            Self::Sigil => "Sigil",
            Self::Ouija => "Ouija",
            Self::Ectoplasm => "Ectoplasm",
            Self::Immolate => "Immolate",
            Self::Ankh => "Ankh",
            Self::DejaVu => "Deja Vu",
            Self::Hex => "Hex",
            Self::Trance => "Trance",
            Self::Medium => "Medium",
            Self::Cryptid => "Cryptid",
            Self::Soul => "The Soul",
            Self::BlackHole => "Black Hole",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::Familiar => {
                "Destroys 1 random card in hand, adds 3 random Enhanced face cards to hand"
            }
            Self::Grim => "Destroys 1 random card in hand, adds 2 random Enhanced Aces to hand",
            Self::Incantation => {
                "Destroys 1 random card in hand, adds 4 random Enhanced numbered cards to hand"
            }
            Self::Talisman => "Adds a Gold Seal to 1 selected card",
            Self::Aura => "Adds Foil, Holographic, or Polychrome to 1 selected card",
            Self::Wraith => "Creates a random Rare Joker, sets money to $0",
            Self::Sigil => "Converts all cards in hand to a single random suit",
            Self::Ouija => "Converts all cards in hand to a single random rank, -1 hand size",
            Self::Ectoplasm => "Adds Negative to a random Joker, -1 hand size",
            Self::Immolate => "Destroys 5 random cards in hand, gain $20",
            Self::Ankh => "Creates a copy of a random Joker, destroys all other Jokers",
            Self::DejaVu => "Adds a Red Seal to 1 selected card",
            Self::Hex => "Adds Polychrome to a random Joker, destroys all other Jokers",
            Self::Trance => "Adds a Blue Seal to 1 selected card",
            Self::Medium => "Adds a Purple Seal to 1 selected card",
            Self::Cryptid => "Creates 2 copies of 1 selected card",
            Self::Soul => "Creates a Legendary Joker (if there is room)",
            Self::BlackHole => "Upgrades every poker hand by 1 level",
        }
    }

    pub fn cost(&self) -> usize {
        4
    }

    pub fn sell_value(&self) -> usize {
        1
    }

    /// `Soul` and `BlackHole` are excluded from Spectral packs' normal equal
    /// odds and instead have an independent 0.3%-per-card chance to appear
    /// in specific booster packs (Soul: Spectral + Arcana; BlackHole:
    /// Spectral + Celestial). They also cannot be created by Sixth Sense /
    /// Seance, and never appear in the Ghost Deck's shop pool.
    pub fn is_rare(&self) -> bool {
        matches!(self, Self::Soul | Self::BlackHole)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn test_spectral_count() {
        assert_eq!(Spectral::iter().count(), 18);
    }

    #[test]
    fn test_spectral_is_rare() {
        assert!(Spectral::Soul.is_rare());
        assert!(Spectral::BlackHole.is_rare());
        assert!(!Spectral::Familiar.is_rare());
    }
}
