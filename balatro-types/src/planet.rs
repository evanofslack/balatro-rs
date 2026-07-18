use crate::rank::{HandRank, Level};
#[cfg(feature = "python")]
use pyo3::pyclass;
use std::fmt;
use strum::{EnumIter, EnumString};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(Debug, Clone, Copy, EnumIter, EnumString, Eq, PartialEq, Hash)]
#[strum(ascii_case_insensitive)]
pub enum Planets {
    Pluto,
    Mercury,
    Uranus,
    Venus,
    Saturn,
    Jupiter,
    Earth,
    Mars,
    Neptune,
    PlanetX,
    Ceres,
    Eris,
}

impl Planets {
    pub fn hand_rank(&self) -> HandRank {
        match self {
            Self::Pluto => HandRank::HighCard,
            Self::Mercury => HandRank::OnePair,
            Self::Uranus => HandRank::TwoPair,
            Self::Venus => HandRank::ThreeOfAKind,
            Self::Saturn => HandRank::Straight,
            Self::Jupiter => HandRank::Flush,
            Self::Earth => HandRank::FullHouse,
            Self::Mars => HandRank::FourOfAKind,
            Self::Neptune => HandRank::StraightFlush,
            Self::PlanetX => HandRank::FiveOfAKind,
            Self::Ceres => HandRank::FlushHouse,
            Self::Eris => HandRank::FlushFive,
        }
    }

    pub fn cost(&self) -> usize {
        3
    }

    pub fn sell_value(&self) -> usize {
        1
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Pluto => "Pluto",
            Self::Mercury => "Mercury",
            Self::Uranus => "Uranus",
            Self::Venus => "Venus",
            Self::Saturn => "Saturn",
            Self::Jupiter => "Jupiter",
            Self::Earth => "Earth",
            Self::Mars => "Mars",
            Self::Neptune => "Neptune",
            Self::PlanetX => "Planet X",
            Self::Ceres => "Ceres",
            Self::Eris => "Eris",
        }
    }

    /// Save-file id for this planet card.
    pub fn id(&self) -> &'static str {
        match self {
            Self::Pluto => "c_pluto",
            Self::Mercury => "c_mercury",
            Self::Uranus => "c_uranus",
            Self::Venus => "c_venus",
            Self::Saturn => "c_saturn",
            Self::Jupiter => "c_jupiter",
            Self::Earth => "c_earth",
            Self::Mars => "c_mars",
            Self::Neptune => "c_neptune",
            Self::PlanetX => "c_planet_x",
            Self::Ceres => "c_ceres",
            Self::Eris => "c_eris",
        }
    }

    /// Parses a save-file id back into a `Planets`.
    pub fn from_id(s: &str) -> Option<Self> {
        match s {
            "c_pluto" => Some(Self::Pluto),
            "c_mercury" => Some(Self::Mercury),
            "c_uranus" => Some(Self::Uranus),
            "c_venus" => Some(Self::Venus),
            "c_saturn" => Some(Self::Saturn),
            "c_jupiter" => Some(Self::Jupiter),
            "c_earth" => Some(Self::Earth),
            "c_mars" => Some(Self::Mars),
            "c_neptune" => Some(Self::Neptune),
            "c_planet_x" => Some(Self::PlanetX),
            "c_ceres" => Some(Self::Ceres),
            "c_eris" => Some(Self::Eris),
            _ => None,
        }
    }

    pub fn desc(&self) -> String {
        format!("Levels up {}", self.hand_rank_name())
    }

    fn hand_rank_name(&self) -> &str {
        match self {
            Self::Pluto => "High Card",
            Self::Mercury => "One Pair",
            Self::Uranus => "Two Pair",
            Self::Venus => "Three of a Kind",
            Self::Saturn => "Straight",
            Self::Jupiter => "Flush",
            Self::Earth => "Full House",
            Self::Mars => "Four of a Kind",
            Self::Neptune => "Straight Flush",
            Self::PlanetX => "Five of a Kind",
            Self::Ceres => "Flush House",
            Self::Eris => "Flush Five",
        }
    }

    /// PlanetX/Ceres/Eris are the "secret" planets (Five of a Kind, Flush
    /// House, Flush Five), only obtainable via those hand types, not sold
    /// in the shop by default.
    pub fn is_secret(&self) -> bool {
        matches!(self, Self::PlanetX | Self::Ceres | Self::Eris)
    }
}

