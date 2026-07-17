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

    /// Save-file id for this boss blind.
    pub fn id(&self) -> &'static str {
        match self {
            Self::Arm => "bl_arm",
            Self::Club => "bl_club",
            Self::Eye => "bl_eye",
            Self::Fish => "bl_fish",
            Self::Flint => "bl_flint",
            Self::Goad => "bl_goad",
            Self::Head => "bl_head",
            Self::Hook => "bl_hook",
            Self::House => "bl_house",
            Self::Manacle => "bl_manacle",
            Self::Mark => "bl_mark",
            Self::Mouth => "bl_mouth",
            Self::Needle => "bl_needle",
            Self::Ox => "bl_ox",
            Self::Pillar => "bl_pillar",
            Self::Plant => "bl_plant",
            Self::Psychic => "bl_psychic",
            Self::Serpent => "bl_serpent",
            Self::Tooth => "bl_tooth",
            Self::Wall => "bl_wall",
            Self::Water => "bl_water",
            Self::Wheel => "bl_wheel",
            Self::Window => "bl_window",
            Self::AmberAcorn => "bl_final_acorn",
            Self::CeruleanBell => "bl_final_bell",
            Self::CrimsonHeart => "bl_final_heart",
            Self::VerdantLeaf => "bl_final_leaf",
            Self::VioletVessel => "bl_final_vessel",
        }
    }

    /// Parses a save-file id back into a `BossBlind`.
    pub fn from_id(s: &str) -> Option<Self> {
        match s {
            "bl_arm" => Some(Self::Arm),
            "bl_club" => Some(Self::Club),
            "bl_eye" => Some(Self::Eye),
            "bl_fish" => Some(Self::Fish),
            "bl_flint" => Some(Self::Flint),
            "bl_goad" => Some(Self::Goad),
            "bl_head" => Some(Self::Head),
            "bl_hook" => Some(Self::Hook),
            "bl_house" => Some(Self::House),
            "bl_manacle" => Some(Self::Manacle),
            "bl_mark" => Some(Self::Mark),
            "bl_mouth" => Some(Self::Mouth),
            "bl_needle" => Some(Self::Needle),
            "bl_ox" => Some(Self::Ox),
            "bl_pillar" => Some(Self::Pillar),
            "bl_plant" => Some(Self::Plant),
            "bl_psychic" => Some(Self::Psychic),
            "bl_serpent" => Some(Self::Serpent),
            "bl_tooth" => Some(Self::Tooth),
            "bl_wall" => Some(Self::Wall),
            "bl_water" => Some(Self::Water),
            "bl_wheel" => Some(Self::Wheel),
            "bl_window" => Some(Self::Window),
            "bl_final_acorn" => Some(Self::AmberAcorn),
            "bl_final_bell" => Some(Self::CeruleanBell),
            "bl_final_heart" => Some(Self::CrimsonHeart),
            "bl_final_leaf" => Some(Self::VerdantLeaf),
            "bl_final_vessel" => Some(Self::VioletVessel),
            _ => None,
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
    fn test_boss_blind_id_round_trip() {
        for b in BossBlind::iter() {
            assert_eq!(BossBlind::from_id(b.id()), Some(b));
        }
    }

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
