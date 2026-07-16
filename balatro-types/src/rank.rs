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
    fn test_hand_rank_count() {
        assert_eq!(HandRank::iter().count(), 13);
    }

    #[test]
    fn test_royal_flush_scores_as_straight_flush() {
        assert_eq!(HandRank::RoyalFlush.scoring_rank(), HandRank::StraightFlush);
        assert_eq!(HandRank::Flush.scoring_rank(), HandRank::Flush);
    }
}
