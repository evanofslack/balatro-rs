use crate::action::Action;
use crate::card::{Card, Edition, Enhancement, Seal, Suit, Value};
use crate::consumable::Consumable;
use crate::error::GameError;
use crate::joker::{jokers_by_rarity, Jokers, Rarity};
use crate::pack::{Pack, PackCategory, PackContent, PackSize};
use crate::planet::{Planetarium, Planets};
use crate::rank::HandRank;
use crate::rng::{Backend, GeneratedItem, RngBackend};
use crate::tarot::Tarot;
use crate::voucher::{discounted, Voucher, Vouchers};
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use strum::IntoEnumIterator;

/// Everything shop generation reads off the `Game`. Bundled into one
/// struct because vouchers made the parameter list unreadable — every
/// field here was already being threaded through positionally.
pub(crate) struct ShopContext<'a> {
    pub planetarium: &'a Planetarium,
    /// Consumables the player already holds — excluded from the roll so
    /// the shop doesn't offer a duplicate.
    pub held_consumables: &'a [Consumable],
    pub held_jokers: &'a [Jokers],
    pub vouchers: &'a Vouchers,
    pub prob_mult: u32,
    pub ante: i32,
    /// Which hand Telescope should guarantee a Planet card for.
    pub most_played: HandRank,
}

impl ShopContext<'_> {
    /// Card slots the shop fills, before packs: 2 by default, more with
    /// Overstock.
    pub(crate) fn card_slots(&self) -> usize {
        DEFAULT_SHOP_CARD_SLOTS + self.vouchers.shop_card_slots_bonus()
    }

    /// Edition probabilities are scaled by both the game-wide probability
    /// multiplier and Hone/Glow Up.
    pub(crate) fn edition_prob_mult(&self) -> u32 {
        self.prob_mult
            .saturating_mul(self.vouchers.edition_rate_mult())
    }
}

#[cfg(test)]
impl<'a> ShopContext<'a> {
    /// Bare context for tests: nothing held, no vouchers redeemed.
    pub(crate) fn for_test(
        planetarium: &'a Planetarium,
        vouchers: &'a Vouchers,
        ante: i32,
    ) -> Self {
        ShopContext {
            planetarium,
            held_consumables: &[],
            held_jokers: &[],
            vouchers,
            prob_mult: 1,
            ante,
            most_played: HandRank::HighCard,
        }
    }
}

const DEFAULT_SHOP_CARD_SLOTS: usize = 2;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct Shop {
    pub jokers: Vec<Jokers>,
    pub consumables: Vec<Consumable>,
    /// Playing cards for sale. Always empty until Magic Trick is redeemed.
    #[cfg_attr(feature = "serde", serde(default))]
    pub cards: Vec<Card>,
    pub packs: Vec<Pack>,
    /// This ante's voucher offer, if it hasn't been bought yet. Unlike
    /// everything else in the shop it survives rerolls and persists
    /// across every shop visit of the ante.
    #[cfg_attr(feature = "serde", serde(default))]
    pub voucher: Option<Voucher>,
}

impl Shop {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Shop {
    pub(crate) fn refresh_cards(&mut self, ctx: &ShopContext, backend: &mut Backend) {
        self.jokers.clear();
        self.consumables.clear();
        self.cards.clear();

        let mut excl_jokers: Vec<Jokers> = ctx.held_jokers.to_vec();
        let mut excl_tarots: Vec<Tarot> = ctx
            .held_consumables
            .iter()
            .filter_map(|c| {
                if let Consumable::Tarot(t) = c {
                    Some(*t)
                } else {
                    None
                }
            })
            .collect();
        let mut excl_planets: Vec<Planets> = ctx
            .held_consumables
            .iter()
            .filter_map(|c| {
                if let Consumable::Planet(p) = c {
                    Some(*p)
                } else {
                    None
                }
            })
            .collect();

        for _ in 0..ctx.card_slots() {
            match backend.gen_shop_item(ctx, &excl_jokers, &excl_tarots, &excl_planets) {
                GeneratedItem::Joker(joker) => {
                    excl_jokers.push(joker.clone());
                    self.jokers.push(joker);
                }
                GeneratedItem::Consumable(c) => {
                    match &c {
                        Consumable::Tarot(t) => excl_tarots.push(*t),
                        Consumable::Planet(p) => excl_planets.push(*p),
                        Consumable::Spectral(_) => {}
                    }
                    self.consumables.push(c);
                }
                GeneratedItem::PlayingCard(card) => self.cards.push(card),
            }
        }
    }

