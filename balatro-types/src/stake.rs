use strum::EnumIter;

/// Difficulty stakes, in ascending order. Each stake's modifiers stack on
/// top of every stake before it.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, EnumIter)]
pub enum Stake {
    White,
    Red,
    Green,
    Black,
    Blue,
    Purple,
    Orange,
    Gold,
}

impl Stake {
    pub fn name(&self) -> &str {
        match self {
            Self::White => "White Stake",
            Self::Red => "Red Stake",
            Self::Green => "Green Stake",
            Self::Black => "Black Stake",
            Self::Blue => "Blue Stake",
            Self::Purple => "Purple Stake",
            Self::Orange => "Orange Stake",
            Self::Gold => "Gold Stake",
        }
    }

    /// The modifier this stake adds on top of every stake before it.
    pub fn added_modifier(&self) -> &str {
        match self {
            Self::White => "Base difficulty",
            Self::Red => "Small Blind no longer gives a cash reward on win",
            Self::Green => "Score requirements scale up faster each Ante",
            Self::Black => "Shop Jokers may spawn with the Eternal sticker",
            Self::Blue => "-1 discard every round",
            Self::Purple => "Score requirements scale up faster again",
            Self::Orange => "Shop Jokers may spawn with the Perishable sticker",
            Self::Gold => "Shop/pack Jokers may spawn with the Rental sticker",
        }
    }

    pub fn ordinal(&self) -> usize {
        match self {
            Self::White => 0,
            Self::Red => 1,
            Self::Green => 2,
            Self::Black => 3,
            Self::Blue => 4,
            Self::Purple => 5,
            Self::Orange => 6,
            Self::Gold => 7,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn test_stake_count() {
        assert_eq!(Stake::iter().count(), 8);
    }

    #[test]
    fn test_stake_order() {
        assert!(Stake::White < Stake::Gold);
        assert_eq!(Stake::White.ordinal(), 0);
        assert_eq!(Stake::Gold.ordinal(), 7);
    }
}
