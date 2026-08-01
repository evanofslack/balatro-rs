use crate::card::Seal;
use crate::error::GameError;
use crate::game::Game;
use crate::rng::RngBackend;

use crate::card::Card;
use balatro_types::Edition;
use balatro_types::HandRank;
use balatro_types::Rarity;
pub use balatro_types::Spectral;
use balatro_types::Value::{self};
use strum::IntoEnumIterator;

/// Engine behavior for `Spectral`
pub trait SpectralEffect {
    fn apply(&self, game: &mut Game) -> Result<(), GameError>;
}

impl SpectralEffect for Spectral {
    fn apply(&self, game: &mut Game) -> Result<(), GameError> {
        match self {
            Self::Talisman => {
                let selected = game.available.selected();
                if selected.is_empty() || selected.len() > 1 {
                    return Err(GameError::InvalidAction);
                }
                let card = selected[0];
                game.mutate_card(card.id, |c| c.seal = Some(Seal::Gold));
            }
            Self::DejaVu => {
                let selected = game.available.selected();
                if selected.is_empty() || selected.len() > 1 {
                    return Err(GameError::InvalidAction);
                }
                let card = selected[0];
                game.mutate_card(card.id, |c| c.seal = Some(Seal::Red));
            }
            Self::Trance => {
                let selected = game.available.selected();
                if selected.is_empty() || selected.len() > 1 {
                    return Err(GameError::InvalidAction);
                }
                let card = selected[0];
                game.mutate_card(card.id, |c| c.seal = Some(Seal::Blue));
            }
            Self::Medium => {
                let selected = game.available.selected();
                if selected.is_empty() || selected.len() > 1 {
                    return Err(GameError::InvalidAction);
                }
                let card = selected[0];
                game.mutate_card(card.id, |c| c.seal = Some(Seal::Purple));
            }
            Self::Aura => {
                let selected = game.available.selected();
                if selected.is_empty() || selected.len() > 1 {
                    return Err(GameError::InvalidAction);
                }
                let card = selected[0];
                let edition = game.backend.roll_random_edition();
                game.mutate_card(card.id, |c| c.edition = edition);
            }
            Self::Cryptid => {
                let selected = game.available.selected();
                if selected.is_empty() || selected.len() > 1 {
                    return Err(GameError::InvalidAction);
                }
                let og = selected[0];
                let copy_one = Card::new(og.value, og.suit);
                let copy_two = Card::new(og.value, og.suit);
                let (id_one, id_two) = (copy_one.id, copy_two.id);
                game.available.extend(vec![copy_one, copy_two]);
                let copy_fields = |c: &mut Card| {
                    c.enhancement = og.enhancement;
                    c.edition = og.edition;
                    c.seal = og.seal;
                };
                game.mutate_card(id_one, copy_fields);
                game.mutate_card(id_two, copy_fields);
            }
            Self::Familiar => {
                let card = game.backend.pick_random_card(game.available.cards());
                game.destroy_card(card.id);
                let values: [Value; 3] = [Value::King, Value::Queen, Value::Jack];
                // TODO: does enhanced mean only enhance, no roll for seal/edition?
                let card_one = game
                    .backend
                    .gen_random_card(game.prob_mult, true, Some(&values));
                let card_two = game
                    .backend
                    .gen_random_card(game.prob_mult, true, Some(&values));
                let card_three = game
                    .backend
                    .gen_random_card(game.prob_mult, true, Some(&values));
                game.available
                    .extend([card_one, card_two, card_three].to_vec());
            }
            Self::Grim => {
                let card = game.backend.pick_random_card(game.available.cards());
                game.destroy_card(card.id);
                let values: [Value; 1] = [Value::Ace];
                let card_one = game
                    .backend
                    .gen_random_card(game.prob_mult, true, Some(&values));
                let card_two = game
                    .backend
                    .gen_random_card(game.prob_mult, true, Some(&values));
                game.available.extend([card_one, card_two].to_vec());
            }
            Self::Incantation => {
                let card = game.backend.pick_random_card(game.available.cards());
                game.destroy_card(card.id);
                let values: [Value; 9] = [
                    Value::Two,
                    Value::Three,
                    Value::Four,
                    Value::Five,
                    Value::Six,
                    Value::Seven,
                    Value::Eight,
                    Value::Nine,
                    Value::Ten,
                ];
                let card_one = game
                    .backend
                    .gen_random_card(game.prob_mult, true, Some(&values));
                let card_two = game
                    .backend
                    .gen_random_card(game.prob_mult, true, Some(&values));
                let card_three = game
                    .backend
                    .gen_random_card(game.prob_mult, true, Some(&values));
                let card_four = game
                    .backend
                    .gen_random_card(game.prob_mult, true, Some(&values));
                game.available
                    .extend([card_one, card_two, card_three, card_four].to_vec());
            }
            Self::Sigil => {
                let suit = game.backend.roll_random_suit();
                for card in game.available.cards() {
                    game.mutate_card(card.id, |c| c.suit = suit);
                }
            }
            Self::Ouija => {
                let value = game.backend.roll_random_value();
                for card in game.available.cards() {
                    game.mutate_card(card.id, |c| c.value = value);
                }
                game.config.available = game.config.available.saturating_sub(1);
            }
            Self::Immolate => {
                let count_destroy = 5.min(game.available.cards().len());
                for _ in 0..count_destroy {
                    let card = game.backend.pick_random_card(game.available.cards());
                    game.destroy_card(card.id);
                }
                game.money += 20;
            }
            Self::BlackHole => {
                // `RoyalFlush` shares its level slot with `StraightFlush`
                // (see `Planetarium::level_up`) - leveling both would
                // double-level that one slot.
                for rank in HandRank::iter().filter(|r| *r != HandRank::RoyalFlush) {
                    game.planetarium.level_up(rank);
                }
            }
            Self::Wraith => {
                if game.jokers.len() < game.config.joker_slots {
                    let ante = game.ante_current as i32;
                    let exclude = game.jokers.clone();
                    let prob_mult = game.prob_mult;
                    let joker =
                        game.backend
                            .gen_joker_of_rarity(ante, prob_mult, &exclude, Rarity::Rare);
                    game.jokers.push(joker);
                }
                game.money = 0;
            }
            Self::Soul => {
                if game.jokers.len() < game.config.joker_slots {
                    let ante = game.ante_current as i32;
                    let exclude = game.jokers.clone();
                    let prob_mult = game.prob_mult;
                    let joker = game.backend.gen_joker_of_rarity(
                        ante,
                        prob_mult,
                        &exclude,
                        Rarity::Legendary,
                    );
                    game.jokers.push(joker);
                }
            }
            Self::Ectoplasm => {
                if !game.jokers.is_empty() {
                    let picked = game.backend.pick_random_joker(game.jokers.clone());
                    if let Some(j) = game
                        .jokers
                        .iter_mut()
                        .find(|j| j.instance_id() == picked.instance_id())
                    {
                        j.set_edition(Edition::Negative);
                    }
                    game.config.available = game.config.available.saturating_sub(1);
                }
            }
            Self::Ankh => {
                if !game.jokers.is_empty() {
                    let original = game.backend.pick_random_joker(game.jokers.clone());
                    let mut clone = game.backend.clone_joker(original.clone());
                    // Strip negative
                    if clone.edition() == Edition::Negative {
                        clone.set_edition(Edition::Base);
                    }
                    game.jokers = vec![original, clone];
                }
            }
            Self::Hex => {
                if !game.jokers.is_empty() {
                    let mut original = game.backend.pick_random_joker(game.jokers.clone());
                    original.set_edition(Edition::Polychrome);
                    game.jokers = vec![original];
                }
            }
        }
        Ok(())
    }
}
