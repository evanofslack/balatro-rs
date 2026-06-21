use crate::action::{Action, MoveDirection};
use crate::config::Config;
use crate::error::ActionSpaceError;
use crate::game::Game;
use crate::stage::Blind;
#[cfg(feature = "python")]
use pyo3::pyclass;

// Hard code a bounded action space.
// Given constraints:
// available max = 24
// store consumable slots max = 4
// consumable slots = 2
//
// 0-23: select card
// 24-46: move card (left)
// 47-69: move card (right)
// 70: play
// 71: discard
// 72: cashout
// 73-76: buy joker
// 77: next round
// 78: select blind
// 79-80: buy consumable
// 81-82: use consumable
// 83: apply tarot
//
// We end up with a vector of length 84 where each index
// represents a potential action.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ActionSpace {
    pub select_card: Vec<usize>,
    pub move_card_left: Vec<usize>,
    pub move_card_right: Vec<usize>,
    pub play: Vec<usize>,
    pub discard: Vec<usize>,
    pub cash_out: Vec<usize>,
    pub buy_joker: Vec<usize>,
    pub next_round: Vec<usize>,
    pub select_blind: Vec<usize>,
    pub buy_consumable: Vec<usize>,
    pub use_consumable: Vec<usize>,
    pub apply_tarot: Vec<usize>,
}

impl ActionSpace {
    pub fn size(&self) -> usize {
        self.select_card.len()
            + self.move_card_left.len()
            + self.move_card_right.len()
            + self.play.len()
            + self.discard.len()
            + self.cash_out.len()
            + self.buy_joker.len()
            + self.next_round.len()
            + self.select_blind.len()
            + self.buy_consumable.len()
            + self.use_consumable.len()
            + self.apply_tarot.len()
    }

    fn select_card_min(&self) -> usize {
        0
    }

    fn select_card_max(&self) -> usize {
        self.select_card_min() + self.select_card.len() - 1
    }

    fn move_card_left_min(&self) -> usize {
        self.select_card_max() + 1
    }

    fn move_card_left_max(&self) -> usize {
        self.move_card_left_min() + self.select_card.len() - 2
    }

    fn move_card_right_min(&self) -> usize {
        self.move_card_left_max() + 1
    }

    fn move_card_right_max(&self) -> usize {
        self.move_card_right_min() + self.select_card.len() - 2
    }

    fn play_min(&self) -> usize {
        self.move_card_right_max() + 1
    }

    fn play_max(&self) -> usize {
        self.play_min() + self.play.len() - 1
    }

    fn discard_min(&self) -> usize {
        self.play_max() + 1
    }

    fn discard_max(&self) -> usize {
        self.discard_min() + self.discard.len() - 1
    }

    fn cash_out_min(&self) -> usize {
        self.discard_max() + 1
    }

    fn cash_out_max(&self) -> usize {
        self.cash_out_min() + self.cash_out.len() - 1
    }

    fn buy_joker_min(&self) -> usize {
        self.cash_out_max() + 1
    }

    fn buy_joker_max(&self) -> usize {
        self.buy_joker_min() + self.buy_joker.len() - 1
    }

    fn next_round_min(&self) -> usize {
        self.buy_joker_max() + 1
    }

    fn next_round_max(&self) -> usize {
        self.next_round_min() + self.next_round.len() - 1
    }

    fn select_blind_min(&self) -> usize {
        self.next_round_max() + 1
    }

    fn select_blind_max(&self) -> usize {
        self.select_blind_min() + self.select_blind.len() - 1
    }

    fn buy_consumable_min(&self) -> usize {
        self.select_blind_max() + 1
    }

    fn buy_consumable_max(&self) -> usize {
        self.buy_consumable_min() + self.buy_consumable.len() - 1
    }

    fn use_consumable_min(&self) -> usize {
        self.buy_consumable_max() + 1
    }

    fn use_consumable_max(&self) -> usize {
        self.use_consumable_min() + self.use_consumable.len() - 1
    }

    fn apply_tarot_min(&self) -> usize {
        self.use_consumable_max() + 1
    }

