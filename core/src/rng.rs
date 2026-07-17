//! Two backends for shop/pack generation, switched by `Config::rng_mode`:
//! `FastBackend` (today's `rand_chacha`-based generation, unchanged) and
//! `RealBackend` (a byte-accurate port of the real Balatro seed algorithm,
//! `balatro-seed`). Named `rng` rather than `generator` to avoid colliding
//! with the existing `generator.rs` (unrelated legal-move/action-space
//! generation) and `shop.rs`'s `JokerGenerator`/`ConsumableGenerator`/
//! `PackGenerator` struct names.
//!
//! Only shop-item and pack generation go through this — deck shuffling,
//! `prob_roll` (Lucky/Glass), and the skip-blind tag draw keep using
//! `Game.rng` directly, untouched by `RngMode`.

use crate::card::Card;
use crate::consumable::Consumable;
use crate::joker::Jokers;
use crate::pack::{Pack, PackCategory, PackContent, PackSize};
use crate::planet::{Planetarium, Planets};
use crate::shop::{ConsumableGenerator, JokerGenerator, PackGenerator};
use crate::tarot::Tarot;
use balatro_seed::Instance;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

/// What a single shop-item generation call produced. Not a bare `Jokers`/
/// `Consumable` split at the call site, because that split (joker vs.
/// tarot vs. planet vs. ...) has to happen *inside* the backend: `Real`
/// mode's category roll is bundled into one `Instance::next_shop_item`
/// call, not separable into "pick a category, then call a type-specific
/// generator" the way `Fast` mode's `WeightedIndex` roll is.
pub(crate) enum GeneratedItem {
    Joker(Jokers),
    Consumable(Consumable),
}

pub(crate) trait RngBackend {
    fn gen_shop_item(
        &mut self,
        ante: i32,
        planetarium: &Planetarium,
        prob_mult: u32,
        exclude_jokers: &[Jokers],
        exclude_tarots: &[Tarot],
        exclude_planets: &[Planets],
    ) -> GeneratedItem;

    fn gen_pack(
        &mut self,
        ante: i32,
        planetarium: &Planetarium,
        prob_mult: u32,
        exclude: Option<(&PackCategory, &PackSize)>,
        held_jokers: &[Jokers],
    ) -> Pack;

    /// Owned-joker dedup hook: called on buy/sell so `Real` mode's lock
    /// table stays accurate. No-op for `Fast` mode, whose `exclude_*`/
    /// `held_jokers` parameters above already carry this fresh each call.
    fn on_joker_bought(&mut self, joker: &Jokers);
    fn on_joker_sold(&mut self, joker: &Jokers);
    /// Jokers::Showman's real effect: disables duplicate-avoidance
    /// entirely. No-op for `Fast` mode until it grows an equivalent flag.
    /// No call site yet — wiring this up is `Jokers::Showman` getting real
    /// `effects()` logic (tracked in `jokers.md`), not part of this phase.
    #[allow(dead_code)]
    fn set_showman(&mut self, owned: bool);
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub(crate) struct FastBackend {
    rng: ChaCha8Rng,
    joker_gen: JokerGenerator,
    consumable_gen: ConsumableGenerator,
    pack_gen: PackGenerator,
}

impl FastBackend {
    pub(crate) fn new(rng: ChaCha8Rng) -> Self {
        FastBackend {
            rng,
            joker_gen: JokerGenerator::new(),
            consumable_gen: ConsumableGenerator::new(),
            pack_gen: PackGenerator {},
        }
    }
}

impl RngBackend for FastBackend {
    fn gen_shop_item(
        &mut self,
        _ante: i32,
        planetarium: &Planetarium,
        prob_mult: u32,
        exclude_jokers: &[Jokers],
        exclude_tarots: &[Tarot],
        exclude_planets: &[Planets],
    ) -> GeneratedItem {
        // Joker=20, Tarot=4, Planet=4 — same weights as the old inline
        // roll in `Shop::refresh_cards`, just relocated here.
        let weights = [20u32, 4, 4];
        let dist = WeightedIndex::new(weights).unwrap();
        match dist.sample(&mut self.rng) {
            0 => GeneratedItem::Joker(self.joker_gen.gen_joker(
                prob_mult,
                exclude_jokers,
                &mut self.rng,
            )),
            1 => GeneratedItem::Consumable(
                self.consumable_gen
                    .gen_tarot_consumable(exclude_tarots, &mut self.rng),
            ),
            _ => GeneratedItem::Consumable(self.consumable_gen.gen_planet_consumable(
                planetarium,
                exclude_planets,
                &mut self.rng,
            )),
        }
    }