    /// Fills any card slots the shop has gained since it was stocked,
    /// leaving what's already on the shelf alone. Redeeming Overstock
    /// mid-shop is the only thing that widens a live shop.
    pub(crate) fn top_up_cards(&mut self, ctx: &ShopContext, backend: &mut Backend) {
        let filled = self.jokers.len() + self.consumables.len() + self.cards.len();
        let mut excl_jokers: Vec<Jokers> = ctx.held_jokers.to_vec();
        excl_jokers.extend(self.jokers.clone());
        let mut excl_tarots: Vec<Tarot> = Vec::new();
        let mut excl_planets: Vec<Planets> = Vec::new();
        for c in ctx.held_consumables.iter().chain(self.consumables.iter()) {
            match c {
                Consumable::Tarot(t) => excl_tarots.push(*t),
                Consumable::Planet(p) => excl_planets.push(*p),
                Consumable::Spectral(_) => {}
            }
        }

        for _ in filled..ctx.card_slots() {
            match backend.gen_shop_item(ctx, &excl_jokers, &excl_tarots, &excl_planets) {
                GeneratedItem::Joker(joker) => {
                    excl_jokers.push(joker.clone());
                    self.jokers.push(joker);
                }
                GeneratedItem::Consumable(c) => {
                    match &c {
                        Consumable::Tarot(t) => excl_tarots.push(*t),
                        Consumable::Planet(p) => excl_planets.push(*p),
                        Consumable::Spectral(_) => {}
                    }
                    self.consumables.push(c);
                }
                GeneratedItem::PlayingCard(card) => self.cards.push(card),
            }
        }
    }

    pub(crate) fn refresh(&mut self, ctx: &ShopContext, backend: &mut Backend) {
        self.refresh_cards(ctx, backend);

        let p1 = backend.gen_pack(ctx, None);
        let exclude = (&p1.category, &p1.size);
        let p2 = backend.gen_pack(ctx, Some(exclude));
        self.packs = vec![p1, p2];
        for pack in &mut self.packs {
            apply_telescope(pack, ctx);
        }
    }

    pub(crate) fn joker_from_index(&self, i: usize) -> Option<Jokers> {
        Some(self.jokers[i].clone())
    }

    pub(crate) fn consumable_from_index(&self, i: usize) -> Option<Consumable> {
        self.consumables.get(i).cloned()
    }

    pub(crate) fn pack_from_index(&self, i: usize) -> Option<Pack> {
        self.packs.get(i).cloned()
    }

    pub(crate) fn card_from_index(&self, i: usize) -> Option<Card> {
        self.cards.get(i).copied()
    }

    pub(crate) fn buy_joker(&mut self, joker: &Jokers) -> Result<Jokers, GameError> {
        let i = self
            .jokers
            .iter()
            .position(|j| j == joker)
            .ok_or(GameError::NoJokerMatch)?;
        Ok(self.jokers.remove(i))
    }

    pub(crate) fn buy_consumable(
        &mut self,
        consumable: &Consumable,
    ) -> Result<Consumable, GameError> {
        let i = self
            .consumables
            .iter()
            .position(|c| c == consumable)
            .ok_or(GameError::NoConsumableMatch)?;
        Ok(self.consumables.remove(i))
    }

    pub(crate) fn buy_pack(&mut self, pack: &Pack) -> Result<Pack, GameError> {
        let i = self
            .packs
            .iter()
            .position(|p| p == pack)
            .ok_or(GameError::InvalidAction)?;
        Ok(self.packs.remove(i))
    }

