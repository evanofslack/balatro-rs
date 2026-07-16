#[cfg(feature = "python")]
use pyo3::pyclass;
use strum::EnumIter;

/// Which of the three blind slots in a round this is. Distinct from
/// `BossBlind`, which identifies the specific boss occupying the `Boss`
/// slot.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, EnumIter)]
pub enum Blind {
    Small,
    Big,
    Boss,
}

/// The 28 Boss Blinds.
/// 23 regular (any Ante) plus 5 "Finisher" blinds
/// exclusive to Ante 8.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, EnumIter)]
pub enum BossBlind {
    Arm,
    Club,
    Eye,
    Fish,
    Flint,
    Goad,
    Head,
    Hook,
    House,
    Manacle,
    Mark,
    Mouth,
    Needle,
    Ox,
    Pillar,
    Plant,
    Psychic,
    Serpent,
    Tooth,
    Wall,
    Water,
    Wheel,
    Window,
    AmberAcorn,
    CeruleanBell,
    CrimsonHeart,
    VerdantLeaf,
    VioletVessel,
}

impl BossBlind {
    pub fn name(&self) -> &str {
        match self {
            Self::Arm => "The Arm",
            Self::Club => "The Club",
            Self::Eye => "The Eye",
            Self::Fish => "The Fish",
            Self::Flint => "The Flint",
            Self::Goad => "The Goad",
            Self::Head => "The Head",
            Self::Hook => "The Hook",
            Self::House => "The House",
            Self::Manacle => "The Manacle",
            Self::Mark => "The Mark",
            Self::Mouth => "The Mouth",
            Self::Needle => "The Needle",
            Self::Ox => "The Ox",
            Self::Pillar => "The Pillar",
            Self::Plant => "The Plant",
            Self::Psychic => "The Psychic",
            Self::Serpent => "The Serpent",
            Self::Tooth => "The Tooth",
            Self::Wall => "The Wall",
            Self::Water => "The Water",
            Self::Wheel => "The Wheel",
            Self::Window => "The Window",
            Self::AmberAcorn => "Amber Acorn",
            Self::CeruleanBell => "Cerulean Bell",
            Self::CrimsonHeart => "Crimson Heart",
            Self::VerdantLeaf => "Verdant Leaf",
            Self::VioletVessel => "Violet Vessel",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::Arm => "Delevels the poker hand played to level 1",
            Self::Club => "Debuffs all Club cards",
            Self::Eye => "Cannot play the same poker hand type twice in a row",
            Self::Fish => "Cards drawn face down after each played/discarded hand, accumulating",
            Self::Flint => "Halves all base Chips and Mult for played hands",
            Self::Goad => "Debuffs all Spade cards",
            Self::Head => "Debuffs all Heart cards",
            Self::Hook => "Discards 2 random unplayed cards after every played hand",
            Self::House => "First hand of the round is dealt face down",
            Self::Manacle => "-1 hand size",
            Self::Mark => "Face cards are dealt face down",
            Self::Mouth => "Only the poker hand type of the first played hand can be played",
            Self::Needle => "Only 1 hand may be played the entire round",
            Self::Ox => "Sets money to $0 when your most-played poker hand is played",
            Self::Pillar => "Cards already played this Ante are debuffed",
            Self::Plant => "Debuffs all face cards",
            Self::Psychic => "Must play exactly 5 cards every hand",
            Self::Serpent => "Draw 3 cards after each played/discarded hand regardless of size",
            Self::Tooth => "Lose $1 per card played",
            Self::Wall => "Requires 4x base chips instead of 2x",
            Self::Water => "Start the round with 0 discards",
            Self::Wheel => "1-in-4 chance for each played card to add an Edition to a random Joker",
            Self::Window => "Debuffs all Diamond cards",
            Self::AmberAcorn => "Flips all Jokers face down and shuffles their positions",
            Self::CeruleanBell => "Forces a random card to always be selected each hand",
            Self::CrimsonHeart => "Debuffs a random Joker each hand",
            Self::VerdantLeaf => "All cards are debuffed until a Joker is sold",
            Self::VioletVessel => "Requires 6x base chips instead of 2x",
        }
    }

    /// The 5 blinds exclusive to Ante 8 (the "Finisher" bosses).
    pub fn is_finisher(&self) -> bool {
        matches!(
            self,
            Self::AmberAcorn
                | Self::CeruleanBell
                | Self::CrimsonHeart
                | Self::VerdantLeaf
                | Self::VioletVessel
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn test_blind_count() {
        assert_eq!(Blind::iter().count(), 3);
    }

    #[test]
    fn test_boss_blind_count() {
        assert_eq!(BossBlind::iter().count(), 28);
    }

    #[test]
    fn test_boss_blind_finisher_count() {
        assert_eq!(BossBlind::iter().filter(|b| b.is_finisher()).count(), 5);
    }
}
