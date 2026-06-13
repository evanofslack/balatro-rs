use crate::action::Action;
use crate::consumable::Consumable;
use crate::error::GameError;
use crate::joker::{Joker, Jokers, Rarity};
use crate::planet::{Planetarium, Planets};
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

    pub(crate) fn refresh(&mut self, planetarium: &Planetarium) {
        let j1 = self.joker_gen.gen_joker();
        let j2 = self.joker_gen.gen_joker();
        self.jokers = vec![j1, j2];

        let c1 = self.consumable_gen.gen_consumable(planetarium);
        let c2 = self.consumable_gen.gen_consumable(planetarium);
        self.consumables = vec![c1, c2];
    }

    pub(crate) fn joker_from_index(&self, i: usize) -> Option<Jokers> {
        return Some(self.jokers[i].clone());
    }

    pub(crate) fn consumable_from_index(&self, i: usize) -> Option<Consumable> {
        return self.consumables.get(i).cloned();
    }

    pub(crate) fn buy_joker(&mut self, joker: &Jokers) -> Result<Jokers, GameError> {
        let i = self
            .jokers
            .iter()
            .position(|j| j == joker)
            .ok_or(GameError::NoJokerMatch)?;
        let out = self.jokers.remove(i);
        return Ok(out);
    }

    pub(crate) fn buy_consumable(&mut self, consumable: &Consumable) -> Result<Consumable, GameError> {
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
        if self.jokers.len() == 0 {
            return None;
        }
        let buys = self
            .jokers
            .clone()
            .into_iter()
            .filter(move |j| j.cost() <= balance)
            .map(|j| Action::BuyJoker(j));
        return Some(buys);
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
            .map(|c| Action::BuyConsumable(c));
        Some(buys)
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct JokerGenerator {}

impl JokerGenerator {
    // Randomly generate rarity of new joker.
    // 70% chance Common, 25% chance Uncommon, 5% chance Rare.
    // Legendary can only appear from Soul Spectral Card.
    fn gen_rarity(&self) -> Rarity {
        // For now, we only have common jokers...
        return Rarity::Common;
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
        return choices[i].clone();
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub(crate) struct ConsumableGenerator {}

impl ConsumableGenerator {
    /// Generate a random planet, excluding secret planets unless their hand has been played.
    fn gen_planet(&self, planetarium: &Planetarium) -> Planets {
        let available: Vec<Planets> = Planets::iter()
            .filter(|p| {
                if p.is_secret() {
                    planetarium.level(p.hand_rank()).plays > 0
                } else {
                    true
                }
            })
            .collect();
        let i = thread_rng().gen_range(0..available.len());
        available[i].clone()
    }

    pub(crate) fn gen_consumable(&self, planetarium: &Planetarium) -> Consumable {
        Consumable::Planet(self.gen_planet(planetarium))
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
        shop.refresh(&planetarium);
        assert_eq!(shop.jokers.len(), 2);
        assert_eq!(shop.consumables.len(), 2);
    }

    #[test]
    fn test_shop_buy_joker() {
        let mut shop = Shop::new();
        let planetarium = Planetarium::new();
        shop.refresh(&planetarium);
        assert_eq!(shop.jokers.len(), 2);
        let j1 = shop.jokers[0].clone();
        assert_eq!(shop.joker_from_index(0).expect("first joker"), j1.clone());
        shop.buy_joker(&j1).expect("buy joker");
    }

    #[test]
    fn test_shop_buy_consumable() {
        let mut shop = Shop::new();
        let planetarium = Planetarium::new();
        shop.refresh(&planetarium);
        assert_eq!(shop.consumables.len(), 2);
        let c1 = shop.consumables[0].clone();
        shop.buy_consumable(&c1).expect("buy consumable");
        assert_eq!(shop.consumables.len(), 1);
    }

    #[test]
    fn test_secret_planet_gating() {
        let planetarium = Planetarium::new();
        let gen = ConsumableGenerator {};
        for _ in 0..500 {
            let c = gen.gen_consumable(&planetarium);
            let Consumable::Planet(planet) = c;
            assert!(!planet.is_secret(), "secret planet generated before discovery");
        }
    }

    #[test]
    fn test_gen_moves_buy_consumable_slots_full() {
        let mut shop = Shop::new();
        let planetarium = Planetarium::new();
        shop.refresh(&planetarium);
        // slots full (held == consumable_slots)
        let moves = shop.gen_moves_buy_consumable(100, 2, 2);
        assert!(moves.is_none());
    }

    #[test]
    fn test_gen_moves_buy_consumable_no_funds() {
        let mut shop = Shop::new();
        let planetarium = Planetarium::new();
        shop.refresh(&planetarium);
        // 0 money can't afford any planet ($3)
        let moves: Option<Vec<Action>> =
            shop.gen_moves_buy_consumable(0, 2, 0).map(|i| i.collect());
        assert!(moves.map_or(true, |v| v.is_empty()));
    }
}
