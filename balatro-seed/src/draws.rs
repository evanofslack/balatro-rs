//! Typed public draw API, mirroring `TheSoul/include/functions.hpp`'s
//! `Instance` methods. Resolves raw pool strings (`pools.rs`) to
//! `balatro_types` values via `resolve.rs`, and panics loudly on any
//! unresolved name rather than silently returning something wrong — see
//! `resolve::tests::pool_names_all_resolve` for the up-front check that
//! should catch these before a panic ever fires here.
//!
//! Scope cuts from the full Immolate port, tracked as follow-ups:
//! - Joker stickers (eternal/perishable/rental) — stake-gated, not ported.
//! - Playing-card shop items (Magic Trick voucher) — `ShopItem::PlayingCard`
//!   stays a placeholder deliberately: Immolate's own `nextShopItem` never
//!   implements this branch either ("Todo: Magic Trick support" in the
//!   source), so there's no reference to verify against. Standard *packs*
//!   are implemented (`next_standard_card`/`next_standard_pack`).
//! - `initLocks`'s `freshProfile`/`freshRun` blocks are ported and callable
//!   (`Instance::init_locks`'s two bools), but callers still hardcode both
//!   to `true` (assume nothing ever discovered) — seeding `fresh_profile`'s
//!   lock list from a real `Profile.unlocked`/`discovered` set instead is a
//!   later follow-up, not done here.
//! - Pre-10099-version joker pool variants (see `pools.rs`).

use crate::instance::Instance;
use crate::pools;
use crate::resolve;
use balatro_types::{BossBlind, Card, Consumable, Edition, Jokers, Seal, Spectral, Tag, Voucher};

pub enum ShopItem {
    Joker(Jokers),
    Consumable(Consumable),
    PlayingCard,
}

/// Given a drawn base voucher, returns its upgrade tier if one exists.
/// `VOUCHERS` (`pools.rs`) is laid out as (base, upgrade) pairs at
/// consecutive indices — mirrors the pairing loop in `TheSoul`'s own
/// `index.html` demo (`Immolate.VOUCHERS`), used there to decide which
/// voucher becomes drawable next.
pub fn voucher_upgrade(voucher: Voucher) -> Option<Voucher> {
    let name = voucher.name();
    let idx = pools::VOUCHERS.iter().position(|&n| n == name)?;
    if idx % 2 == 0 {
        resolve::resolve_voucher(pools::VOUCHERS[idx + 1])
    } else {
        None
    }
}

/// `functions.hpp::packInfo`: decodes a drawn pack name (as returned by
/// [`Instance::next_pack`]) into its base category (e.g. "Jumbo Celestial
/// Pack" -> "Celestial Pack") and card count. `Mega`/`Jumbo` sizes are 4/2
/// for Buffoon and Spectral packs, 5/3 for every other category — detected
/// by checking specific character positions, exactly as the source does,
/// since "Spectral" and "Standard" only diverge at their second letter.
pub fn pack_info(pack: &str) -> (&str, i32) {
    let bytes = pack.as_bytes();
    if bytes[0] == b'M' {
        let size = if bytes[5] == b'B' || bytes[6] == b'p' {
            4
        } else {
            5
        };
        (&pack[5..], size)
    } else if bytes[0] == b'J' {
        let size = if bytes[6] == b'B' || bytes[7] == b'p' {
            4
        } else {
            5
        };
        (&pack[6..], size)
    } else {
        let size = if bytes[0] == b'B' || bytes[1] == b'p' {
            2
        } else {
            3
        };
        (pack, size)
    }
}

fn resolved_joker(name: &str) -> Jokers {
    resolve::resolve_joker(name)
        .unwrap_or_else(|| panic!("pool name {name:?} has no balatro_types::Jokers match"))
}