    pub(crate) fn buy_card(&mut self, card: &Card) -> Result<Card, GameError> {
        let i = self
            .cards
            .iter()
            .position(|c| c.id == card.id)
            .ok_or(GameError::NoCardMatch)?;
        Ok(self.cards.remove(i))
    }

    /// Takes the offered voucher off the shelf. Unlike the other buys it
    /// clears the slot rather than removing from a list — there is only
    /// ever one voucher on offer per ante.
    pub(crate) fn buy_voucher(&mut self, voucher: &Voucher) -> Result<Voucher, GameError> {
        match self.voucher {
            Some(v) if v == *voucher => {
                self.voucher = None;
                Ok(v)
            }
            _ => Err(GameError::NoVoucherMatch),
        }
    }

    pub(crate) fn gen_moves_buy_joker(
        &self,
        balance: usize,
        vouchers: &Vouchers,
    ) -> Option<impl Iterator<Item = Action>> {
        if self.jokers.is_empty() {
            return None;
        }
        let discount = vouchers.discount_pct();
        let buys = self
            .jokers
            .clone()
            .into_iter()
            .filter(move |j| discounted(j.cost(), discount) <= balance)
            .map(Action::BuyJoker);
        Some(buys)
    }

    pub(crate) fn gen_moves_buy_consumable(
        &self,
        balance: usize,
        consumable_slots: usize,
        held: usize,
        vouchers: &Vouchers,
    ) -> Option<impl Iterator<Item = Action>> {
        if self.consumables.is_empty() || held >= consumable_slots {
            return None;
        }
        let discount = vouchers.discount_pct();
        let buys = self
            .consumables
            .clone()
            .into_iter()
            .filter(move |c| discounted(c.cost(), discount) <= balance)
            .map(Action::BuyConsumable);
        Some(buys)
    }

    pub(crate) fn gen_moves_buy_pack(
        &self,
        balance: usize,
        vouchers: &Vouchers,
    ) -> Option<impl Iterator<Item = Action>> {
        if self.packs.is_empty() {
            return None;
        }
        let discount = vouchers.discount_pct();
        let buys = self
            .packs
            .clone()
            .into_iter()
            .filter(move |p| discounted(p.cost(), discount) <= balance)
            .map(Action::BuyPack);
        Some(buys)
    }

    pub(crate) fn gen_moves_buy_card(
        &self,
        balance: usize,
        vouchers: &Vouchers,
    ) -> Option<impl Iterator<Item = Action>> {
        if self.cards.is_empty() {
            return None;
        }
        let discount = vouchers.discount_pct();
        let buys = self
            .cards
            .clone()
            .into_iter()
            .filter(move |c| discounted(c.shop_cost(), discount) <= balance)
            .map(Action::BuyPlayingCard);
        Some(buys)
    }

