use crate::action::Action;
use crate::consumable::Consumable;
use crate::error::GameError;
use crate::joker::{Joker, Jokers, Rarity};
use crate::planet::{Planetarium, Planets};
use crate::tarot::Tarot;
use rand::prelude::*;
use strum::IntoEnumIterator;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Shop {
    pub jokers: Vec<Jokers>,
    pub consumables: Vec<Consumable>,
    joker_gen: JokerGenerator,
    consumable_gen: ConsumableGenerator,
}

impl Shop {
    pub fn new() -> Self {
        Shop {
            joker_gen: JokerGenerator {},
            consumable_gen: ConsumableGenerator {},
            jokers: Vec::new(),
            consumables: Vec::new(),
        }
    }
}

impl Default for Shop {
    fn default() -> Self {
        Self::new()
    }
}

impl Shop {
    pub(crate) fn refresh(
        &mut self,
        planetarium: &Planetarium,
        held: &[Consumable],
        allow_duplicates: bool,
    ) {
        let j1 = self.joker_gen.gen_joker();
        let j2 = self.joker_gen.gen_joker();
        self.jokers = vec![j1, j2];

        let held_planets: Vec<Planets> = if allow_duplicates {
            vec![]
        } else {
            held.iter()
                .filter_map(|c| {
                    if let Consumable::Planet(p) = c {
                        Some(p.clone())
                    } else {
                        None
                    }
                })
                .collect()
        };

        // Randomly pick consumable mix: 0=both planets, 1=mixed, 2=both tarots
        // TODO: implement packs instead of raw consumable cards
        let mix = thread_rng().gen_range(0..3usize);
        let (c1, c2) = match mix {
            0 => {
                let p1 = self
                    .consumable_gen
                    .gen_planet_consumable(planetarium, &held_planets);
                let mut exclude2 = held_planets.clone();
                if !allow_duplicates {
                    if let Consumable::Planet(p) = &p1 {
                        exclude2.push(p.clone());
                    }
                }
                let p2 = self
                    .consumable_gen
                    .gen_planet_consumable(planetarium, &exclude2);
                (p1, p2)
            }
            1 => {
                let p = self
                    .consumable_gen
                    .gen_planet_consumable(planetarium, &held_planets);
                let t = self.consumable_gen.gen_tarot_consumable();
                (p, t)
            }
            _ => {
                let t1 = self.consumable_gen.gen_tarot_consumable();
                let t2 = self.consumable_gen.gen_tarot_consumable();
                (t1, t2)
            }
        };
        self.consumables = vec![c1, c2];
    }

    pub(crate) fn joker_from_index(&self, i: usize) -> Option<Jokers> {
        Some(self.jokers[i].clone())
    }

    pub(crate) fn consumable_from_index(&self, i: usize) -> Option<Consumable> {
        self.consumables.get(i).cloned()
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

    // Generate a random new joker
    pub(crate) fn gen_joker(&self) -> Jokers {
        let rarity = self.gen_rarity();
        let choices = Jokers::by_rarity(rarity);
        let i = thread_rng().gen_range(0..choices.len());
        // TODO: don't regenerate already generated jokers.
        // track with hashmap.
        choices[i].clone()
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
    fn gen_planet(&self, planetarium: &Planetarium, exclude: &[Planets]) -> Planets {
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
            return self.gen_planet(planetarium, &[]);
        }
        let i = thread_rng().gen_range(0..available.len());
        available[i].clone()
    }

    pub(crate) fn gen_planet_consumable(
        &self,
        planetarium: &Planetarium,
        exclude: &[Planets],
    ) -> Consumable {
        Consumable::Planet(self.gen_planet(planetarium, exclude))
    }

    fn gen_tarot(&self) -> Tarot {
        Tarot::random()
    }

    pub(crate) fn gen_tarot_consumable(&self) -> Consumable {
        Consumable::Tarot(self.gen_tarot())
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
        assert_eq!(shop.consumables.len(), 0);
        shop.refresh(&planetarium, &[], false);
        assert_eq!(shop.jokers.len(), 2);
        assert_eq!(shop.consumables.len(), 2);
    }

    #[test]
    fn test_shop_buy_joker() {
        let mut shop = Shop::new();
        let planetarium = Planetarium::new();
        shop.refresh(&planetarium, &[], false);
        assert_eq!(shop.jokers.len(), 2);
        let j1 = shop.jokers[0].clone();
        assert_eq!(shop.joker_from_index(0).expect("first joker"), j1.clone());
        shop.buy_joker(&j1).expect("buy joker");
    }

    #[test]
    fn test_shop_buy_consumable() {
        let mut shop = Shop::new();
        let planetarium = Planetarium::new();
        shop.refresh(&planetarium, &[], false);
        assert_eq!(shop.consumables.len(), 2);
        let c1 = shop.consumables[0].clone();
        shop.buy_consumable(&c1).expect("buy consumable");
        assert_eq!(shop.consumables.len(), 1);
    }

    #[test]
    fn test_secret_planet_gating() {
        let planetarium = Planetarium::new();
        let gen = ConsumableGenerator::new();
        for _ in 0..500 {
            let c = gen.gen_planet_consumable(&planetarium, &[]);
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
        shop.refresh(&planetarium, &[], false);
        // slots full (held == consumable_slots)
        let moves = shop.gen_moves_buy_consumable(100, 2, 2);
        assert!(moves.is_none());
    }

    #[test]
    fn test_gen_moves_buy_consumable_no_funds() {
        let mut shop = Shop::new();
        let planetarium = Planetarium::new();
        shop.refresh(&planetarium, &[], false);
        // 0 money can't afford any planet ($3)
        let moves: Option<Vec<Action>> =
            shop.gen_moves_buy_consumable(0, 2, 0).map(|i| i.collect());
        assert!(moves.is_none_or(|v| v.is_empty()));
    }
}