impl Instance {
    /// Full port of `functions.hpp::initLocks`. `fresh_profile` gates
    /// permanent, profile-level achievement locks (things a real save's
    /// `Profile.unlocked` would eventually override — see draws.rs module
    /// docs); `fresh_run` gates locks that reset every run regardless of
    /// profile, because the requirement is inherently in-run (Gros Michel
    /// destroyed this run for Cavendish, a secret hand played this run for
    /// Planet X/Ceres/Eris, ...) and can't be satisfied by profile state
    /// alone.
    pub fn init_locks(&mut self, ante: i32, fresh_profile: bool, fresh_run: bool) {
        if ante < 2 {
            for name in [
                "The Mouth",
                "The Fish",
                "The Wall",
                "The House",
                "The Mark",
                "The Wheel",
                "The Arm",
                "The Water",
                "The Needle",
                "The Flint",
                "Negative Tag",
                "Standard Tag",
                "Meteor Tag",
                "Buffoon Tag",
                "Handy Tag",
                "Garbage Tag",
                "Ethereal Tag",
                "Top-up Tag",
                "Orbital Tag",
            ] {
                self.lock(name);
            }
        }
        if ante < 3 {
            self.lock("The Tooth");
            self.lock("The Eye");
        }
        if ante < 4 {
            self.lock("The Plant");
        }
        if ante < 5 {
            self.lock("The Serpent");
        }
        if ante < 6 {
            self.lock("The Ox");
        }
        if fresh_profile {
            for name in [
                "Negative Tag",
                "Foil Tag",
                "Holographic Tag",
                "Polychrome Tag",
                "Rare Tag",
                "Golden Ticket",
                "Mr. Bones",
                "Acrobat",
                "Sock and Buskin",
                "Swashbuckler",
                "Troubadour",
                "Certificate",
                "Smeared Joker",
                "Throwback",
                "Hanging Chad",
                "Rough Gem",
                "Bloodstone",
                "Arrowhead",
                "Onyx Agate",
                "Glass Joker",
                "Showman",
                "Flower Pot",
                "Blueprint",
                "Wee Joker",
                "Merry Andy",
                "Oops! All 6s",
                "The Idol",
                "Seeing Double",
                "Matador",
                "Hit the Road",
                "The Duo",
                "The Trio",
                "The Family",
                "The Order",
                "The Tribe",
                "Stuntman",
                "Invisible Joker",
                "Brainstorm",
                "Satellite",
                "Shoot the Moon",
                "Driver's License",
                "Cartomancer",
                "Astronomer",
                "Burnt Joker",
                "Bootstraps",
                "Overstock Plus",
                "Liquidation",
                "Glow Up",
                "Reroll Glut",
                "Omen Globe",
                "Observatory",
                "Nacho Tong",
                "Recyclomancy",
                "Tarot Tycoon",
                "Planet Tycoon",
                "Money Tree",
                "Antimatter",
                "Illusion",
                "Petroglyph",
                "Retcon",
                "Palette",
            ] {
                self.lock(name);
            }
        }
        if fresh_run {
            for name in [
                "Planet X",
                "Ceres",
                "Eris",
                "Five of a Kind",
                "Flush House",
                "Flush Five",
                "Stone Joker",
                "Steel Joker",
                "Glass Joker",
                "Golden Ticket",
                "Lucky Cat",
                "Cavendish",
                "Overstock Plus",
                "Liquidation",
                "Glow Up",
                "Reroll Glut",
                "Omen Globe",
                "Observatory",
                "Nacho Tong",
                "Recyclomancy",
                "Tarot Tycoon",
                "Planet Tycoon",
                "Money Tree",
                "Antimatter",
                "Illusion",
                "Petroglyph",
                "Retcon",
                "Palette",
            ] {
                self.lock(name);
            }
        }
    }

