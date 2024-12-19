use crate::action::MoveDirection;
use crate::config::Config;
use crate::error::ActionSpaceError;
use pyo3::pyclass;

// Hard code a bounded action space.
// Given constraints:
// available max = 24
// store consumable slots max = 4
//
// 0-23: select_card
// 24-46: move_card (left)
// 47-69: move_card (right)
// 70: play
// 71: discard
// 72: cashout
// 73-76: buy joker
// 77: next round
// 78: select blind
//
// We end up with a vector of length 35 where each index
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
}

impl ActionSpace {
    pub fn size(&self) -> usize {
        return self.select_card.len()
            + self.move_card_left.len()
            + self.move_card_right.len()
            + self.play.len()
            + self.discard.len()
            + self.cash_out.len()
            + self.buy_joker.len()
            + self.next_round.len()
            + self.select_blind.len();
    }

    // Not all actions are always legal, by default all actions
    // are masked out, but provide methods to unmask valid.
    pub fn unmask_select_card(&mut self, i: usize) -> Result<(), ActionSpaceError> {
        if i >= self.select_card.len() {
            return Err(ActionSpaceError::InvalidIndex);
        }
        self.select_card[i] = 1;
        return Ok(());
    }

    pub fn unmask_move_card_left(&mut self, i: usize) -> Result<(), ActionSpaceError> {
        if i >= self.move_card_left.len() {
            return Err(ActionSpaceError::InvalidIndex);
        }
        self.move_card_left[i] = 1;
        return Ok(());
    }

    pub fn unmask_move_card_right(&mut self, i: usize) -> Result<(), ActionSpaceError> {
        if i >= self.move_card_right.len() {
            return Err(ActionSpaceError::InvalidIndex);
        }
        self.move_card_right[i] = 1;
        return Ok(());
    }

    pub fn unmask_play(&mut self) {
        self.play[0] = 1;
    }

    pub fn unmask_discard(&mut self) {
        self.discard[0] = 1;
    }

    pub fn unmask_cash_out(&mut self) {
        self.cash_out[0] = 1;
    }

    pub fn unmask_buy_joker(&mut self, i: usize) -> Result<(), ActionSpaceError> {
        if i >= self.buy_joker.len() {
            return Err(ActionSpaceError::InvalidIndex);
        }
        self.buy_joker[i] = 1;
        return Ok(());
    }

    pub fn unmask_next_round(&mut self) {
        self.next_round[0] = 1;
    }

    pub fn unmask_select_blind(&mut self) {
        self.select_blind[0] = 1;
    }
}

impl From<Config> for ActionSpace {
    fn from(c: Config) -> Self {
        return ActionSpace {
            select_card: vec![0; c.available_max],
            move_card_left: vec![0; c.available_max - 1], // every card but leftmost can move left
            move_card_right: vec![0; c.available_max - 1], // every card but rightmost can move right
            play: vec![0; 1],
            discard: vec![0; 1],
            cash_out: vec![0; 1],
            buy_joker: vec![0; c.store_consumable_slots_max],
            next_round: vec![0; 1],
            select_blind: vec![0; 1],
        };
    }
}

// Generate an action space vector, masked based on current state
impl From<ActionSpace> for Vec<usize> {
    fn from(a: ActionSpace) -> Vec<usize> {
        return [
            a.select_card,
            a.move_card,
            a.play,
            a.discard,
            a.cash_out,
            a.buy_joker,
            a.next_round,
            a.select_blind,
        ]
        .concat();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

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
}
