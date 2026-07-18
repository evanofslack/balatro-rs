//! Two backends for shop/pack generation, switched by `Config::rng_mode`:
//! `FastBackend` (`rand_chacha`-based) and `RealBackend` (byte-accurate
//! port of the real Balatro seed algorithm, `balatro-seed`). Only
//! shop-item and pack generation go through this — deck shuffling,
//! `prob_roll`, and the skip-blind tag draw stay on `Game.rng` directly.

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

/// What a single shop-item generation call produced. The joker/consumable
/// split has to happen inside the backend, since `Real` mode's category
/// roll is bundled into one `Instance::next_shop_item` call.
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
    /// table stays accurate. No-op for `Fast` mode.
    fn on_joker_bought(&mut self, joker: &Jokers);
    fn on_joker_sold(&mut self, joker: &Jokers);
    /// Jokers::Showman's real effect. No call site yet — see `jokers.md`.
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
        // Joker=20, Tarot=4, Planet=4.
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

/// `balatro_types::Card` has no `id`; `core::Card` adds one, so only
/// `value`/`suit` go through the id-assigning constructor.
fn seed_card_to_core_card(c: balatro_types::Card) -> Card {
    let mut card = Card::new(c.value, c.suit);
    card.edition = c.edition;
    card.enhancement = c.enhancement;
    card.seal = c.seal;
    card
}

/// `Consumable` may carry Soul/Black Hole (a Spectral) even from a
/// nominally Tarot/Planet draw; `PackContent` has a matching variant for
/// each case, so this is lossless.
fn consumable_to_pack_content(c: Consumable) -> PackContent {
    match c {
        Consumable::Tarot(t) => PackContent::Tarot(t),
        Consumable::Planet(p) => PackContent::Planet(p),
        Consumable::Spectral(s) => PackContent::Spectral(s),
    }
}

impl RngBackend for RealBackend {
    // `planetarium` is unused: `Fast` mode uses it to gate secret planets
    // behind discovery state, which isn't wired into `Real` mode (known gap).
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
                // Unreachable: needs Magic Trick active, and core has no
                // voucher-shop mechanic to ever activate it.
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
        let (category, size) = self.instance.next_pack(ante);
        let count = balatro_seed::pack_card_count(category, size);
        let contents = self.gen_pack_contents(ante, category, count);
        Pack {
            category,
            size,
            contents,
        }
    }

    fn on_joker_bought(&mut self, joker: &Jokers) {
        self.instance.lock(joker);
    }

    fn on_joker_sold(&mut self, joker: &Jokers) {
        self.instance.unlock(joker);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_backend_set_showman_propagates() {
        let mut backend = RealBackend::new("TESTSEED");
        assert!(!backend.instance.params.showman);
        backend.set_showman(true);
        assert!(backend.instance.params.showman);
        backend.set_showman(false);
        assert!(!backend.instance.params.showman);
    }

    // Structural check on gen_pack's typed plumbing across many draws.
    #[test]
    fn real_backend_gen_pack_contents_match_category_and_count() {
        let mut backend = RealBackend::new("TESTSEED");
        let planetarium = Planetarium::new();
        let mut seen_categories: std::collections::HashSet<PackCategory> =
            std::collections::HashSet::new();

        for ante in 1..=300 {
            let pack = backend.gen_pack(ante, &planetarium, 1, None, &[]);
            seen_categories.insert(pack.category);

            let expected_count = balatro_seed::pack_card_count(pack.category, pack.size);
            assert_eq!(
                pack.contents.len() as i32,
                expected_count,
                "ante {ante}: {:?}/{:?} should hold {expected_count} cards",
                pack.category,
                pack.size
            );

            for content in &pack.contents {
                let matches = match pack.category {
                    PackCategory::Buffoon => matches!(content, PackContent::Joker(_)),
                    PackCategory::Standard => matches!(content, PackContent::PlayingCard(_)),
                    PackCategory::Spectral => matches!(content, PackContent::Spectral(_)),
                    PackCategory::Arcana => {
                        matches!(content, PackContent::Tarot(_) | PackContent::Spectral(_))
                    }
                    PackCategory::Celestial => {
                        matches!(content, PackContent::Planet(_) | PackContent::Spectral(_))
                    }
                };
                assert!(
                    matches,
                    "ante {ante}: {:?} pack produced unexpected content {content:?}",
                    pack.category
                );
            }
        }

        assert_eq!(
            seen_categories.len(),
            5,
            "expected all 5 pack categories across 300 draws, saw {seen_categories:?}"
        );
    }
}
