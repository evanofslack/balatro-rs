use crate::card::{Enhancement, Suit};
use crate::error::GameError;
use crate::game::Game;
use crate::joker::Joker;
#[cfg(feature = "python")]
use pyo3::pyclass;
use strum::{EnumIter, IntoEnumIterator};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, EnumIter)]
pub enum Tarot {
    Fool,
    Magician,
    HighPriestess,
    Empress,
    Emperor,
    Hierophant,
    Lovers,
    Chariot,
    Justice,
    Hermit,
    WheelOfFortune,
    Strength,
    HangedMan,
    Death,
    Temperance,
    Devil,
    Tower,
    Star,
    Moon,
    Sun,
    Judgement,
    World,
}

impl Tarot {
    pub fn name(&self) -> &str {
        match self {
            Self::Fool => "The Fool",
            Self::Magician => "The Magician",
            Self::HighPriestess => "The High Priestess",
            Self::Empress => "The Empress",
            Self::Emperor => "The Emperor",
            Self::Hierophant => "The Hierophant",
            Self::Lovers => "The Lovers",
            Self::Chariot => "The Chariot",
            Self::Justice => "Justice",
            Self::Hermit => "The Hermit",
            Self::WheelOfFortune => "The Wheel of Fortune",
            Self::Strength => "Strength",
            Self::HangedMan => "The Hanged Man",
            Self::Death => "Death",
            Self::Temperance => "Temperance",
            Self::Devil => "The Devil",
            Self::Tower => "The Tower",
            Self::Star => "The Star",
            Self::Moon => "The Moon",
            Self::Sun => "The Sun",
            Self::Judgement => "Judgement",
            Self::World => "The World",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::Fool => "Creates a copy of the last Tarot used",
            Self::Magician => "Enhances up to 2 selected cards into Lucky Cards",
            Self::HighPriestess => "Gives 2 random Planet cards",
            Self::Empress => "Enhances up to 2 selected cards into Mult Cards",
            Self::Emperor => "Gives 2 random Tarot cards",
            Self::Hierophant => "Enhances up to 2 selected cards into Bonus Cards",
            Self::Lovers => "Enhances 1 selected card into a Wild Card",
            Self::Chariot => "Enhances 1 selected card into a Steel Card",
            Self::Justice => "Enhances 1 selected card into a Glass Card",
            Self::Hermit => "Doubles your money (up to $20)",
            Self::WheelOfFortune => "Adds a Foil, Holo or Polychrome to 1 random Joker",
            Self::Strength => "Increases the rank of up to 2 selected cards",
            Self::HangedMan => "Destroys up to 2 selected cards",
            Self::Death => "Converts the left selected card into the right",
            Self::Temperance => "Gives $1 per Joker sell value (max $50)",
            Self::Devil => "Enhances 1 selected card into a Gold Card",
            Self::Tower => "Enhances 1 selected card into a Stone Card",
            Self::Star => "Converts up to 3 selected cards to Diamonds",
            Self::Moon => "Converts up to 3 selected cards to Clubs",
            Self::Sun => "Converts up to 3 selected cards to Hearts",
            Self::Judgement => "Creates a random Joker card",
            Self::World => "Converts up to 3 selected cards to Spades",
        }
    }

    pub fn cost(&self) -> usize {
        3
    }

    pub fn sell_value(&self) -> usize {
        1
    }

    pub fn min_targets(&self) -> usize {
        match self {
            Self::Fool
            | Self::HighPriestess
            | Self::Emperor
            | Self::Hermit
            | Self::WheelOfFortune
            | Self::Temperance
            | Self::Judgement => 0,
            Self::Lovers
            | Self::Chariot
            | Self::Justice
            | Self::Devil
            | Self::Tower
            | Self::Magician
            | Self::Empress
            | Self::Hierophant
            | Self::Strength
            | Self::HangedMan
            | Self::Star
            | Self::Moon
            | Self::Sun
            | Self::World => 1,
            Self::Death => 2,
        }
    }

