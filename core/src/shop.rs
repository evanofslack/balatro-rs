use crate::action::Action;
use crate::card::{Card, Edition, Enhancement, Seal, Suit, Value};
use crate::consumable::Consumable;
use crate::error::GameError;
use crate::joker::{Joker, Jokers, Rarity};
use crate::pack::{Pack, PackCategory, PackContent, PackSize};
use crate::planet::{Planetarium, Planets};
use crate::tarot::Tarot;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use strum::IntoEnumIterator;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Shop {
    pub jokers: Vec<Jokers>,
    pub consumables: Vec<Consumable>,
    pub packs: Vec<Pack>,
    joker_gen: JokerGenerator,
    consumable_gen: ConsumableGenerator,
    pack_gen: PackGenerator,
}

impl Shop {
    pub fn new() -> Self {
        Shop {
            joker_gen: JokerGenerator {},
            consumable_gen: ConsumableGenerator {},
            pack_gen: PackGenerator {},
            jokers: Vec::new(),
            consumables: Vec::new(),
            packs: Vec::new(),
        }
    }
}

impl Default for Shop {
    fn default() -> Self {
        Self::new()
    }
}

impl Shop {
    pub(crate) fn refresh_cards(
        &mut self,
        planetarium: &Planetarium,
        held: &[Consumable],
        prob_mult: u32,
        held_jokers: &[Jokers],
        rng: &mut impl Rng,
    ) {
        self.jokers.clear();
        self.consumables.clear();

        let mut excl_jokers: Vec<Jokers> = held_jokers.to_vec();
        let mut excl_tarots: Vec<Tarot> = held
            .iter()
            .filter_map(|c| {
                if let Consumable::Tarot(t) = c {
                    Some(*t)
                } else {
                    None
                }
            })
            .collect();
        let mut excl_planets: Vec<Planets> = held
            .iter()
            .filter_map(|c| {
                if let Consumable::Planet(p) = c {
                    Some(p.clone())
                } else {
                    None
                }
            })
            .collect();

        // Joker=20, Tarot=4, Planet=4
        let weights = [20u32, 4, 4];
        let dist = WeightedIndex::new(weights).unwrap();

        for _ in 0..2 {
            match dist.sample(rng) {
                0 => {
                    let joker = self.joker_gen.gen_joker(prob_mult, &excl_jokers, rng);
                    excl_jokers.push(joker.clone());
                    self.jokers.push(joker);
                }
                1 => {
                    let c = self.consumable_gen.gen_tarot_consumable(&excl_tarots, rng);
                    if let Consumable::Tarot(t) = &c {
                        excl_tarots.push(*t);
                    }
                    self.consumables.push(c);
                }
                _ => {
                    let c =
                        self.consumable_gen
                            .gen_planet_consumable(planetarium, &excl_planets, rng);
                    if let Consumable::Planet(p) = &c {
                        excl_planets.push(p.clone());
                    }
                    self.consumables.push(c);
                }
            }
        }
    }

