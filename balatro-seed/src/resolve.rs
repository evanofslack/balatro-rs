//! Maps a raw pool string (as transcribed from Immolate/TheSoul in
//! `pools.rs`) onto the matching `balatro_types` variant, by display-name
//! lookup (`T::iter().find(|v| v.name() == name)`).
//!
//! This is the safety net for the transcription risk called out in the
//! plan: `pool_names_all_resolve` below exercises every table entry and
//! fails loudly, listing every mismatch at once, rather than letting a
//! naming discrepancy between TheSoul's strings and `balatro_types`'
//! declared names silently produce a wrong draw.

use balatro_types::{
    BossBlind, Card, Enhancement, Jokers, Planets, Spectral, Suit, Tag, Tarot, Value, Voucher,
};
use strum::IntoEnumIterator;

pub fn resolve_joker(name: &str) -> Option<Jokers> {
    Jokers::iter().find(|j| j.name() == name)
}

pub fn resolve_tarot(name: &str) -> Option<Tarot> {
    Tarot::iter().find(|t| t.name() == name)
}

pub fn resolve_planet(name: &str) -> Option<Planets> {
    Planets::iter().find(|p| p.name() == name)
}

pub fn resolve_spectral(name: &str) -> Option<Spectral> {
    Spectral::iter().find(|s| s.name() == name)
}

pub fn resolve_voucher(name: &str) -> Option<Voucher> {
    Voucher::iter().find(|v| v.name() == name)
}

pub fn resolve_tag(name: &str) -> Option<Tag> {
    Tag::iter().find(|t| t.name() == name)
}

pub fn resolve_boss(name: &str) -> Option<BossBlind> {
    BossBlind::iter().find(|b| b.name() == name)
}

/// `Enhancement`/`Value` derive `EnumString` and their pool strings are
/// exactly the variant names ("Bonus", "Two", ...), so a direct parse
/// works here — unlike the other resolvers above, which match against a
/// separate declared display name.
pub fn resolve_enhancement(name: &str) -> Option<Enhancement> {
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

/// Parses a `CARDS` pool entry (`"{suit}_{rank}"`, e.g. `"S_T"`) into a
/// bare `Card` — a positional format, not a display-name lookup, so this
/// doesn't fit the `T::iter().find(...)` pattern the rest of this module
/// uses.
pub fn resolve_card_base(base: &str) -> Option<Card> {
    let bytes = base.as_bytes();
    let suit = suit_from_char(*bytes.first()? as char)?;
    let value = value_from_char(*bytes.get(2)? as char)?;
    Some(Card::new(value, suit))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pools;

    #[test]
    fn pool_names_all_resolve() {
        let mut failures: Vec<String> = Vec::new();

        for &name in pools::TAROTS {
            if resolve_tarot(name).is_none() {
                failures.push(format!("tarot: {name:?}"));
            }
        }
        for &name in pools::PLANETS {
            if resolve_planet(name).is_none() {
                failures.push(format!("planet: {name:?}"));
            }
        }
        for &name in pools::SPECTRALS {
            if name != "RETRY" && resolve_spectral(name).is_none() {
                failures.push(format!("spectral: {name:?}"));
            }
        }
        for &name in pools::COMMON_JOKERS
            .iter()
            .chain(pools::UNCOMMON_JOKERS)
            .chain(pools::RARE_JOKERS)
            .chain(pools::LEGENDARY_JOKERS)
        {
            if resolve_joker(name).is_none() {
                failures.push(format!("joker: {name:?}"));
            }
        }
        for &name in pools::VOUCHERS {
            if resolve_voucher(name).is_none() {
                failures.push(format!("voucher: {name:?}"));
            }
        }
        for &name in pools::TAGS {
            if resolve_tag(name).is_none() {
                failures.push(format!("tag: {name:?}"));
            }
        }
        for &name in pools::BOSSES {
            if resolve_boss(name).is_none() {
                failures.push(format!("boss: {name:?}"));
            }
        }
        for &name in pools::ENHANCEMENTS {
            if resolve_enhancement(name).is_none() {
                failures.push(format!("enhancement: {name:?}"));
            }
        }
        for &base in pools::CARDS {
            if resolve_card_base(base).is_none() {
                failures.push(format!("card base: {base:?}"));
            }
        }

        assert!(
            failures.is_empty(),
            "{} pool name(s) failed to resolve against balatro_types:\n{}",
            failures.len(),
            failures.join("\n")
        );
    }
}
