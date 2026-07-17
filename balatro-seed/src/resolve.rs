//! Maps a raw pool string (as transcribed from Immolate/TheSoul in
//! `pools.rs`) onto the matching `balatro_types` variant, by display-name
//! lookup (`T::iter().find(|v| v.name() == name)`).
//!
//! This is the safety net for the transcription risk called out in the
//! plan: `pool_names_all_resolve` below exercises every table entry and
//! fails loudly, listing every mismatch at once, rather than letting a
//! naming discrepancy between TheSoul's strings and `balatro_types`'
//! declared names silently produce a wrong draw.

use balatro_types::{BossBlind, Jokers, Planets, Spectral, Tag, Tarot, Voucher};
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

        assert!(
            failures.is_empty(),
            "{} pool name(s) failed to resolve against balatro_types:\n{}",
            failures.len(),
            failures.join("\n")
        );
    }
}
