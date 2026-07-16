use crate::card::{Card, Suit, Value};
use rand::{seq::SliceRandom, Rng};
use strum::IntoEnumIterator;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        Self { cards: Vec::new() }
    }
    pub(crate) fn draw(&mut self, n: usize) -> Option<Vec<Card>> {
        if self.cards.len() < n {
            return None;
        }
        Some(self.cards.drain(0..n).collect())
    }
    pub(crate) fn len(&self) -> usize {
        self.cards.len()
    }

    pub(crate) fn shuffle(&mut self, rng: &mut impl Rng) {
        self.cards.shuffle(rng);
    }

    pub(crate) fn append(&mut self, other: &mut Vec<Card>) {
        self.cards.append(other);
    }

    pub(crate) fn extend(&mut self, other: Vec<Card>) {
        self.cards.extend(other);
    }

    pub(crate) fn push(&mut self, card: Card) {
        self.cards.push(card);
    }

    pub(crate) fn remove_by_id(&mut self, id: usize) {
        self.cards.retain(|c| c.id != id);
    }

    pub(crate) fn mutate_card<F: Fn(&mut Card)>(&mut self, id: usize, f: F) {
        if let Some(c) = self.cards.iter_mut().find(|c| c.id == id) {
            f(c);
        }
    }

    pub fn cards(&self) -> Vec<Card> {
        self.cards.clone()
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
        for v in Value::iter() {
            for s in Suit::iter() {
                let c = Card::new(v, s);
                cards.push(c);
            }
        }
        Self { cards }
    }
}
