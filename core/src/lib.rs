pub mod action;
pub mod ante;

pub fn seed_from_str(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}
pub mod available;
pub mod card;
pub mod config;
pub mod consumable;
pub mod deck;
pub mod effect;
pub mod error;
pub mod game;
pub mod generator;
pub mod hand;
pub mod joker;
pub mod pack;
pub mod planet;
pub mod rank;
pub mod shop;
pub mod space;
pub mod stage;
pub mod tarot;

#[cfg(test)]
mod tests {
    use crate::action::Action;
    use crate::consumable::Consumable;
    use crate::error::GameError;
    use crate::game::Game;
    use crate::planet::Planets;
    use crate::rank::HandRank;
    use crate::stage::Stage;

    use rand::Rng;

    #[test]
    fn test_planetarium_level_up_changes_scoring() {
        let mut g = Game::default();
        let before = g.planetarium.level(HandRank::OnePair);
        g.planetarium.level_up(HandRank::OnePair);
        let after = g.planetarium.level(HandRank::OnePair);
        assert_eq!(after.chips, before.chips + 15);
        assert_eq!(after.mult, before.mult + 1);
        assert_eq!(after.level, before.level + 1);
    }

    #[test]
    fn test_buy_consumable() {
        let mut g = Game::default();
        g.start();
        g.money = 100;
        g.stage = Stage::Shop();
        g.shop.consumables = vec![Consumable::Planet(Planets::Mercury)];
        g.handle_action(Action::BuyConsumable(Consumable::Planet(Planets::Mercury)))
            .unwrap();
        assert_eq!(g.consumables.len(), 1);
        assert_eq!(g.money, 97);
        assert!(g.shop.consumables.is_empty());
    }

    #[test]
    fn test_use_consumable_planet() {
        let mut g = Game::default();
        g.start();
        g.stage = Stage::Shop();
        g.consumables = vec![Consumable::Planet(Planets::Mercury)];
        let before = g.planetarium.level(HandRank::OnePair);
        g.handle_action(Action::UseConsumable(Consumable::Planet(Planets::Mercury)))
            .unwrap();
        assert!(g.consumables.is_empty());
        let after = g.planetarium.level(HandRank::OnePair);
        assert_eq!(after.chips, before.chips + 15);
        assert_eq!(after.mult, before.mult + 1);
    }

    #[test]
    fn test_use_consumable_in_postblind() {
        let mut g = Game::default();
        g.start();
        g.stage = Stage::PostBlind();
        g.consumables = vec![Consumable::Planet(Planets::Jupiter)];
        let before = g.planetarium.level(HandRank::Flush);
        g.handle_action(Action::UseConsumable(Consumable::Planet(Planets::Jupiter)))
            .unwrap();
        assert!(g.consumables.is_empty());
        let after = g.planetarium.level(HandRank::Flush);
        assert_eq!(after.chips, before.chips + 15);
        assert_eq!(after.mult, before.mult + 2);
    }

    #[test]
    fn test_buy_consumable_slots_full() {
        let mut g = Game::default();
        g.start();
        g.money = 100;
        g.stage = Stage::Shop();
        g.consumables = vec![
            Consumable::Planet(Planets::Pluto),
            Consumable::Planet(Planets::Mercury),
        ];
        g.shop.consumables = vec![Consumable::Planet(Planets::Venus)];
        let res = g.handle_action(Action::BuyConsumable(Consumable::Planet(Planets::Venus)));
        assert!(matches!(res, Err(GameError::NoAvailableSlot)));
    }

    #[test]
    fn test_buy_consumable_insufficient_funds() {
        let mut g = Game::default();
        g.start();
        g.money = 0;
        g.stage = Stage::Shop();
        g.shop.consumables = vec![Consumable::Planet(Planets::Mercury)];
        let res = g.handle_action(Action::BuyConsumable(Consumable::Planet(Planets::Mercury)));
        assert!(matches!(res, Err(GameError::InvalidBalance)));
    }

    #[test]
    fn test_use_consumable_valid_in_blind() {
        let mut g = Game::default();
        g.start();
        g.stage = Stage::Blind(crate::stage::Blind::Small);
        g.consumables = vec![Consumable::Planet(Planets::Mercury)];
        let before = g.planetarium.level(HandRank::OnePair);
        let res = g.handle_action(Action::UseConsumable(Consumable::Planet(Planets::Mercury)));
        assert!(res.is_ok());
        assert_eq!(
            g.planetarium.level(HandRank::OnePair).chips,
            before.chips + 15
        );
    }

    #[test]
    fn test_use_consumable_valid_in_preblind() {
        let mut g = Game::default();
        g.start();
        g.consumables = vec![Consumable::Planet(Planets::Saturn)];
        let before = g.planetarium.level(HandRank::Straight);
        let res = g.handle_action(Action::UseConsumable(Consumable::Planet(Planets::Saturn)));
        assert!(res.is_ok());
        assert_eq!(
            g.planetarium.level(HandRank::Straight).chips,
            before.chips + 30
        );
    }

    #[test]
    fn test_use_consumable_invalid_at_end() {
        let mut g = Game::default();
        g.start();
        g.stage = Stage::End(crate::stage::End::Win);
        g.consumables = vec![Consumable::Planet(Planets::Mercury)];
        let res = g.handle_action(Action::UseConsumable(Consumable::Planet(Planets::Mercury)));
        assert!(matches!(res, Err(GameError::InvalidAction)));
    }