/// Tracks the current level, chip/mult values, and play count for each
/// scored hand rank. Only 12 slots are stored, `RoyalFlush` shares
/// `StraightFlush`'s slot via `HandRank::scoring_rank`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Planetarium {
    highcard: Level,
    onepair: Level,
    twopair: Level,
    threeofkind: Level,
    straight: Level,
    flush: Level,
    fullhouse: Level,
    fourofkind: Level,
    straightflush: Level,
    fiveofkind: Level,
    flushhouse: Level,
    flushfive: Level,
}

impl Planetarium {
    pub fn new() -> Self {
        Planetarium {
            highcard: Level {
                level: 1,
                chips: 5,
                mult: 1,
                plays: 0,
            },
            onepair: Level {
                level: 1,
                chips: 10,
                mult: 2,
                plays: 0,
            },
            twopair: Level {
                level: 1,
                chips: 20,
                mult: 2,
                plays: 0,
            },
            threeofkind: Level {
                level: 1,
                chips: 30,
                mult: 3,
                plays: 0,
            },
            straight: Level {
                level: 1,
                chips: 30,
                mult: 4,
                plays: 0,
            },
            flush: Level {
                level: 1,
                chips: 35,
                mult: 4,
                plays: 0,
            },
            fullhouse: Level {
                level: 1,
                chips: 40,
                mult: 4,
                plays: 0,
            },
            fourofkind: Level {
                level: 1,
                chips: 60,
                mult: 7,
                plays: 0,
            },
            straightflush: Level {
                level: 1,
                chips: 100,
                mult: 8,
                plays: 0,
            },
            fiveofkind: Level {
                level: 1,
                chips: 120,
                mult: 12,
                plays: 0,
            },
            flushhouse: Level {
                level: 1,
                chips: 140,
                mult: 14,
                plays: 0,
            },
            flushfive: Level {
                level: 1,
                chips: 160,
                mult: 16,
                plays: 0,
            },
        }
    }
}

impl Default for Planetarium {
    fn default() -> Self {
        Self::new()
    }
}

impl Planetarium {
    /// Increment play count for the rank and return current level data.
    pub fn play(&mut self, rank: HandRank) -> Level {
        match rank.scoring_rank() {
            HandRank::HighCard => self.highcard.plays += 1,
            HandRank::OnePair => self.onepair.plays += 1,
            HandRank::TwoPair => self.twopair.plays += 1,
            HandRank::ThreeOfAKind => self.threeofkind.plays += 1,
            HandRank::Straight => self.straight.plays += 1,
            HandRank::Flush => self.flush.plays += 1,
            HandRank::FullHouse => self.fullhouse.plays += 1,
            HandRank::FourOfAKind => self.fourofkind.plays += 1,
            HandRank::StraightFlush => self.straightflush.plays += 1,
            HandRank::FiveOfAKind => self.fiveofkind.plays += 1,
            HandRank::FlushHouse => self.flushhouse.plays += 1,
            HandRank::FlushFive => self.flushfive.plays += 1,
            HandRank::RoyalFlush => unreachable!("scoring_rank() never returns RoyalFlush"),
        }
        self.level(rank)
    }

    pub fn level(&self, rank: HandRank) -> Level {
        match rank.scoring_rank() {
            HandRank::HighCard => self.highcard,
            HandRank::OnePair => self.onepair,
            HandRank::TwoPair => self.twopair,
            HandRank::ThreeOfAKind => self.threeofkind,
            HandRank::Straight => self.straight,
            HandRank::Flush => self.flush,
            HandRank::FullHouse => self.fullhouse,
            HandRank::FourOfAKind => self.fourofkind,
            HandRank::StraightFlush => self.straightflush,
            HandRank::FiveOfAKind => self.fiveofkind,
            HandRank::FlushHouse => self.flushhouse,
            HandRank::FlushFive => self.flushfive,
            HandRank::RoyalFlush => unreachable!("scoring_rank() never returns RoyalFlush"),
        }
    }

