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
