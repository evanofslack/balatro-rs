use crate::instance::Instance;
use crate::node_id::NodeId;
use crate::pools;
use crate::resolve;
use balatro_types::{
    BossBlind, Card, Consumable, Edition, Jokers, PackCategory, PackSize, Seal, Spectral, Tag,
    Voucher,
};

pub enum ShopItem {
    Joker(Jokers),
    Consumable(Consumable),
    PlayingCard,
}

/// Given a drawn base voucher, returns its upgrade tier if one exists.
/// `VOUCHERS` is laid out as (base, upgrade) pairs at consecutive indices.
pub fn voucher_upgrade(voucher: Voucher) -> Option<Voucher> {
    let name = voucher.name();
    let idx = pools::VOUCHERS.iter().position(|&n| n == name)?;
    if idx % 2 == 0 {
        resolve::resolve_voucher(pools::VOUCHERS[idx + 1])
    } else {
        None
    }
}

/// Decodes a drawn pack name into its typed category and size.
fn parse_pack_name(name: &str) -> (PackCategory, PackSize) {
    let (category_str, size) = if let Some(rest) = name.strip_prefix("Mega ") {
        (rest, PackSize::Mega)
    } else if let Some(rest) = name.strip_prefix("Jumbo ") {
        (rest, PackSize::Jumbo)
    } else {
        (name, PackSize::Normal)
    };
    let category = match category_str {
        "Arcana Pack" => PackCategory::Arcana,
        "Celestial Pack" => PackCategory::Celestial,
        "Buffoon Pack" => PackCategory::Buffoon,
        "Standard Pack" => PackCategory::Standard,
        "Spectral Pack" => PackCategory::Spectral,
        other => panic!("pack name {other:?} has no balatro_types::PackCategory match"),
    };
    (category, size)
}

/// Card count for a pack: Buffoon/Spectral hold 2 (Normal) / 4 (else);
/// every other category holds 3 (Normal) / 5 (else).
pub fn pack_card_count(category: PackCategory, size: PackSize) -> i32 {
    match (category, size) {
        (PackCategory::Buffoon | PackCategory::Spectral, PackSize::Normal) => 2,
        (PackCategory::Buffoon | PackCategory::Spectral, _) => 4,
        (_, PackSize::Normal) => 3,
        _ => 5,
    }
}

