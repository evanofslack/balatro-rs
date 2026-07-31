use crate::card::Card;
use crate::consumable::Consumable;
use crate::deck::Deck;
use crate::joker::Jokers;
use crate::pack::{Pack, PackCategory, PackContent, PackSize};
use crate::planet::{Planetarium, Planets};
use crate::shop::{gen_random_playing_card, ConsumableGenerator, JokerGenerator, PackGenerator};
use crate::tag::Tag;
use crate::tarot::Tarot;
use balatro_seed::Instance;
use balatro_types::{Edition, Suit, Value};
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use strum::IntoEnumIterator;

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
    /// Jokers::Showman's real effect. No call site yet.
    #[allow(dead_code)]
    fn set_showman(&mut self, owned: bool);

    /// A single freshly generated joker, outside the shop/pack context —
    /// currently only Judgement. Distinct from `gen_shop_item`'s joker arm
    /// since a shop slot's category (joker vs. consumable) is its own roll.
    fn gen_joker(&mut self, ante: i32, prob_mult: u32, exclude: &[Jokers]) -> Jokers;
    fn shuffle_deck(&mut self, deck: &mut Deck);
    fn prob_roll(&mut self, numerator: u32, denominator: u32) -> bool;
    /// Returns `(small_blind_tag, big_blind_tag)` for a fresh ante.
    fn draw_ante_tags(&mut self) -> (Tag, Tag);
    /// A single random Tarot consumable, respecting `exclude` — used by
    /// the purple-seal discard trigger and the Emperor/High Priestess
    /// tarot effects.
    fn roll_random_tarot(&mut self, exclude: &[Tarot]) -> Consumable;
    /// A single random Planet consumable, respecting `exclude` — used by
    /// the blue-seal round-end trigger and the High Priestess tarot effect.
    fn roll_random_planet(&mut self, planetarium: &Planetarium, exclude: &[Planets]) -> Consumable;
    /// Castle/MailInRebate's per-round reroll (`Game::clear_blind`). Mint-time
    /// rolling for freshly generated jokers already happens inside
    /// `gen_joker`/`gen_shop_item`/`gen_pack`, each already using the right
    /// stream per backend — this is only for rerolling an *existing* joker.
    fn roll_discard_selector(&mut self, j: &mut Jokers);
    fn roll_random_edition(&mut self) -> Edition;
    fn roll_random_suit(&mut self) -> Suit;
    fn roll_random_value(&mut self) -> Value;
    fn pick_random_card(&mut self, available: Vec<Card>) -> Card;
    fn gen_random_card(
        &mut self,
        prob_mult: u32,
        force_enhance: bool,
        force_values: Option<&[Value]>,
    ) -> Card;
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

    fn gen_joker(&mut self, _ante: i32, prob_mult: u32, exclude: &[Jokers]) -> Jokers {
        self.joker_gen.gen_joker(prob_mult, exclude, &mut self.rng)
    }

    fn shuffle_deck(&mut self, deck: &mut Deck) {
        deck.shuffle(&mut self.rng);
    }

    fn prob_roll(&mut self, numerator: u32, denominator: u32) -> bool {
        self.rng.gen_ratio(numerator.min(denominator), denominator)
    }

    fn draw_ante_tags(&mut self) -> (Tag, Tag) {
        let tags: Vec<Tag> = Tag::iter().collect();
        let small = *tags.choose(&mut self.rng).unwrap();
        let big = *tags.choose(&mut self.rng).unwrap();
        (small, big)
    }

    fn roll_random_tarot(&mut self, exclude: &[Tarot]) -> Consumable {
        self.consumable_gen
            .gen_tarot_consumable(exclude, &mut self.rng)
    }

    fn roll_random_planet(&mut self, planetarium: &Planetarium, exclude: &[Planets]) -> Consumable {
        self.consumable_gen
            .gen_planet_consumable(planetarium, exclude, &mut self.rng)
    }

    fn roll_discard_selector(&mut self, j: &mut Jokers) {
        crate::joker::roll_discard_selector(&mut self.rng, j);
    }

    fn roll_random_edition(&mut self) -> Edition {
        const EDITIONS: [Edition; 3] = [Edition::Foil, Edition::Holographic, Edition::Polychrome];
        EDITIONS[self.rng.gen_range(0..EDITIONS.len())]
    }

    fn roll_random_suit(&mut self) -> Suit {
        let suits: Vec<Suit> = Suit::iter().collect();
        suits[self.rng.gen_range(0..suits.len())]
    }

    fn roll_random_value(&mut self) -> Value {
        let values: Vec<Value> = Value::iter().collect();
        values[self.rng.gen_range(0..values.len())]
    }

    fn pick_random_card(&mut self, available: Vec<Card>) -> Card {
        available[self.rng.gen_range(0..available.len())]
    }

    fn gen_random_card(
        &mut self,
        prob_mult: u32,
        force_enhance: bool,
        force_values: Option<&[Value]>,
    ) -> Card {
        gen_random_playing_card(prob_mult, &mut self.rng, force_enhance, force_values)
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub(crate) struct RealBackend {
    instance: Instance,
    // `Instance` only replays real Balatro's actual seed algorithm,
    // need a separate dedicated fast rng for some other things atm.
    extra_rng: ChaCha8Rng,
}

impl RealBackend {
    pub(crate) fn new(seed: &str) -> Self {
        RealBackend {
            instance: Instance::new(seed),
            extra_rng: ChaCha8Rng::seed_from_u64(crate::seed_from_str(seed).wrapping_add(1)),
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
                .map(|j| seed_joker_with_id(j, &mut self.extra_rng))
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

/// Joker generation needs some engine only init at create time
fn seed_joker_with_id(mut j: Jokers, rng: &mut impl Rng) -> Jokers {
    j.set_instance_id(crate::joker::mint_joker_id());
    crate::joker::roll_discard_selector(rng, &mut j);
    j
}

/// `Consumable` may carry Soul/Black Hole (a Spectral) even from a
/// nominally Tarot/Planet draw.
fn consumable_to_pack_content(c: Consumable) -> PackContent {
    match c {
        Consumable::Tarot(t) => PackContent::Tarot(t),
        Consumable::Planet(p) => PackContent::Planet(p),
        Consumable::Spectral(s) => PackContent::Spectral(s),
    }
}

impl RngBackend for RealBackend {
    // `planetarium` is unused: `Fast` mode uses it to gate secret planets
    // behind discovery state, which isn't wired into `Real` mode TODO.
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
            balatro_seed::ShopItem::Joker(j) => {
                GeneratedItem::Joker(seed_joker_with_id(j, &mut self.extra_rng))
            }
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

    // `Instance::next_joker(source, ante)` exists and isn't wired up yet,
    // real accuracy here is a follow-up. Reuses the same Fast-style
    // generator `FastBackend` uses, seeded off `extra_rng` instead.
    fn gen_joker(&mut self, _ante: i32, prob_mult: u32, exclude: &[Jokers]) -> Jokers {
        JokerGenerator::new().gen_joker(prob_mult, exclude, &mut self.extra_rng)
    }

    // No real Instance equivalent for deck draw order, yet
    fn shuffle_deck(&mut self, deck: &mut Deck) {
        deck.shuffle(&mut self.extra_rng);
    }

    // No real Instance equivalent for scoring-time probability procs
    // (Lucky/Misprint/SpaceJoker/OopsAllSixes-class effects).
    fn prob_roll(&mut self, numerator: u32, denominator: u32) -> bool {
        self.extra_rng
            .gen_ratio(numerator.min(denominator), denominator)
    }

    // `Instance::next_tag(ante)` exists and isn't wired up yet - real
    // accuracy here is a follow-up.
    fn draw_ante_tags(&mut self) -> (Tag, Tag) {
        let tags: Vec<Tag> = Tag::iter().collect();
        let small = *tags.choose(&mut self.extra_rng).unwrap();
        let big = *tags.choose(&mut self.extra_rng).unwrap();
        (small, big)
    }

    // `Instance::next_tarot(source, ante, soulable)` exists and isn't
    // wired up yet - real accuracy here is a follow-up.
    fn roll_random_tarot(&mut self, exclude: &[Tarot]) -> Consumable {
        ConsumableGenerator::new().gen_tarot_consumable(exclude, &mut self.extra_rng)
    }

    // `Instance::next_planet(source, ante, soulable)` exists and isn't
    // wired up yet - real accuracy here is a follow-up.
    fn roll_random_planet(&mut self, planetarium: &Planetarium, exclude: &[Planets]) -> Consumable {
        ConsumableGenerator::new().gen_planet_consumable(planetarium, exclude, &mut self.extra_rng)
    }

    fn roll_discard_selector(&mut self, j: &mut Jokers) {
        crate::joker::roll_discard_selector(&mut self.extra_rng, j);
    }

    fn roll_random_edition(&mut self) -> Edition {
        const EDITIONS: [Edition; 3] = [Edition::Foil, Edition::Holographic, Edition::Polychrome];
        EDITIONS[self.extra_rng.gen_range(0..EDITIONS.len())]
    }

    fn roll_random_suit(&mut self) -> Suit {
        let suits: Vec<Suit> = Suit::iter().collect();
        suits[self.extra_rng.gen_range(0..suits.len())]
    }

    fn roll_random_value(&mut self) -> Value {
        let values: Vec<Value> = Value::iter().collect();
        values[self.extra_rng.gen_range(0..values.len())]
    }

    fn pick_random_card(&mut self, available: Vec<Card>) -> Card {
        available[self.extra_rng.gen_range(0..available.len())]
    }

    fn gen_random_card(
        &mut self,
        prob_mult: u32,
        force_enhance: bool,
        force_values: Option<&[Value]>,
    ) -> Card {
        gen_random_playing_card(prob_mult, &mut self.extra_rng, force_enhance, force_values)
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

    fn gen_joker(&mut self, ante: i32, prob_mult: u32, exclude: &[Jokers]) -> Jokers {
        match self {
            Backend::Fast(b) => b.gen_joker(ante, prob_mult, exclude),
            Backend::Real(b) => b.gen_joker(ante, prob_mult, exclude),
        }
    }

    fn shuffle_deck(&mut self, deck: &mut Deck) {
        match self {
            Backend::Fast(b) => b.shuffle_deck(deck),
            Backend::Real(b) => b.shuffle_deck(deck),
        }
    }

    fn prob_roll(&mut self, numerator: u32, denominator: u32) -> bool {
        match self {
            Backend::Fast(b) => b.prob_roll(numerator, denominator),
            Backend::Real(b) => b.prob_roll(numerator, denominator),
        }
    }

    fn draw_ante_tags(&mut self) -> (Tag, Tag) {
        match self {
            Backend::Fast(b) => b.draw_ante_tags(),
            Backend::Real(b) => b.draw_ante_tags(),
        }
    }

    fn roll_random_tarot(&mut self, exclude: &[Tarot]) -> Consumable {
        match self {
            Backend::Fast(b) => b.roll_random_tarot(exclude),
            Backend::Real(b) => b.roll_random_tarot(exclude),
        }
    }

    fn roll_random_planet(&mut self, planetarium: &Planetarium, exclude: &[Planets]) -> Consumable {
        match self {
            Backend::Fast(b) => b.roll_random_planet(planetarium, exclude),
            Backend::Real(b) => b.roll_random_planet(planetarium, exclude),
        }
    }

    fn roll_discard_selector(&mut self, j: &mut Jokers) {
        match self {
            Backend::Fast(b) => b.roll_discard_selector(j),
            Backend::Real(b) => b.roll_discard_selector(j),
        }
    }
    fn roll_random_edition(&mut self) -> Edition {
        match self {
            Backend::Fast(b) => b.roll_random_edition(),
            Backend::Real(b) => b.roll_random_edition(),
        }
    }

    fn roll_random_suit(&mut self) -> Suit {
        match self {
            Backend::Fast(b) => b.roll_random_suit(),
            Backend::Real(b) => b.roll_random_suit(),
        }
    }

    fn roll_random_value(&mut self) -> Value {
        match self {
            Backend::Fast(b) => b.roll_random_value(),
            Backend::Real(b) => b.roll_random_value(),
        }
    }

    fn pick_random_card(&mut self, available: Vec<Card>) -> Card {
        match self {
            Backend::Fast(b) => b.pick_random_card(available),
            Backend::Real(b) => b.pick_random_card(available),
        }
    }

    fn gen_random_card(
        &mut self,
        prob_mult: u32,
        force_enhance: bool,
        force_values: Option<&[Value]>,
    ) -> Card {
        match self {
            Backend::Fast(b) => b.gen_random_card(prob_mult, force_enhance, force_values),
            Backend::Real(b) => b.gen_random_card(prob_mult, force_enhance, force_values),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::joker::JokerEffects;

    #[test]
    fn fast_backend_draw_ante_tags_draws_two_valid_tags() {
        let mut backend = FastBackend::new(ChaCha8Rng::seed_from_u64(1));
        let (small, big) = backend.draw_ante_tags();
        assert!(Tag::iter().any(|t| t == small));
        assert!(Tag::iter().any(|t| t == big));
    }

    #[test]
    fn fast_backend_prob_roll_respects_ratio_edges() {
        let mut backend = FastBackend::new(ChaCha8Rng::seed_from_u64(1));
        // numerator 0 -> never; numerator == denominator -> always,
        // regardless of what's drawn.
        for _ in 0..20 {
            assert!(!backend.prob_roll(0, 10));
        }
        for _ in 0..20 {
            assert!(backend.prob_roll(10, 10));
        }
        // numerator > denominator gets clamped to denominator (always true),
        // not a `gen_ratio` panic.
        assert!(backend.prob_roll(50, 10));
    }

    #[test]
    fn fast_backend_shuffle_deck_reorders_without_losing_cards() {
        let mut backend = FastBackend::new(ChaCha8Rng::seed_from_u64(1));
        let mut deck = crate::deck::Deck::default();
        let before = deck.cards();
        backend.shuffle_deck(&mut deck);
        let after = deck.cards();

        assert_eq!(before.len(), after.len());
        assert_ne!(before, after, "shuffle with a fixed seed should reorder");
        let mut sorted_before = before.clone();
        let mut sorted_after = after.clone();
        sorted_before.sort();
        sorted_after.sort();
        assert_eq!(
            sorted_before, sorted_after,
            "shuffle must permute, not lose or duplicate cards"
        );
    }

    // `RealBackend`'s new methods all fall back to `extra_rng` (no real
    // `Instance` equivalent wired up yet, see each method's doc comment) -
    // this is a smoke test that the fallback wiring is complete and doesn't
    // panic, not a claim of real-Balatro accuracy.
    #[test]
    fn real_backend_fallback_methods_all_work() {
        let mut backend = RealBackend::new("TESTSEED");

        let mut deck = crate::deck::Deck::default();
        let before = deck.cards();
        backend.shuffle_deck(&mut deck);
        assert_ne!(before, deck.cards());

        assert!(!backend.prob_roll(0, 10));
        assert!(backend.prob_roll(10, 10));

        let (small, big) = backend.draw_ante_tags();
        assert!(Tag::iter().any(|t| t == small));
        assert!(Tag::iter().any(|t| t == big));

        let planetarium = Planetarium::new();
        let _ = backend.roll_random_planet(&planetarium, &[]);
        let _ = backend.roll_random_tarot(&[]);

        let joker = backend.gen_joker(1, 1, &[]);
        assert!(joker.is_implemented());
    }

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

    // `seed_joker_with_id` is `RealBackend`'s equivalent mint chokepoint to
    // `JokerGenerator::gen_joker` (Fast mode) - Castle/MailInRebate must
    // come out with a selector already set via `RealBackend`'s own
    // `extra_rng`, not left `None`. Calls it directly with constructed
    // jokers rather than sampling real-seed shop/pack generation and hoping
    // to land on these two specific jokers among real Balatro's full
    // 150-joker pool, which isn't reliably hit in any bounded sample.
    #[test]
    fn real_backend_seed_joker_with_id_rolls_discard_selector() {
        use crate::joker::{Castle, MailInRebate};

        let mut backend = RealBackend::new("TESTSEED");
        let castle = seed_joker_with_id(Jokers::Castle(Castle::default()), &mut backend.extra_rng);
        assert!(
            castle.state().selector.is_some(),
            "Castle minted with no selector"
        );
        let mail = seed_joker_with_id(
            Jokers::MailInRebate(MailInRebate::default()),
            &mut backend.extra_rng,
        );
        assert!(
            mail.state().selector.is_some(),
            "MailInRebate minted with no selector"
        );
    }
}