    /// Ante-gated unlocks (`functions.hpp::initUnlocks`).
    pub fn init_unlocks(&mut self, ante: i32, fresh_profile: bool) {
        if ante == 2 {
            for name in [
                "The Mouth",
                "The Fish",
                "The Wall",
                "The House",
                "The Mark",
                "The Wheel",
                "The Arm",
                "The Water",
                "The Needle",
                "The Flint",
                "Standard Tag",
                "Meteor Tag",
                "Buffoon Tag",
                "Handy Tag",
                "Garbage Tag",
                "Ethereal Tag",
                "Top-up Tag",
                "Orbital Tag",
            ] {
                self.unlock(name);
            }
            if !fresh_profile {
                self.unlock("Negative Tag");
            }
        }
        if ante == 3 {
            self.unlock("The Tooth");
            self.unlock("The Eye");
        }
        if ante == 4 {
            self.unlock("The Plant");
        }
        if ante == 5 {
            self.unlock("The Serpent");
        }
        if ante == 6 {
            self.unlock("The Ox");
        }
    }

    /// `functions.hpp::nextTarot`. Can return `The Soul` (a Spectral card)
    /// instead of a real tarot — a real-game quirk, not a bug.
    pub fn next_tarot(&mut self, source: &str, ante: i32, soulable: bool) -> Consumable {
        let ante_str = ante.to_string();
        if soulable
            && (self.params.showman || !self.is_locked("The Soul"))
            && self.random(&format!("soul_Tarot{ante_str}")) > 0.997
        {
            return Consumable::Spectral(Spectral::Soul);
        }
        let name = self.randchoice(&format!("Tarot{source}{ante_str}"), pools::TAROTS);
        let tarot = resolve::resolve_tarot(name)
            .unwrap_or_else(|| panic!("pool name {name:?} has no balatro_types::Tarot match"));
        Consumable::Tarot(tarot)
    }

    /// `functions.hpp::nextPlanet`. Can return `Black Hole` (a Spectral
    /// card) instead of a real planet.
    pub fn next_planet(&mut self, source: &str, ante: i32, soulable: bool) -> Consumable {
        let ante_str = ante.to_string();
        if soulable
            && (self.params.showman || !self.is_locked("Black Hole"))
            && self.random(&format!("soul_Planet{ante_str}")) > 0.997
        {
            return Consumable::Spectral(Spectral::BlackHole);
        }
        let name = self.randchoice(&format!("Planet{source}{ante_str}"), pools::PLANETS);
        let planet = resolve::resolve_planet(name)
            .unwrap_or_else(|| panic!("pool name {name:?} has no balatro_types::Planets match"));
        Consumable::Planet(planet)
    }

    /// `functions.hpp::nextSpectral`. Note both the Soul and Black Hole
    /// checks draw from the *same* node ID (`"soul_Spectral"+ante`), called
    /// twice in a row — each call still consumes a fresh value because the
    /// node cache mutates on every access, and if both checks succeed,
    /// Black Hole wins (checked second, unconditionally overwrites).
    pub fn next_spectral(&mut self, source: &str, ante: i32, soulable: bool) -> Consumable {
        let ante_str = ante.to_string();
        if soulable {
            let mut forced: Option<Spectral> = None;
            if (self.params.showman || !self.is_locked("The Soul"))
                && self.random(&format!("soul_Spectral{ante_str}")) > 0.997
            {
                forced = Some(Spectral::Soul);
            }
            if (self.params.showman || !self.is_locked("Black Hole"))
                && self.random(&format!("soul_Spectral{ante_str}")) > 0.997
            {
                forced = Some(Spectral::BlackHole);
            }
            if let Some(s) = forced {
                return Consumable::Spectral(s);
            }
        }
        let name = self.randchoice(&format!("Spectral{source}{ante_str}"), pools::SPECTRALS);
        let spectral = resolve::resolve_spectral(name)
            .unwrap_or_else(|| panic!("pool name {name:?} has no balatro_types::Spectral match"));
        Consumable::Spectral(spectral)
    }