    fn gen_pack(
        &mut self,
        _ante: i32,
        planetarium: &Planetarium,
        prob_mult: u32,
        exclude: Option<(&PackCategory, &PackSize)>,
        held_jokers: &[Jokers],
    ) -> Pack {
        self.pack_gen
            .gen_pack(planetarium, prob_mult, exclude, held_jokers, &mut self.rng)
    }

    fn on_joker_bought(&mut self, _joker: &Jokers) {}
    fn on_joker_sold(&mut self, _joker: &Jokers) {}
    fn set_showman(&mut self, _owned: bool) {}
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub(crate) struct RealBackend {
    instance: Instance,
}

impl RealBackend {
    pub(crate) fn new(seed: &str) -> Self {
        RealBackend {
            instance: Instance::new(seed),
        }
    }

    fn map_category(name: &str) -> PackCategory {
        match name {
            "Arcana Pack" => PackCategory::Arcana,
            "Celestial Pack" => PackCategory::Celestial,
            "Buffoon Pack" => PackCategory::Buffoon,
            "Standard Pack" => PackCategory::Standard,
            "Spectral Pack" => PackCategory::Spectral,
            other => panic!("balatro-seed pack category {other:?} has no PackCategory mapping"),
        }
    }

    fn gen_pack_contents(
        &mut self,
        ante: i32,
        category: PackCategory,
        count: i32,
    ) -> Vec<PackContent> {
        match category {
            PackCategory::Arcana => self
                .instance
                .next_arcana_pack(count, ante)
                .into_iter()
                .map(consumable_to_pack_content)
                .collect(),
            PackCategory::Celestial => self
                .instance
                .next_celestial_pack(count, ante)
                .into_iter()
                .map(consumable_to_pack_content)
                .collect(),
            PackCategory::Spectral => self
                .instance
                .next_spectral_pack(count, ante)
                .into_iter()
                .map(consumable_to_pack_content)
                .collect(),
            PackCategory::Buffoon => self
                .instance
                .next_buffoon_pack(count, ante)
                .into_iter()
                .map(PackContent::Joker)
                .collect(),
            PackCategory::Standard => self
                .instance
                .next_standard_pack(count, ante)
                .into_iter()
                .map(|c| PackContent::PlayingCard(seed_card_to_core_card(c)))
                .collect(),
        }
    }
}

/// `balatro_seed`'s `Card` (`balatro_types::Card`) has no `id` — `core`'s
/// own `Card` (`crate::card::Card`) adds one (auto-assigned, for
/// dedup/selection tracking) but reuses the same `Value`/`Suit`/`Edition`/
/// `Enhancement`/`Seal` types directly, so only `value`/`suit` go through
/// the id-assigning constructor and the rest are plain field copies.
fn seed_card_to_core_card(c: balatro_types::Card) -> Card {
    let mut card = Card::new(c.value, c.suit);
    card.edition = c.edition;
    card.enhancement = c.enhancement;
    card.seal = c.seal;
    card
}

/// `balatro_seed`'s `Consumable` may carry Soul/Black Hole (a Spectral)
/// even from a nominally Tarot/Planet draw — `PackContent` already has a
/// matching variant for each `Consumable` case, so this is a direct,
/// lossless translation, not a lossy one.
fn consumable_to_pack_content(c: Consumable) -> PackContent {
    match c {
        Consumable::Tarot(t) => PackContent::Tarot(t),
        Consumable::Planet(p) => PackContent::Planet(p),
        Consumable::Spectral(s) => PackContent::Spectral(s),
    }
}

impl RngBackend for RealBackend {
    // NOTE: `planetarium` is accepted but unused here — `Fast` mode uses it
    // to gate "secret" planets (Planet X/Ceres/Eris) behind their hand type
    // having been discovered (`ConsumableGenerator::gen_planet`); this
    // gating isn't wired into `Real` mode. `balatro-seed`'s lock table
    // *could* carry the same rule (lock/unlock "Planet X" as discovery
    // state changes, exactly like the joker-ownership hooks below), but
    // that's additional wiring this phase didn't include — flagged as a
    // known gap, same category as the `freshProfile`/`freshRun` locks
    // already noted as out of scope in `ARCHITECTURE.md`.
    fn gen_shop_item(
        &mut self,
        ante: i32,
        _planetarium: &Planetarium,
        _prob_mult: u32,
        _exclude_jokers: &[Jokers],
        _exclude_tarots: &[Tarot],
        _exclude_planets: &[Planets],
    ) -> GeneratedItem {
        match self.instance.next_shop_item(ante) {
            balatro_seed::ShopItem::Joker(j) => GeneratedItem::Joker(j),
            balatro_seed::ShopItem::Consumable(c) => GeneratedItem::Consumable(c),
            balatro_seed::ShopItem::PlayingCard => {
                // Unreachable given core's current feature set: this only
                // has nonzero odds when the Magic Trick voucher is active,
                // and nothing in `core` can ever activate a voucher (no
                // voucher-shop mechanic exists yet — see
                // ARCHITECTURE.md's "Open / deferred decisions"). Panic
                // rather than silently drop the slot, so this gets noticed
                // immediately if that ever changes.
                panic!(
                    "balatro-seed produced a shop playing card; core has no \
                     voucher mechanic to ever enable Magic Trick's nonzero rate"
                )
            }
        }
    }

