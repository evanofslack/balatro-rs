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

    /// Save-file id for this spectral card.
    pub fn id(&self) -> &'static str {
        match self {
            Self::Familiar => "c_familiar",
            Self::Grim => "c_grim",
            Self::Incantation => "c_incantation",
            Self::Talisman => "c_talisman",
            Self::Aura => "c_aura",
            Self::Wraith => "c_wraith",
            Self::Sigil => "c_sigil",
            Self::Ouija => "c_ouija",
            Self::Ectoplasm => "c_ectoplasm",
            Self::Immolate => "c_immolate",
            Self::Ankh => "c_ankh",
            Self::DejaVu => "c_deja_vu",
            Self::Hex => "c_hex",
            Self::Trance => "c_trance",
            Self::Medium => "c_medium",
            Self::Cryptid => "c_cryptid",
            Self::Soul => "c_soul",
            Self::BlackHole => "c_black_hole",
        }
    }

    /// Parses a save-file id back into a `Spectral`.
    pub fn from_id(s: &str) -> Option<Self> {
        match s {
            "c_familiar" => Some(Self::Familiar),
            "c_grim" => Some(Self::Grim),
            "c_incantation" => Some(Self::Incantation),
            "c_talisman" => Some(Self::Talisman),
            "c_aura" => Some(Self::Aura),
            "c_wraith" => Some(Self::Wraith),
            "c_sigil" => Some(Self::Sigil),
            "c_ouija" => Some(Self::Ouija),
            "c_ectoplasm" => Some(Self::Ectoplasm),
            "c_immolate" => Some(Self::Immolate),
            "c_ankh" => Some(Self::Ankh),
            "c_deja_vu" => Some(Self::DejaVu),
            "c_hex" => Some(Self::Hex),
            "c_trance" => Some(Self::Trance),
            "c_medium" => Some(Self::Medium),
            "c_cryptid" => Some(Self::Cryptid),
            "c_soul" => Some(Self::Soul),
            "c_black_hole" => Some(Self::BlackHole),
            _ => None,
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
    fn test_spectral_id_round_trip() {
        for s in Spectral::iter() {
            assert_eq!(Spectral::from_id(s.id()), Some(s));
        }
    }

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