    fn apply_tarot_max(&self) -> usize {
        self.apply_tarot_min()
    }

    // Not all actions are always legal, by default all actions
    // are masked out, but provide methods to unmask valid.
    pub(crate) fn unmask_select_card(&mut self, i: usize) -> Result<(), ActionSpaceError> {
        if i >= self.select_card.len() {
            return Err(ActionSpaceError::InvalidIndex);
        }
        self.select_card[i] = 1;
        Ok(())
    }

    pub(crate) fn unmask_move_card_left(&mut self, i: usize) -> Result<(), ActionSpaceError> {
        if i >= self.move_card_left.len() {
            return Err(ActionSpaceError::InvalidIndex);
        }
        self.move_card_left[i] = 1;
        Ok(())
    }

    pub(crate) fn unmask_move_card_right(&mut self, i: usize) -> Result<(), ActionSpaceError> {
        if i >= self.move_card_right.len() {
            return Err(ActionSpaceError::InvalidIndex);
        }
        self.move_card_right[i] = 1;
        Ok(())
    }

    pub(crate) fn unmask_play(&mut self) {
        self.play[0] = 1;
    }

    pub(crate) fn unmask_discard(&mut self) {
        self.discard[0] = 1;
    }

    pub(crate) fn unmask_cash_out(&mut self) {
        self.cash_out[0] = 1;
    }

    pub(crate) fn unmask_buy_joker(&mut self, i: usize) -> Result<(), ActionSpaceError> {
        if i >= self.buy_joker.len() {
            return Err(ActionSpaceError::InvalidIndex);
        }
        self.buy_joker[i] = 1;
        Ok(())
    }

    pub(crate) fn unmask_next_round(&mut self) {
        self.next_round[0] = 1;
    }

    pub(crate) fn unmask_select_blind(&mut self) {
        self.select_blind[0] = 1;
    }

    pub(crate) fn unmask_buy_consumable(&mut self, i: usize) -> Result<(), ActionSpaceError> {
        if i >= self.buy_consumable.len() {
            return Err(ActionSpaceError::InvalidIndex);
        }
        self.buy_consumable[i] = 1;
        Ok(())
    }

    pub(crate) fn unmask_use_consumable(&mut self, i: usize) -> Result<(), ActionSpaceError> {
        if i >= self.use_consumable.len() {
            return Err(ActionSpaceError::InvalidIndex);
        }
        self.use_consumable[i] = 1;
        Ok(())
    }

    pub(crate) fn unmask_apply_tarot(&mut self) {
        self.apply_tarot[0] = 1;
    }

