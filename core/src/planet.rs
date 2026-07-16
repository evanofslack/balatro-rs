pub use balatro_types::{Planetarium, Planets};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rank::HandRank;

    #[test]
    fn test_planet_hand_ranks() {
        assert_eq!(Planets::Pluto.hand_rank(), HandRank::HighCard);
        assert_eq!(Planets::Mercury.hand_rank(), HandRank::OnePair);
        assert_eq!(Planets::Uranus.hand_rank(), HandRank::TwoPair);
        assert_eq!(Planets::Venus.hand_rank(), HandRank::ThreeOfAKind);
        assert_eq!(Planets::Saturn.hand_rank(), HandRank::Straight);
        assert_eq!(Planets::Jupiter.hand_rank(), HandRank::Flush);
        assert_eq!(Planets::Earth.hand_rank(), HandRank::FullHouse);
        assert_eq!(Planets::Mars.hand_rank(), HandRank::FourOfAKind);
        assert_eq!(Planets::Neptune.hand_rank(), HandRank::StraightFlush);
        assert_eq!(Planets::PlanetX.hand_rank(), HandRank::FiveOfAKind);
        assert_eq!(Planets::Ceres.hand_rank(), HandRank::FlushHouse);
        assert_eq!(Planets::Eris.hand_rank(), HandRank::FlushFive);
    }

    #[test]
    fn test_planet_is_secret() {
        assert!(!Planets::Pluto.is_secret());
        assert!(!Planets::Neptune.is_secret());
        assert!(Planets::PlanetX.is_secret());
        assert!(Planets::Ceres.is_secret());
        assert!(Planets::Eris.is_secret());
    }

    #[test]
    fn test_planetarium_level_up_highcard() {
        let mut p = Planetarium::new();
        let before = p.level(HandRank::HighCard);
        p.level_up(HandRank::HighCard);
        let after = p.level(HandRank::HighCard);
        assert_eq!(after.level, before.level + 1);
        assert_eq!(after.chips, before.chips + 10);
        assert_eq!(after.mult, before.mult + 1);
    }

    #[test]
    fn test_planetarium_level_up_threeofkind() {
        let mut p = Planetarium::new();
        let before = p.level(HandRank::ThreeOfAKind);
        p.level_up(HandRank::ThreeOfAKind);
        let after = p.level(HandRank::ThreeOfAKind);
        assert_eq!(after.level, before.level + 1);
        assert_eq!(after.chips, before.chips + 20);
        assert_eq!(after.mult, before.mult + 2);
    }

    #[test]
    fn test_planetarium_level_up_royalflush_colevels_with_straightflush() {
        let mut p = Planetarium::new();
        let sf_before = p.level(HandRank::StraightFlush);
        let rf_before = p.level(HandRank::RoyalFlush);
        p.level_up(HandRank::StraightFlush);
        let sf_after = p.level(HandRank::StraightFlush);
        let rf_after = p.level(HandRank::RoyalFlush);
        assert_eq!(sf_after.level, sf_before.level + 1);
        assert_eq!(sf_after.chips, sf_before.chips + 40);
        assert_eq!(sf_after.mult, sf_before.mult + 4);
        assert_eq!(rf_after.level, rf_before.level + 1);
        assert_eq!(rf_after.chips, rf_before.chips + 40);
        assert_eq!(rf_after.mult, rf_before.mult + 4);
    }

    // RoyalFlush no longer has its own storage slot, it shares
    // StraightFlush's
    #[test]
    fn test_planetarium_royalflush_level_up_shares_straightflush_slot() {
        let mut p = Planetarium::new();
        p.level_up(HandRank::RoyalFlush);
        assert_eq!(
            p.level(HandRank::RoyalFlush),
            p.level(HandRank::StraightFlush)
        );
        assert_eq!(p.level(HandRank::StraightFlush).level, 2);
    }

    #[test]
    fn test_planetarium_play_increments_count() {
        let mut p = Planetarium::new();
        assert_eq!(p.level(HandRank::OnePair).plays, 0);
        p.play(HandRank::OnePair);
        assert_eq!(p.level(HandRank::OnePair).plays, 1);
        p.play(HandRank::OnePair);
        assert_eq!(p.level(HandRank::OnePair).plays, 2);
        assert_eq!(p.level(HandRank::HighCard).plays, 0);
    }

    #[test]
    fn test_planetarium_play_returns_current_level() {
        let mut p = Planetarium::new();
        p.level_up(HandRank::OnePair);
        let level = p.play(HandRank::OnePair);
        assert_eq!(level.chips, 10 + 15);
        assert_eq!(level.mult, 2 + 1);
        assert_eq!(level.plays, 1);
    }
}