    pub fn max_targets(&self) -> usize {
        match self {
            Self::Fool
            | Self::HighPriestess
            | Self::Emperor
            | Self::Hermit
            | Self::WheelOfFortune
            | Self::Temperance
            | Self::Judgement => 0,
            Self::Lovers | Self::Chariot | Self::Justice | Self::Devil | Self::Tower => 1,
            Self::Magician
            | Self::Empress
            | Self::Hierophant
            | Self::Strength
            | Self::HangedMan
            | Self::Death => 2,
            Self::Star | Self::Moon | Self::Sun | Self::World => 3,
        }
    }

    pub fn requires_targets(&self) -> bool {
        self.min_targets() > 0
    }

    pub fn apply(&self, game: &mut Game) -> Result<(), GameError> {
        match self {
            Self::Magician => {
                let selected = game.available.selected();
                for card in selected {
                    game.mutate_card(card.id, |c| c.enhancement = Some(Enhancement::Lucky));
                }
            }
            Self::Empress => {
                let selected = game.available.selected();
                for card in selected {
                    game.mutate_card(card.id, |c| c.enhancement = Some(Enhancement::Mult));
                }
            }
            Self::Hierophant => {
                let selected = game.available.selected();
                for card in selected {
                    game.mutate_card(card.id, |c| c.enhancement = Some(Enhancement::Bonus));
                }
            }
            Self::Lovers => {
                let selected = game.available.selected();
                for card in selected {
                    game.mutate_card(card.id, |c| c.enhancement = Some(Enhancement::Wild));
                }
            }
            Self::Chariot => {
                let selected = game.available.selected();
                for card in selected {
                    game.mutate_card(card.id, |c| c.enhancement = Some(Enhancement::Steel));
                }
            }
            Self::Justice => {
                let selected = game.available.selected();
                for card in selected {
                    game.mutate_card(card.id, |c| c.enhancement = Some(Enhancement::Glass));
                }
            }
            Self::Devil => {
                let selected = game.available.selected();
                for card in selected {
                    game.mutate_card(card.id, |c| c.enhancement = Some(Enhancement::Gold));
                }
            }
            Self::Tower => {
                let selected = game.available.selected();
                for card in selected {
                    game.mutate_card(card.id, |c| c.enhancement = Some(Enhancement::Stone));
                }
            }
            Self::Star => {
                let selected = game.available.selected();
                for card in selected {
                    game.mutate_card(card.id, |c| c.suit = Suit::Diamond);
                }
            }
            Self::Moon => {
                let selected = game.available.selected();
                for card in selected {
                    game.mutate_card(card.id, |c| c.suit = Suit::Club);
                }
            }
            Self::Sun => {
                let selected = game.available.selected();
                for card in selected {
                    game.mutate_card(card.id, |c| c.suit = Suit::Heart);
                }
            }
            Self::World => {
                let selected = game.available.selected();
                for card in selected {
                    game.mutate_card(card.id, |c| c.suit = Suit::Spade);
                }
            }
            Self::Strength => {
                let selected = game.available.selected();
                for card in selected {
                    game.mutate_card(card.id, |c| c.value = c.value.next());
                }
            }
            Self::HangedMan => {
                let selected = game.available.selected();
                for card in selected {
                    game.destroy_card(card.id);
                }
            }
            Self::Death => {
                let selected = game.available.selected();
                if selected.len() < 2 {
                    return Err(GameError::InvalidAction);
                }
                let left_id = selected[0].id;
                let right = selected[1];
                game.mutate_card(left_id, |c| {
                    c.value = right.value;
                    c.suit = right.suit;
                    c.enhancement = right.enhancement;
                    c.edition = right.edition;
                    c.seal = right.seal;
                    c.face_card_override = right.face_card_override;
                });
            }
            Self::Hermit => {
                let gain = game.money.min(20);
                game.money += gain;
            }
            Self::Temperance => {
                let total: usize = game.jokers.iter().map(|j| j.sell_value()).sum();
                game.money += total.min(50);
            }
            Self::WheelOfFortune => {
                // TODO: Joker editions not implemented yet
            }
            Self::HighPriestess => {
                let slots = game.config.consumable_slots;
                let gen = crate::shop::ConsumableGenerator::new();
                for _ in 0..2 {
                    if game.consumables.len() >= slots {
                        break;
                    }
                    let planetarium = game.planetarium.clone();
                    let planet = gen.gen_planet_consumable(&planetarium, &[], &mut game.rng);
                    game.consumables.push(planet);
                }
            }
            Self::Emperor => {
                let slots = game.config.consumable_slots;
                let gen = crate::shop::ConsumableGenerator::new();
                for _ in 0..2 {
                    if game.consumables.len() >= slots {
                        break;
                    }
                    let tarot = gen.gen_tarot_consumable(&mut game.rng);
                    game.consumables.push(tarot);
                }
            }
            Self::Judgement => {
                if game.jokers.len() < game.config.joker_slots {
                    let prob_mult = game.prob_mult;
                    let joker = crate::shop::JokerGenerator::new().gen_joker(prob_mult, &mut game.rng);
                    game.jokers.push(joker);
                }
            }
            Self::Fool => {
                if let Some(last) = game.last_consumable_used.clone() {
                    if game.consumables.len() < game.config.consumable_slots {
                        game.consumables.push(last);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn random(rng: &mut impl rand::Rng) -> Self {
        let all: Vec<Self> = Self::iter().collect();
        let i = rng.gen_range(0..all.len());
        all[i]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{Card, Suit, Value};
    use crate::consumable::Consumable;
    use crate::game::Game;
    use crate::stage::{Blind, Stage};

    fn game_in_blind() -> Game {
        Game {
            stage: Stage::Blind(Blind::Small),
            ..Default::default()
        }
    }

    #[test]
    fn test_target_counts() {
        assert_eq!(Tarot::Fool.min_targets(), 0);
        assert_eq!(Tarot::Fool.max_targets(), 0);
        assert_eq!(Tarot::Lovers.min_targets(), 1);
        assert_eq!(Tarot::Lovers.max_targets(), 1);
        assert_eq!(Tarot::Magician.min_targets(), 1);
        assert_eq!(Tarot::Magician.max_targets(), 2);
        assert_eq!(Tarot::Death.min_targets(), 2);
        assert_eq!(Tarot::Death.max_targets(), 2);
        assert_eq!(Tarot::Star.min_targets(), 1);
        assert_eq!(Tarot::Star.max_targets(), 3);
    }

    #[test]
    fn test_requires_targets() {
        assert!(!Tarot::Fool.requires_targets());
        assert!(!Tarot::Hermit.requires_targets());
        assert!(Tarot::Lovers.requires_targets());
        assert!(Tarot::Death.requires_targets());
    }

    #[test]
    fn test_strength_rank_wrap() {
        assert_eq!(Value::King.next(), Value::Ace);
        assert_eq!(Value::Ace.next(), Value::Two);
        assert_eq!(Value::Two.next(), Value::Three);
    }

    #[test]
    fn test_hermit_cap() {
        let mut g = game_in_blind();
        g.money = 4;
        Tarot::Hermit.apply(&mut g).unwrap();
        assert_eq!(g.money, 8);

        let mut g = game_in_blind();
        g.money = 25;
        Tarot::Hermit.apply(&mut g).unwrap();
        assert_eq!(g.money, 45);

        let mut g = game_in_blind();
        g.money = 20;
        Tarot::Hermit.apply(&mut g).unwrap();
        assert_eq!(g.money, 40);
    }

    #[test]
    fn test_death_copy_semantics() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let king = Card::new(Value::King, Suit::Diamond);
        let ace_id = ace.id;

        let mut g = game_in_blind();
        g.available.extend(vec![ace, king]);
        g.available.select_card(ace).unwrap();
        g.available.select_card(king).unwrap();
        g.deck.extend(vec![ace, king]);

        Tarot::Death.apply(&mut g).unwrap();

        let cards = g.available.cards();
        let left = cards.iter().find(|c| c.id == ace_id).unwrap();
        assert_eq!(left.value, Value::King);
        assert_eq!(left.suit, Suit::Diamond);
        assert_eq!(left.id, ace_id);
    }

    #[test]
    fn test_enhancement_tarot() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let ace_id = ace.id;

        let mut g = game_in_blind();
        g.available.extend(vec![ace]);
        g.available.select_card(ace).unwrap();
        g.deck.extend(vec![ace]);

        Tarot::Justice.apply(&mut g).unwrap();

        let cards = g.available.cards();
        let card = cards.iter().find(|c| c.id == ace_id).unwrap();
        assert_eq!(card.enhancement, Some(Enhancement::Glass));

        let deck_cards = g.deck.cards();
        let deck_card = deck_cards.iter().find(|c| c.id == ace_id).unwrap();
        assert_eq!(deck_card.enhancement, Some(Enhancement::Glass));
    }

    #[test]
    fn test_suit_tarot() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let ace_id = ace.id;

        let mut g = game_in_blind();
        g.available.extend(vec![ace]);
        g.available.select_card(ace).unwrap();
        g.deck.extend(vec![ace]);

        Tarot::Star.apply(&mut g).unwrap();

        let cards = g.available.cards();
        let card = cards.iter().find(|c| c.id == ace_id).unwrap();
        assert_eq!(card.suit, Suit::Diamond);
    }

    #[test]
    fn test_hanged_man_destroys() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let king = Card::new(Value::King, Suit::Diamond);
        let ace_id = ace.id;

        let mut g = game_in_blind();
        g.available.extend(vec![ace, king]);
        g.available.select_card(ace).unwrap();
        g.deck.extend(vec![ace, king]);

        let deck_before = g.deck.cards().len();
        assert_eq!(g.available.cards().len(), 2);
        Tarot::HangedMan.apply(&mut g).unwrap();
        assert_eq!(g.available.cards().len(), 1);
        assert_eq!(g.deck.cards().len(), deck_before - 1);
        assert!(!g.deck.cards().iter().any(|c| c.id == ace_id));
    }

    #[test]
    fn test_fool_copies_last_consumable() {
        use crate::planet::Planets;

        let mut g = game_in_blind();
        g.config.consumable_slots = 2;
        g.last_consumable_used = Some(Consumable::Planet(Planets::Mercury));

        Tarot::Fool.apply(&mut g).unwrap();
        assert_eq!(g.consumables.len(), 1);
        assert_eq!(g.consumables[0], Consumable::Planet(Planets::Mercury));
    }

    #[test]
    fn test_fool_noop_when_no_last() {
        let mut g = game_in_blind();
        Tarot::Fool.apply(&mut g).unwrap();
        assert!(g.consumables.is_empty());
    }

    #[test]
    fn test_strength_wraps_ace() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let ace_id = ace.id;

        let mut g = game_in_blind();
        g.available.extend(vec![ace]);
        g.available.select_card(ace).unwrap();
        g.deck.extend(vec![ace]);

        Tarot::Strength.apply(&mut g).unwrap();

        let cards = g.available.cards();
        let card = cards.iter().find(|c| c.id == ace_id).unwrap();
        assert_eq!(card.value, Value::Two);
    }

    #[test]
    fn test_strength_ten_to_jack_becomes_face() {
        let ten = Card::new(Value::Ten, Suit::Heart);
        let ten_id = ten.id;
        assert!(!ten.is_face_card());

        let mut g = game_in_blind();
        g.available.extend(vec![ten]);
        g.available.select_card(ten).unwrap();
        g.deck.extend(vec![ten]);

        Tarot::Strength.apply(&mut g).unwrap();

        let cards = g.available.cards();
        let card = cards.iter().find(|c| c.id == ten_id).unwrap();
        assert_eq!(card.value, Value::Jack);
        assert!(card.is_face_card());
    }

    #[test]
    fn test_strength_king_to_ace_loses_face() {
        let king = Card::new(Value::King, Suit::Heart);
        let king_id = king.id;
        assert!(king.is_face_card());

        let mut g = game_in_blind();
        g.available.extend(vec![king]);
        g.available.select_card(king).unwrap();
        g.deck.extend(vec![king]);

        Tarot::Strength.apply(&mut g).unwrap();

        let cards = g.available.cards();
        let card = cards.iter().find(|c| c.id == king_id).unwrap();
        assert_eq!(card.value, Value::Ace);
        assert!(!card.is_face_card());
    }
}