    fn gen_pack(
        &mut self,
        ante: i32,
        _planetarium: &Planetarium,
        _prob_mult: u32,
        _exclude: Option<(&PackCategory, &PackSize)>,
        _held_jokers: &[Jokers],
    ) -> Pack {
        let name = self.instance.next_pack(ante);
        let size = if name.starts_with("Jumbo ") {
            PackSize::Jumbo
        } else if name.starts_with("Mega ") {
            PackSize::Mega
        } else {
            PackSize::Normal
        };
        let (category_name, count) = balatro_seed::pack_info(name);
        let category = Self::map_category(category_name);
        let contents = self.gen_pack_contents(ante, category, count);
        Pack {
            category,
            size,
            contents,
        }
    }

    fn on_joker_bought(&mut self, joker: &Jokers) {
        self.instance.lock(joker.name());
    }

    fn on_joker_sold(&mut self, joker: &Jokers) {
        self.instance.unlock(joker.name());
    }

    fn set_showman(&mut self, owned: bool) {
        self.instance.params.showman = owned;
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub(crate) enum Backend {
    Fast(FastBackend),
    Real(RealBackend),
}

impl RngBackend for Backend {
    fn gen_shop_item(
        &mut self,
        ante: i32,
        planetarium: &Planetarium,
        prob_mult: u32,
        exclude_jokers: &[Jokers],
        exclude_tarots: &[Tarot],
        exclude_planets: &[Planets],
    ) -> GeneratedItem {
        match self {
            Backend::Fast(b) => b.gen_shop_item(
                ante,
                planetarium,
                prob_mult,
                exclude_jokers,
                exclude_tarots,
                exclude_planets,
            ),
            Backend::Real(b) => b.gen_shop_item(
                ante,
                planetarium,
                prob_mult,
                exclude_jokers,
                exclude_tarots,
                exclude_planets,
            ),
        }
    }

    fn gen_pack(
        &mut self,
        ante: i32,
        planetarium: &Planetarium,
        prob_mult: u32,
        exclude: Option<(&PackCategory, &PackSize)>,
        held_jokers: &[Jokers],
    ) -> Pack {
        match self {
            Backend::Fast(b) => b.gen_pack(ante, planetarium, prob_mult, exclude, held_jokers),
            Backend::Real(b) => b.gen_pack(ante, planetarium, prob_mult, exclude, held_jokers),
        }
    }

    fn on_joker_bought(&mut self, joker: &Jokers) {
        match self {
            Backend::Fast(b) => b.on_joker_bought(joker),
            Backend::Real(b) => b.on_joker_bought(joker),
        }
    }

    fn on_joker_sold(&mut self, joker: &Jokers) {
        match self {
            Backend::Fast(b) => b.on_joker_sold(joker),
            Backend::Real(b) => b.on_joker_sold(joker),
        }
    }

    fn set_showman(&mut self, owned: bool) {
        match self {
            Backend::Fast(b) => b.set_showman(owned),
            Backend::Real(b) => b.set_showman(owned),
        }
    }
}