    /// `functions.hpp::nextJoker`, minus sticker generation (see module
    /// docs). `source` selects a forced rarity for a few callers
    /// ("sou" = legendary/Soul pull, "wra"/"rta" = rare, "uta" = uncommon),
    /// otherwise rarity is rolled.
    pub fn next_joker(&mut self, source: &str, ante: i32) -> Jokers {
        let ante_str = ante.to_string();

        let rarity: &str = match source {
            "sou" => "4",
            "wra" | "rta" => "3",
            "uta" => "2",
            _ => {
                let poll = self.random(&format!("rarity{ante_str}{source}"));
                if poll > 0.95 {
                    "3"
                } else if poll > 0.7 {
                    "2"
                } else {
                    "1"
                }
            }
        };

        let edition_rate: f64 = if self.is_voucher_active("Glow Up") {
            4.0
        } else if self.is_voucher_active("Hone") {
            2.0
        } else {
            1.0
        };
        let edition_poll = self.random(&format!("edi{source}{ante_str}"));
        let edition = if edition_poll > 0.997 {
            Edition::Negative
        } else if edition_poll > 1.0 - 0.006 * edition_rate {
            Edition::Polychrome
        } else if edition_poll > 1.0 - 0.02 * edition_rate {
            Edition::Holographic
        } else if edition_poll > 1.0 - 0.04 * edition_rate {
            Edition::Foil
        } else {
            Edition::Base
        };

        let name = match rarity {
            "4" => self.randchoice("Joker4", pools::LEGENDARY_JOKERS),
            "3" => self.randchoice(&format!("Joker3{source}{ante_str}"), pools::RARE_JOKERS),
            "2" => self.randchoice(&format!("Joker2{source}{ante_str}"), pools::UNCOMMON_JOKERS),
            _ => self.randchoice(&format!("Joker1{source}{ante_str}"), pools::COMMON_JOKERS),
        };

        let mut joker = resolved_joker(name);
        joker.set_edition(edition);
        joker
    }

    /// `functions.hpp::nextVoucher`. Deliberately doesn't lock the returned
    /// voucher itself — that's a caller decision, not this method's. Callers
    /// wanting a TheSoul-style "assume every offered voucher gets bought"
    /// preview (what `explore.rs` does, to stay diffable against the
    /// reference site) should lock it themselves right after calling this.
    /// A future save-state-aware caller should instead lock only on
    /// confirmed purchase (from real run/profile data), since an unbought
    /// voucher can legitimately reappear in a later ante in the real game —
    /// don't copy `explore.rs`'s lock-on-draw loop verbatim for that case.
    pub fn next_voucher(&mut self, ante: i32) -> Voucher {
        let name = self.randchoice(&format!("Voucher{ante}"), pools::VOUCHERS);
        resolve::resolve_voucher(name)
            .unwrap_or_else(|| panic!("pool name {name:?} has no balatro_types::Voucher match"))
    }

    /// `functions.hpp::nextTag`.
    pub fn next_tag(&mut self, ante: i32) -> Tag {
        let name = self.randchoice(&format!("Tag{ante}"), pools::TAGS);
        resolve::resolve_tag(name)
            .unwrap_or_else(|| panic!("pool name {name:?} has no balatro_types::Tag match"))
    }

    /// `functions.hpp::nextBoss`. Filters the boss pool to the current
    /// ante's category (finisher on `ante % 8 == 0`, else regular — the
    /// real game distinguishes these by whether the name starts with "The
    /// "), retrying with a full unlock if that category's pool is
    /// exhausted (every candidate already locked from a prior ante).
    pub fn next_boss(&mut self, ante: i32) -> BossBlind {
        let is_finisher_ante = ante % 8 == 0;
        let matches_category = |name: &str| {
            let is_t = name.starts_with('T');
            (is_finisher_ante && !is_t) || (!is_finisher_ante && is_t)
        };

        let pool: Vec<&str> = pools::BOSSES
            .iter()
            .copied()
            .filter(|&name| !self.is_locked(name) && matches_category(name))
            .collect();

        if pool.is_empty() {
            for &name in pools::BOSSES {
                if matches_category(name) {
                    self.unlock(name);
                }
            }
            return self.next_boss(ante);
        }

        let chosen = self.randchoice("boss", &pool);
        self.lock(chosen);
        resolve::resolve_boss(chosen)
            .unwrap_or_else(|| panic!("pool name {chosen:?} has no balatro_types::BossBlind match"))
    }

