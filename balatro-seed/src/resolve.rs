//! Maps a raw pool string (`pools.rs`) onto the matching `balatro_types`
//! variant, by display-name lookup. Used to build `pools.rs`'s `Pool<T>`
//! constants.

use balatro_types::{
    BossBlind, Card, Enhancement, Jokers, Planets, Spectral, Suit, Tag, Tarot, Value, Voucher,
};
use strum::IntoEnumIterator;

pub(crate) fn resolve_joker(name: &str) -> Option<Jokers> {
    Jokers::iter().find(|j| j.name() == name)
}

pub(crate) fn resolve_tarot(name: &str) -> Option<Tarot> {
    Tarot::iter().find(|t| t.name() == name)
}

pub(crate) fn resolve_planet(name: &str) -> Option<Planets> {
    Planets::iter().find(|p| p.name() == name)
}

pub(crate) fn resolve_spectral(name: &str) -> Option<Spectral> {
    Spectral::iter().find(|s| s.name() == name)
}

pub(crate) fn resolve_voucher(name: &str) -> Option<Voucher> {
    Voucher::iter().find(|v| v.name() == name)
}

pub(crate) fn resolve_tag(name: &str) -> Option<Tag> {
    Tag::iter().find(|t| t.name() == name)
}

pub(crate) fn resolve_boss(name: &str) -> Option<BossBlind> {
    BossBlind::iter().find(|b| b.name() == name)
}

/// Pool strings are exactly the variant names, so a direct parse works
/// here unlike the other resolvers above.
pub(crate) fn resolve_enhancement(name: &str) -> Option<Enhancement> {
    name.parse().ok()
}

fn value_from_char(c: char) -> Option<Value> {
    match c {
        '2' => Some(Value::Two),
        '3' => Some(Value::Three),
        '4' => Some(Value::Four),
        '5' => Some(Value::Five),
        '6' => Some(Value::Six),
        '7' => Some(Value::Seven),
        '8' => Some(Value::Eight),
        '9' => Some(Value::Nine),
        'T' => Some(Value::Ten),
        'J' => Some(Value::Jack),
        'Q' => Some(Value::Queen),
        'K' => Some(Value::King),
        'A' => Some(Value::Ace),
        _ => None,
    }
}

fn suit_from_char(c: char) -> Option<Suit> {
    match c {
        'C' => Some(Suit::Club),
        'D' => Some(Suit::Diamond),
        'H' => Some(Suit::Heart),
        'S' => Some(Suit::Spade),
        _ => None,
    }
}

/// Parses a `CARDS` pool entry (`"{suit}_{rank}"`, e.g. `"S_T"`) — a
/// positional format, unlike the display-name lookups above.
pub(crate) fn resolve_card_base(base: &str) -> Option<Card> {
    let bytes = base.as_bytes();
    let suit = suit_from_char(*bytes.first()? as char)?;
    let value = value_from_char(*bytes.get(2)? as char)?;
    Some(Card::new(value, suit))
}