    pub fn to_action(&self, index: usize, game: &Game) -> Result<Action, ActionSpaceError> {
        let vec = self.to_vec();
        if let Some(v) = vec.get(index) {
            if *v == 0 {
                return Err(ActionSpaceError::MaskedAction);
            }
        } else {
            return Err(ActionSpaceError::InvalidIndex);
        }
        match index {
            // Cannot reference runtime values in patterns, so this is workaround
            n if (self.select_card_min()..=self.select_card_max()).contains(&n) => game
                .available
                .card_from_index(index)
                .map(Action::SelectCard)
                .ok_or(ActionSpaceError::InvalidActionConversion),
            n if (self.move_card_left_min()..=self.move_card_left_max()).contains(&n) => {
                // Index shifted to right (+1), since leftmost card cannot move left
                let n_offset = n - self.move_card_left_min() + 1;
                game.available
                    .card_from_index(n_offset)
                    .map(|card| Action::MoveCard(MoveDirection::Left, card))
                    .ok_or(ActionSpaceError::InvalidActionConversion)
            }
            n if (self.move_card_right_min()..=self.move_card_right_max()).contains(&n) => {
                let n_offset = n - self.move_card_right_min();
                game.available
                    .card_from_index(n_offset)
                    .map(|card| Action::MoveCard(MoveDirection::Right, card))
                    .ok_or(ActionSpaceError::InvalidActionConversion)
            }
            n if (self.play_min()..=self.play_max()).contains(&n) => Ok(Action::Play()),
            n if (self.discard_min()..=self.discard_max()).contains(&n) => Ok(Action::Discard()),
            n if (self.cash_out_min()..=self.cash_out_max()).contains(&n) => {
                Ok(Action::CashOut(game.reward))
            }
            n if (self.buy_joker_min()..=self.buy_joker_max()).contains(&n) => {
                let n_offset = n - self.buy_joker_min();
                game.shop
                    .joker_from_index(n_offset)
                    .map(Action::BuyJoker)
                    .ok_or(ActionSpaceError::InvalidActionConversion)
            }
            n if (self.next_round_min()..=self.next_round_max()).contains(&n) => {
                Ok(Action::NextRound())
            }
            n if (self.select_blind_min()..=self.select_blind_max()).contains(&n) => {
                match game.blind {
                    Some(blind) => Ok(Action::SelectBlind(blind.next())),
                    None => Ok(Action::SelectBlind(Blind::Small)),
                }
            }
            n if (self.buy_consumable_min()..=self.buy_consumable_max()).contains(&n) => {
                let n_offset = n - self.buy_consumable_min();
                game.shop
                    .consumable_from_index(n_offset)
                    .map(Action::BuyConsumable)
                    .ok_or(ActionSpaceError::InvalidActionConversion)
            }
            n if (self.use_consumable_min()..=self.use_consumable_max()).contains(&n) => {
                let n_offset = n - self.use_consumable_min();
                game.consumables
                    .get(n_offset)
                    .cloned()
                    .map(Action::UseConsumable)
                    .ok_or(ActionSpaceError::InvalidActionConversion)
            }
            n if (self.apply_tarot_min()..=self.apply_tarot_max()).contains(&n) => {
                Ok(Action::ApplyTarot())
            }
            _ => Err(ActionSpaceError::InvalidActionConversion),
        }
    }

    pub fn to_vec(&self) -> Vec<usize> {
        [
            self.select_card.clone(),
            self.move_card_left.clone(),
            self.move_card_right.clone(),
            self.play.clone(),
            self.discard.clone(),
            self.cash_out.clone(),
            self.buy_joker.clone(),
            self.next_round.clone(),
            self.select_blind.clone(),
            self.buy_consumable.clone(),
            self.use_consumable.clone(),
            self.apply_tarot.clone(),
        ]
        .concat()
    }

    // True is all elements are masked
    pub fn is_empty(&self) -> bool {
        let vec = self.to_vec();
        (*vec.iter().min().unwrap() == 0) && (*vec.iter().max().unwrap() == 0)
    }
}

impl From<Config> for ActionSpace {
    fn from(c: Config) -> Self {
        ActionSpace {
            select_card: vec![0; c.available_max],
            move_card_left: vec![0; c.available_max - 1], // every card but leftmost can move left
            move_card_right: vec![0; c.available_max - 1], // every card but rightmost can move right
            play: vec![0; 1],
            discard: vec![0; 1],
            cash_out: vec![0; 1],
            buy_joker: vec![0; c.store_consumable_slots_max],
            next_round: vec![0; 1],
            select_blind: vec![0; 1],
            buy_consumable: vec![0; c.consumable_slots],
            use_consumable: vec![0; c.consumable_slots],
            apply_tarot: vec![0; 1],
        }
    }
}

