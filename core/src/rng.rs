//! Two backends for shop/pack generation, switched by `Config::rng_mode`:
//! `FastBackend` (`rand_chacha`-based) and `RealBackend` (byte-accurate
//! port of the real Balatro seed algorithm, `balatro-seed`). Only
//! shop-item and pack generation go through this — deck shuffling,
//! `prob_roll`, and the skip-blind tag draw stay on `Game.rng` directly.

use crate::card::Card;
use crate::consumable::Consumable;
use crate::joker::Jokers;
use crate::pack::{Pack, PackCategory, PackContent, PackSize};
use crate::planet::Planets;
use crate::shop::{
    gen_shop_playing_card, ConsumableGenerator, JokerGenerator, PackGenerator, ShopContext,
};
use crate::tarot::Tarot;
use crate::voucher::{Voucher, Vouchers};
use balatro_seed::Instance;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

/// What a single shop-item generation call produced. The category split
/// has to happen inside the backend, since `Real` mode's category roll is
/// bundled into one `Instance::next_shop_item` call.
pub(crate) enum GeneratedItem {
    Joker(Jokers),
    Consumable(Consumable),
    /// Only ever produced with the Magic Trick voucher redeemed.
    PlayingCard(Card),
}

pub(crate) trait RngBackend {
    fn gen_shop_item(
        &mut self,
        ctx: &ShopContext,
        exclude_jokers: &[Jokers],
        exclude_tarots: &[Tarot],
        exclude_planets: &[Planets],
    ) -> GeneratedItem;

    fn gen_pack(&mut self, ctx: &ShopContext, exclude: Option<(&PackCategory, &PackSize)>) -> Pack;

    /// This ante's voucher offer, drawn from the vouchers still
    /// offerable (see `Vouchers::offerable`). `None` once all 32 are
    /// redeemed.
    fn gen_voucher(&mut self, ante: i32, vouchers: &Vouchers) -> Option<Voucher>;

    /// Owned-joker dedup hook: called on buy/sell so `Real` mode's lock
    /// table stays accurate. No-op for `Fast` mode.
    fn on_joker_bought(&mut self, joker: &Jokers);
    fn on_joker_sold(&mut self, joker: &Jokers);
    /// Redeemed-voucher hook: `Real` mode's draw rates read straight off
    /// its own active-voucher list, so it has to be told. No-op for
    /// `Fast` mode, which reads `ShopContext::vouchers` per call instead.
    fn on_voucher_bought(&mut self, voucher: &Voucher);
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
        ctx: &ShopContext,
        exclude_jokers: &[Jokers],
        exclude_tarots: &[Tarot],
        exclude_planets: &[Planets],
    ) -> GeneratedItem {
        // Joker=20, Tarot/Planet=4 each before the Merchant/Tycoon
        // vouchers raise them; playing cards weigh 0 until Magic Trick.
        let weights = [
            20.0,
            ctx.vouchers.tarot_rate(),
            ctx.vouchers.planet_rate(),
            ctx.vouchers.playing_card_rate(),
        ];
        let dist = WeightedIndex::new(weights).expect("shop item weights are finite and nonzero");
        match dist.sample(&mut self.rng) {
            0 => GeneratedItem::Joker(self.joker_gen.gen_joker(
                ctx.edition_prob_mult(),
                exclude_jokers,
                &mut self.rng,
            )),
            1 => GeneratedItem::Consumable(
                self.consumable_gen
                    .gen_tarot_consumable(exclude_tarots, &mut self.rng),
            ),
            2 => GeneratedItem::Consumable(self.consumable_gen.gen_planet_consumable(
                ctx.planetarium,
                exclude_planets,
                &mut self.rng,
            )),
            _ => GeneratedItem::PlayingCard(gen_shop_playing_card(
                ctx.edition_prob_mult(),
                ctx.vouchers.shop_cards_are_modified(),
                &mut self.rng,
            )),
        }
    }

    fn gen_pack(&mut self, ctx: &ShopContext, exclude: Option<(&PackCategory, &PackSize)>) -> Pack {
        self.pack_gen.gen_pack(
            ctx.planetarium,
            ctx.edition_prob_mult(),
            exclude,
            ctx.held_jokers,
            &mut self.rng,
        )
    }

    fn gen_voucher(&mut self, _ante: i32, vouchers: &Vouchers) -> Option<Voucher> {
        let offerable = vouchers.offerable();
        if offerable.is_empty() {
            return None;
        }
        Some(offerable[self.rng.gen_range(0..offerable.len())])
    }

    fn on_joker_bought(&mut self, _joker: &Jokers) {}
    fn on_joker_sold(&mut self, _joker: &Jokers) {}
    fn on_voucher_bought(&mut self, _voucher: &Voucher) {}
    fn set_showman(&mut self, _owned: bool) {}
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub(crate) struct RealBackend {
    instance: Instance,
    /// Side channel for shop playing cards only. `balatro-seed`'s
    /// `ShopItem::PlayingCard` is a marker that carries no card data, and
    /// drawing one out of the `Instance` would advance nodes the real
    /// game doesn't, desyncing every later draw. Seeded from the same
    /// seed, so runs stay reproducible.
    card_rng: ChaCha8Rng,
}