impl Instance {
    /// `fresh_profile` gates profile-level achievement locks; `fresh_run`
    /// gates locks whose requirement is inherently in-run.
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
        if soulable
            && (self.params.showman || !self.is_locked(&Spectral::Soul))
            && self.random(NodeId::SoulTarot(ante)) > 0.997
        {
            return Consumable::Spectral(Spectral::Soul);
        }
        let tarot = self.randchoice_typed(NodeId::Tarot { source, ante }, &pools::TAROTS_POOL);
        Consumable::Tarot(tarot)
    }

    /// `functions.hpp::nextPlanet`. Can return `Black Hole` (a Spectral
    /// card) instead of a real planet.
    pub fn next_planet(&mut self, source: &str, ante: i32, soulable: bool) -> Consumable {
        if soulable
            && (self.params.showman || !self.is_locked(&Spectral::BlackHole))
            && self.random(NodeId::SoulPlanet(ante)) > 0.997
        {
            return Consumable::Spectral(Spectral::BlackHole);
        }
        let planet = self.randchoice_typed(NodeId::Planet { source, ante }, &pools::PLANETS_POOL);
        Consumable::Planet(planet)
    }

    /// `functions.hpp::nextSpectral`. Soul and Black Hole both draw from the
    /// same node ID; if both succeed, Black Hole wins (checked second).
    pub fn next_spectral(&mut self, source: &str, ante: i32, soulable: bool) -> Consumable {
        if soulable {
            let mut forced: Option<Spectral> = None;
            if (self.params.showman || !self.is_locked(&Spectral::Soul))
                && self.random(NodeId::SoulSpectral(ante)) > 0.997
            {
                forced = Some(Spectral::Soul);
            }
            if (self.params.showman || !self.is_locked(&Spectral::BlackHole))
                && self.random(NodeId::SoulSpectral(ante)) > 0.997
            {
                forced = Some(Spectral::BlackHole);
            }
            if let Some(s) = forced {
                return Consumable::Spectral(s);
            }
        }
        let spectral =
            self.randchoice_typed(NodeId::Spectral { source, ante }, &pools::SPECTRALS_POOL);
        Consumable::Spectral(spectral)
    }

    /// `source` selects a forced rarity for a few callers ("sou" =
    /// legendary, "wra"/"rta" = rare, "uta" = uncommon), else rarity is rolled.
    pub fn next_joker(&mut self, source: &str, ante: i32) -> Jokers {
        let rarity: &str = match source {
            "sou" => "4",
            "wra" | "rta" => "3",
            "uta" => "2",
            _ => {
                let poll = self.random(NodeId::Rarity { source, ante });
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
        let edition_poll = self.random(NodeId::Edition { source, ante });
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

        let mut joker: Jokers = match rarity {
            "4" => self.randchoice_typed(NodeId::Joker4, &pools::LEGENDARY_JOKERS_POOL),
            "3" => self.randchoice_typed(NodeId::Joker3 { source, ante }, &pools::RARE_JOKERS_POOL),
            "2" => self.randchoice_typed(
                NodeId::Joker2 { source, ante },
                &pools::UNCOMMON_JOKERS_POOL,
            ),
            _ => self.randchoice_typed(NodeId::Joker1 { source, ante }, &pools::COMMON_JOKERS_POOL),
        };
        joker.set_edition(edition);
        joker
    }

    /// Does not lock the result — locking on purchase is the caller's job.
    pub fn next_voucher(&mut self, ante: i32) -> Voucher {
        self.randchoice_typed(NodeId::Voucher(ante), &pools::VOUCHERS_POOL)
    }

    /// `functions.hpp::nextTag`.
    pub fn next_tag(&mut self, ante: i32) -> Tag {
        self.randchoice_typed(NodeId::Tag(ante), &pools::TAGS_POOL)
    }

    /// Filters to the current ante's category (finisher on `ante % 8 == 0`),
    /// retrying with a full unlock if that category's pool is exhausted.
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

        let chosen = self.randchoice(NodeId::Boss, &pool);
        self.lock(chosen);
        pools::BOSSES_POOL.resolve(chosen)
    }

    /// `functions.hpp::nextPack`. The run's first pack (ante <= 2, only
    /// once) is always a Buffoon pack.
    pub fn next_pack(&mut self, ante: i32) -> (PackCategory, PackSize) {
        if ante <= 2 && !self.generated_first_pack && self.params.version > 10099 {
            self.generated_first_pack = true;
            return (PackCategory::Buffoon, PackSize::Normal);
        }
        let name = self.randweightedchoice(NodeId::ShopPack(ante), pools::PACKS);
        parse_pack_name(name)
    }

    /// Locks each drawn card so the pack can't repeat one, then unlocks
    /// them all once fully drawn — bypassed under `showman`.
    pub fn next_arcana_pack(&mut self, size: i32, ante: i32) -> Vec<Consumable> {
        let mut pack: Vec<Consumable> = Vec::new();
        for _ in 0..size {
            let item =
                if self.is_voucher_active("Omen Globe") && self.random(NodeId::OmenGlobe) > 0.8 {
                    self.next_spectral("ar2", ante, true)
                } else {
                    self.next_tarot("ar1", ante, true)
                };
            if !self.params.showman {
                self.lock(&item);
            }
            pack.push(item);
        }
        for item in &pack {
            self.unlock(item);
        }
        pack
    }

    /// `functions.hpp::nextCelestialPack`.
    pub fn next_celestial_pack(&mut self, size: i32, ante: i32) -> Vec<Consumable> {
        let mut pack: Vec<Consumable> = Vec::new();
        for _ in 0..size {
            let item = self.next_planet("pl1", ante, true);
            if !self.params.showman {
                self.lock(&item);
            }
            pack.push(item);
        }
        for item in &pack {
            self.unlock(item);
        }
        pack
    }

    /// `functions.hpp::nextSpectralPack`.
    pub fn next_spectral_pack(&mut self, size: i32, ante: i32) -> Vec<Consumable> {
        let mut pack: Vec<Consumable> = Vec::new();
        for _ in 0..size {
            let item = self.next_spectral("spe", ante, true);
            if !self.params.showman {
                self.lock(&item);
            }
            pack.push(item);
        }
        for item in &pack {
            self.unlock(item);
        }
        pack
    }

    /// `functions.hpp::nextBuffoonPack`.
    pub fn next_buffoon_pack(&mut self, size: i32, ante: i32) -> Vec<Jokers> {
        let mut pack: Vec<Jokers> = Vec::new();
        for _ in 0..size {
            let joker = self.next_joker("buf", ante);
            if !self.params.showman {
                self.lock(&joker);
            }
            pack.push(joker);
        }
        for joker in &pack {
            self.unlock(joker);
        }
        pack
    }

    /// `functions.hpp::nextStandardCard`.
    pub fn next_standard_card(&mut self, ante: i32) -> Card {
        let enhancement = if self.random(NodeId::StdSet(ante)) <= 0.6 {
            None
        } else {
            Some(self.randchoice_typed(NodeId::EnhancedStandard(ante), &pools::ENHANCEMENTS_POOL))
        };

        let mut card: Card = self.randchoice_typed(NodeId::FrontStandard(ante), &pools::CARDS_POOL);
        card.enhancement = enhancement;

        let edition_poll = self.random(NodeId::StandardEdition(ante));
        card.edition = if edition_poll > 0.988 {
            Edition::Polychrome
        } else if edition_poll > 0.96 {
            Edition::Holographic
        } else if edition_poll > 0.92 {
            Edition::Foil
        } else {
            Edition::Base
        };

        card.seal = if self.random(NodeId::StdSeal(ante)) <= 0.8 {
            None
        } else {
            let seal_poll = self.random(NodeId::StdSealType(ante));
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

    /// Unlike every other pack-content method, never locks drawn cards —
    /// duplicates within one Standard Pack are expected.
    pub fn next_standard_pack(&mut self, size: i32, ante: i32) -> Vec<Card> {
        (0..size).map(|_| self.next_standard_card(ante)).collect()
    }

    /// `functions.hpp::nextShopItem` (via `getShopInstance` inlined). The
    /// `PlayingCard` branch is a placeholder — see module docs.
    pub fn next_shop_item(&mut self, ante: i32) -> ShopItem {
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

        let mut poll = self.random(NodeId::Cdt(ante)) * total;

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

    // Smoke test: confirms the whole draw surface is stable and non-panicking.
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

            let (category, size) = inst.next_pack(1);
            match category {
                PackCategory::Buffoon => {
                    inst.next_buffoon_pack(pack_card_count(category, size), 1);
                }
                PackCategory::Arcana => {
                    inst.next_arcana_pack(pack_card_count(category, size), 1);
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
