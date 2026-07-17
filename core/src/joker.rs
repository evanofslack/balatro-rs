use crate::card::{Card, Enhancement, Suit, Value};
use crate::effect::Effects;
use crate::game::Game;
use crate::hand::{MadeHand, SelectHand};
use std::sync::{Arc, Mutex};
use strum::IntoEnumIterator;

pub use balatro_types::joker::*;

/// `balatro_types::Jokers` already supplies all static joker data
/// (name/rarity/cost/desc/category/etc.) as inherent methods.
/// The one thing that can't live there is game behavior.
pub trait JokerEffects {
    fn effects(&self, game: &Game) -> Vec<Effects>;

    /// Whether this joker's `effects()` has real logic behind it.
    /// Keeps unimplemented jokers out of shop/pack generation.
    fn is_implemented(&self) -> bool;
}

impl JokerEffects for Jokers {
    fn effects(&self, game: &Game) -> Vec<Effects> {
        let _ = game;
        match self {
            Self::TheJoker(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    g.mult += 4;
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::GreedyJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    let diamonds = hand
                        .hand
                        .cards()
                        .iter()
                        .filter(|c| c.matches_suit(Suit::Diamond))
                        .count();
                    g.mult += diamonds * 3
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::LustyJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    let hearts = hand
                        .hand
                        .cards()
                        .iter()
                        .filter(|c| c.matches_suit(Suit::Heart))
                        .count();
                    g.mult += hearts * 3
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::WrathfulJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    let spades = hand
                        .hand
                        .cards()
                        .iter()
                        .filter(|c| c.matches_suit(Suit::Spade))
                        .count();
                    g.mult += spades * 3
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::GluttonousJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    let clubs = hand
                        .hand
                        .cards()
                        .iter()
                        .filter(|c| c.matches_suit(Suit::Club))
                        .count();
                    g.mult += clubs * 3
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::JollyJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_pair().is_some() {
                        g.mult += 8
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::ZanyJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_three_of_kind().is_some() {
                        g.mult += 12
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::MadJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_two_pair().is_some() {
                        g.mult += 10
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::CrazyJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_straight().is_some() {
                        g.mult += 12
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::DrollJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_flush().is_some() {
                        g.mult += 10
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::SlyJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_pair().is_some() {
                        g.chips += 50
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::WilyJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_three_of_kind().is_some() {
                        g.chips += 100
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::CleverJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_two_pair().is_some() {
                        g.chips += 80
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::DeviousJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_straight().is_some() {
                        g.chips += 100
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::CraftyJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_flush().is_some() {
                        g.chips += 80
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::HalfJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.len() <= 3 {
                        g.mult += 20;
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::JokerStencil(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    let empty = g.config.joker_slots.saturating_sub(g.jokers.len());
                    if empty > 0 {
                        g.mult *= empty;
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::Banner(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    g.chips += 30 * g.discards;
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::MysticSummit(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    if g.discards == 0 {
                        g.mult += 15;
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::Fibonacci(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    for card in _hand.hand.cards() {
                        if card.value == Value::Ace
                            || card.value == Value::Two
                            || card.value == Value::Three
                            || card.value == Value::Five
                            || card.value == Value::Eight
                        {
                            g.mult += 8;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::ScaryFace(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    for card in _hand.hand.cards() {
                        if card.is_face_card() {
                            g.chips += 30;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::AbstractJoker(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    g.mult += g.jokers.len() * 3;
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::Pareidolia(_) => {
                fn apply(_g: &mut Game, hand: &mut MadeHand) {
                    for card in &mut hand.all {
                        card.face_card_override = true;
                    }
                    let cards: Vec<Card> = hand
                        .hand
                        .cards()
                        .into_iter()
                        .map(|mut c| {
                            c.face_card_override = true;
                            c
                        })
                        .collect();
                    hand.hand = SelectHand::new(cards);
                }
                vec![Effects::OnModifyHand(Arc::new(Mutex::new(apply)))]
            }
            Self::EvenSteven(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    for card in _hand.hand.cards() {
                        if card.is_even() {
                            g.mult += 4;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::OddTodd(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    for card in _hand.hand.cards() {
                        if card.is_odd() {
                            g.chips += 31;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::Scholar(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    for card in _hand.hand.cards() {
                        if card.value == Value::Ace {
                            g.chips += 20;
                            g.mult += 4;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::BusinessCard(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    for card in _hand.hand.cards() {
                        if card.is_face_card() && g.prob_roll(1, 2) {
                            g.money += 2;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::FacelessJoker(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    if _hand.all.len() >= 5 {
                        g.money += 5;
                    }
                }
                vec![Effects::OnDiscard(Arc::new(Mutex::new(apply)))]
            }
            Self::Baron(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    let kings = g
                        .available
                        .not_selected()
                        .iter()
                        .filter(|c| c.value == Value::King)
                        .count();
                    for _ in 0..kings {
                        g.mult = (g.mult as f64 * 1.5) as usize;
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::MidasMask(_) => {
                fn apply(_g: &mut Game, hand: &mut MadeHand) {
                    for card in &mut hand.all {
                        card.enhancement = Some(Enhancement::Gold);
                    }
                    let cards: Vec<Card> = hand
                        .hand
                        .cards()
                        .into_iter()
                        .map(|mut c| {
                            c.enhancement = Some(Enhancement::Gold);
                            c
                        })
                        .collect();
                    hand.hand = SelectHand::new(cards);
                }
                vec![Effects::OnModifyHand(Arc::new(Mutex::new(apply)))]
            }
            Self::Photograph(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    for card in _hand.hand.cards() {
                        if card.is_face_card() {
                            g.mult *= 2;
                            break;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::ReservedParking(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    for card in g.available.not_selected() {
                        if card.is_face_card() && g.prob_roll(1, 2) {
                            g.money += 1;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::BaseballCard(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    let uncommon = g
                        .jokers
                        .iter()
                        .filter(|j| j.rarity() == Rarity::Uncommon)
                        .count();
                    for _ in 0..uncommon {
                        g.mult = (g.mult as f64 * 1.5) as usize;
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::Bull(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    g.chips += g.money * 2;
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::WalkieTalkie(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    for card in _hand.hand.cards() {
                        if card.value == Value::Ten || card.value == Value::Four {
                            g.chips += 10;
                            g.mult += 4;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::SmileyFace(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    for card in _hand.hand.cards() {
                        if card.is_face_card() {
                            g.mult += 5;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::GoldenTicket(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    for card in _hand.hand.cards() {
                        if card.enhancement == Some(Enhancement::Gold) {
                            g.money += 4;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::Acrobat(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    if g.plays == 0 {
                        g.mult *= 3;
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::RoughGem(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    for card in _hand.hand.cards() {
                        if card.matches_suit(Suit::Diamond) {
                            g.money += 1;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::Bloodstone(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    for card in _hand.hand.cards() {
                        if card.matches_suit(Suit::Heart) && g.prob_roll(1, 2) {
                            g.mult += g.mult / 2;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::Arrowhead(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    for card in hand.hand.cards() {
                        if card.matches_suit(Suit::Spade) {
                            g.chips += 50;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::OnyxAgate(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    for card in hand.hand.cards() {
                        if card.matches_suit(Suit::Club) {
                            g.mult += 7;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::ShootTheMoon(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    for card in g.available.not_selected() {
                        if card.value == Value::Queen {
                            g.mult += 13;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::RaisedFist(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    // no dedicated numeric "rank" accessor exists; chips() is the
                    // closest analog to the card's face rank
                    // TODO: need to do this with rank ord?
                    let lowest = g.available.not_selected().iter().map(|c| c.chips()).min();
                    if let Some(lowest) = lowest {
                        g.mult += lowest * 2;
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::TheDuo(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_pair().is_some() {
                        g.mult *= 2;
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::TheTrio(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_three_of_kind().is_some() {
                        g.mult *= 3;
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::TheFamily(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_four_of_kind().is_some() {
                        g.mult *= 4;
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::TheOrder(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_straight().is_some() {
                        g.mult *= 3;
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::TheTribe(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_flush().is_some() {
                        g.mult *= 2;
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::StoneJoker(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    let count = g
                        .full_deck()
                        .iter()
                        .filter(|c| c.enhancement == Some(Enhancement::Stone))
                        .count();
                    g.chips += count * 25;
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::SteelJoker(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    let count = g
                        .full_deck()
                        .iter()
                        .filter(|c| c.enhancement == Some(Enhancement::Steel))
                        .count();
                    g.mult += (g.mult as f64 * 0.2 * count as f64) as usize;
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::BlueJoker(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    g.chips += g.deck.len() * 2;
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::Erosion(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    let deficit = 52usize.saturating_sub(g.full_deck().len());
                    g.mult += deficit * 4;
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::DriversLicense(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    let enhanced = g
                        .full_deck()
                        .iter()
                        .filter(|c| c.enhancement.is_some())
                        .count();
                    if enhanced >= 16 {
                        g.mult *= 3;
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            _ => vec![],
        }
    }

    fn is_implemented(&self) -> bool {
        matches!(
            self,
            Self::TheJoker(_)
                | Self::GreedyJoker(_)
                | Self::LustyJoker(_)
                | Self::WrathfulJoker(_)
                | Self::GluttonousJoker(_)
                | Self::JollyJoker(_)
                | Self::ZanyJoker(_)
                | Self::MadJoker(_)
                | Self::CrazyJoker(_)
                | Self::DrollJoker(_)
                | Self::SlyJoker(_)
                | Self::WilyJoker(_)
                | Self::CleverJoker(_)
                | Self::DeviousJoker(_)
                | Self::CraftyJoker(_)
                | Self::HalfJoker(_)
                | Self::JokerStencil(_)
                | Self::Banner(_)
                | Self::MysticSummit(_)
                | Self::Fibonacci(_)
                | Self::ScaryFace(_)
                | Self::AbstractJoker(_)
                | Self::Pareidolia(_)
                | Self::EvenSteven(_)
                | Self::OddTodd(_)
                | Self::Scholar(_)
                | Self::BusinessCard(_)
                | Self::FacelessJoker(_)
                | Self::Baron(_)
                | Self::MidasMask(_)
                | Self::Photograph(_)
                | Self::ReservedParking(_)
                | Self::BaseballCard(_)
                | Self::Bull(_)
                | Self::WalkieTalkie(_)
                | Self::SmileyFace(_)
                | Self::GoldenTicket(_)
                | Self::Acrobat(_)
                | Self::RoughGem(_)
                | Self::Bloodstone(_)
                | Self::Arrowhead(_)
                | Self::OnyxAgate(_)
                | Self::ShootTheMoon(_)
                | Self::RaisedFist(_)
                | Self::TheDuo(_)
                | Self::TheTrio(_)
                | Self::TheFamily(_)
                | Self::TheOrder(_)
                | Self::TheTribe(_)
                | Self::StoneJoker(_)
                | Self::SteelJoker(_)
                | Self::BlueJoker(_)
                | Self::Erosion(_)
                | Self::DriversLicense(_)
        )
    }
}

/// Only returns jokers with real `effects()` behavior implemented - callers
/// (shop/pack generation) must never offer a not-yet-implemented joker to
/// the player. A free function rather than an inherent method since
/// `Jokers` is now a foreign type (orphan rule blocks inherent impls on it).
pub(crate) fn jokers_by_rarity(rarity: Rarity) -> Vec<Jokers> {
    Jokers::iter()
        .filter(|j| j.rarity() == rarity && j.is_implemented())
        .collect()
}

/// `Display` can't be implemented on `Jokers` directly (foreign trait +
/// foreign type both fail the orphan rule), so this free function stands in
/// for the old `impl fmt::Display for Jokers`.
pub fn joker_display(j: &Jokers) -> String {
    format!("{} [${}, {}] {}", j.name(), j.cost(), j.rarity(), j.desc())
}

#[cfg(test)]
mod tests {
    use crate::card::{Card, Enhancement, Suit, Value};
    use crate::hand::SelectHand;
    use crate::stage::{Blind, Stage};

    use super::*;

    // balatro_types::Jokers now defines all jokers but only a subset have
    // `effects()` behavior implemented. Shop/pack generation
    // must never offer joker that silently does nothing.
    #[test]
    fn test_exactly_54_jokers_implemented() {
        let count = Jokers::iter().filter(|j| j.is_implemented()).count();
        assert_eq!(count, 54);
    }

    #[test]
    fn test_jokers_by_rarity_never_returns_unimplemented() {
        for rarity in [
            Rarity::Common,
            Rarity::Uncommon,
            Rarity::Rare,
            Rarity::Legendary,
        ] {
            for j in jokers_by_rarity(rarity) {
                assert!(
                    j.is_implemented(),
                    "jokers_by_rarity({rarity}) returned unimplemented joker {}",
                    j.name()
                );
            }
        }
    }

    #[test]
    fn test_shop_joker_generation_never_produces_unimplemented() {
        use rand::SeedableRng;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(42);
        let gen = crate::shop::JokerGenerator::new();
        for _ in 0..500 {
            let joker = gen.gen_joker(1, &[], &mut rng);
            assert!(
                joker.is_implemented(),
                "shop generated unimplemented joker {}",
                joker.name()
            );
        }
    }

    fn score_before_after_joker(joker: Jokers, hand: SelectHand, before: usize, after: usize) {
        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };

        // First score without joker
        let score = g.calc_score(hand.best_hand().unwrap());
        assert_eq!(score, before);

        // Buy (and apply) the joker
        g.money += 1000; // Give adequate money to buy
        g.stage = Stage::Shop();
        g.shop.jokers.push(joker.clone());
        g.buy_joker(joker).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        // Second score with joker applied
        let score = g.calc_score(hand.best_hand().unwrap());
        assert_eq!(score, after);
    }

    #[test]
    fn test_the_joker() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);

        // Score Ace high without joker
        // High card (level 1) -> 5 chips, 1 mult
        // Played cards (1 ace) -> 11 chips
        // (5 + 11) * (1) = 16
        let before = 16;
        // Score Ace high with the Joker
        // High card (level 1) -> 5 chips, 1 mult
        // Played cards (1 ace) -> 11 chips
        // Joker (The Joker) -> 4 mult
        // (5 + 11) * (1 + 4) = 80
        let after = 80;

        let j = Jokers::TheJoker(TheJoker::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_lusty_joker() {
        let ah = Card::new(Value::Ace, Suit::Heart);
        let ac = Card::new(Value::Ace, Suit::Club);
        let ad = Card::new(Value::Ace, Suit::Diamond);
        let hand = SelectHand::new(vec![ah, ah, ac, ad]);

        // Score 4ok without joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // (60 + 44) * (7) = 728
        let before = 728;
        // Score 4ok (2 hearts) with joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // joker w/ 2 hearts = +6 mult
        // (60 + 44) * (7 + 6) = 1352
        let after = 1352;

        let j = Jokers::LustyJoker(LustyJoker::default());
        score_before_after_joker(j, hand, before, after)
    }

    #[test]
    fn test_greedy_joker() {
        let ah = Card::new(Value::Ace, Suit::Heart);
        let ad = Card::new(Value::Ace, Suit::Diamond);
        let hand = SelectHand::new(vec![ad, ad, ad, ah]);

        // Score 4ok without joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // (60 + 44) * (7) = 728
        let before = 728;
        // Score 4ok (3 diamonds) with joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // joker w/ 3 diamonds = +9 mult
        // (60 + 44) * (7 + 9) = 1664
        let after = 1664;

        let j = Jokers::GreedyJoker(GreedyJoker::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_wrathful_joker() {
        let asp = Card::new(Value::Ace, Suit::Spade);
        let ad = Card::new(Value::Ace, Suit::Diamond);
        let hand = SelectHand::new(vec![asp, ad, ad, ad]);

        // Score 4ok without joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // (60 + 44) * (7) = 728
        let before = 728;
        // Score 4ok (1 spade) with joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // joker w/ 1 spade = +3 mult
        // (60 + 44) * (7 + 3) = 1040
        let after = 1040;

        let j = Jokers::WrathfulJoker(WrathfulJoker::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_gluttonous_joker() {
        let ac = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![ac, ac, ac, ac]);

        // Score 4ok without joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // (60 + 44) * (7) = 728
        let before = 728;
        // Score 4ok (4 clubs) with joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // joker w/ 4 clubs = +12 mult
        // (60 + 44) * (7 + 12) = 1976
        let after = 1976;

        let j = Jokers::GluttonousJoker(GluttonousJoker::default());
        score_before_after_joker(j, hand, before, after)
    }

    #[test]
    fn test_jolly_joker() {
        let ac = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![ac, ac, ac, ac]);

        // Score 4ok without joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // (60 + 44) * (7) = 728
        let before = 728;
        // Score 4ok with joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // joker w/ pair = +8 mult
        // (60 + 44) * (7 + 8) = 1560
        let after = 1560;

        let j = Jokers::JollyJoker(JollyJoker::default());
        score_before_after_joker(j, hand, before, after)
    }

    #[test]
    fn test_zany_joker() {
        let ac = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![ac, ac, ac, ac]);

        // Score 4ok without joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // (60 + 44) * (7) = 728
        let before = 728;
        // Score 4ok with joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // joker w/ 3ok = +12 mult
        // (60 + 44) * (7 + 12) = 1976
        let after = 1976;

        let j = Jokers::ZanyJoker(ZanyJoker::default());
        score_before_after_joker(j, hand, before, after)
    }

    #[test]
    fn test_mad_joker() {
        let ac = Card::new(Value::Ace, Suit::Club);
        let kc = Card::new(Value::King, Suit::Club);
        let hand = SelectHand::new(vec![ac, ac, kc, kc]);

        // Score two pair without joker
        // two pair (level 1) -> 20 chips, 2 mult
        // Played cards (2 ace, 2 king) -> 42 chips
        // (20 + 42) * (2) = 124
        let before = 124;
        let j = Jokers::MadJoker(MadJoker::default());
        // Score two pair with joker
        // two pair (level 1) -> 20 chips, 2 mult
        // Played cards (2 ace, 2 king) -> 42 chips
        // joker w/ two pair = +10 mult
        // (20 + 42) * (2 + 10) = 744
        let after = 744;

        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_crazy_joker() {
        let two = Card::new(Value::Two, Suit::Club);
        let three = Card::new(Value::Three, Suit::Club);
        let four = Card::new(Value::Four, Suit::Club);
        let five = Card::new(Value::Five, Suit::Club);
        let six = Card::new(Value::Six, Suit::Heart);
        let hand = SelectHand::new(vec![two, three, four, five, six]);

        // Score straight without joker
        // straight (level 1) -> 30 chips, 4 mult
        // Played cards (2, 3, 4, 5, 6) -> 20 chips
        // (20 + 30) * (4) = 200
        let before = 200;
        // Score straight with joker
        // straight (level 1) -> 30 chips, 4 mult
        // Played cards (2, 3, 4, 5, 6) -> 20 chips
        // joker w/ straight = +12 mult
        // (20 + 30) * (4 + 12) = 800
        let after = 800;

        let j = Jokers::CrazyJoker(CrazyJoker::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_droll_joker() {
        let two = Card::new(Value::Two, Suit::Club);
        let three = Card::new(Value::Three, Suit::Club);
        let four = Card::new(Value::Four, Suit::Club);
        let five = Card::new(Value::Five, Suit::Club);
        let ten = Card::new(Value::Ten, Suit::Club);
        let hand = SelectHand::new(vec![two, three, four, five, ten]);

        // Score flush without joker
        // flush (level 1) -> 35 chips, 4 mult
        // Played cards (2, 3, 4, 5, 10) -> 24 chips
        // (24 + 35) * (4) = 236
        let before = 236;
        // Score flush with joker
        // flush (level 1) -> 35 chips, 4 mult
        // Played cards (2, 3, 4, 5, 10) -> 24 chips
        // joker w/ flush = +10 mult
        // (24 + 35) * (4 + 10) = 826
        let after = 826;

        let j = Jokers::DrollJoker(DrollJoker::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_the_duo_with_pair() {
        let ac = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![ac, ac, ac, ac]);

        // Score 4ok without joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // (60 + 44) * (7) = 728
        let before = 728;
        // Score 4ok with joker (contains a pair)
        // joker w/ pair = X2 mult
        // (60 + 44) * (7 * 2) = 1456
        let after = 1456;

        let j = Jokers::TheDuo(TheDuo::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_the_duo_no_pair() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);

        let before = 16;
        let after = 16;

        let j = Jokers::TheDuo(TheDuo::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_the_trio_with_three_of_kind() {
        let ac = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![ac, ac, ac, ac]);

        let before = 728;
        // joker w/ three of a kind = X3 mult
        // (60 + 44) * (7 * 3) = 2184
        let after = 2184;

        let j = Jokers::TheTrio(TheTrio::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_the_trio_no_three_of_kind() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);

        let before = 16;
        let after = 16;

        let j = Jokers::TheTrio(TheTrio::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_the_family_with_four_of_kind() {
        let ac = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![ac, ac, ac, ac]);

        let before = 728;
        // joker w/ four of a kind = X4 mult
        // (60 + 44) * (7 * 4) = 2912
        let after = 2912;

        let j = Jokers::TheFamily(TheFamily::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_the_family_no_four_of_kind() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);

        let before = 16;
        let after = 16;

        let j = Jokers::TheFamily(TheFamily::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_the_order_with_straight() {
        let two = Card::new(Value::Two, Suit::Club);
        let three = Card::new(Value::Three, Suit::Club);
        let four = Card::new(Value::Four, Suit::Club);
        let five = Card::new(Value::Five, Suit::Club);
        let six = Card::new(Value::Six, Suit::Heart);
        let hand = SelectHand::new(vec![two, three, four, five, six]);

        // Score straight without joker
        // straight (level 1) -> 30 chips, 4 mult
        // Played cards (2, 3, 4, 5, 6) -> 20 chips
        // (20 + 30) * (4) = 200
        let before = 200;
        // joker w/ straight = X3 mult
        // (20 + 30) * (4 * 3) = 600
        let after = 600;

        let j = Jokers::TheOrder(TheOrder::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_the_order_no_straight() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);

        let before = 16;
        let after = 16;

        let j = Jokers::TheOrder(TheOrder::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_the_tribe_with_flush() {
        let two = Card::new(Value::Two, Suit::Club);
        let three = Card::new(Value::Three, Suit::Club);
        let four = Card::new(Value::Four, Suit::Club);
        let five = Card::new(Value::Five, Suit::Club);
        let ten = Card::new(Value::Ten, Suit::Club);
        let hand = SelectHand::new(vec![two, three, four, five, ten]);

        // Score flush without joker
        // flush (level 1) -> 35 chips, 4 mult
        // Played cards (2, 3, 4, 5, 10) -> 24 chips
        // (24 + 35) * (4) = 236
        let before = 236;
        // joker w/ flush = X2 mult
        // (24 + 35) * (4 * 2) = 472
        let after = 472;

        let j = Jokers::TheTribe(TheTribe::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_the_tribe_no_flush() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);

        let before = 16;
        let after = 16;

        let j = Jokers::TheTribe(TheTribe::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_stone_joker_counts_deck_and_discarded() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::StoneJoker(StoneJoker::default());

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        let mut deck_stone = Card::new(Value::Two, Suit::Diamond);
        deck_stone.enhancement = Some(Enhancement::Stone);
        g.deck.push(deck_stone);

        let mut discarded_stone = Card::new(Value::Three, Suit::Diamond);
        discarded_stone.enhancement = Some(Enhancement::Stone);
        g.discarded.push(discarded_stone);

        assert_eq!(g.calc_score(hand.best_hand().unwrap()), 16);

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // 2 Stone cards total (1 undrawn in deck, 1 already discarded) -> +50 chips
        // (5 + 11 + 50) * 1 = 66
        assert_eq!(g.calc_score(hand.best_hand().unwrap()), 66);
    }

    #[test]
    fn test_stone_joker_no_stone_cards() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);

        let before = 16;
        let after = 16;

        let j = Jokers::StoneJoker(StoneJoker::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_steel_joker_counts_deck_and_discarded() {
        let ac = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![ac, ac, ac, ac]);
        let j = Jokers::SteelJoker(SteelJoker::default());

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        let mut deck_steel = Card::new(Value::Two, Suit::Diamond);
        deck_steel.enhancement = Some(Enhancement::Steel);
        g.deck.push(deck_steel);

        let mut discarded_steel = Card::new(Value::Three, Suit::Diamond);
        discarded_steel.enhancement = Some(Enhancement::Steel);
        g.discarded.push(discarded_steel);

        // Score 4ok without joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // (60 + 44) * 7 = 728
        assert_eq!(g.calc_score(hand.best_hand().unwrap()), 728);

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // 2 Steel cards total -> mult += floor(7 * 0.2 * 2) = 7 + 2 = 9
        // (60 + 44) * 9 = 936
        assert_eq!(g.calc_score(hand.best_hand().unwrap()), 936);
    }

    #[test]
    fn test_steel_joker_no_steel_cards() {
        let ac = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![ac, ac, ac, ac]);

        let before = 728;
        let after = 728;

        let j = Jokers::SteelJoker(SteelJoker::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_blue_joker_excludes_drawn_cards() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::BlueJoker(BlueJoker::default());

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        // simulate 10 cards already drawn out of the deck this round
        let drawn = g.deck.draw(10).unwrap();
        g.available.extend(drawn);
        assert_eq!(g.deck.len(), 42);

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // BlueJoker only counts the undrawn remainder (42), not the 10 held cards
        // High card (level 1): 5 chips, 1 mult
        // Played (1 ace): 11 chips
        // BlueJoker: +2 chips per remaining deck card = 42 * 2 = 84
        // (5 + 11 + 84) * 1 = 100
        assert_eq!(g.calc_score(hand.best_hand().unwrap()), 100);
    }

    #[test]
    fn test_blue_joker_fresh_deck() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);

        // Full 52-card deck untouched -> +2*52 = 104 chips
        // (5 + 11 + 104) * 1 = 120
        let before = 16;
        let after = 120;

        let j = Jokers::BlueJoker(BlueJoker::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_erosion_with_deficit() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::Erosion(Erosion::default());

        let mut g = Game::default();
        // draw 10 cards into hand -- still owned this run, not a deficit
        let drawn = g.deck.draw(10).unwrap();
        g.available.extend(drawn);
        // destroy exactly 1 card -- a genuine deficit of 1 below the 52-card starting size
        let victim = g.deck.cards()[0];
        g.destroy_card(victim.id);
        assert_eq!(g.full_deck().len(), 51);

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // The 10 drawn cards are still owned (no deficit from those); only the
        // destroyed card counts -> deficit of 1 -> +4 mult
        // (5 + 11) * (1 + 4) = 80
        assert_eq!(g.calc_score(hand.best_hand().unwrap()), 80);
    }

    #[test]
    fn test_erosion_no_deficit() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);

        let before = 16;
        let after = 16;

        let j = Jokers::Erosion(Erosion::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_drivers_license_with_enough_enhanced_cards() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::DriversLicense(DriversLicense::default());

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };

        // enhance 10 cards already in the deck
        let ids: Vec<usize> = g.deck.cards().iter().take(10).map(|c| c.id).collect();
        for id in ids {
            g.mutate_card(id, |c| c.enhancement = Some(Enhancement::Bonus));
        }

        // 6 more enhanced cards already discarded this round (not in the deck at all)
        for _ in 0..6 {
            let mut c = Card::new(Value::Two, Suit::Diamond);
            c.enhancement = Some(Enhancement::Bonus);
            g.discarded.push(c);
        }

        assert_eq!(
            g.full_deck()
                .iter()
                .filter(|c| c.enhancement.is_some())
                .count(),
            16
        );

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // 16 enhanced cards across deck+discarded (>= 16 threshold) -> X3 mult
        // (5 + 11) * (1 * 3) = 48
        assert_eq!(g.calc_score(hand.best_hand().unwrap()), 48);
    }

    #[test]
    fn test_drivers_license_not_enough_enhanced_cards() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);

        let before = 16;
        let after = 16;

        let j = Jokers::DriversLicense(DriversLicense::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_sly_joker() {
        let ac = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![ac, ac, ac, ac]);

        // Score 4ok without joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // (60 + 44) * (7) = 728
        let before = 728;
        // Score 4ok with joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // joker w/ pair = +50 chips
        // (60 + 44 + 50) * (7) = 1078
        let after = 1078;

        let j = Jokers::SlyJoker(SlyJoker::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_wily_joker() {
        let ac = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![ac, ac, ac, ac]);

        // Score 4ok without joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // (60 + 44) * (7) = 728
        let before = 728;
        // Score 4ok with joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // joker w/ 3ok = +100 chips
        // (60 + 44 + 100) * (7) = 1428
        let after = 1428;

        let j = Jokers::WilyJoker(WilyJoker::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_clever_joker() {
        let ac = Card::new(Value::Ace, Suit::Club);
        let kc = Card::new(Value::King, Suit::Club);
        let hand = SelectHand::new(vec![ac, ac, kc, kc]);

        // Score two pair without joker
        // two pair (level 1) -> 20 chips, 2 mult
        // Played cards (2 ace, 2 king) -> 42 chips
        // (20 + 42) * (2) = 124
        let before = 124;
        // Score two pair with joker
        // two pair (level 1) -> 20 chips, 2 mult
        // Played cards (2 ace, 2 king) -> 42 chips
        // joker w/ two pair = +80 chips
        // (20 + 42 + 80) * (2) = 284
        let after = 284;

        let j = Jokers::CleverJoker(CleverJoker::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_devious_joker() {
        let two = Card::new(Value::Two, Suit::Club);
        let three = Card::new(Value::Three, Suit::Club);
        let four = Card::new(Value::Four, Suit::Club);
        let five = Card::new(Value::Five, Suit::Club);
        let six = Card::new(Value::Six, Suit::Heart);
        let hand = SelectHand::new(vec![two, three, four, five, six]);

        // Score straight without joker
        // straight (level 1) -> 30 chips, 4 mult
        // Played cards (2, 3, 4, 5, 6) -> 20 chips
        // (20 + 30) * (4) = 200
        let before = 200;
        // Score straight with joker
        // straight (level 1) -> 30 chips, 4 mult
        // Played cards (2, 3, 4, 5, 6) -> 20 chips
        // joker w/ straight = +100 chips
        // (20 + 30 + 100) * (4) = 600
        let after = 600;

        let j = Jokers::DeviousJoker(DeviousJoker::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_crafty_joker() {
        let two = Card::new(Value::Two, Suit::Club);
        let three = Card::new(Value::Three, Suit::Club);
        let four = Card::new(Value::Four, Suit::Club);
        let five = Card::new(Value::Five, Suit::Club);
        let ten = Card::new(Value::Ten, Suit::Club);
        let hand = SelectHand::new(vec![two, three, four, five, ten]);

        // Score flush without joker
        // flush (level 1) -> 35 chips, 4 mult
        // Played cards (2, 3, 4, 5, 10) -> 24 chips
        // (24 + 35) * (4) = 236
        let before = 236;
        // Score flush with joker
        // flush (level 1) -> 35 chips, 4 mult
        // Played cards (2, 3, 4, 5, 10) -> 24 chips
        // joker w/ flush = +80 chips
        // (24 + 35 + 80) * (4) = 556
        let after = 556;
        let j = Jokers::CraftyJoker(CraftyJoker::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_half_joker() {
        let ac = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![ac, ac, ac]);

        // Score 3ok without joker
        // 3ok (level 1) -> 30 chips, 3 mult
        // Played cards (3 ace) -> 33 chips
        // (30 + 33) * 3 = 189
        let before = 189;
        // Score 3ok with joker
        // 3ok (level 1) -> 30 chips, 3 mult
        // Played cards (3 ace) -> 33 chips
        // joker w/ <=3 cards = +20 mult
        // (30 + 33) * (3 + 20) = 1449
        let after = 1449;

        let j = Jokers::HalfJoker(HalfJoker::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_joker_stencil() {
        let ace = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![ace]);
        let best = hand.best_hand().unwrap();

        // High card (level 1) -> 5 chips, 1 mult
        // Played cards (1 ace) -> 11 chips
        // (5 + 11) * 1 = 16
        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        assert_eq!(g.calc_score(best.clone()), 16);

        // Stencil alone in 5 slots = 4 empty -> X4
        // (5 + 11) * (1 * 4) = 64
        let j = Jokers::JokerStencil(JokerStencil::default());
        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        assert_eq!(g.calc_score(best.clone()), 64);

        // Add another joker -> 3 empty -> X3
        let j2 = Jokers::Banner(Banner::default());
        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j2.clone());
        g.buy_joker(j2).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        // (5 + 11 + 4*30) * (1 * 3) = 136 * 3 = 408
        assert_eq!(g.calc_score(best.clone()), 408);
    }

    #[test]
    fn test_banner_joker() {
        let ace = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![ace]);
        let best = hand.best_hand().unwrap();
        let j = Jokers::Banner(Banner::default());

        // High card (level 1) -> 5 chips, 1 mult
        // Played cards (1 ace) -> 11 chips
        // (5 + 11) * (1) = 16
        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        assert_eq!(g.calc_score(best.clone()), 16);

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // Banner: 4 discards * 30 chips = +120
        // (5 + 11 + 120) * 1 = 136
        assert_eq!(g.calc_score(best.clone()), 136);

        g.discards = 0;
        // Banner: +0 chips
        // (5 + 11 + 0) * 1 = 16
        assert_eq!(g.calc_score(best.clone()), 16);
    }

    #[test]
    fn test_mystic_summit() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let best = hand.best_hand().unwrap();
        let j = Jokers::MysticSummit(MysticSummit::default());

        // High card (level 1): 5 chips, 1 mult
        // Played (1 ace): 11 chips
        // (5 + 11) * 1 = 16
        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        assert_eq!(g.calc_score(best.clone()), 16);

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // discards = 4 (default), so Mystic Summit does NOT fire
        // (5 + 11) * 1 = 16
        assert_eq!(g.calc_score(best.clone()), 16);

        // Now set discards to 0 -> +15 mult
        g.discards = 0;
        // (5 + 11) * (1 + 15) = 16 * 16 = 256
        assert_eq!(g.calc_score(best.clone()), 256);
    }

    #[test]
    fn test_fibonacci() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let two = Card::new(Value::Two, Suit::Heart);
        let three = Card::new(Value::Three, Suit::Heart);
        let five = Card::new(Value::Five, Suit::Heart);
        let eight = Card::new(Value::Eight, Suit::Heart);
        let hand = SelectHand::new(vec![ace, two, three, five, eight]);
        let j = Jokers::Fibonacci(Fibonacci::default());

        // Flush (level 1): 35 chips, 4 mult
        // Played (5 cards): 11 + 2 + 3 + 5 + 8 = 29 chips
        // (35 + 29) * 4 = 256
        let before = 256;

        // Fibonacci: 1 ace, 1 two, 1 three, 1 five, 1 eight -> +8 mult each
        // (35 + 29) * (4 + 40) = 64 * 44 = 2816
        let after = 2816;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_scary_face() {
        let ace = Card::new(Value::Ace, Suit::Club);
        let king = Card::new(Value::King, Suit::Club);
        let queen = Card::new(Value::Queen, Suit::Spade);
        let jack = Card::new(Value::Jack, Suit::Heart);
        let ten = Card::new(Value::Ten, Suit::Diamond);
        let hand = SelectHand::new(vec![ace, jack, queen, king, ten]);
        let j = Jokers::ScaryFace(ScaryFace::default());

        // Straight (level 1): 30 chips, 4 mult
        // Played (5 cards): 11 + 10 + 10 + 10 + 10 = 51 chips
        // (30 + 51) * 4 = 324
        let before = 324;

        // Scary Face: jack, queen, king -> +30 chips each
        // (30 + 51 + 90) * 4 = 684
        let after = 684;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_abstract_joker() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let best = hand.best_hand().unwrap();

        // High card (level 1): 5 chips, 1 mult
        // Played (1 ace): 11 chips
        // (5 + 11) * 1 = 16
        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        assert_eq!(g.calc_score(best.clone()), 16);

        // Buy Abstract Joker -> 1 joker, +3 mult
        // (5 + 11) * (1 + 3) = 64
        g.money += 1000;
        g.stage = Stage::Shop();
        let aj = Jokers::AbstractJoker(AbstractJoker::default());
        g.shop.jokers.push(aj.clone());
        g.buy_joker(aj).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        assert_eq!(g.calc_score(best.clone()), 64);

        // Buy Scary Face -> 2 jokers, +6 mult
        // (5 + 11) * (1 + 6) = 112
        g.money += 1000;
        g.stage = Stage::Shop();
        let sf = Jokers::ScaryFace(ScaryFace::default());
        g.shop.jokers.push(sf.clone());
        g.buy_joker(sf).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        assert_eq!(g.calc_score(best.clone()), 112);
    }

    #[test]
    fn test_pareidolia_scary_face() {
        // Ace low straight: no natural face cards
        let ace = Card::new(Value::Ace, Suit::Club);
        let two = Card::new(Value::Two, Suit::Heart);
        let three = Card::new(Value::Three, Suit::Spade);
        let four = Card::new(Value::Four, Suit::Diamond);
        let five = Card::new(Value::Five, Suit::Club);
        let hand = SelectHand::new(vec![ace, two, three, four, five]);
        let best = hand.best_hand().unwrap();

        // Straight (level 1): 30 chips, 4 mult
        // Card chips: 11 + 2 + 3 + 4 + 5 = 25
        // (30 + 25) * 4 = 220
        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        assert_eq!(g.calc_score(best.clone()), 220);

        // Add Scary Face: still no face cards, so still 220
        g.money += 1000;
        g.stage = Stage::Shop();
        let sf = Jokers::ScaryFace(ScaryFace::default());
        g.shop.jokers.push(sf.clone());
        g.buy_joker(sf).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        assert_eq!(g.calc_score(best.clone()), 220);

        // Add Pareidolia: now all cards are face cards
        // Scary Face gives +30 chips × 5 = +150
        // (30 + 25 + 150) * 4 = 820
        g.money += 1000;
        g.stage = Stage::Shop();
        let p = Jokers::Pareidolia(Pareidolia::default());
        g.shop.jokers.push(p.clone());
        g.buy_joker(p).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        assert_eq!(g.calc_score(best.clone()), 820);
    }

    #[test]
    fn test_even_steven() {
        let two = Card::new(Value::Two, Suit::Club);
        let four = Card::new(Value::Four, Suit::Club);
        let six = Card::new(Value::Six, Suit::Club);
        let eight = Card::new(Value::Eight, Suit::Club);
        let ten = Card::new(Value::Ten, Suit::Club);
        let hand = SelectHand::new(vec![two, four, six, eight, ten]);
        let j = Jokers::EvenSteven(EvenSteven::default());

        // Flush (level 1): 35 chips, 4 mult
        // Played (5 cards): 2 + 4 + 6 + 8 + 10 = 30 chips
        // (35 + 30) * 4 = 260
        let before = 260;

        // Even Steven: 5 even cards * +4 mult = +20 mult
        // (35 + 30) * (4 + 20) = 65 * 24 = 1560
        let after = 1560;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_even_steven_odd_cards() {
        let three = Card::new(Value::Three, Suit::Club);
        let five = Card::new(Value::Five, Suit::Club);
        let seven = Card::new(Value::Seven, Suit::Club);
        let nine = Card::new(Value::Nine, Suit::Club);
        let ace = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![three, five, seven, nine, ace]);
        let j = Jokers::EvenSteven(EvenSteven::default());

        // Flush (level 1): 35 chips, 4 mult
        // Played (5 cards): 3 + 5 + 7 + 9 + 11 = 35 chips
        // (35 + 35) * 4 = 280
        let before = 280;

        // Even Steven: 0 even cards -> no bonus
        let after = 280;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_odd_todd() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let three = Card::new(Value::Three, Suit::Heart);
        let five = Card::new(Value::Five, Suit::Heart);
        let seven = Card::new(Value::Seven, Suit::Heart);
        let nine = Card::new(Value::Nine, Suit::Heart);
        let hand = SelectHand::new(vec![ace, three, five, seven, nine]);
        let j = Jokers::OddTodd(OddTodd::default());

        // Flush (level 1): 35 chips, 4 mult
        // Played (5 cards): 11 + 3 + 5 + 7 + 9 = 35 chips
        // (35 + 35) * 4 = 280
        let before = 280;

        // Odd Todd: 5 odd cards * +31 chips = +155 chips
        // (35 + 35 + 155) * 4 = 225 * 4 = 900
        let after = 900;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_odd_todd_even_cards() {
        let two = Card::new(Value::Two, Suit::Club);
        let four = Card::new(Value::Four, Suit::Club);
        let six = Card::new(Value::Six, Suit::Club);
        let eight = Card::new(Value::Eight, Suit::Club);
        let ten = Card::new(Value::Ten, Suit::Club);
        let hand = SelectHand::new(vec![two, four, six, eight, ten]);
        let j = Jokers::OddTodd(OddTodd::default());

        // Flush (level 1): 35 chips, 4 mult
        // Played (5 cards): 2 + 4 + 6 + 8 + 10 = 30 chips
        // (35 + 30) * 4 = 260
        let before = 260;

        // Odd Todd: 0 odd cards -> no bonus
        let after = 260;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_scholar() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace, ace]);
        let j = Jokers::Scholar(Scholar::default());

        // Pair (level 1): 10 chips, 2 mult
        // Played (2 aces): 2 * 11 = 22 chips
        // (10 + 22) * 2 = 64
        let before = 64;

        // Scholar: 2 aces * (+20 chips, +4 mult) = +40 chips, +8 mult
        // (10 + 22 + 40) * (2 + 8) = 72 * 10 = 720
        let after = 720;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_scholar_no_aces() {
        let king = Card::new(Value::King, Suit::Club);
        let hand = SelectHand::new(vec![king]);
        let j = Jokers::Scholar(Scholar::default());

        // High card (level 1): 5 chips, 1 mult
        // Played (1 king): 10 chips
        // (5 + 10) * 1 = 15
        let before = 15;

        // Scholar: 0 aces -> no bonus
        let after = 15;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_business_card_no_face_cards() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace, ace]);
        let j = Jokers::BusinessCard(BusinessCard::default());

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        let score = g.calc_score(hand.best_hand().unwrap());
        assert_eq!(score, 64);

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        let money_before = g.money;
        g.calc_score(hand.best_hand().unwrap());
        assert_eq!(g.money, money_before);
    }

    #[test]
    fn test_business_card_face_cards() {
        let king = Card::new(Value::King, Suit::Heart);
        let queen = Card::new(Value::Queen, Suit::Heart);
        let jack = Card::new(Value::Jack, Suit::Heart);
        let hand = SelectHand::new(vec![king, queen, jack]);
        let j = Jokers::BusinessCard(BusinessCard::default());

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        g.calc_score(hand.best_hand().unwrap());

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        let mut saw_increase = false;
        for _ in 0..100 {
            g.money = 1000;
            g.calc_score(hand.best_hand().unwrap());
            if g.money > 1000 {
                saw_increase = true;
                break;
            }
        }
        assert!(saw_increase, "Business Card should sometimes give money");
    }

    #[test]
    fn test_faceless_joker() {
        let j = Jokers::FacelessJoker(FacelessJoker::default());

        let mut g = Game::default();
        g.start();
        g.stage = Stage::Blind(Blind::Small);
        g.blind = Some(Blind::Small);
        g.deal();

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        let cards: Vec<Card> = g.available.cards().iter().take(5).copied().collect();
        for card in &cards {
            g.available.select_card(*card).expect("can select");
        }
        assert_eq!(g.available.selected().len(), 5);

        g.discard_selected().expect("can discard");
        assert_eq!(g.money, 1001);
    }

    #[test]
    fn test_faceless_joker_few_cards() {
        let j = Jokers::FacelessJoker(FacelessJoker::default());

        let mut g = Game::default();
        g.start();
        g.stage = Stage::Blind(Blind::Small);
        g.blind = Some(Blind::Small);
        g.deal();

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        let cards: Vec<Card> = g.available.cards().iter().take(3).copied().collect();
        for card in &cards {
            g.available.select_card(*card).expect("can select");
        }
        assert_eq!(g.available.selected().len(), 3);

        g.discard_selected().expect("can discard");
        assert_eq!(g.money, 996);
    }

    #[test]
    fn test_baron() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace, ace]);
        let j = Jokers::Baron(Baron::default());

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        g.available.extend(vec![
            Card::new(Value::King, Suit::Club),
            Card::new(Value::King, Suit::Spade),
        ]);
        let best = hand.best_hand().unwrap();

        // Pair (level 1): 10 chips, 2 mult
        // Played (2 aces): 22 chips
        // (10 + 22) * 2 = 64
        assert_eq!(g.calc_score(best.clone()), 64);

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // Baron: 2 kings -> 2 * 1.5 = 3, then 3 * 1.5 = 4 (truncated)
        // (10 + 22) * (2 * 1.5 * 1.5) = 32 * 4 = 128
        assert_eq!(g.calc_score(best.clone()), 128);
    }

    #[test]
    fn test_baron_no_kings() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace, ace]);
        let j = Jokers::Baron(Baron::default());

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        g.available.extend(vec![
            Card::new(Value::Queen, Suit::Club),
            Card::new(Value::Jack, Suit::Spade),
        ]);
        let best = hand.best_hand().unwrap();

        // Pair (level 1): 10 chips, 2 mult
        // Played (2 aces): 22 chips
        // (10 + 22) * 2 = 64
        assert_eq!(g.calc_score(best.clone()), 64);

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // Baron: 0 kings -> no bonus
        assert_eq!(g.calc_score(best.clone()), 64);
    }

    #[test]
    fn test_midas_mask() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace, ace]);
        let j = Jokers::MidasMask(MidasMask::default());

        // Pair (level 1): 10 chips, 2 mult
        // Played (2 aces): 22 chips
        // (10 + 22) * 2 = 64
        let before = 64;

        // Midas Mask converts to Gold but Gold has no game logic yet -> same score
        let after = 64;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_photograph() {
        let king = Card::new(Value::King, Suit::Heart);
        let hand = SelectHand::new(vec![king]);
        let j = Jokers::Photograph(Photograph::default());

        // High card (level 1): 5 chips, 1 mult
        // Played (1 king): 10 chips
        // (5 + 10) * 1 = 15
        let before = 15;

        // Photograph: 1 face card -> X2 mult
        // (5 + 10) * (1 * 2) = 30
        let after = 30;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_photograph_no_face() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::Photograph(Photograph::default());

        // High card (level 1): 5 chips, 1 mult
        // Played (1 ace): 11 chips
        // (5 + 11) * 1 = 16
        let before = 16;

        // Photograph: 0 face cards -> no bonus
        let after = 16;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_reserved_parking() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::ReservedParking(ReservedParking::default());

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        g.available.extend(vec![
            Card::new(Value::King, Suit::Club),
            Card::new(Value::Queen, Suit::Spade),
            Card::new(Value::Jack, Suit::Heart),
        ]);
        let best = hand.best_hand().unwrap();
        g.calc_score(best.clone());

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        let mut saw_increase = false;
        for _ in 0..100 {
            g.money = 994;
            g.calc_score(best.clone());
            if g.money > 994 {
                saw_increase = true;
                break;
            }
        }
        assert!(saw_increase, "Reserved Parking should sometimes give money");
    }

    #[test]
    fn test_reserved_parking_no_face() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::ReservedParking(ReservedParking::default());

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        g.available.extend(vec![
            Card::new(Value::Ace, Suit::Club),
            Card::new(Value::Two, Suit::Spade),
        ]);
        let best = hand.best_hand().unwrap();
        g.calc_score(best.clone());

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        let money_before = g.money;
        g.calc_score(best.clone());
        assert_eq!(g.money, money_before);
    }

    #[test]
    fn test_baseball_card() {
        let ten = Card::new(Value::Ten, Suit::Heart);
        let hand = SelectHand::new(vec![ten, ten]);
        let best = hand.best_hand().unwrap();

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };

        // Pair (level 1): 10 chips, 2 mult
        // Played (2 tens): 20 chips
        // (10 + 20) * 2 = 60
        assert_eq!(g.calc_score(best.clone()), 60);

        // Buy 2 uncommon jokers (MidasMask, Pareidolia) and BaseballCard
        let midas = Jokers::MidasMask(MidasMask::default());
        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(midas.clone());
        g.buy_joker(midas).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        g.calc_score(best.clone());

        let pareidolia = Jokers::Pareidolia(Pareidolia::default());
        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(pareidolia.clone());
        g.buy_joker(pareidolia).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        g.calc_score(best.clone());

        let bb = Jokers::BaseballCard(BaseballCard::default());
        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(bb.clone());
        g.buy_joker(bb).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // BaseballCard: 2 uncommon * X1.5
        // (10 + 20) * (2 * 1.5 * 1.5) = 30 * 4 = 120
        assert_eq!(g.calc_score(best.clone()), 120);
    }

    #[test]
    fn test_baseball_card_no_uncommon() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let best = hand.best_hand().unwrap();

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };

        // Buy BaseballCard with no uncommon jokers
        let bb = Jokers::BaseballCard(BaseballCard::default());
        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(bb.clone());
        g.buy_joker(bb).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // High card (level 1): 5 chips, 1 mult
        // Played (1 ace): 11 chips
        // (5 + 11) * 1 = 16
        assert_eq!(g.calc_score(best.clone()), 16);
    }

    #[test]
    fn test_bull() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::Bull(Bull::default());

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            money: 100,
            ..Default::default()
        };
        let best = hand.best_hand().unwrap();

        // High card (level 1): 5 chips, 1 mult
        // Played (1 ace): 11 chips
        // (5 + 11) * 1 = 16
        assert_eq!(g.calc_score(best.clone()), 16);

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // Bull: (100 + 1000 - 6) * 2 = 1094 * 2 = 2188 chips
        // (5 + 11 + 2188) * 1 = 2204
        assert_eq!(g.calc_score(best.clone()), 2204);
    }

    #[test]
    fn test_bull_no_money() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::Bull(Bull::default());

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        let best = hand.best_hand().unwrap();

        // High card (level 1): 5 chips, 1 mult
        // Played (1 ace): 11 chips
        // (5 + 11) * 1 = 16
        assert_eq!(g.calc_score(best.clone()), 16);

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        g.money = 0;

        // Bull: 0 * 2 = 0 chips
        // (5 + 11) * 1 = 16
        assert_eq!(g.calc_score(best.clone()), 16);
    }

    #[test]
    fn test_walkie_talkie() {
        let ten = Card::new(Value::Ten, Suit::Heart);
        let hand = SelectHand::new(vec![ten, ten]);
        let j = Jokers::WalkieTalkie(WalkieTalkie::default());

        // Pair (level 1): 10 chips, 2 mult
        // Played (2 tens): 20 chips
        // (10 + 20) * 2 = 60
        let before = 60;

        // WalkieTalkie: 2 tens * (+10 chips, +4 mult) = +20 chips, +8 mult
        // (10 + 20 + 20) * (2 + 8) = 50 * 10 = 500
        let after = 500;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_walkie_talkie_other_cards() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::WalkieTalkie(WalkieTalkie::default());

        // High card (level 1): 5 chips, 1 mult
        // Played (1 ace): 11 chips
        // (5 + 11) * 1 = 16
        let before = 16;

        // WalkieTalkie: 0 tens or fours -> no bonus
        let after = 16;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_smiley_face() {
        let king = Card::new(Value::King, Suit::Heart);
        let king2 = Card::new(Value::King, Suit::Diamond);
        let hand = SelectHand::new(vec![king, king2]);
        let j = Jokers::SmileyFace(SmileyFace::default());

        // Pair (level 1): 10 chips, 2 mult
        // Played (2 kings): 10 + 10 = 20 chips
        // (10 + 20) * 2 = 60
        let before = 60;

        // Smiley Face: 2 face cards * +5 mult = +10 mult
        // (10 + 20) * (2 + 10) = 30 * 12 = 360
        let after = 360;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_smiley_face_no_face() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::SmileyFace(SmileyFace::default());

        // High card (level 1): 5 chips, 1 mult
        // Played (1 ace): 11 chips
        // (5 + 11) * 1 = 16
        let before = 16;

        // Smiley Face: 0 face cards -> no bonus
        let after = 16;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_golden_ticket_no_gold() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace, ace]);
        let j = Jokers::GoldenTicket(GoldenTicket::default());

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        let score = g.calc_score(hand.best_hand().unwrap());
        assert_eq!(score, 64);

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        let money_before = g.money;
        g.calc_score(hand.best_hand().unwrap());
        assert_eq!(g.money, money_before);
    }

    #[test]
    fn test_golden_ticket_with_gold() {
        let mut ace = Card::new(Value::Ace, Suit::Heart);
        ace.enhancement = Some(Enhancement::Gold);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::GoldenTicket(GoldenTicket::default());

        let mut g = Game::default();
        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        let money_before = g.money;
        g.calc_score(hand.best_hand().unwrap());
        assert_eq!(g.money, money_before + 4);
    }

    #[test]
    fn test_acrobat_final_hand() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::Acrobat(Acrobat::default());

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        g.calc_score(hand.best_hand().unwrap());

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // Final hand: plays == 0
        g.plays = 0;
        // High card ace: (5 + 11) * 1 = 16
        // Acrobat X3: 16 * 3 = 48
        assert_eq!(g.calc_score(hand.best_hand().unwrap()), 48);
    }

    #[test]
    fn test_acrobat_not_final_hand() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::Acrobat(Acrobat::default());

        let mut g = Game::default();
        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // Not final hand: plays > 0
        g.plays = 1;
        // High card ace: (5 + 11) * 1 = 16
        // Acrobat: no bonus
        assert_eq!(g.calc_score(hand.best_hand().unwrap()), 16);
    }

    #[test]
    fn test_rough_gem_no_diamonds() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace, ace]);
        let j = Jokers::RoughGem(RoughGem::default());

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        g.calc_score(hand.best_hand().unwrap());

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        let money_before = g.money;
        g.calc_score(hand.best_hand().unwrap());
        assert_eq!(g.money, money_before);
    }

    #[test]
    fn test_rough_gem_with_diamonds() {
        let dia1 = Card::new(Value::Ace, Suit::Diamond);
        let dia2 = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![dia1, dia2]);
        let j = Jokers::RoughGem(RoughGem::default());

        let mut g = Game::default();
        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        let money_before = g.money;
        g.calc_score(hand.best_hand().unwrap());
        assert_eq!(g.money, money_before + 1);
    }

    #[test]
    fn test_bloodstone_no_hearts() {
        let ace = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::Bloodstone(Bloodstone::default());

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        g.calc_score(hand.best_hand().unwrap());

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // High card club: (5 + 11) * 1 = 16, no hearts -> no Xmult
        assert_eq!(g.calc_score(hand.best_hand().unwrap()), 16);
    }

    #[test]
    fn test_bloodstone_with_hearts() {
        let heart = Card::new(Value::Ace, Suit::Heart);
        let heart2 = Card::new(Value::Ace, Suit::Diamond);
        let hand = SelectHand::new(vec![heart, heart2]);
        let j = Jokers::Bloodstone(Bloodstone::default());

        let mut g = Game::default();
        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // Pair of aces (level 1): 10 chips, 2 mult
        // Played: Ace heart (11 chips) + Ace diamond (11 chips) = 22 chips
        // (10 + 22) * 2 = 64
        // With Bloodstone: 1 heart card, 50% chance X1.5
        // Expected: sometimes 64, sometimes 96
        let mut saw_increase = false;
        let mut saw_no_increase = false;
        for _ in 0..50 {
            let score = g.calc_score(hand.best_hand().unwrap());
            if score == 96 {
                saw_increase = true;
            } else if score == 64 {
                saw_no_increase = true;
            }
        }
        assert!(saw_increase, "Bloodstone should sometimes Xmult");
        assert!(saw_no_increase, "Bloodstone should sometimes not Xmult");
    }

    #[test]
    fn test_arrowhead_with_spades() {
        let ace = Card::new(Value::Ace, Suit::Spade);
        let hand = SelectHand::new(vec![ace]);

        // High card (level 1): 5 chips, 1 mult
        // Played (1 ace spade): 11 chips
        // (5 + 11) * 1 = 16
        let before = 16;
        // Arrowhead: +50 chips for the spade
        // (5 + 11 + 50) * 1 = 66
        let after = 66;

        let j = Jokers::Arrowhead(Arrowhead::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_arrowhead_no_spades() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);

        let before = 16;
        let after = 16;

        let j = Jokers::Arrowhead(Arrowhead::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_onyx_agate_with_clubs() {
        let ace = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![ace]);

        let before = 16;
        // (5 + 11) * (1 + 7) = 128
        let after = 128;

        let j = Jokers::OnyxAgate(OnyxAgate::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_onyx_agate_no_clubs() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);

        let before = 16;
        let after = 16;

        let j = Jokers::OnyxAgate(OnyxAgate::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_shoot_the_moon_with_held_queen() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::ShootTheMoon(ShootTheMoon::default());

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        g.available
            .extend(vec![Card::new(Value::Queen, Suit::Spade)]);

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // High card (level 1): 5 chips, 1 mult
        // Played (1 ace): 11 chips
        // Held Queen: +13 mult
        // (5 + 11) * (1 + 13) = 224
        assert_eq!(g.calc_score(hand.best_hand().unwrap()), 224);
    }

    #[test]
    fn test_shoot_the_moon_no_held_queen() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::ShootTheMoon(ShootTheMoon::default());

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // No held cards -> no bonus; (5 + 11) * 1 = 16
        assert_eq!(g.calc_score(hand.best_hand().unwrap()), 16);
    }

    #[test]
    fn test_raised_fist_uses_lowest_held_card() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::RaisedFist(RaisedFist::default());

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        g.available.extend(vec![
            Card::new(Value::Two, Suit::Club),
            Card::new(Value::King, Suit::Spade),
        ]);

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // High card (level 1): 5 chips, 1 mult
        // Played (1 ace): 11 chips
        // Lowest held card: Two (2 chips) -> +4 mult
        // (5 + 11) * (1 + 4) = 80
        assert_eq!(g.calc_score(hand.best_hand().unwrap()), 80);
    }

    #[test]
    fn test_raised_fist_no_held_cards() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::RaisedFist(RaisedFist::default());

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // No held cards -> no bonus; (5 + 11) * 1 = 16
        assert_eq!(g.calc_score(hand.best_hand().unwrap()), 16);
    }

    #[test]
    fn test_wild_counts_for_suit_jokers() {
        // Two Aces so they form a Pair, one is Wild (Spade but counts as all suits)
        let mut wild = Card::new(Value::Ace, Suit::Spade);
        wild.enhancement = Some(Enhancement::Wild);
        let heart = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![heart, wild]);

        // Pair (level 1): 10 chips, 2 mult
        // Played: Ace (11) + Ace (11) = 22 chips
        // (10 + 22) * 2 = 64
        let before = 64;

        // LustyJoker: heart (1) + wild-as-heart (1) = 2 hearts -> +6 mult
        // (10 + 22) * (2 + 6) = 32 * 8 = 256
        let after = 256;
        let j = Jokers::LustyJoker(LustyJoker::default());
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_wild_counts_for_rough_gem() {
        let mut wild = Card::new(Value::Ace, Suit::Heart);
        wild.enhancement = Some(Enhancement::Wild);
        let hand = SelectHand::new(vec![wild]);

        let j = Jokers::RoughGem(RoughGem::default());
        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        let money_before = g.money;
        g.calc_score(hand.best_hand().unwrap());
        assert_eq!(
            g.money,
            money_before + 1,
            "Wild should count as Diamond for RoughGem"
        );
    }
}