    /// Apply one level-up for the given rank, using the per-planet chip/mult
    /// increments. Since `RoyalFlush` shares `StraightFlush`'s slot,
    /// levelling up either one levels up the same stored data.
    pub fn level_up(&mut self, rank: HandRank) {
        match rank.scoring_rank() {
            HandRank::HighCard => {
                self.highcard.level += 1;
                self.highcard.chips += 10;
                self.highcard.mult += 1;
            }
            HandRank::OnePair => {
                self.onepair.level += 1;
                self.onepair.chips += 15;
                self.onepair.mult += 1;
            }
            HandRank::TwoPair => {
                self.twopair.level += 1;
                self.twopair.chips += 20;
                self.twopair.mult += 1;
            }
            HandRank::ThreeOfAKind => {
                self.threeofkind.level += 1;
                self.threeofkind.chips += 20;
                self.threeofkind.mult += 2;
            }
            HandRank::Straight => {
                self.straight.level += 1;
                self.straight.chips += 30;
                self.straight.mult += 3;
            }
            HandRank::Flush => {
                self.flush.level += 1;
                self.flush.chips += 15;
                self.flush.mult += 2;
            }
            HandRank::FullHouse => {
                self.fullhouse.level += 1;
                self.fullhouse.chips += 25;
                self.fullhouse.mult += 2;
            }
            HandRank::FourOfAKind => {
                self.fourofkind.level += 1;
                self.fourofkind.chips += 30;
                self.fourofkind.mult += 3;
            }
            HandRank::StraightFlush => {
                self.straightflush.level += 1;
                self.straightflush.chips += 40;
                self.straightflush.mult += 4;
            }
            HandRank::FiveOfAKind => {
                self.fiveofkind.level += 1;
                self.fiveofkind.chips += 35;
                self.fiveofkind.mult += 3;
            }
            HandRank::FlushHouse => {
                self.flushhouse.level += 1;
                self.flushhouse.chips += 40;
                self.flushhouse.mult += 4;
            }
            HandRank::FlushFive => {
                self.flushfive.level += 1;
                self.flushfive.chips += 50;
                self.flushfive.mult += 3;
            }
            HandRank::RoyalFlush => unreachable!("scoring_rank() never returns RoyalFlush"),
        }
    }
}

impl fmt::Display for Planetarium {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ranks = [
            ("HC", self.highcard),
            ("1P", self.onepair),
            ("2P", self.twopair),
            ("3K", self.threeofkind),
            ("ST", self.straight),
            ("FL", self.flush),
            ("FH", self.fullhouse),
            ("4K", self.fourofkind),
            ("SF", self.straightflush),
            ("5K", self.fiveofkind),
            ("FLH", self.flushhouse),
            ("FF", self.flushfive),
        ];
        let parts: Vec<String> = ranks
            .iter()
            .map(|(abbr, lvl)| format!("{}:L{}", abbr, lvl.level))
            .collect();
        write!(f, "{}", parts.join(" | "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn test_planets_id_round_trip() {
        for p in Planets::iter() {
            assert_eq!(Planets::from_id(p.id()), Some(p));
        }
    }

    #[test]
    fn test_planet_count() {
        assert_eq!(Planets::iter().count(), 12);
    }

    #[test]
    fn test_planet_hand_ranks() {
        assert_eq!(Planets::Pluto.hand_rank(), HandRank::HighCard);
        assert_eq!(Planets::Neptune.hand_rank(), HandRank::StraightFlush);
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
    fn test_planet_cost_and_sell_value() {
        assert_eq!(Planets::Mercury.cost(), 3);
        assert_eq!(Planets::Mercury.sell_value(), 1);
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
    fn test_planetarium_royal_flush_shares_straight_flush_slot() {
        let mut p = Planetarium::new();
        p.level_up(HandRank::StraightFlush);
        assert_eq!(
            p.level(HandRank::RoyalFlush),
            p.level(HandRank::StraightFlush)
        );
    }

    #[test]
    fn test_planetarium_play_increments_count() {
        let mut p = Planetarium::new();
        assert_eq!(p.level(HandRank::OnePair).plays, 0);
        p.play(HandRank::OnePair);
        p.play(HandRank::OnePair);
        assert_eq!(p.level(HandRank::OnePair).plays, 2);
        assert_eq!(p.level(HandRank::HighCard).plays, 0);
    }

    #[test]
    fn test_planets_from_str_round_trip() {
        for planet in Planets::iter() {
            assert_eq!(format!("{planet:?}").parse::<Planets>(), Ok(planet));
        }
    }

    #[test]
    fn test_planets_from_str_case_insensitive() {
        assert_eq!("planetx".parse::<Planets>(), Ok(Planets::PlanetX));
        assert_eq!("PLANETX".parse::<Planets>(), Ok(Planets::PlanetX));
    }

    #[test]
    fn test_planets_from_str_invalid() {
        assert!("NotAPlanet".parse::<Planets>().is_err());
    }
}