    pub(crate) fn gen_moves_buy_voucher(
        &self,
        balance: usize,
    ) -> Option<impl Iterator<Item = Action>> {
        // Vouchers are never discounted, so no `Vouchers` argument here.
        let voucher = self.voucher.filter(|v| v.cost() <= balance)?;
        Some(std::iter::once(Action::BuyVoucher(voucher)))
    }
}

/// Telescope guarantees the most played hand's Planet card in every
/// Celestial Pack. Applied after generation so it works the same for both
/// RNG backends.
fn apply_telescope(pack: &mut Pack, ctx: &ShopContext) {
    if !ctx.vouchers.telescope() || pack.category != PackCategory::Celestial {
        return;
    }
    let Some(wanted) = Planets::iter().find(|p| p.hand_rank() == ctx.most_played) else {
        return;
    };
    if pack
        .contents
        .iter()
        .any(|c| matches!(c, PackContent::Planet(p) if *p == wanted))
    {
        return;
    }
    if let Some(first) = pack.contents.first_mut() {
        *first = PackContent::Planet(wanted);
    }
}

pub(crate) fn gen_edition(prob_mult: u32, rng: &mut impl Rng) -> Edition {
    if rng.gen_ratio(3u32.saturating_mul(prob_mult).min(1000), 1000) {
        return Edition::Negative;
    }
    if rng.gen_ratio(3u32.saturating_mul(prob_mult).min(1000), 1000) {
        return Edition::Polychrome;
    }
    if rng.gen_ratio(14u32.saturating_mul(prob_mult).min(1000), 1000) {
        return Edition::Holographic;
    }
    if rng.gen_ratio(20u32.saturating_mul(prob_mult).min(1000), 1000) {
        return Edition::Foil;
    }
    Edition::Base
}

/// A shop playing card (Magic Trick). Plain unless Illusion is also
/// redeemed, which is the only thing that lets it roll an enhancement,
/// edition, or seal.
pub(crate) fn gen_shop_playing_card(prob_mult: u32, illusion: bool, rng: &mut impl Rng) -> Card {
    if illusion {
        return gen_random_playing_card(prob_mult, rng);
    }
    let values: Vec<Value> = Value::iter().collect();
    let suits: Vec<Suit> = Suit::iter().collect();
    let v = values[rng.gen_range(0..values.len())];
    let s = suits[rng.gen_range(0..suits.len())];
    Card::new(v, s)
}

pub(crate) fn gen_random_playing_card(prob_mult: u32, rng: &mut impl Rng) -> Card {
    let values: Vec<Value> = Value::iter().collect();
    let suits: Vec<Suit> = Suit::iter().collect();
    let v = values[rng.gen_range(0..values.len())];
    let s = suits[rng.gen_range(0..suits.len())];
    let mut card = Card::new(v, s);

    if rng.gen_ratio(1, 5) {
        const ENHANCEMENTS: [Enhancement; 8] = [
            Enhancement::Bonus,
            Enhancement::Mult,
            Enhancement::Wild,
            Enhancement::Glass,
            Enhancement::Steel,
            Enhancement::Stone,
            Enhancement::Gold,
            Enhancement::Lucky,
        ];
        card.enhancement = Some(ENHANCEMENTS[rng.gen_range(0..ENHANCEMENTS.len())]);
    }

    card.edition = gen_edition(prob_mult, rng);

    if rng.gen_ratio(1, 10) {
        const SEALS: [Seal; 4] = [Seal::Gold, Seal::Red, Seal::Blue, Seal::Purple];
        card.seal = Some(SEALS[rng.gen_range(0..SEALS.len())]);
    }

    card
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub(crate) struct JokerGenerator {}

impl JokerGenerator {
    pub(crate) fn new() -> Self {
        JokerGenerator {}
    }

    // Randomly generate rarity of new joker.
    // 70% chance Common, 25% chance Uncommon, 5% chance Rare.
    // Legendary can only appear from Soul Spectral Card.
    fn gen_rarity(&self, rng: &mut impl Rng) -> Rarity {
        let choices = [Rarity::Common, Rarity::Uncommon, Rarity::Rare];
        let weights = [70u32, 25, 5];
        let dist = WeightedIndex::new(weights).unwrap();
        choices[dist.sample(rng)]
    }

    pub(crate) fn gen_joker(
        &self,
        prob_mult: u32,
        exclude: &[Jokers],
        rng: &mut impl Rng,
    ) -> Jokers {
        let rarity = self.gen_rarity(rng);
        let all = jokers_by_rarity(rarity);
        let choices: Vec<_> = all
            .iter()
            .filter(|j| {
                !exclude
                    .iter()
                    .any(|e| std::mem::discriminant(e) == std::mem::discriminant(*j))
            })
            .cloned()
            .collect();
        let choices = if choices.is_empty() { all } else { choices };
        let i = rng.gen_range(0..choices.len());
        let mut joker = choices[i].clone();
        joker.set_edition(gen_edition(prob_mult, rng));
        joker.set_instance_id(crate::joker::mint_joker_id());
        joker
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub(crate) struct ConsumableGenerator {}

impl ConsumableGenerator {
    pub(crate) fn new() -> Self {
        ConsumableGenerator {}
    }

    /// Generate a random planet, excluding secret planets unless their hand has been played,
    /// and excluding any already-picked planets.
    pub(crate) fn gen_planet(
        &self,
        planetarium: &Planetarium,
        exclude: &[Planets],
        rng: &mut impl Rng,
    ) -> Planets {
        let available: Vec<Planets> = Planets::iter()
            .filter(|p| {
                if exclude.contains(p) {
                    return false;
                }
                if p.is_secret() {
                    planetarium.level(p.hand_rank()).plays > 0
                } else {
                    true
                }
            })
            .collect();
        if available.is_empty() {
            return self.gen_planet(planetarium, &[], rng);
        }
        let i = rng.gen_range(0..available.len());
        available[i]
    }

    pub(crate) fn gen_planet_consumable(
        &self,
        planetarium: &Planetarium,
        exclude: &[Planets],
        rng: &mut impl Rng,
    ) -> Consumable {
        Consumable::Planet(self.gen_planet(planetarium, exclude, rng))
    }

    fn gen_tarot(&self, exclude: &[Tarot], rng: &mut impl Rng) -> Tarot {
        let all: Vec<Tarot> = Tarot::iter().collect();
        let choices: Vec<_> = all.iter().filter(|t| !exclude.contains(t)).collect();
        let choices = if choices.is_empty() {
            all.iter().collect()
        } else {
            choices
        };
        let i = rng.gen_range(0..choices.len());
        *choices[i]
    }

    pub(crate) fn gen_tarot_consumable(&self, exclude: &[Tarot], rng: &mut impl Rng) -> Consumable {
        Consumable::Tarot(self.gen_tarot(exclude, rng))
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub(crate) struct PackGenerator {}

impl PackGenerator {
    // Weighted random pack selection per spec.
    // Standard/Arcana/Celestial: Normal=4, Jumbo=2, Mega=0.5 (scaled *10 -> 40,20,5)
    // Buffoon: Normal=1.2, Jumbo=0.6, Mega=0.15 (scaled *10 -> 12,6,2)
    // Spectral: excluded
    pub(crate) fn gen_pack(
        &self,
        planetarium: &Planetarium,
        prob_mult: u32,
        exclude: Option<(&PackCategory, &PackSize)>,
        held_jokers: &[Jokers],
        rng: &mut impl Rng,
    ) -> Pack {
        #[rustfmt::skip]
        let all_choices: &[(PackCategory, PackSize, u32)] = &[
            (PackCategory::Standard,  PackSize::Normal, 40),
            (PackCategory::Standard,  PackSize::Jumbo,  20),
            (PackCategory::Standard,  PackSize::Mega,    5),
            (PackCategory::Arcana,    PackSize::Normal, 40),
            (PackCategory::Arcana,    PackSize::Jumbo,  20),
            (PackCategory::Arcana,    PackSize::Mega,    5),
            (PackCategory::Celestial, PackSize::Normal, 40),
            (PackCategory::Celestial, PackSize::Jumbo,  20),
            (PackCategory::Celestial, PackSize::Mega,    5),
            (PackCategory::Buffoon,   PackSize::Normal, 12),
            (PackCategory::Buffoon,   PackSize::Jumbo,   6),
            (PackCategory::Buffoon,   PackSize::Mega,    2),
        ];

        let choices: Vec<&(PackCategory, PackSize, u32)> = all_choices
            .iter()
            .filter(|(cat, sz, _)| exclude.is_none_or(|(ec, es)| cat != ec || sz != es))
            .collect();

        let weights: Vec<u32> = choices.iter().map(|(_, _, w)| *w).collect();
        let dist = WeightedIndex::new(&weights).unwrap();
        let idx = dist.sample(rng);
        let (category, size, _) = choices[idx];

        let count = match (category, size) {
            (PackCategory::Buffoon, PackSize::Normal) => 2,
            (PackCategory::Buffoon, _) => 4,
            (_, PackSize::Normal) => 3,
            _ => 5,
        };

        let contents = self.gen_contents(category, count, planetarium, prob_mult, held_jokers, rng);

        Pack {
            category: *category,
            size: *size,
            contents,
        }
    }

    fn gen_contents(
        &self,
        category: &PackCategory,
        count: usize,
        planetarium: &Planetarium,
        prob_mult: u32,
        held_jokers: &[Jokers],
        rng: &mut impl Rng,
    ) -> Vec<PackContent> {
        let joker_gen = JokerGenerator {};
        let consumable_gen = ConsumableGenerator {};

        match category {
            PackCategory::Arcana => (0..count)
                .map(|_| PackContent::Tarot(crate::tarot::random_tarot(rng)))
                .collect(),
            PackCategory::Celestial => {
                let mut exclude: Vec<Planets> = vec![];
                (0..count)
                    .map(|_| {
                        let planet = consumable_gen.gen_planet(planetarium, &exclude, rng);
                        exclude.push(planet);
                        PackContent::Planet(planet)
                    })
                    .collect()
            }
            PackCategory::Buffoon => {
                let mut seen: Vec<Jokers> = held_jokers.to_vec();
                (0..count)
                    .map(|_| {
                        let joker = joker_gen.gen_joker(prob_mult, &seen, rng);
                        seen.push(joker.clone());
                        PackContent::Joker(joker)
                    })
                    .collect()
            }
            PackCategory::Standard => (0..count)
                .map(|_| PackContent::PlayingCard(gen_random_playing_card(prob_mult, rng)))
                .collect(),
            PackCategory::Spectral => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::FastBackend;
    use rand_chacha::ChaCha8Rng;

    fn fast_backend() -> Backend {
        Backend::Fast(FastBackend::new(ChaCha8Rng::from_entropy()))
    }

    #[test]
    fn test_shop_refresh() {
        let mut shop = Shop::new();
        let planetarium = Planetarium::new();
        assert_eq!(shop.jokers.len(), 0);
        assert_eq!(shop.packs.len(), 0);
        let vouchers = Vouchers::new();
        let ctx = ShopContext::for_test(&planetarium, &vouchers, 1);
        shop.refresh(&ctx, &mut fast_backend());
        assert_eq!(shop.jokers.len() + shop.consumables.len(), 2);
        assert_eq!(shop.packs.len(), 2);
    }

    #[test]
    fn test_shop_buy_joker() {
        let mut shop = Shop::new();
        let j1 = jokers_by_rarity(Rarity::Common)[0].clone();
        shop.jokers = vec![j1.clone()];
        assert_eq!(shop.joker_from_index(0).expect("first joker"), j1.clone());
        shop.buy_joker(&j1).expect("buy joker");
    }

    #[test]
    fn test_shop_buy_joker_disambiguates_duplicates_by_instance_id() {
        // Two copies of the same joker variant, distinguishable only by
        // instance_id (same as real shop generation would produce). Without
        // instance-id-sensitive equality, `buy_joker`'s `.position()` lookup
        // can't tell them apart and may remove the wrong one.
        let mut j1 = jokers_by_rarity(Rarity::Common)[0].clone();
        let mut j2 = j1.clone();
        j1.set_instance_id(1);
        j2.set_instance_id(2);

        let mut shop = Shop::new();
        shop.jokers = vec![j1.clone(), j2.clone()];

        let bought = shop.buy_joker(&j2).expect("buy joker");
        assert_eq!(bought.instance_id(), 2);
        assert_eq!(shop.jokers.len(), 1);
        assert_eq!(shop.jokers[0].instance_id(), 1);
    }

    #[test]
    fn test_shop_buy_consumable() {
        let mut shop = Shop::new();
        shop.consumables = vec![Consumable::Planet(Planets::Mercury)];
        assert_eq!(shop.consumables.len(), 1);
        let c1 = shop.consumables[0].clone();
        shop.buy_consumable(&c1).expect("buy consumable");
        assert_eq!(shop.consumables.len(), 0);
    }

    #[test]
    fn test_shop_buy_pack() {
        let mut shop = Shop::new();
        let planetarium = Planetarium::new();
        let vouchers = Vouchers::new();
        let ctx = ShopContext::for_test(&planetarium, &vouchers, 1);
        shop.refresh(&ctx, &mut fast_backend());
        assert_eq!(shop.packs.len(), 2);
        let p1 = shop.packs[0].clone();
        let bought = shop.buy_pack(&p1).expect("buy pack");
        assert_eq!(bought.category, p1.category);
        assert_eq!(shop.packs.len(), 1);
    }

    #[test]
    fn test_secret_planet_gating() {
        let planetarium = Planetarium::new();
        let gen = ConsumableGenerator::new();
        for _ in 0..500 {
            let c = gen.gen_planet_consumable(&planetarium, &[], &mut rand::thread_rng());
            let Consumable::Planet(planet) = c else {
                continue;
            };
            assert!(
                !planet.is_secret(),
                "secret planet generated before discovery"
            );
        }
    }

    #[test]
    fn test_gen_moves_buy_consumable_slots_full() {
        let mut shop = Shop::new();
        let planetarium = Planetarium::new();
        let vouchers = Vouchers::new();
        let ctx = ShopContext::for_test(&planetarium, &vouchers, 1);
        shop.refresh(&ctx, &mut fast_backend());
        // slots full (held == consumable_slots)
        let moves = shop.gen_moves_buy_consumable(100, 2, 2, &vouchers);
        assert!(moves.is_none());
    }

    #[test]
    fn test_gen_moves_buy_consumable_no_funds() {
        let mut shop = Shop::new();
        let planetarium = Planetarium::new();
        let vouchers = Vouchers::new();
        let ctx = ShopContext::for_test(&planetarium, &vouchers, 1);
        shop.refresh(&ctx, &mut fast_backend());
        // 0 money can't afford any planet ($3)
        let moves: Option<Vec<Action>> = shop
            .gen_moves_buy_consumable(0, 2, 0, &vouchers)
            .map(|i| i.collect());
        assert!(moves.is_none_or(|v| v.is_empty()));
    }

    #[test]
    fn test_pack_gen_produces_valid_packs() {
        let planetarium = Planetarium::new();
        let gen = PackGenerator {};
        for _ in 0..50 {
            let pack = gen.gen_pack(&planetarium, 1, None, &[], &mut rand::thread_rng());
            let expected_count = match (&pack.category, &pack.size) {
                (PackCategory::Buffoon, PackSize::Normal) => 2,
                (PackCategory::Buffoon, _) => 4,
                (_, PackSize::Normal) => 3,
                _ => 5,
            };
            assert_eq!(pack.contents.len(), expected_count);
        }
    }

    #[test]
    fn test_gen_joker_rarity_distribution_roughly_matches_weights() {
        use rand::SeedableRng;
        let mut rng = ChaCha8Rng::seed_from_u64(1234);
        let gen = JokerGenerator::new();
        let n = 2000;
        let mut common = 0;
        let mut uncommon = 0;
        let mut rare = 0;
        for _ in 0..n {
            match gen.gen_joker(1, &[], &mut rng).rarity() {
                Rarity::Common => common += 1,
                Rarity::Uncommon => uncommon += 1,
                Rarity::Rare => rare += 1,
                Rarity::Legendary => panic!("gen_joker should never roll Legendary"),
            }
        }
        // Generous bands around the intended 70/25/5 split - wide enough to
        // never flake, tight enough to catch the roll being broken/reverted
        // to Common-only.
        assert!(
            (1200..1900).contains(&common),
            "expected ~70% Common, got {common}/{n}"
        );
        assert!(
            (300..800).contains(&uncommon),
            "expected ~25% Uncommon, got {uncommon}/{n}"
        );
        assert!(
            (10..250).contains(&rare),
            "expected ~5% Rare, got {rare}/{n}"
        );
    }
}
