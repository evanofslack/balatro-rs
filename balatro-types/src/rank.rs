use strum::EnumIter;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Copy)]
pub struct Level {
    pub level: usize,
    pub chips: usize,
    pub mult: usize,
    pub plays: usize,
}

/// All the different possible hand ranks.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Copy, EnumIter)]
pub enum HandRank {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    Straight,
    Flush,
    FullHouse,
    FourOfAKind,
    StraightFlush,
    RoyalFlush,
    FiveOfAKind,
    FlushHouse,
    FlushFive,
}

impl HandRank {
    /// Save-file id for this hand rank. Three are spelled with a lowercase
    /// "of" in the real data (`FiveofaKind`). `RoyalFlush`'s id is inferred,
    /// unconfirmed.
    pub fn id(&self) -> &'static str {
        match self {
            Self::HighCard => "HighCard",
            Self::OnePair => "Pair",
            Self::TwoPair => "TwoPair",
            Self::ThreeOfAKind => "ThreeofaKind",
            Self::Straight => "Straight",
            Self::Flush => "Flush",
            Self::FullHouse => "FullHouse",
            Self::FourOfAKind => "FourofaKind",
            Self::StraightFlush => "StraightFlush",
            Self::RoyalFlush => "RoyalFlush",
            Self::FiveOfAKind => "FiveofaKind",
            Self::FlushHouse => "FlushHouse",
            Self::FlushFive => "FlushFive",
        }
    }

    /// Parses a save-file id back into a `HandRank`.
    pub fn from_id(s: &str) -> Option<Self> {
        match s {
            "HighCard" => Some(Self::HighCard),
            "Pair" => Some(Self::OnePair),
            "TwoPair" => Some(Self::TwoPair),
            "ThreeofaKind" => Some(Self::ThreeOfAKind),
            "Straight" => Some(Self::Straight),
            "Flush" => Some(Self::Flush),
            "FullHouse" => Some(Self::FullHouse),
            "FourofaKind" => Some(Self::FourOfAKind),
            "StraightFlush" => Some(Self::StraightFlush),
            "RoyalFlush" => Some(Self::RoyalFlush),
            "FiveofaKind" => Some(Self::FiveOfAKind),
            "FlushHouse" => Some(Self::FlushHouse),
            "FlushFive" => Some(Self::FlushFive),
            _ => None,
        }
    }

    /// The rank whose `Level` this hand actually scores against.
    ///
    /// `RoyalFlush` is identified separately for detection purposes, but it
    /// levels up (and scores) identically to `StraightFlush`, there is no
    /// separate Royal Flush entry among the 12 Planet-leveled hand types.
    pub fn scoring_rank(&self) -> HandRank {
        match self {
            HandRank::RoyalFlush => HandRank::StraightFlush,
            other => *other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn test_hand_rank_id_round_trip() {
        for h in HandRank::iter() {
            assert_eq!(HandRank::from_id(h.id()), Some(h));
        }
    }

    #[test]
    fn test_hand_rank_id_known_spellings() {
        assert_eq!(HandRank::FiveOfAKind.id(), "FiveofaKind");
        assert_eq!(HandRank::FourOfAKind.id(), "FourofaKind");
        assert_eq!(HandRank::ThreeOfAKind.id(), "ThreeofaKind");
        assert_eq!(HandRank::OnePair.id(), "Pair");
    }

    #[test]
    fn test_hand_rank_count() {
        assert_eq!(HandRank::iter().count(), 13);
    }

    #[test]
    fn test_royal_flush_scores_as_straight_flush() {
        assert_eq!(HandRank::RoyalFlush.scoring_rank(), HandRank::StraightFlush);
        assert_eq!(HandRank::Flush.scoring_rank(), HandRank::Flush);
    }
}
