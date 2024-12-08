use crate::core::card::{Card, Suit, Value};
use rand::{seq::SliceRandom, thread_rng};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        Self { cards: Vec::new() }
    }
    pub fn remove(&mut self, c: &Card) -> bool {
        if let Some(pos) = self.cards.iter().position(|x| x == c) {
            self.cards.remove(pos);
            return true;
        }
        return false;
    }
    pub(crate) fn draw(&mut self, n: usize) -> Option<Vec<Card>> {
        if self.cards.len() < n {
            return None;
        }
        return Some(self.cards.drain(0..n).collect());
    }
    pub(crate) fn len(&self) -> usize {
        self.cards.len()
    }

    pub(crate) fn shuffle(&mut self) {
        self.cards.shuffle(&mut thread_rng());
    }

    pub(crate) fn append(&mut self, other: &mut Vec<Card>) {
        self.cards.append(other);
    }
    // // Loops through cards, assigning index to each equal to index in deck
    // pub(crate) fn index_cards(&mut self) {
    //     let mut i = 0;
    //     for card in &mut self.cards {
    //         card.index = Some(i);
    //         i += 1;
    //     }
    // }
}

impl Default for Deck {
    fn default() -> Self {
        let mut cards: Vec<Card> = Vec::new();
        for v in &Value::values() {
            for s in &Suit::suits() {
                let c = Card::new(*v, *s);
                cards.push(c);
            }
        }
        Self { cards }
    }
}
