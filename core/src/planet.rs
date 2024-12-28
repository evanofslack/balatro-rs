use pyo3::pyclass;
use std::sync::{Arc, Mutex};

pub type PlanetEffect = Arc<Mutex<dyn Fn(&mut Planetarium) + Send + 'static>>;

pub trait Planet {
    fn name(&self) -> String;
    fn effect(&self) -> PlanetEffect;
}

// Similar to what we do for jokers.
// We could pass around `Box<dyn Planet>` but it doesn't work so nice with pyo3 and serde.
// Since we know all variants (one for each planet), we define an enum that implements
// our `Planet` trait.
macro_rules! make_planets {
    ($($x:ident), *) => {
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "python", pyclass(eq))]
        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        pub enum Planets {
            $(
                $x($x),
            )*
        }

        impl Planet for Planets {
            fn name(&self) -> String {
                match self {
                    $(
                        Planets::$x(planet) => planet.name(),
                    )*
                }
            }
            fn effect(&self) -> PlanetEffect {
                match self {
                    $(
                        Planets::$x(planet) => planet.effect(),
                    )*
                }
            }
        }
    }
}

make_planets!(
    Pluto, Mercury, Uranus, Venus, Saturn, Jupiter, Earth, Mars, Neptune, PlanetX, Ceres, Eris
);

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct Pluto {}

