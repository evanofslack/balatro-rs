use crate::action::MoveDirection;
use crate::card::Card;
use crate::error::GameError;
use itertools::Itertools;

/// Available is the set of cards drawn from deck and available for
/// moving, selecting, playing and discarding.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct Available {
    // Tuple (card, bool) where bool represents if card is selected or not
    cards: Vec<(Card, bool)>,
}

impl Available {
    pub(crate) fn select_card(&mut self, card: Card) -> Result<(), GameError> {
        if let Some((i, _)) = self.cards.iter().find_position(|(c, _a)| c.id == card.id) {
            self.cards[i].1 = true;
            Ok(())
        } else {
            Err(GameError::NoCardMatch)
        }
    }

    pub fn selected(&self) -> Vec<Card> {
        self.cards
            .iter()
            .filter(|(_c, a)| *a)
            .map(|(c, _a)| *c)
            .collect()
    }

    pub fn not_selected(&self) -> Vec<Card> {
        self.cards
            .iter()
            .filter(|(_, s)| !*s)
            .map(|(c, _)| *c)
            .collect()
    }

    pub(crate) fn card_from_index(&self, i: usize) -> Option<Card> {
        if i >= self.cards.len() {
            return None;
        }
        Some(self.cards[i].0)
    }

    pub(crate) fn remove_selected(&mut self) -> usize {
        let remove_count = self.selected().len();
        self.cards.retain(|(_c, a)| !*a);
        remove_count
    }

    pub(crate) fn move_card(
        &mut self,
        direction: MoveDirection,
        card: Card,
    ) -> Result<(), GameError> {
        if let Some((i, _)) = self.cards.iter().find_position(|(c, _)| c.id == card.id) {
            match direction {
                MoveDirection::Left => {
                    if i == 0 {
                        return Err(GameError::InvalidMoveDirection);
                    }
                    self.cards.swap(i, i - 1);
                    Ok(())
                }
                MoveDirection::Right => {
                    if i >= self.cards.len() - 1 {
                        return Err(GameError::InvalidMoveDirection);
                    }
                    self.cards.swap(i, i + 1);
                    Ok(())
                }
            }
        } else {
            Err(GameError::NoCardMatch)
        }
    }

    pub(crate) fn remove_by_id(&mut self, id: usize) {
        self.cards.retain(|(c, _)| c.id != id);
    }

    pub(crate) fn mutate_card<F: Fn(&mut Card)>(&mut self, id: usize, f: F) {
        if let Some((c, _)) = self.cards.iter_mut().find(|(c, _)| c.id == id) {
            f(c);
        }
    }

    pub(crate) fn empty(&mut self) {
        self.cards = Vec::new();
    }

    pub(crate) fn extend(&mut self, cards: Vec<Card>) {
        for c in cards {
            self.cards.push((c, false));
        }
    }

    pub fn cards(&self) -> Vec<Card> {
        self.cards.iter().map(|(c, _)| *c).collect()
    }

    pub(crate) fn cards_and_selected(&self) -> Vec<(Card, bool)> {
        self.cards.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{Suit, Value};

    #[test]
    fn test_select_card() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let king = Card::new(Value::King, Suit::Diamond);
        let mut a = Available::default();
        a.extend(vec![ace, king]);
        assert_eq!(a.selected().len(), 0);

        a.select_card(ace).unwrap();
        assert_eq!(a.selected().len(), 1);

        let selected = a.selected();
        assert_eq!(selected[0], ace);
        let not_selected = a.not_selected();
        assert_eq!(not_selected[0], king);
    }

    #[test]
    fn test_card_from_index() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let king = Card::new(Value::King, Suit::Diamond);
        let mut a = Available::default();
        assert_eq!(a.card_from_index(0), None);

        a.extend(vec![ace, king]);
        assert_eq!(a.card_from_index(0), Some(ace));
        assert_eq!(a.card_from_index(1), Some(king));
    }

    #[test]
    fn test_move_card() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let king = Card::new(Value::King, Suit::Diamond);
        let mut a = Available::default();
        a.extend(vec![ace, king]);
        assert_eq!(a.card_from_index(0), Some(ace));
        assert_eq!(a.card_from_index(1), Some(king));

        a.move_card(MoveDirection::Right, ace).unwrap();
        assert_eq!(a.card_from_index(0), Some(king));
        assert_eq!(a.card_from_index(1), Some(ace));

        let res = a.move_card(MoveDirection::Right, ace);
        assert!(res.is_err());
    }
}