    /// `functions.hpp::nextPack`. The run's first pack (ante <= 2, only
    /// once) is always a Buffoon pack.
    pub fn next_pack(&mut self, ante: i32) -> &'static str {
        if ante <= 2 && !self.generated_first_pack && self.params.version > 10099 {
            self.generated_first_pack = true;
            return "Buffoon Pack";
        }
        self.randweightedchoice(&format!("shop_pack{ante}"), pools::PACKS)
    }

    /// `functions.hpp::nextArcanaPack`. Locks each drawn card as it's
    /// assigned so the pack can't repeat a card, then unlocks all of them
    /// once the pack is fully drawn — bypassed entirely under `showman`.
    pub fn next_arcana_pack(&mut self, size: i32, ante: i32) -> Vec<Consumable> {
        let mut pack: Vec<Consumable> = Vec::new();
        for _ in 0..size {
            let item = if self.is_voucher_active("Omen Globe") && self.random("omen_globe") > 0.8 {
                self.next_spectral("ar2", ante, true)
            } else {
                self.next_tarot("ar1", ante, true)
            };
            if !self.params.showman {
                self.lock(&item.name());
            }
            pack.push(item);
        }
        for item in &pack {
            self.unlock(&item.name());
        }
        pack
    }

    /// `functions.hpp::nextCelestialPack`.
    pub fn next_celestial_pack(&mut self, size: i32, ante: i32) -> Vec<Consumable> {
        let mut pack: Vec<Consumable> = Vec::new();
        for _ in 0..size {
            let item = self.next_planet("pl1", ante, true);
            if !self.params.showman {
                self.lock(&item.name());
            }
            pack.push(item);
        }
        for item in &pack {
            self.unlock(&item.name());
        }
        pack
    }

    /// `functions.hpp::nextSpectralPack`.
    pub fn next_spectral_pack(&mut self, size: i32, ante: i32) -> Vec<Consumable> {
        let mut pack: Vec<Consumable> = Vec::new();
        for _ in 0..size {
            let item = self.next_spectral("spe", ante, true);
            if !self.params.showman {
                self.lock(&item.name());
            }
            pack.push(item);
        }
        for item in &pack {
            self.unlock(&item.name());
        }
        pack
    }

    /// `functions.hpp::nextBuffoonPack`.
    pub fn next_buffoon_pack(&mut self, size: i32, ante: i32) -> Vec<Jokers> {
        let mut pack: Vec<Jokers> = Vec::new();
        for _ in 0..size {
            let joker = self.next_joker("buf", ante);
            if !self.params.showman {
                self.lock(joker.name());
            }
            pack.push(joker);
        }
        for joker in &pack {
            self.unlock(joker.name());
        }
        pack
    }

    /// `functions.hpp::nextStandardCard`.
    pub fn next_standard_card(&mut self, ante: i32) -> Card {
        let ante_str = ante.to_string();

        let enhancement = if self.random(&format!("stdset{ante_str}")) <= 0.6 {
            None
        } else {
            let name = self.randchoice(&format!("Enhancedsta{ante_str}"), pools::ENHANCEMENTS);
            Some(resolve::resolve_enhancement(name).unwrap_or_else(|| {
                panic!("pool name {name:?} has no balatro_types::Enhancement match")
            }))
        };

        let base = self.randchoice(&format!("frontsta{ante_str}"), pools::CARDS);
        let mut card = resolve::resolve_card_base(base)
            .unwrap_or_else(|| panic!("pool base {base:?} has no balatro_types::Card mapping"));
        card.enhancement = enhancement;

        let edition_poll = self.random(&format!("standard_edition{ante_str}"));
        card.edition = if edition_poll > 0.988 {
            Edition::Polychrome
        } else if edition_poll > 0.96 {
            Edition::Holographic
        } else if edition_poll > 0.92 {
            Edition::Foil
        } else {
            Edition::Base
        };

        card.seal = if self.random(&format!("stdseal{ante_str}")) <= 0.8 {
            None
        } else {
            let seal_poll = self.random(&format!("stdsealtype{ante_str}"));
            Some(if seal_poll > 0.75 {
                Seal::Red
            } else if seal_poll > 0.5 {
                Seal::Blue
            } else if seal_poll > 0.25 {
                Seal::Gold
            } else {
                Seal::Purple
            })
        };

        card
    }

    /// `functions.hpp::nextStandardPack`. Unlike every other pack-content
    /// method, the source never locks cards as they're drawn — duplicate
    /// cards within one Standard Pack are expected, not a bug.
    pub fn next_standard_pack(&mut self, size: i32, ante: i32) -> Vec<Card> {
        (0..size).map(|_| self.next_standard_card(ante)).collect()
    }

    /// `functions.hpp::nextShopItem` (via `getShopInstance` inlined). The
    /// `PlayingCard` branch is a placeholder — see module docs.
    pub fn next_shop_item(&mut self, ante: i32) -> ShopItem {
        let ante_str = ante.to_string();

        let joker_rate = 20.0f64;
        let mut tarot_rate = 4.0;
        let mut planet_rate = 4.0;
        let mut playing_card_rate = 0.0;
        let mut spectral_rate = 0.0;
        if self.params.deck == "Ghost Deck" {
            spectral_rate = 2.0;
        }
        if self.is_voucher_active("Tarot Tycoon") {
            tarot_rate = 32.0;
        } else if self.is_voucher_active("Tarot Merchant") {
            tarot_rate = 9.6;
        }
        if self.is_voucher_active("Planet Tycoon") {
            planet_rate = 32.0;
        } else if self.is_voucher_active("Planet Merchant") {
            planet_rate = 9.6;
        }
        if self.is_voucher_active("Magic Trick") {
            playing_card_rate = 4.0;
        }
        let total = joker_rate + tarot_rate + planet_rate + playing_card_rate + spectral_rate;

        let mut poll = self.random(&format!("cdt{ante_str}")) * total;

        if poll < joker_rate {
            return ShopItem::Joker(self.next_joker("sho", ante));
        }
        poll -= joker_rate;
        if poll < tarot_rate {
            return ShopItem::Consumable(self.next_tarot("sho", ante, false));
        }
        poll -= tarot_rate;
        if poll < planet_rate {
            return ShopItem::Consumable(self.next_planet("sho", ante, false));
        }
        poll -= planet_rate;
        if poll < playing_card_rate {
            return ShopItem::PlayingCard;
        }
        ShopItem::Consumable(self.next_spectral("sho", ante, false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Coarse smoke test: a fixed seed's ante-1 draws should be stable and
    // non-panicking across the whole draw surface. Not a correctness check
    // against real Balatro (that's the planned manual TheSoul comparison) —
    // just confirms the plumbing holds together end to end.
    #[test]
    fn ante_one_smoke_test_is_stable() {
        fn run() -> (String, String, Vec<String>) {
            let mut inst = Instance::new("TESTSEED");
            inst.init_locks(1, false, true);
            inst.init_unlocks(1, false);

            let boss = format!("{:?}", inst.next_boss(1));
            let voucher = format!("{:?}", inst.next_voucher(1));

            let mut shop_items = Vec::new();
            for _ in 0..5 {
                let item = match inst.next_shop_item(1) {
                    ShopItem::Joker(j) => format!("Joker({})", j.name()),
                    ShopItem::Consumable(c) => format!("Consumable({})", c.name()),
                    ShopItem::PlayingCard => "PlayingCard".to_string(),
                };
                shop_items.push(item);
            }

            let pack = inst.next_pack(1);
            match pack {
                "Buffoon Pack" => {
                    inst.next_buffoon_pack(2, 1);
                }
                "Arcana Pack" => {
                    inst.next_arcana_pack(3, 1);
                }
                _ => {}
            }

            (boss, voucher, shop_items)
        }

        let a = run();
        let b = run();
        assert_eq!(
            a, b,
            "same seed, same fixed draw sequence must repeat exactly"
        );
    }
}