    #[test]
    fn test_gen_actions_buy_consumable_in_shop() {
        let mut g = Game::default();
        g.start();
        g.money = 100;
        g.stage = Stage::Shop();
        g.shop.consumables = vec![Consumable::Planet(Planets::Mercury)];
        let actions: Vec<Action> = g.gen_actions().collect();
        assert!(actions.contains(&Action::BuyConsumable(Consumable::Planet(Planets::Mercury))));
    }

    #[test]
    fn test_gen_actions_use_consumable_in_shop() {
        let mut g = Game::default();
        g.start();
        g.stage = Stage::Shop();
        g.consumables = vec![Consumable::Planet(Planets::Earth)];
        let actions: Vec<Action> = g.gen_actions().collect();
        assert!(actions.contains(&Action::UseConsumable(Consumable::Planet(Planets::Earth))));
    }

    #[test]
    fn test_gen_actions_use_consumable_in_blind() {
        let mut g = Game::default();
        g.start();
        g.stage = Stage::Blind(crate::stage::Blind::Small);
        g.consumables = vec![Consumable::Planet(Planets::Mercury)];
        let actions: Vec<Action> = g.gen_actions().collect();
        assert!(actions
            .iter()
            .any(|a| matches!(a, Action::UseConsumable(_))));
        // buy is still shop-only
        assert!(!actions
            .iter()
            .any(|a| matches!(a, Action::BuyConsumable(_))));
    }

    #[test]
    // Test executing a full game using the gen_actions api
    fn test_game_gen_actions() {
        let mut g = Game::default();

        g.start();
        while !g.is_over() {
            // Get all available actions
            let actions: Vec<Action> = g.gen_actions().collect();
            if actions.is_empty() {
                break;
            }

            // Pick a random move and execute it
            let i = rand::thread_rng().gen_range(0..actions.len());
            let action = actions[i].clone();
            dbg!("game state:\n{}", g.clone());
            dbg!("play action: {}", action.clone());
            let action_res = g.handle_action(action.clone());
            dbg!(action);
            assert!(action_res.is_ok());
        }
        let result = g.result();
        // Ensure game is over at end
        assert!(result.is_some());
        // Check game state at end
        assert!(matches!(g.stage, Stage::End(_)));
        dbg!("game action history: {:?}", g.action_history);
    }

    #[test]
    fn test_tarot_hand_from_shop_draws_exactly_one_hand() {
        use crate::tarot::Tarot;
        let mut g = Game::default();
        g.start();
        // at Shop, available is empty (clear_blind returns cards to deck without re-dealing)
        g.stage = Stage::Shop();
        assert_eq!(g.available.cards().len(), 0);
        g.consumables = vec![Consumable::Tarot(Tarot::Magician)];
        g.handle_action(Action::UseConsumable(Consumable::Tarot(Tarot::Magician)))
            .unwrap();
        assert!(matches!(g.stage, Stage::TarotHand(Tarot::Magician)));
        assert_eq!(g.available.cards().len(), g.config.available);
    }

    #[test]
    fn test_tarot_hand_selection_capped_at_max_targets() {
        use crate::tarot::Tarot;
        let mut g = Game::default();
        g.start();
        g.stage = Stage::Shop();
        g.consumables = vec![Consumable::Tarot(Tarot::Magician)];
        g.handle_action(Action::UseConsumable(Consumable::Tarot(Tarot::Magician)))
            .unwrap();
        assert!(matches!(g.stage, Stage::TarotHand(Tarot::Magician)));
        let cards = g.available.cards();
        g.handle_action(Action::SelectCard(cards[0])).unwrap();
        g.handle_action(Action::SelectCard(cards[1])).unwrap();
        assert_eq!(g.available.selected().len(), 2);
        let res = g.handle_action(Action::SelectCard(cards[2]));
        assert!(matches!(res, Err(GameError::InvalidAction)));
        let actions: Vec<Action> = g.gen_actions().collect();
        assert!(!actions.iter().any(|a| matches!(a, Action::SelectCard(_))));
        assert!(actions.contains(&Action::ApplyTarot()));
    }

    #[test]
    // Test executing a full game using the gen_action_space (vector) api
    fn test_game_action_space() {
        let mut g = Game::default();

        g.start();
        while !g.is_over() {
            // Get action space and vector
            let space = g.gen_action_space();
            let space_vec = space.to_vec();
            if space.is_empty() {
                break;
            }

            // Pick a random move and ensure its unmasked
            let mut i: usize;
            loop {
                i = rand::thread_rng().gen_range(0..space_vec.len());
                if space_vec[i] == 1 {
                    break;
                }
            }
            let action = space.to_action(i, &g).expect("valid index to action");
            dbg!("game state:\n{}", g.clone());
            dbg!("play action: {}", action.clone());
            let action_res = g.handle_action(action.clone());
            dbg!(action);
            assert!(action_res.is_ok());
        }
        let result = g.result();
        // Ensure game is over at end
        assert!(result.is_some());
        // Check game state at end
        assert!(matches!(g.stage, Stage::End(_)));
        dbg!("game action history: {:?}", g.action_history);
    }
}
