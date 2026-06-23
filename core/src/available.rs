use crate::action::{MoveDirection, SortBy};
use crate::card::{Card, Enhancement, Suit};
use crate::error::GameError;
use itertools::Itertools;
use std::cmp::Ordering;

fn suit_order(card: &Card) -> u8 {
    if card.enhancement == Some(Enhancement::Wild) {
        return 4;
    }
    match card.suit {
        Suit::Spade => 0,
        Suit::Heart => 1,
        Suit::Club => 2,
        Suit::Diamond => 3,
    }
}

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

    pub(crate) fn deselect_card(&mut self, card: Card) -> Result<(), GameError> {
        if let Some((i, _)) = self.cards.iter().find_position(|(c, _a)| c.id == card.id) {
            self.cards[i].1 = false;
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

    pub(crate) fn sort(&mut self, sort_by: SortBy) {
        match sort_by {
            SortBy::Rank => self.sort_by_rank(),
            SortBy::Suit => self.sort_by_suit(),
        }
    }

    fn sort_by_rank(&mut self) {
        self.cards.sort_by(|(a, _), (b, _)| {
            let a_stone = a.enhancement == Some(Enhancement::Stone);
            let b_stone = b.enhancement == Some(Enhancement::Stone);
            match (a_stone, b_stone) {
                (true, false) => Ordering::Greater,
                (false, true) => Ordering::Less,
                _ => b
                    .value
                    .cmp(&a.value)
                    .then_with(|| suit_order(a).cmp(&suit_order(b))),
            }
        });
    }

    fn sort_by_suit(&mut self) {
        self.cards.sort_by(|(a, _), (b, _)| {
            suit_order(a)
                .cmp(&suit_order(b))
                .then_with(|| b.value.cmp(&a.value))
        });
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

    #[test]
    fn test_sort_by_rank() {
        let two = Card::new(Value::Two, Suit::Heart);
        let ace = Card::new(Value::Ace, Suit::Spade);
        let king = Card::new(Value::King, Suit::Diamond);
        let mut stone = Card::new(Value::Five, Suit::Club);
        stone.enhancement = Some(Enhancement::Stone);

        let mut a = Available::default();
        a.extend(vec![two, stone, king, ace]);
        a.sort(SortBy::Rank);

        let cards = a.cards();
        assert_eq!(cards[0].value, Value::Ace);
        assert_eq!(cards[1].value, Value::King);
        assert_eq!(cards[2].value, Value::Two);
        assert_eq!(cards[3].enhancement, Some(Enhancement::Stone));
    }

    #[test]
    fn test_sort_by_suit() {
        let spade = Card::new(Value::Two, Suit::Spade);
        let heart = Card::new(Value::King, Suit::Heart);
        let club = Card::new(Value::Ace, Suit::Club);
        let diamond = Card::new(Value::Three, Suit::Diamond);
        let mut wild = Card::new(Value::Five, Suit::Heart);
        wild.enhancement = Some(Enhancement::Wild);

        let mut a = Available::default();
        a.extend(vec![diamond, wild, club, heart, spade]);
        a.sort(SortBy::Suit);

        let cards = a.cards();
        assert_eq!(cards[0].suit, Suit::Spade);
        assert_eq!(cards[1].suit, Suit::Heart);
        assert_eq!(cards[2].suit, Suit::Club);
        assert_eq!(cards[3].suit, Suit::Diamond);
        assert_eq!(cards[4].enhancement, Some(Enhancement::Wild));
    }
}
