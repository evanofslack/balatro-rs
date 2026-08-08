use crate::card::{Card, Edition, Enhancement, Suit, Value};
use crate::effect::{Effects, RuleFlag};
use crate::game::Game;
use crate::hand::{MadeHand, SelectHand};
use crate::rank::HandRank;
use rand::Rng;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use strum::IntoEnumIterator;

pub use balatro_types::joker::*;

static JOKER_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

pub(crate) fn mint_joker_id() -> usize {
    JOKER_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

pub(crate) fn ensure_joker_id_counter_past(max_seen: usize) {
    JOKER_ID_COUNTER.fetch_max(max_seen + 1, Ordering::SeqCst);
}

/// Assigns a fresh rotating selector to joker if it's a selector-type joker.
pub(crate) fn roll_discard_selector<R: Rng + ?Sized>(rng: &mut R, j: &mut Jokers) {
    const SUITS: [Suit; 4] = [Suit::Spade, Suit::Club, Suit::Heart, Suit::Diamond];
    const VALUES: [Value; 13] = [
        Value::Two,
        Value::Three,
        Value::Four,
        Value::Five,
        Value::Six,
        Value::Seven,
        Value::Eight,
        Value::Nine,
        Value::Ten,
        Value::Jack,
        Value::Queen,
        Value::King,
        Value::Ace,
    ];
    match j {
        Jokers::Castle(_) => {
            let suit = SUITS[rng.gen_range(0..SUITS.len())];
            j.state_mut().selector = Some(SelectorValue::Suit(suit));
        }
        Jokers::MailInRebate(_) => {
            let value = VALUES[rng.gen_range(0..VALUES.len())];
            j.state_mut().selector = Some(SelectorValue::Value(value));
        }
        _ => {}
    }
}

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
                    let diamonds = g
                        .non_debuffed(hand.hand.cards().iter())
                        .iter()
                        .filter(|c| c.matches_suit(Suit::Diamond))
                        .count();
                    g.mult += diamonds * 3
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::LustyJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    let hearts = g
                        .non_debuffed(hand.hand.cards().iter())
                        .iter()
                        .filter(|c| c.matches_suit(Suit::Heart))
                        .count();
                    g.mult += hearts * 3
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::WrathfulJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    let spades = g
                        .non_debuffed(hand.hand.cards().iter())
                        .iter()
                        .filter(|c| c.matches_suit(Suit::Spade))
                        .count();
                    g.mult += spades * 3
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::GluttonousJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    let clubs = g
                        .non_debuffed(hand.hand.cards().iter())
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
                    g.chips += 30 * g.discards();
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::MysticSummit(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    if g.discards() == 0 {
                        g.mult += 15;
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::Supernova(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    g.mult += g.planetarium.level(hand.rank).plays;
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::RideTheBus(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    let has_face = g
                        .non_debuffed(hand.hand.cards().iter())
                        .iter()
                        .any(|c| g.is_face_card(c));
                    if has_face {
                        g.consecutive_hands_without_face_card = 0;
                    } else {
                        g.consecutive_hands_without_face_card += 1;
                        g.mult += g.consecutive_hands_without_face_card;
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::CardSharp(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if g.hand_ranks_played_this_round.contains(&hand.rank) {
                        g.mult *= 3;
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::Obelisk(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    // RoyalFlush shares Planetarium storage with
                    // StraightFlush, normalize this
                    let played_rank = if hand.rank == HandRank::RoyalFlush {
                        HandRank::StraightFlush
                    } else {
                        hand.rank
                    };
                    if played_rank == g.most_played_hand_rank() {
                        g.consecutive_hands_not_most_played_type = 0;
                    } else {
                        g.consecutive_hands_not_most_played_type += 1;
                        g.mult +=
                            (g.mult as f64 * 0.2 * g.consecutive_hands_not_most_played_type as f64)
                                as usize;
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::WeeJoker(wee) => {
                let id = wee.instance_id;
                let apply = move |g: &mut Game, hand: MadeHand| {
                    let twos = hand
                        .hand
                        .cards()
                        .iter()
                        .filter(|c| c.value == Value::Two)
                        .count();
                    let counter = match g.joker_state_mut(id) {
                        Some(state) => {
                            state.counter += 8.0 * twos as f32;
                            state.counter
                        }
                        None => return,
                    };
                    g.chips += counter as usize;
                };
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::SpareTrousers(st) => {
                let id = st.instance_id;
                let apply = move |g: &mut Game, hand: MadeHand| {
                    let has_two_pair = hand.hand.is_two_pair().is_some();
                    let counter = match g.joker_state_mut(id) {
                        Some(state) => {
                            if has_two_pair {
                                state.counter += 2.0;
                            }
                            state.counter
                        }
                        None => return,
                    };
                    g.mult += counter as usize;
                };
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
                    for card in g.non_debuffed(_hand.hand.cards().iter()) {
                        if g.is_face_card(&card) {
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
            Self::Pareidolia(_) => vec![Effects::RuleFlag(RuleFlag::AllCardsAreFace)],
            Self::EvenSteven(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    for card in _hand.hand.cards() {
                        if g.is_even(&card) {
                            g.mult += 4;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::OddTodd(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    for card in _hand.hand.cards() {
                        if g.is_odd(&card) {
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
                    for card in g.non_debuffed(_hand.hand.cards().iter()) {
                        if g.is_face_card(&card) && g.prob_roll(1, 2) {
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
                        .non_debuffed(g.available.not_selected().iter())
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
                fn apply(g: &mut Game, hand: &mut MadeHand) {
                    // only cards that are actually scored (hand.hand), not
                    // unscored kickers (hand.all), become Gold
                    let cards: Vec<Card> = hand
                        .hand
                        .cards()
                        .into_iter()
                        .map(|mut c| {
                            if g.is_face_card(&c) && !g.is_card_debuffed(&c) {
                                c.enhancement = Some(Enhancement::Gold);
                                g.mutate_card(c.id, |c| c.enhancement = Some(Enhancement::Gold));
                            }
                            c
                        })
                        .collect();
                    hand.hand = SelectHand::new(cards);
                }
                vec![Effects::OnModifyHand(Arc::new(Mutex::new(apply)))]
            }
            Self::Photograph(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    for card in g.non_debuffed(_hand.hand.cards().iter()) {
                        if g.is_face_card(&card) {
                            g.mult *= 2;
                            break;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::ReservedParking(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    for card in g.non_debuffed(g.available.not_selected().iter()) {
                        if g.is_face_card(&card) && g.prob_roll(1, 2) {
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
                    for card in g.non_debuffed(_hand.hand.cards().iter()) {
                        if g.is_face_card(&card) {
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
                    for card in g.non_debuffed(_hand.hand.cards().iter()) {
                        if card.matches_suit(Suit::Diamond) {
                            g.money += 1;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::Bloodstone(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    for card in g.non_debuffed(_hand.hand.cards().iter()) {
                        if card.matches_suit(Suit::Heart) && g.prob_roll(1, 2) {
                            g.mult += g.mult / 2;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::Arrowhead(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    for card in g.non_debuffed(hand.hand.cards().iter()) {
                        if card.matches_suit(Suit::Spade) {
                            g.chips += 50;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::OnyxAgate(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    for card in g.non_debuffed(hand.hand.cards().iter()) {
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
            Self::Dusk(_) => {
                fn extra(g: &mut Game, _card: Card, _is_first: bool) -> usize {
                    if g.plays == 0 {
                        1
                    } else {
                        0
                    }
                }
                vec![Effects::TriggerCountPlayed(Arc::new(Mutex::new(extra)))]
            }
            Self::Hack(_) => {
                fn extra(_g: &mut Game, card: Card, _is_first: bool) -> usize {
                    if matches!(
                        card.value,
                        Value::Two | Value::Three | Value::Four | Value::Five
                    ) {
                        1
                    } else {
                        0
                    }
                }
                vec![Effects::TriggerCountPlayed(Arc::new(Mutex::new(extra)))]
            }
            Self::SockAndBuskin(_) => {
                fn extra(g: &mut Game, card: Card, _is_first: bool) -> usize {
                    if g.is_face_card(&card) {
                        1
                    } else {
                        0
                    }
                }
                vec![Effects::TriggerCountPlayed(Arc::new(Mutex::new(extra)))]
            }
            Self::HangingChad(_) => {
                fn extra(_g: &mut Game, _card: Card, is_first: bool) -> usize {
                    if is_first {
                        2
                    } else {
                        0
                    }
                }
                vec![Effects::TriggerCountPlayed(Arc::new(Mutex::new(extra)))]
            }
            Self::Mime(_) => {
                fn extra(_g: &mut Game, _card: Card, _is_first: bool) -> usize {
                    1
                }
                vec![Effects::TriggerCountHeld(Arc::new(Mutex::new(extra)))]
            }
            Self::TradingCard(_) => {
                // "First discard of round" == self.discards already
                // decremented to config.discards - 1 by the time this fires
                // (discard_selected decrements before invoking OnDiscard).
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.all.len() == 1
                        && g.discards_remaining == g.config.discards.saturating_sub(1)
                    {
                        g.destroy_card(hand.all[0].id);
                        g.money += 3;
                    }
                }
                vec![Effects::OnDiscard(Arc::new(Mutex::new(apply)))]
            }
            Self::GreenJoker(_) => {
                // config.plays/config.discards - remaining == used this
                // round, both already reset to config values in clear_blind.
                fn apply(g: &mut Game, _hand: MadeHand) {
                    let hands_played = g.config.plays - g.plays;
                    let discards_used = g.config.discards - g.discards_remaining;
                    let delta = hands_played as i64 - discards_used as i64;
                    g.mult = (g.mult as i64 + delta).max(0) as usize;
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::HitTheRoad(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    let jacks = g
                        .non_debuffed(g.discarded_this_round.iter())
                        .iter()
                        .filter(|c| c.value == Value::Jack)
                        .count();
                    g.mult += (g.mult as f64 * 0.5 * jacks as f64) as usize;
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::Yorick(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    let levels = g.total_cards_discarded / 23;
                    g.mult += (g.mult as f64 * levels as f64) as usize;
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::Castle(castle) => {
                // Selector is picked in Game::clear_blind (round-scoped, "changes
                // every round"), *after* effects() is typically last rebuilt
                // (buy/sell/pack-add), must be looked up live.
                let id = castle.instance_id;
                let apply = move |g: &mut Game, _hand: MadeHand| {
                    let Some(SelectorValue::Suit(suit)) =
                        g.joker_state_mut(id).and_then(|s| s.selector)
                    else {
                        return;
                    };
                    let count = g
                        .non_debuffed(g.discarded_this_round.iter())
                        .iter()
                        .filter(|c| c.matches_suit(suit))
                        .count();
                    g.chips += count * 3;
                };
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::MailInRebate(mail) => {
                // Immediate payout on the triggering discard itself, not an
                // accumulated round total - unlike Castle/HitTheRoad, matches
                // real card text ("Earn $5 for each discarded [rank]"). Selector
                // looked up live for the same reason as Castle above.
                let id = mail.instance_id;
                let apply = move |g: &mut Game, hand: MadeHand| {
                    let Some(SelectorValue::Value(value)) =
                        g.joker_state_mut(id).and_then(|s| s.selector)
                    else {
                        return;
                    };
                    let count = hand.all.iter().filter(|c| c.value == value).count();
                    g.money += count * 5;
                };
                vec![Effects::OnDiscard(Arc::new(Mutex::new(apply)))]
            }
            Self::Luchador(_) => {
                fn apply(g: &mut Game) {
                    g.boss_disabled_by_luchador = true;
                }
                vec![Effects::OnSell(Arc::new(Mutex::new(apply)))]
            }
            Self::Matador(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    if g.boss_triggered_this_hand {
                        g.money += 8;
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
                | Self::Supernova(_)
                | Self::RideTheBus(_)
                | Self::CardSharp(_)
                | Self::Obelisk(_)
                | Self::WeeJoker(_)
                | Self::SpareTrousers(_)
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
                | Self::Mime(_)
                | Self::Dusk(_)
                | Self::Hack(_)
                | Self::SockAndBuskin(_)
                | Self::HangingChad(_)
                | Self::TradingCard(_)
                | Self::GreenJoker(_)
                | Self::HitTheRoad(_)
                | Self::Yorick(_)
                | Self::Castle(_)
                | Self::MailInRebate(_)
                | Self::Luchador(_)
                | Self::Chicot(_)
                | Self::Matador(_)
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
    let edition = j.edition();
    let edition_str = if edition == Edition::Base {
        String::new()
    } else {
        format!(" [{:?}]", edition)
    };
    let stickers = j.stickers();
    let mut sticker_flags = Vec::new();
    if stickers.eternal {
        sticker_flags.push("Eternal");
    }
    if stickers.perishable {
        sticker_flags.push("Perishable");
    }
    if stickers.rental {
        sticker_flags.push("Rental");
    }
    let sticker_str = if sticker_flags.is_empty() {
        String::new()
    } else {
        format!(" [{}]", sticker_flags.join(","))
    };
    format!(
        "{} [{}]{}{} {}",
        j.name(),
        j.rarity(),
        edition_str,
        sticker_str,
        j.desc()
    )
}

#[cfg(test)]
mod tests {
    use crate::card::{Card, Enhancement, Suit, Value};
    use crate::hand::SelectHand;
    use crate::stage::{Blind, Stage};
    use balatro_types::BossBlind;

    use super::*;

    // balatro_types::Jokers now defines all jokers but only a subset have
    // `effects()` behavior implemented. Shop/pack generation
    // must never offer joker that silently does nothing.
    #[test]
    fn test_exactly_74_jokers_implemented() {
        let count = Jokers::iter().filter(|j| j.is_implemented()).count();
        assert_eq!(count, 74);
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
        let drawn = g.deck.draw(10);
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
        let drawn = g.deck.draw(10);
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

        g.discards_remaining = 0;
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
        g.discards_remaining = 0;
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
    fn test_even_steven_pareidolia_disqualifies_even_cards() {
        // High Card Two, alone: (5 + 2) * 1 = 7
        // + EvenSteven: Two is even -> +4 mult: (5 + 2) * (1 + 4) = 35
        // + Pareidolia: Two now counts as a face card, so no longer "even": back to 7
        let two = Card::new(Value::Two, Suit::Heart);
        let hand = SelectHand::new(vec![two]);

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        let best = hand.best_hand().unwrap();
        assert_eq!(g.calc_score(best.clone()), 7);

        g.money += 1000;
        g.stage = Stage::Shop();
        let es = Jokers::EvenSteven(EvenSteven::default());
        g.shop.jokers.push(es.clone());
        g.buy_joker(es).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        assert_eq!(g.calc_score(best.clone()), 35);

        g.money += 1000;
        g.stage = Stage::Shop();
        let p = Jokers::Pareidolia(Pareidolia::default());
        g.shop.jokers.push(p.clone());
        g.buy_joker(p).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        assert_eq!(g.calc_score(best.clone()), 7);
    }

    #[test]
    fn test_odd_todd_pareidolia_disqualifies_odd_cards() {
        // High Card Three, alone: (5 + 3) * 1 = 8
        // + OddTodd: Three is odd -> +31 chips: (5 + 3 + 31) * 1 = 39
        // + Pareidolia: Three now counts as a face card, so no longer "odd": back to 8
        let three = Card::new(Value::Three, Suit::Heart);
        let hand = SelectHand::new(vec![three]);

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        let best = hand.best_hand().unwrap();
        assert_eq!(g.calc_score(best.clone()), 8);

        g.money += 1000;
        g.stage = Stage::Shop();
        let ot = Jokers::OddTodd(OddTodd::default());
        g.shop.jokers.push(ot.clone());
        g.buy_joker(ot).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        assert_eq!(g.calc_score(best.clone()), 39);

        g.money += 1000;
        g.stage = Stage::Shop();
        let p = Jokers::Pareidolia(Pareidolia::default());
        g.shop.jokers.push(p.clone());
        g.buy_joker(p).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        assert_eq!(g.calc_score(best.clone()), 8);
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

        // Aces aren't face cards, so Midas Mask doesn't touch them -> same score
        let after = 64;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_midas_mask_converts_played_face_card_to_gold() {
        let king = Card::new(Value::King, Suit::Heart);

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        g.available.extend(vec![king]);

        g.money += 1000;
        g.stage = Stage::Shop();
        let j = Jokers::MidasMask(MidasMask::default());
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        let hand = SelectHand::new(vec![king]);
        g.calc_score(hand.best_hand().unwrap());

        // the enhancement must persist onto the real card, not just the
        // transient MadeHand used for this scoring pass
        let scored = g.available.cards().into_iter().find(|c| c.id == king.id);
        assert_eq!(scored.unwrap().enhancement, Some(Enhancement::Gold));
    }

    #[test]
    fn test_midas_mask_pareidolia_converts_non_face_card_to_gold() {
        let two = Card::new(Value::Two, Suit::Heart);

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        g.available.extend(vec![two]);

        g.money += 1000;
        g.stage = Stage::Shop();
        let mm = Jokers::MidasMask(MidasMask::default());
        g.shop.jokers.push(mm.clone());
        g.buy_joker(mm).unwrap();
        let p = Jokers::Pareidolia(Pareidolia::default());
        g.shop.jokers.push(p.clone());
        g.buy_joker(p).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        let hand = SelectHand::new(vec![two]);
        g.calc_score(hand.best_hand().unwrap());

        let scored = g.available.cards().into_iter().find(|c| c.id == two.id);
        assert_eq!(scored.unwrap().enhancement, Some(Enhancement::Gold));
    }

    #[test]
    fn test_midas_mask_does_not_convert_unscored_kicker() {
        let king1 = Card::new(Value::King, Suit::Heart);
        let king2 = Card::new(Value::King, Suit::Spade);
        let queen = Card::new(Value::Queen, Suit::Diamond);

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        g.available.extend(vec![king1, king2, queen]);

        g.money += 1000;
        g.stage = Stage::Shop();
        let j = Jokers::MidasMask(MidasMask::default());
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // Pair of Kings + Queen kicker: only the pair is scored, the queen
        // kicker never scores and so should stay unenhanced
        let hand = SelectHand::new(vec![king1, king2, queen]);
        g.calc_score(hand.best_hand().unwrap());

        let cards = g.available.cards();
        let scored_king = cards.iter().find(|c| c.id == king1.id).unwrap();
        let kicker_queen = cards.iter().find(|c| c.id == queen.id).unwrap();
        assert_eq!(scored_king.enhancement, Some(Enhancement::Gold));
        assert_eq!(kicker_queen.enhancement, None);
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
    fn test_reserved_parking_pareidolia_counts_non_face_cards() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let rp = Jokers::ReservedParking(ReservedParking::default());
        let p = Jokers::Pareidolia(Pareidolia::default());

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
        g.shop.jokers.push(rp.clone());
        g.buy_joker(rp).unwrap();
        g.shop.jokers.push(p.clone());
        g.buy_joker(p).unwrap();
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
        assert!(
            saw_increase,
            "Pareidolia should make non-face held cards count for Reserved Parking"
        );
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

    #[test]
    fn test_hanging_chad_buy_flow() {
        // High Card Ace, alone -> first (only) card retriggers 2 extra times.
        // before: (5 + 11) * 1 = 16; after: (5 + 11*3) * 1 = 38
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::HangingChad(HangingChad::default());
        score_before_after_joker(j, hand, 16, 38);
    }

    #[test]
    fn test_hanging_chad_skips_unscored_leading_kicker() {
        // Pair of Kings with an unmatched Queen kicker played first. The Queen
        // never scores, so "first played card used in scoring" must resolve
        // to the first King, not the Queen.
        // Pair (level 1): 10 chips, 2 mult. Kings score 10 chips each.
        // before: (10 + 10 + 10) * 2 = 60
        // after: king1 retriggers (1 base + 2 from HangingChad = 3 triggers),
        // king2 doesn't: (10 + 10*3 + 10*1) * 2 = 100
        let queen = Card::new(Value::Queen, Suit::Diamond);
        let king1 = Card::new(Value::King, Suit::Heart);
        let king2 = Card::new(Value::King, Suit::Spade);
        let hand = SelectHand::new(vec![queen, king1, king2]);
        let j = Jokers::HangingChad(HangingChad::default());
        score_before_after_joker(j, hand, 60, 100);
    }

    #[test]
    fn test_hack_buy_flow() {
        // High Card Three, alone -> retriggers since 3 is in 2-5.
        // before: (5 + 3) * 1 = 8; after: (5 + 3 + 3) * 1 = 11
        let three = Card::new(Value::Three, Suit::Heart);
        let hand = SelectHand::new(vec![three]);
        let j = Jokers::Hack(Hack::default());
        score_before_after_joker(j, hand, 8, 11);
    }

    #[test]
    fn test_sock_and_buskin_buy_flow() {
        // High Card Jack, alone -> retriggers since Jack is a face card.
        // before: (5 + 10) * 1 = 15; after: (5 + 10 + 10) * 1 = 25
        let jack = Card::new(Value::Jack, Suit::Heart);
        let hand = SelectHand::new(vec![jack]);
        let j = Jokers::SockAndBuskin(SockAndBuskin::default());
        score_before_after_joker(j, hand, 15, 25);
    }

    #[test]
    fn test_sock_and_buskin_pareidolia_retriggers_non_face_card() {
        // High Card Two, alone. Two isn't a face card, so SockAndBuskin alone
        // doesn't retrigger it. Once Pareidolia is also owned, it should.
        // no jokers: (5 + 2) * 1 = 7
        // + SockAndBuskin only: still 7 (not a face card)
        // + Pareidolia: (5 + 2*2) * 1 = 9 (now retriggers once)
        let two = Card::new(Value::Two, Suit::Heart);
        let hand = SelectHand::new(vec![two]);

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        let best = hand.best_hand().unwrap();
        assert_eq!(g.calc_score(best.clone()), 7);

        g.money += 1000;
        g.stage = Stage::Shop();
        let sb = Jokers::SockAndBuskin(SockAndBuskin::default());
        g.shop.jokers.push(sb.clone());
        g.buy_joker(sb).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        assert_eq!(g.calc_score(best.clone()), 7);

        g.money += 1000;
        g.stage = Stage::Shop();
        let p = Jokers::Pareidolia(Pareidolia::default());
        g.shop.jokers.push(p.clone());
        g.buy_joker(p).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        assert_eq!(g.calc_score(best.clone()), 9);
    }

    #[test]
    fn test_multiple_retrigger_jokers_stack() {
        // High Card Three, alone. Three qualifies for Hack (2-5)
        // and (with Pareidolia active) also for SockAndBuskin.
        // HangingChad adds a third trigger source.
        // Each trigger re-scores card's chip value (3), so total
        // trigger count directly multiplies.
        let three = Card::new(Value::Three, Suit::Heart);
        let hand = SelectHand::new(vec![three]);

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        let best = hand.best_hand().unwrap();
        // no jokers: (5 + 3) * 1 = 8
        assert_eq!(g.calc_score(best.clone()), 8);

        g.money += 1000;
        g.stage = Stage::Shop();
        let hack = Jokers::Hack(Hack::default());
        g.shop.jokers.push(hack.clone());
        g.buy_joker(hack).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        // + Hack: 2 triggers (base 1 + Hack's 1) -> (5 + 3*2) * 1 = 11
        assert_eq!(g.calc_score(best.clone()), 11);

        g.money += 1000;
        g.stage = Stage::Shop();
        let sb = Jokers::SockAndBuskin(SockAndBuskin::default());
        g.shop.jokers.push(sb.clone());
        g.buy_joker(sb).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        // + SockAndBuskin, no Pareidolia yet: Three isn't a face card -> unchanged
        assert_eq!(g.calc_score(best.clone()), 11);

        g.money += 1000;
        g.stage = Stage::Shop();
        let p = Jokers::Pareidolia(Pareidolia::default());
        g.shop.jokers.push(p.clone());
        g.buy_joker(p).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        // + Pareidolia: SockAndBuskin's check now passes too -> 3 triggers
        // (base 1 + Hack 1 + SockAndBuskin 1) -> (5 + 3*3) * 1 = 14
        assert_eq!(g.calc_score(best.clone()), 14);

        g.money += 1000;
        g.stage = Stage::Shop();
        let hc = Jokers::HangingChad(HangingChad::default());
        g.shop.jokers.push(hc.clone());
        g.buy_joker(hc).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        // + HangingChad: card is first -> +2 more -> 5 triggers total
        // (5 + 3*5) * 1 = 20
        assert_eq!(g.calc_score(best.clone()), 20);
    }

    #[test]
    fn test_mime_buy_flow_retriggers_held_steel() {
        // Same shape/numbers as test_seal_red_retrigger_steel_held (game.rs):
        // mult 2 -> 3 (floor(2*1.5)) -> 4 (floor(3*1.5)); score = 30 * 4 = 120
        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        let king1 = Card::new(Value::King, Suit::Heart);
        let king2 = Card::new(Value::King, Suit::Diamond);
        let mut steel_king = Card::new(Value::King, Suit::Spade);
        steel_king.enhancement = Some(Enhancement::Steel);
        g.available.extend(vec![steel_king]);
        let hand = SelectHand::new(vec![king1, king2]).best_hand().unwrap();

        let score = g.calc_score(hand.clone());
        assert_eq!(score, 90);

        g.money += 1000;
        g.stage = Stage::Shop();
        let j = Jokers::Mime(Mime::default());
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        let score = g.calc_score(hand);
        assert_eq!(score, 120);
    }

    #[test]
    fn test_dusk_buy_flow_retriggers_final_hand() {
        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        g.plays = 0;
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]).best_hand().unwrap();

        let score = g.calc_score(hand.clone());
        assert_eq!(score, 16);

        g.money += 1000;
        g.stage = Stage::Shop();
        let j = Jokers::Dusk(Dusk::default());
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        let score = g.calc_score(hand);
        assert_eq!(score, 27);
    }

    #[test]
    fn test_supernova_scales_with_hand_type_plays_this_run() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]).best_hand().unwrap(); // HighCard

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        g.money += 1000;
        g.stage = Stage::Shop();
        let j = Jokers::Supernova(Supernova::default());
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // 1st play of HighCard this run: planetarium plays 0 -> 1, +1 mult
        // (5 + 11) * (1 + 1) = 32
        assert_eq!(g.calc_score(hand.clone()), 32);
        // 2nd play of HighCard this run: plays 1 -> 2, +2 mult
        // (5 + 11) * (1 + 2) = 48
        assert_eq!(g.calc_score(hand), 48);
    }

    #[test]
    fn test_ride_the_bus_resets_on_scoring_face_card() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let no_face_hand = SelectHand::new(vec![ace]).best_hand().unwrap();

        let king = Card::new(Value::King, Suit::Heart);
        let face_hand = SelectHand::new(vec![king]).best_hand().unwrap();

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        g.money += 1000;
        g.stage = Stage::Shop();
        let j = Jokers::RideTheBus(RideTheBus::default());
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // no face card: streak 0 -> 1, +1 mult -> (5 + 11) * (1 + 1) = 32
        assert_eq!(g.calc_score(no_face_hand.clone()), 32);
        // no face card again: streak 1 -> 2, +2 mult -> (5 + 11) * (1 + 2) = 48
        assert_eq!(g.calc_score(no_face_hand), 48);
        // scoring face card resets the streak, no bonus this hand
        // (5 + 10) * 1 = 15
        assert_eq!(g.calc_score(face_hand), 15);
    }

    #[test]
    fn test_card_sharp_triples_mult_on_repeat_hand_type_same_round() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]).best_hand().unwrap(); // HighCard

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        g.money += 1000;
        g.stage = Stage::Shop();
        let j = Jokers::CardSharp(CardSharp::default());
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // 1st play of HighCard this round: not yet played -> no bonus
        assert_eq!(g.calc_score(hand.clone()), 16);
        // 2nd play of HighCard, same round: already played -> X3 mult
        // (5 + 11) * (1 * 3) = 48
        assert_eq!(g.calc_score(hand.clone()), 48);

        g.clear_blind();

        // new round: the played-this-round set was cleared -> no bonus again
        assert_eq!(g.calc_score(hand), 16);
    }

    #[test]
    fn test_obelisk_scales_mult_for_hands_not_matching_most_played_type() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let high_card_hand = SelectHand::new(vec![ace]).best_hand().unwrap(); // HighCard

        let four_aces = Card::new(Value::Ace, Suit::Club);
        let four_of_kind_hand = SelectHand::new(vec![four_aces, four_aces, four_aces, four_aces])
            .best_hand()
            .unwrap(); // FourOfAKind

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };

        // Establish HighCard as the run's most-played type before Obelisk exists.
        for _ in 0..3 {
            g.calc_score(high_card_hand.clone());
        }

        g.money += 1000;
        g.stage = Stage::Shop();
        let j = Jokers::Obelisk(Obelisk::default());
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // FourOfAKind != most-played (HighCard, 3 plays) -> streak 0 -> 1
        // (60 + 44) * (7 + (7 * 0.2 * 1) as usize) = 104 * (7 + 1) = 832
        assert_eq!(g.calc_score(four_of_kind_hand.clone()), 832);
        // still != most-played (HighCard now leads 3 to 2) -> streak 1 -> 2
        // (60 + 44) * (7 + (7 * 0.2 * 2) as usize) = 104 * (7 + 2) = 936
        assert_eq!(g.calc_score(four_of_kind_hand), 936);
        // HighCard == most-played (now 4 plays, still highest) -> streak resets
        // (5 + 11) * 1 = 16
        assert_eq!(g.calc_score(high_card_hand), 16);
    }

    #[test]
    fn test_wee_joker_accumulates_chips_per_scored_two() {
        let two = Card::new(Value::Two, Suit::Heart);
        let two_hand = SelectHand::new(vec![two]).best_hand().unwrap(); // HighCard, one 2

        let ace = Card::new(Value::Ace, Suit::Heart);
        let ace_hand = SelectHand::new(vec![ace]).best_hand().unwrap(); // HighCard, no 2s

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        g.money += 1000;
        g.stage = Stage::Shop();
        let mut j = Jokers::WeeJoker(WeeJoker::default());
        j.set_instance_id(42);
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // scored 2: counter 0 -> 8; chips = level(5) + card(2) + counter(8) = 15
        assert_eq!(g.calc_score(two_hand.clone()), 15);
        // scored 2 again: counter 8 -> 16; chips = 5 + 2 + 16 = 23
        assert_eq!(g.calc_score(two_hand), 23);
        // no 2 this hand, but the accumulated bonus persists: chips = 5 + 11 + 16 = 32
        assert_eq!(g.calc_score(ace_hand), 32);
    }

    #[test]
    fn test_spare_trousers_accumulates_mult_per_two_pair_hand() {
        let ac = Card::new(Value::Ace, Suit::Club);
        let kc = Card::new(Value::King, Suit::Club);
        let two_pair_hand = SelectHand::new(vec![ac, ac, kc, kc]).best_hand().unwrap(); // TwoPair

        let ace = Card::new(Value::Ace, Suit::Heart);
        let ace_hand = SelectHand::new(vec![ace]).best_hand().unwrap(); // HighCard, no two pair

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        g.money += 1000;
        g.stage = Stage::Shop();
        let mut j = Jokers::SpareTrousers(SpareTrousers::default());
        j.set_instance_id(99);
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // two pair: counter 0 -> 2; mult = level(2) + counter(2) = 4
        // chips = level(20) + cards(2*11 + 2*10 = 42) = 62; score = 62 * 4 = 248
        assert_eq!(g.calc_score(two_pair_hand.clone()), 248);
        // two pair again: counter 2 -> 4; mult = 2 + 4 = 6; score = 62 * 6 = 372
        assert_eq!(g.calc_score(two_pair_hand), 372);
        // no two pair this hand, bonus persists: mult = level(1) + counter(4) = 5
        // chips = 5 + 11 = 16; score = 16 * 5 = 80
        assert_eq!(g.calc_score(ace_hand), 80);
    }

    #[test]
    fn test_trading_card_destroys_first_single_discard_only() {
        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        g.money += 1000;
        g.stage = Stage::Shop();
        let mut j = Jokers::TradingCard(TradingCard::default());
        j.set_instance_id(1);
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        let c1 = Card::new(Value::Two, Suit::Heart);
        g.available.extend(vec![c1]);
        g.select_card(c1).unwrap();
        let money_before = g.money;
        g.discard_selected().unwrap();
        // first discard of the round, exactly 1 card: destroyed + $3
        assert_eq!(g.money, money_before + 3);
        assert!(!g.discarded.iter().any(|c| c.id == c1.id));

        let c2 = Card::new(Value::Three, Suit::Club);
        g.available.extend(vec![c2]);
        g.select_card(c2).unwrap();
        let money_before = g.money;
        g.discard_selected().unwrap();
        // second discard of the round, even though it's also 1 card: no-op
        assert_eq!(g.money, money_before);
        assert!(g.discarded.iter().any(|c| c.id == c2.id));
    }

    #[test]
    fn test_green_joker_mult_tracks_hands_played_minus_discards() {
        let ace_hand = SelectHand::new(vec![Card::new(Value::Ace, Suit::Heart)])
            .best_hand()
            .unwrap();

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        g.money += 1000;
        g.stage = Stage::Shop();
        let mut j = Jokers::GreenJoker(GreenJoker::default());
        j.set_instance_id(1);
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // chips = level(5) + card(11) = 16
        // no hands played, no discards: mult = level(1) + 0 = 1; score = 16
        assert_eq!(g.calc_score(ace_hand.clone()), 16);

        g.plays -= 1; // 1 hand played
                      // mult = 1 + (1 - 0) = 2; score = 16 * 2 = 32
        assert_eq!(g.calc_score(ace_hand.clone()), 32);

        g.discards_remaining -= 1; // 1 discard used
                         // mult = 1 + (1 - 1) = 1; score = 16
        assert_eq!(g.calc_score(ace_hand.clone()), 16);

        g.discards_remaining -= 1; // 2 discards used
                         // mult = 1 + (1 - 2) = 0 (floored, not negative); score = 0
        assert_eq!(g.calc_score(ace_hand), 0);
    }

    #[test]
    fn test_hit_the_road_mult_from_jacks_discarded_this_round() {
        let ace_hand = SelectHand::new(vec![Card::new(Value::Ace, Suit::Heart)])
            .best_hand()
            .unwrap();

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        g.money += 1000;
        g.stage = Stage::Shop();
        let mut j = Jokers::HitTheRoad(HitTheRoad::default());
        j.set_instance_id(1);
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // play a hand containing a Jack first - it lands in `g.discarded`
        // (the deck-recycling pile) but must NOT count toward HitTheRoad's
        // bonus, which is scoped to `discarded_this_round` (Discard-action
        // cards only). If the joker's filter ever regressed to reading
        // `g.discarded` instead, this Jack would wrongly inflate the count
        // and the very next assertion would fail.
        let played_jack = Card::new(Value::Jack, Suit::Diamond);
        g.available.extend(vec![played_jack]);
        g.select_card(played_jack).unwrap();
        g.play_selected().unwrap();

        let jack1 = Card::new(Value::Jack, Suit::Club);
        g.available.extend(vec![jack1]);
        g.select_card(jack1).unwrap();
        g.discard_selected().unwrap();
        // 1 jack discarded (the played jack above doesn't count):
        // bonus = mult(1) * 0.5 * 1 = 0.5, truncates to 0
        assert_eq!(g.calc_score(ace_hand.clone()), 16);

        let jack2 = Card::new(Value::Jack, Suit::Spade);
        g.available.extend(vec![jack2]);
        g.select_card(jack2).unwrap();
        g.discard_selected().unwrap();
        // 2 jacks discarded across 2 separate discards this round: bonus = 1 * 0.5 * 2 = 1
        // mult = 1 + 1 = 2; score = 16 * 2 = 32
        assert_eq!(g.calc_score(ace_hand), 32);
    }

    #[test]
    fn test_yorick_mult_scales_with_run_total_cards_discarded() {
        let ace_hand = SelectHand::new(vec![Card::new(Value::Ace, Suit::Heart)])
            .best_hand()
            .unwrap();

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        g.money += 1000;
        g.stage = Stage::Shop();
        let mut j = Jokers::Yorick(Yorick::default());
        j.set_instance_id(1);
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        g.total_cards_discarded = 22;
        // 22 / 23 = 0 levels; score = 16
        assert_eq!(g.calc_score(ace_hand.clone()), 16);

        g.total_cards_discarded = 23;
        // 1 level: mult = 1 + (1 * 1) = 2; score = 16 * 2 = 32
        assert_eq!(g.calc_score(ace_hand.clone()), 32);

        g.total_cards_discarded = 46;
        // 2 levels: mult = 1 + (1 * 2) = 3; score = 16 * 3 = 48
        assert_eq!(g.calc_score(ace_hand), 48);
    }

    #[test]
    fn test_castle_chips_from_discarded_suit_this_round() {
        let ace_hand = SelectHand::new(vec![Card::new(Value::Ace, Suit::Club)])
            .best_hand()
            .unwrap();

        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        g.money += 1000;
        g.stage = Stage::Shop();
        let mut j = Jokers::Castle(Castle::default());
        j.set_instance_id(1);
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        g.joker_state_mut(1).unwrap().selector = Some(SelectorValue::Suit(Suit::Heart));

        // play a Heart first - it lands in `g.discarded` (deck-recycling
        // pile) but must NOT count toward Castle's bonus, which is scoped
        // to `discarded_this_round` (Discard-action cards only). If the
        // joker's filter ever regressed to reading `g.discarded` instead,
        // this Heart would wrongly inflate the count and the assertion
        // below would fail.
        let played_heart = Card::new(Value::Five, Suit::Heart);
        g.available.extend(vec![played_heart]);
        g.select_card(played_heart).unwrap();
        g.play_selected().unwrap();

        let h1 = Card::new(Value::Two, Suit::Heart);
        let h2 = Card::new(Value::Three, Suit::Heart);
        let c1 = Card::new(Value::Four, Suit::Club); // non-matching suit, doesn't count
        g.available.extend(vec![h1, h2, c1]);
        g.select_card(h1).unwrap();
        g.select_card(h2).unwrap();
        g.select_card(c1).unwrap();
        g.discard_selected().unwrap();

        // chips = level(5) + card(11) + 2 discarded hearts (not the played
        // one) * 3 = 22; mult = level(1); score = 22
        assert_eq!(g.calc_score(ace_hand), 22);
    }

    #[test]
    fn test_mail_in_rebate_pays_per_matching_rank_discarded() {
        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        g.money += 1000;
        g.stage = Stage::Shop();
        let mut j = Jokers::MailInRebate(MailInRebate::default());
        j.set_instance_id(1);
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        g.joker_state_mut(1).unwrap().selector = Some(SelectorValue::Value(Value::King));

        let k1 = Card::new(Value::King, Suit::Heart);
        let k2 = Card::new(Value::King, Suit::Spade);
        let three = Card::new(Value::Three, Suit::Club);
        g.available.extend(vec![k1, k2, three]);
        g.select_card(k1).unwrap();
        g.select_card(k2).unwrap();
        g.select_card(three).unwrap();
        let money_before = g.money;
        g.discard_selected().unwrap();
        // 2 kings discarded in this action: +$5 each
        assert_eq!(g.money, money_before + 10);

        let k3 = Card::new(Value::King, Suit::Diamond);
        g.available.extend(vec![k3]);
        g.select_card(k3).unwrap();
        let money_before = g.money;
        g.discard_selected().unwrap();
        // immediate per-event payout, not accumulated from prior discards
        assert_eq!(g.money, money_before + 5);
    }

    #[test]
    fn test_discard_selectors_reroll_on_clear_blind() {
        let mut g = Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        };
        g.money += 1000;
        g.stage = Stage::Shop();
        let mut castle = Jokers::Castle(Castle::default());
        castle.set_instance_id(1);
        g.shop.jokers.push(castle.clone());
        g.buy_joker(castle).unwrap();
        let mut mail = Jokers::MailInRebate(MailInRebate::default());
        mail.set_instance_id(2);
        g.shop.jokers.push(mail.clone());
        g.buy_joker(mail).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        assert_eq!(g.joker_state_mut(1).unwrap().selector, None);
        assert_eq!(g.joker_state_mut(2).unwrap().selector, None);

        g.clear_blind();

        assert!(g.joker_state_mut(1).unwrap().selector.is_some());
        assert!(g.joker_state_mut(2).unwrap().selector.is_some());
    }

    #[test]
    fn test_roll_discard_selector_sets_castle_and_mail_in_rebate() {
        // `roll_discard_selector` is called directly inside both real mint
        // chokepoints (`JokerGenerator::gen_joker`, `rng::seed_joker_with_id`)
        // - tested here directly rather than via `gen_joker`'s own random
        // rarity/pool selection. An earlier version of this test sampled
        // `gen_joker` in a loop hoping to draw both jokers within N tries,
        // which ties the test's pass/fail to incidental RNG-sequence
        // details - a future change to the joker roster or rarity weights
        // could shift whether a fixed seed lands on both within a bounded
        // number of draws, failing this test for no real regression.
        use rand::SeedableRng;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(7);

        let mut castle = Jokers::Castle(Castle::default());
        roll_discard_selector(&mut rng, &mut castle);
        assert!(
            castle.state().selector.is_some(),
            "Castle minted with no selector"
        );

        let mut mail = Jokers::MailInRebate(MailInRebate::default());
        roll_discard_selector(&mut rng, &mut mail);
        assert!(
            mail.state().selector.is_some(),
            "MailInRebate minted with no selector"
        );
    }

    // --- Group A boss debuff x joker interactions ---
    // Each test plays/holds/discards a card that would normally trigger the
    // joker, but the card is debuffed by the relevant Group A boss (via
    // `non_debuffed`) - the joker's bonus must not apply.

    #[test]
    fn test_greedy_joker_does_not_count_debuffed_diamond() {
        let mut g = Game {
            current_boss: Some(BossBlind::Window),
            blind: Some(Blind::Boss),
            jokers: vec![Jokers::GreedyJoker(GreedyJoker::default())],
            ..Default::default()
        };
        let ace = Card::new(Value::Ace, Suit::Diamond);
        let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
        // (5) * 1 = 5 - debuffed card contributes nothing and isn't counted.
        assert_eq!(g.calc_score(hand), 5);
    }

    #[test]
    fn test_lusty_joker_does_not_count_debuffed_heart() {
        let mut g = Game {
            current_boss: Some(BossBlind::Head),
            blind: Some(Blind::Boss),
            jokers: vec![Jokers::LustyJoker(LustyJoker::default())],
            ..Default::default()
        };
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
        assert_eq!(g.calc_score(hand), 5);
    }

    #[test]
    fn test_wrathful_joker_does_not_count_debuffed_spade() {
        let mut g = Game {
            current_boss: Some(BossBlind::Goad),
            blind: Some(Blind::Boss),
            jokers: vec![Jokers::WrathfulJoker(WrathfulJoker::default())],
            ..Default::default()
        };
        let ace = Card::new(Value::Ace, Suit::Spade);
        let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
        assert_eq!(g.calc_score(hand), 5);
    }

    #[test]
    fn test_gluttonous_joker_does_not_count_debuffed_club() {
        let mut g = Game {
            current_boss: Some(BossBlind::Club),
            blind: Some(Blind::Boss),
            jokers: vec![Jokers::GluttonousJoker(GluttonousJoker::default())],
            ..Default::default()
        };
        let ace = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
        assert_eq!(g.calc_score(hand), 5);
    }

    #[test]
    fn test_rough_gem_does_not_pay_for_debuffed_diamond() {
        let mut g = Game {
            current_boss: Some(BossBlind::Window),
            blind: Some(Blind::Boss),
            jokers: vec![Jokers::RoughGem(RoughGem::default())],
            ..Default::default()
        };
        let money_before = g.money;
        let ace = Card::new(Value::Ace, Suit::Diamond);
        let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
        g.calc_score(hand);
        assert_eq!(g.money, money_before);
    }

    #[test]
    fn test_bloodstone_does_not_trigger_on_debuffed_heart() {
        let mut g = Game {
            current_boss: Some(BossBlind::Head),
            blind: Some(Blind::Boss),
            jokers: vec![Jokers::Bloodstone(Bloodstone::default())],
            ..Default::default()
        };
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
        // debuffed card is filtered before the probabilistic roll ever
        // happens, so this is deterministic despite Bloodstone's own RNG.
        assert_eq!(g.calc_score(hand), 5);
    }

    #[test]
    fn test_arrowhead_does_not_add_chips_for_debuffed_spade() {
        let mut g = Game {
            current_boss: Some(BossBlind::Goad),
            blind: Some(Blind::Boss),
            jokers: vec![Jokers::Arrowhead(Arrowhead::default())],
            ..Default::default()
        };
        let ace = Card::new(Value::Ace, Suit::Spade);
        let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
        assert_eq!(g.calc_score(hand), 5);
    }

    #[test]
    fn test_onyx_agate_does_not_add_mult_for_debuffed_club() {
        let mut g = Game {
            current_boss: Some(BossBlind::Club),
            blind: Some(Blind::Boss),
            jokers: vec![Jokers::OnyxAgate(OnyxAgate::default())],
            ..Default::default()
        };
        let ace = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
        assert_eq!(g.calc_score(hand), 5);
    }

    #[test]
    fn test_castle_does_not_count_debuffed_discarded_suit() {
        let mut j = Jokers::Castle(Castle::default());
        j.set_instance_id(1);
        let mut g = Game {
            current_boss: Some(BossBlind::Club),
            blind: Some(Blind::Boss),
            jokers: vec![j],
            ..Default::default()
        };
        g.joker_state_mut(1).unwrap().selector = Some(SelectorValue::Suit(Suit::Club));

        let club_card = Card::new(Value::Four, Suit::Club);
        g.available.extend(vec![club_card]);
        g.select_card(club_card).unwrap();
        g.discard_selected().unwrap();

        let ace_hand = SelectHand::new(vec![Card::new(Value::Ace, Suit::Heart)])
            .best_hand()
            .unwrap();
        // debuffed discarded Club doesn't count -> Castle contributes 0
        // chips. (5 + 11) * 1 = 16, not 16 + 3 = 19.
        assert_eq!(g.calc_score(ace_hand), 16);
    }

    #[test]
    fn test_hit_the_road_does_not_count_debuffed_discarded_jack() {
        let mut g = Game {
            current_boss: Some(BossBlind::Club),
            blind: Some(Blind::Boss),
            jokers: vec![Jokers::HitTheRoad(HitTheRoad::default())],
            ..Default::default()
        };
        let jack_club = Card::new(Value::Jack, Suit::Club);
        g.available.extend(vec![jack_club]);
        g.select_card(jack_club).unwrap();
        g.discard_selected().unwrap();

        let ace_hand = SelectHand::new(vec![Card::new(Value::Ace, Suit::Heart)])
            .best_hand()
            .unwrap();
        // debuffed discarded Jack doesn't count -> HitTheRoad contributes
        // 0 mult. (5 + 11) * 1 = 16, not (5 + 11) * 1.5 = 24.
        assert_eq!(g.calc_score(ace_hand), 16);
    }

    #[test]
    fn test_smiley_face_does_not_count_debuffed_face_card() {
        let mut g = Game {
            current_boss: Some(BossBlind::Plant),
            blind: Some(Blind::Boss),
            jokers: vec![Jokers::SmileyFace(SmileyFace::default())],
            ..Default::default()
        };
        let king = Card::new(Value::King, Suit::Heart);
        let hand = SelectHand::new(vec![king]).best_hand().unwrap();
        assert_eq!(g.calc_score(hand), 5);
    }

    #[test]
    fn test_scary_face_does_not_add_chips_for_debuffed_face_card() {
        let mut g = Game {
            current_boss: Some(BossBlind::Plant),
            blind: Some(Blind::Boss),
            jokers: vec![Jokers::ScaryFace(ScaryFace::default())],
            ..Default::default()
        };
        let king = Card::new(Value::King, Suit::Heart);
        let hand = SelectHand::new(vec![king]).best_hand().unwrap();
        assert_eq!(g.calc_score(hand), 5);
    }

    #[test]
    fn test_business_card_does_not_trigger_on_debuffed_face_card() {
        let mut g = Game {
            current_boss: Some(BossBlind::Plant),
            blind: Some(Blind::Boss),
            jokers: vec![Jokers::BusinessCard(BusinessCard::default())],
            ..Default::default()
        };
        let money_before = g.money;
        let king = Card::new(Value::King, Suit::Heart);
        let hand = SelectHand::new(vec![king]).best_hand().unwrap();
        g.calc_score(hand);
        assert_eq!(g.money, money_before);
    }

    #[test]
    fn test_photograph_does_not_double_mult_for_debuffed_face_card() {
        let mut g = Game {
            current_boss: Some(BossBlind::Plant),
            blind: Some(Blind::Boss),
            jokers: vec![Jokers::Photograph(Photograph::default())],
            ..Default::default()
        };
        let king = Card::new(Value::King, Suit::Heart);
        let hand = SelectHand::new(vec![king]).best_hand().unwrap();
        // mult stays at 1 (never doubled) -> 5 * 1 = 5, not 5 * 2 = 10.
        assert_eq!(g.calc_score(hand), 5);
    }

    #[test]
    fn test_ride_the_bus_ignores_debuffed_face_card() {
        let mut g = Game {
            current_boss: Some(BossBlind::Plant),
            blind: Some(Blind::Boss),
            jokers: vec![Jokers::RideTheBus(RideTheBus::default())],
            ..Default::default()
        };
        let king = Card::new(Value::King, Suit::Heart);
        let hand = SelectHand::new(vec![king]).best_hand().unwrap();
        // debuffed King doesn't count as a face card -> streak 0 -> 1,
        // +1 mult. chips = 5 (debuffed), mult = 1 + 1 = 2 -> 5 * 2 = 10.
        // If the debuff were ignored here, the King would reset the streak
        // instead (mult stays 1, score 5).
        assert_eq!(g.calc_score(hand), 10);
    }

    #[test]
    fn test_midas_mask_does_not_convert_debuffed_face_card() {
        let mut g = Game {
            current_boss: Some(BossBlind::Plant),
            blind: Some(Blind::Boss),
            jokers: vec![Jokers::MidasMask(MidasMask::default())],
            ..Default::default()
        };
        let king = Card::new(Value::King, Suit::Heart);
        g.available.extend(vec![king]);

        let hand = SelectHand::new(vec![king]);
        g.calc_score(hand.best_hand().unwrap());

        let scored = g.available.cards().into_iter().find(|c| c.id == king.id);
        assert_eq!(scored.unwrap().enhancement, None);
    }

    #[test]
    fn test_reserved_parking_does_not_trigger_on_debuffed_held_face_card() {
        let mut g = Game {
            current_boss: Some(BossBlind::Plant),
            blind: Some(Blind::Boss),
            jokers: vec![Jokers::ReservedParking(ReservedParking::default())],
            ..Default::default()
        };
        let money_before = g.money;
        let king = Card::new(Value::King, Suit::Heart);
        g.available.extend(vec![king]); // held, not played

        let ace = Card::new(Value::Ace, Suit::Diamond);
        let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
        g.calc_score(hand);
        assert_eq!(g.money, money_before);
    }

    #[test]
    fn test_baron_does_not_count_debuffed_held_king() {
        let mut g = Game {
            current_boss: Some(BossBlind::Plant),
            blind: Some(Blind::Boss),
            jokers: vec![Jokers::Baron(Baron::default())],
            ..Default::default()
        };
        let king = Card::new(Value::King, Suit::Heart);
        g.available.extend(vec![king]); // held, not played

        let ace = Card::new(Value::Ace, Suit::Diamond);
        let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
        // Baron never sees the held King (debuffed) -> mult stays 1.
        // (5 + 11) * 1 = 16, not * 1.5 = 24.
        assert_eq!(g.calc_score(hand), 16);
    }

    #[test]
    fn test_matador_pays_out_when_boss_ability_triggers() {
        // Club is Matador-Yes (docs/boss-blinds.md §2): a played Club card
        // gets debuffed, which sets boss_triggered_this_hand.
        let mut g = Game {
            current_boss: Some(BossBlind::Club),
            blind: Some(Blind::Boss),
            jokers: vec![Jokers::Matador(Matador::default())],
            ..Default::default()
        };
        let money_before = g.money;
        let king = Card::new(Value::King, Suit::Club);
        let hand = SelectHand::new(vec![king]).best_hand().unwrap();
        g.calc_score(hand);
        assert_eq!(g.money, money_before + 8);
    }

    #[test]
    fn test_matador_no_payout_without_boss_trigger() {
        let mut g = Game {
            current_boss: Some(BossBlind::Club),
            blind: Some(Blind::Boss),
            jokers: vec![Jokers::Matador(Matador::default())],
            ..Default::default()
        };
        let money_before = g.money;
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
        g.calc_score(hand);
        assert_eq!(g.money, money_before);
    }
}