    pub(crate) fn refresh(
        &mut self,
        planetarium: &Planetarium,
        held: &[Consumable],
        allow_duplicates: bool,
        prob_mult: u32,
        held_jokers: &[Jokers],
        rng: &mut impl Rng,
    ) {
        let _ = allow_duplicates;
        self.refresh_cards(planetarium, held, prob_mult, held_jokers, rng);

        let p1 = self
            .pack_gen
            .gen_pack(planetarium, prob_mult, None, held_jokers, rng);
        let exclude = (&p1.category, &p1.size);
        let p2 = self
            .pack_gen
            .gen_pack(planetarium, prob_mult, Some(exclude), held_jokers, rng);
        self.packs = vec![p1, p2];
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

    pub(crate) fn gen_moves_buy_joker(
        &self,
        balance: usize,
    ) -> Option<impl Iterator<Item = Action>> {
        if self.jokers.is_empty() {
            return None;
        }
        let buys = self
            .jokers
            .clone()
            .into_iter()
            .filter(move |j| j.cost() <= balance)
            .map(Action::BuyJoker);
        Some(buys)
    }

    pub(crate) fn gen_moves_buy_consumable(
        &self,
        balance: usize,
        consumable_slots: usize,
        held: usize,
    ) -> Option<impl Iterator<Item = Action>> {
        if self.consumables.is_empty() || held >= consumable_slots {
            return None;
        }
        let buys = self
            .consumables
            .clone()
            .into_iter()
            .filter(move |c| c.cost() <= balance)
            .map(Action::BuyConsumable);
        Some(buys)
    }

    pub(crate) fn gen_moves_buy_pack(
        &self,
        balance: usize,
    ) -> Option<impl Iterator<Item = Action>> {
        if self.packs.is_empty() {
            return None;
        }
        let buys = self
            .packs
            .clone()
            .into_iter()
            .filter(move |p| p.cost() <= balance)
            .map(Action::BuyPack);
        Some(buys)
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

pub(crate) fn gen_random_playing_card(prob_mult: u32, rng: &mut impl Rng) -> Card {
    let values = Value::values();
    let suits = Suit::suits();
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
    fn gen_rarity(&self) -> Rarity {
        // For now, we only have common jokers...
        Rarity::Common
        // let choices = [Rarity::Common, Rarity::Uncommon, Rarity::Rare];
        // let weights = [70, 25, 5];
        // let dist = WeightedIndex::new(&weights).unwrap();
        // let mut rng = thread_rng();
        // return choices[dist.sample(&mut rng)].clone();
    }

    pub(crate) fn gen_joker(
        &self,
        prob_mult: u32,
        exclude: &[Jokers],
        rng: &mut impl Rng,
    ) -> Jokers {
        let rarity = self.gen_rarity();
        let all = Jokers::by_rarity(rarity);
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
        available[i].clone()
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
            category: category.clone(),
            size: size.clone(),
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
                .map(|_| PackContent::Tarot(Tarot::random(rng)))
                .collect(),
            PackCategory::Celestial => {
                let mut exclude: Vec<Planets> = vec![];
                (0..count)
                    .map(|_| {
                        let planet = consumable_gen.gen_planet(planetarium, &exclude, rng);
                        exclude.push(planet.clone());
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

    #[test]
    fn test_shop_refresh() {
        let mut shop = Shop::new();
        let planetarium = Planetarium::new();
        assert_eq!(shop.jokers.len(), 0);
        assert_eq!(shop.packs.len(), 0);
        shop.refresh(&planetarium, &[], false, 1, &[], &mut rand::thread_rng());
        assert_eq!(shop.jokers.len() + shop.consumables.len(), 2);
        assert_eq!(shop.packs.len(), 2);
    }

    #[test]
    fn test_shop_buy_joker() {
        let mut shop = Shop::new();
        let j1 = Jokers::by_rarity(Rarity::Common)[0].clone();
        shop.jokers = vec![j1.clone()];
        assert_eq!(shop.joker_from_index(0).expect("first joker"), j1.clone());
        shop.buy_joker(&j1).expect("buy joker");
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
        shop.refresh(&planetarium, &[], false, 1, &[], &mut rand::thread_rng());
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
        shop.refresh(&planetarium, &[], false, 1, &[], &mut rand::thread_rng());
        // slots full (held == consumable_slots)
        let moves = shop.gen_moves_buy_consumable(100, 2, 2);
        assert!(moves.is_none());
    }

    #[test]
    fn test_gen_moves_buy_consumable_no_funds() {
        let mut shop = Shop::new();
        let planetarium = Planetarium::new();
        shop.refresh(&planetarium, &[], false, 1, &[], &mut rand::thread_rng());
        // 0 money can't afford any planet ($3)
        let moves: Option<Vec<Action>> =
            shop.gen_moves_buy_consumable(0, 2, 0).map(|i| i.collect());
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
}