impl RealBackend {
    pub(crate) fn new(seed: &str) -> Self {
        RealBackend {
            instance: Instance::new(seed),
            card_rng: ChaCha8Rng::seed_from_u64(crate::seed_from_str(seed)),
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
                .map(seed_joker_with_id)
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

/// Joker generation doesn't mints a real instance id (it has
/// no notion of instance identity), assign one here.
/// (same translation `seed_card_to_core_card` does for cards)
fn seed_joker_with_id(mut j: Jokers) -> Jokers {
    j.set_instance_id(crate::joker::mint_joker_id());
    j
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
    // `ctx.planetarium` is unused: `Fast` mode uses it to gate secret
    // planets behind discovery state, which isn't wired into `Real` mode
    // TODO. Voucher-driven rates come from the `Instance`'s own active
    // list (kept current by `on_voucher_bought`), not `ctx.vouchers`.
    fn gen_shop_item(
        &mut self,
        ctx: &ShopContext,
        _exclude_jokers: &[Jokers],
        _exclude_tarots: &[Tarot],
        _exclude_planets: &[Planets],
    ) -> GeneratedItem {
        match self.instance.next_shop_item(ctx.ante) {
            balatro_seed::ShopItem::Joker(j) => GeneratedItem::Joker(seed_joker_with_id(j)),
            balatro_seed::ShopItem::Consumable(c) => GeneratedItem::Consumable(c),
            balatro_seed::ShopItem::PlayingCard => {
                GeneratedItem::PlayingCard(gen_shop_playing_card(
                    1,
                    ctx.vouchers.shop_cards_are_modified(),
                    &mut self.card_rng,
                ))
            }
        }
    }

    fn gen_pack(
        &mut self,
        ctx: &ShopContext,
        _exclude: Option<(&PackCategory, &PackSize)>,
    ) -> Pack {
        let (category, size) = self.instance.next_pack(ctx.ante);
        let count = balatro_seed::pack_card_count(category, size);
        let contents = self.gen_pack_contents(ctx.ante, category, count);
        Pack {
            category,
            size,
            contents,
        }
    }

    // Unlike `Fast` mode this draws from the full pool and then filters:
    // the draw has to consume the same node the real game would, whether
    // or not the result is still offerable. A base voucher already owned
    // is upgraded in place (`voucher_upgrade`), matching the real game.
    fn gen_voucher(&mut self, ante: i32, vouchers: &Vouchers) -> Option<Voucher> {
        let drawn = self.instance.next_voucher(ante);
        let offer = if vouchers.has(drawn) {
            balatro_seed::voucher_upgrade(drawn).filter(|u| vouchers.is_offerable(*u))
        } else {
            Some(drawn)
        };
        if let Some(v) = offer {
            self.instance.lock(&v);
        }
        offer
    }

    fn on_joker_bought(&mut self, joker: &Jokers) {
        self.instance.lock(joker);
    }

    fn on_joker_sold(&mut self, joker: &Jokers) {
        self.instance.unlock(joker);
    }

    fn on_voucher_bought(&mut self, voucher: &Voucher) {
        self.instance.activate_voucher(voucher);
    }

    fn set_showman(&mut self, owned: bool) {
        self.instance.params.showman = owned;
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
// Both variants are inherently chunky — each owns an rng, and `Real` also
// owns the seed `Instance`'s node and lock tables. Exactly one exists per
// `Game`, so boxing would only trade a few hundred bytes for an
// indirection on every draw.
#[allow(clippy::large_enum_variant)]
pub(crate) enum Backend {
    Fast(FastBackend),
    Real(RealBackend),
}

impl RngBackend for Backend {
    fn gen_shop_item(
        &mut self,
        ctx: &ShopContext,
        exclude_jokers: &[Jokers],
        exclude_tarots: &[Tarot],
        exclude_planets: &[Planets],
    ) -> GeneratedItem {
        match self {
            Backend::Fast(b) => {
                b.gen_shop_item(ctx, exclude_jokers, exclude_tarots, exclude_planets)
            }
            Backend::Real(b) => {
                b.gen_shop_item(ctx, exclude_jokers, exclude_tarots, exclude_planets)
            }
        }
    }

    fn gen_pack(&mut self, ctx: &ShopContext, exclude: Option<(&PackCategory, &PackSize)>) -> Pack {
        match self {
            Backend::Fast(b) => b.gen_pack(ctx, exclude),
            Backend::Real(b) => b.gen_pack(ctx, exclude),
        }
    }

    fn gen_voucher(&mut self, ante: i32, vouchers: &Vouchers) -> Option<Voucher> {
        match self {
            Backend::Fast(b) => b.gen_voucher(ante, vouchers),
            Backend::Real(b) => b.gen_voucher(ante, vouchers),
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

    fn on_voucher_bought(&mut self, voucher: &Voucher) {
        match self {
            Backend::Fast(b) => b.on_voucher_bought(voucher),
            Backend::Real(b) => b.on_voucher_bought(voucher),
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
        let planetarium = crate::planet::Planetarium::new();
        let vouchers = Vouchers::new();
        let mut seen_categories: std::collections::HashSet<PackCategory> =
            std::collections::HashSet::new();

        for ante in 1..=300 {
            let ctx = ShopContext::for_test(&planetarium, &vouchers, ante);
            let pack = backend.gen_pack(&ctx, None);
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