// Generate an action space vector, masked based on current state
impl From<ActionSpace> for Vec<usize> {
    fn from(a: ActionSpace) -> Vec<usize> {
        [
            a.select_card,
            a.move_card_left,
            a.move_card_right,
            a.play,
            a.discard,
            a.cash_out,
            a.buy_joker,
            a.next_round,
            a.select_blind,
            a.buy_consumable,
            a.use_consumable,
            a.apply_tarot,
        ]
        .concat()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::stage::Blind;

    #[test]
    fn test_unmask() {
        let c = Config::default();
        let mut a = ActionSpace::from(c);

        a.unmask_select_card(1).unwrap();

        let v: Vec<usize> = Vec::from(a);
        dbg!(v.clone());
        assert_eq!(v[0], 0);
        assert_eq!(v[1], 1);
    }

    #[test]
    fn test_unmask_max_index() {
        let c = Config::default();
        let mut a = ActionSpace::from(c);

        a.unmask_select_card(23).unwrap();

        let v: Vec<usize> = Vec::from(a.clone());
        dbg!(v.clone());
        assert_eq!(v[0], 0);
        assert_eq!(v[23], 1);

        let res = a.unmask_select_card(24);
        assert!(res.is_err());
    }

    #[test]
    fn test_action_space_size() {
        let c = Config::default();
        let a = ActionSpace::from(c.clone());
        // 24 select + 23 move_left + 23 move_right + 1 play + 1 discard
        // + 1 cashout + 4 buy_joker + 1 next_round + 1 select_blind
        // + 2 buy_consumable + 2 use_consumable + 1 apply_tarot = 84
        assert_eq!(a.size(), 84);
        assert_eq!(a.to_vec().len(), 84);
    }

    #[test]
    fn test_index_to_action() {
        let mut g = Game::default();
        let space = g.gen_action_space();
        let space_vec = g.gen_action_space().to_vec();

        // Game hasn't started yet, so only valid action is select blind
        for b in space_vec.iter().rev().skip(1) {
            // skip last (select_blind at index 78) AND skip use_consumable/buy_consumable
            // Actually just check select_blind is unmasked
            let _ = b;
        }
        // select_blind is at index 78
        assert_eq!(space_vec[78], 1);
        let action = space.to_action(78, &g).expect("to action");
        assert_eq!(action, Action::SelectBlind(Blind::Small));
        g.handle_action(action).unwrap();

        // Game now in small blind, we can select, move, play, discard
        let space = g.gen_action_space();
        let space_vec = g.gen_action_space().to_vec();
        assert_eq!(space_vec[0], 1);

        // We can select first card
        assert_eq!(g.available.selected().len(), 0);
        let index = space.select_card_min();
        let action = space.to_action(index, &g).expect("to action");
        assert_eq!(
            action,
            Action::SelectCard(g.available.card_from_index(0).expect("first card"))
        );
        g.handle_action(action).unwrap();
        assert_eq!(g.available.selected().len(), 1);

        // Regenerate space, cannot select first card, can select second
        let space = g.gen_action_space();
        let space_vec = g.gen_action_space().to_vec();
        assert_eq!(space_vec[0], 0);
        assert_eq!(space_vec[1], 1);
        assert_eq!(g.available.selected().len(), 1);

        // Ensure select second is unmasked, convert to action and handle
        let index = space.select_card_min() + 1;
        let action = space.to_action(index, &g).expect("to action");
        assert_eq!(
            action,
            Action::SelectCard(g.available.card_from_index(1).expect("second card"))
        );
        g.handle_action(action).unwrap();
        assert_eq!(g.available.selected().len(), 2);

        // Regenerate space, cannot select first or second, can play and discard
        let space = g.gen_action_space();
        let space_vec = g.gen_action_space().to_vec();
        assert_eq!(space_vec[0], 0);
        assert_eq!(space_vec[1], 0);
        assert_eq!(g.available.selected().len(), 2);

        let index = space.play_min();
        let action_play = space.to_action(index, &g).expect("to action");
        assert_eq!(action_play, Action::Play());

        let index = space.discard_min();
        let action_discard = space.to_action(index, &g).expect("to action");
        assert_eq!(action_discard, Action::Discard());

        // Play
        g.handle_action(action_play).unwrap();
    }

    #[test]
    fn test_unmask_buy_consumable() {
        let c = Config::default();
        let mut a = ActionSpace::from(c);
        assert_eq!(a.buy_consumable[0], 0);
        a.unmask_buy_consumable(0).unwrap();
        assert_eq!(a.buy_consumable[0], 1);
        assert!(a.unmask_buy_consumable(2).is_err());
    }

    #[test]
    fn test_unmask_use_consumable() {
        let c = Config::default();
        let mut a = ActionSpace::from(c);
        assert_eq!(a.use_consumable[0], 0);
        a.unmask_use_consumable(0).unwrap();
        assert_eq!(a.use_consumable[0], 1);
        assert!(a.unmask_use_consumable(2).is_err());
    }
}