impl Planet for Pluto {
    fn name(&self) -> String {
        "Pluto".to_string()
    }
    fn effect(&self) -> PlanetEffect {
        fn apply(p: &mut Planetarium) {
            p.level_up(HandRank::HighCard);
        }
        Arc::new(Mutex::new(apply))
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct Mercury {}

impl Planet for Mercury {
    fn name(&self) -> String {
        "Mercury".to_string()
    }
    fn effect(&self) -> PlanetEffect {
        fn apply(p: &mut Planetarium) {
            p.level_up(HandRank::OnePair);
        }
        Arc::new(Mutex::new(apply))
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct Uranus {}

impl Planet for Uranus {
    fn name(&self) -> String {
        "Uranus".to_string()
    }
    fn effect(&self) -> PlanetEffect {
        fn apply(p: &mut Planetarium) {
            p.level_up(HandRank::TwoPair);
        }
        Arc::new(Mutex::new(apply))
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct Venus {}

impl Planet for Venus {
    fn name(&self) -> String {
        "Venus".to_string()
    }
    fn effect(&self) -> PlanetEffect {
        fn apply(p: &mut Planetarium) {
            p.level_up(HandRank::ThreeOfAKind);
        }
        Arc::new(Mutex::new(apply))
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct Saturn {}

impl Planet for Saturn {
    fn name(&self) -> String {
        "Saturn".to_string()
    }
    fn effect(&self) -> PlanetEffect {
        fn apply(p: &mut Planetarium) {
            p.level_up(HandRank::Straight);
        }
        Arc::new(Mutex::new(apply))
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct Jupiter {}

impl Planet for Jupiter {
    fn name(&self) -> String {
        "Jupiter".to_string()
    }
    fn effect(&self) -> PlanetEffect {
        fn apply(p: &mut Planetarium) {
            p.level_up(HandRank::Flush);
        }
        Arc::new(Mutex::new(apply))
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct Earth {}

impl Planet for Earth {
    fn name(&self) -> String {
        "Earth".to_string()
    }
    fn effect(&self) -> PlanetEffect {
        fn apply(p: &mut Planetarium) {
            p.level_up(HandRank::FullHouse);
        }
        Arc::new(Mutex::new(apply))
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct Mars {}

impl Planet for Mars {
    fn name(&self) -> String {
        "Mars".to_string()
    }
    fn effect(&self) -> PlanetEffect {
        fn apply(p: &mut Planetarium) {
            p.level_up(HandRank::FourOfAKind);
        }
        Arc::new(Mutex::new(apply))
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct Neptune {}

impl Planet for Neptune {
    fn name(&self) -> String {
        "Neptune".to_string()
    }
    fn effect(&self) -> PlanetEffect {
        fn apply(p: &mut Planetarium) {
            p.level_up(HandRank::StraightFlush);
        }
        Arc::new(Mutex::new(apply))
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct PlanetX {}

impl Planet for PlanetX {
    fn name(&self) -> String {
        "Planet X".to_string()
    }
    fn effect(&self) -> PlanetEffect {
        fn apply(p: &mut Planetarium) {
            p.level_up(HandRank::FiveOfAKind);
        }
        Arc::new(Mutex::new(apply))
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct Ceres {}

impl Planet for Ceres {
    fn name(&self) -> String {
        "Ceres".to_string()
    }
    fn effect(&self) -> PlanetEffect {
        fn apply(p: &mut Planetarium) {
            p.level_up(HandRank::FlushHouse);
        }
        Arc::new(Mutex::new(apply))
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct Eris {}

impl Planet for Eris {
    fn name(&self) -> String {
        "Eris".to_string()
    }
    fn effect(&self) -> PlanetEffect {
        fn apply(p: &mut Planetarium) {
            p.level_up(HandRank::FlushFive);
        }
        Arc::new(Mutex::new(apply))
    }
}

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
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Copy)]
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

// Planetarium tracks hand leveling and play count
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
    royalflush: Level,
    fiveofkind: Level,
    flushhouse: Level,
    flushfive: Level,
}

impl Planetarium {
    pub fn new() -> Self {
        return Planetarium {
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
            royalflush: Level {
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
        };
    }

    pub fn play(&mut self, rank: HandRank) -> Level {
        match rank {
            HandRank::HighCard => self.highcard.plays += 1,
            HandRank::OnePair => self.onepair.plays += 1,
            HandRank::TwoPair => self.twopair.plays += 1,
            HandRank::ThreeOfAKind => self.threeofkind.plays += 1,
            HandRank::Straight => self.straight.plays += 1,
            HandRank::Flush => self.flush.plays += 1,
            HandRank::FullHouse => self.fullhouse.plays += 1,
            HandRank::FourOfAKind => self.fourofkind.plays += 1,
            HandRank::StraightFlush => self.straightflush.plays += 1,
            HandRank::RoyalFlush => self.royalflush.plays += 1,
            HandRank::FiveOfAKind => self.fiveofkind.plays += 1,
            HandRank::FlushHouse => self.flushhouse.plays += 1,
            HandRank::FlushFive => self.flushfive.plays += 1,
        }
        return self.level(rank);
    }

    fn level(&self, rank: HandRank) -> Level {
        match rank {
            HandRank::HighCard => return self.highcard.clone(),
            HandRank::OnePair => return self.onepair.clone(),
            HandRank::TwoPair => return self.twopair.clone(),
            HandRank::ThreeOfAKind => return self.threeofkind.clone(),
            HandRank::Straight => return self.straight.clone(),
            HandRank::Flush => return self.flush.clone(),
            HandRank::FullHouse => return self.fullhouse.clone(),
            HandRank::FourOfAKind => return self.fourofkind.clone(),
            HandRank::StraightFlush => return self.straightflush.clone(),
            HandRank::RoyalFlush => return self.royalflush.clone(),
            HandRank::FiveOfAKind => return self.fiveofkind.clone(),
            HandRank::FlushHouse => return self.flushhouse.clone(),
            HandRank::FlushFive => return self.flushfive.clone(),
        }
    }

    pub fn level_up(&mut self, rank: HandRank) {
        match rank {
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
                self.threeofkind.level += 2;
                self.threeofkind.chips += 20;
                self.threeofkind.mult += 1;
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

                self.royalflush.level += 1;
                self.royalflush.chips += 40;
                self.royalflush.mult += 4;
            }
            HandRank::RoyalFlush => {}
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
        }
    }
}
