#[cfg(feature = "colored")]
use colored::Colorize;
#[cfg(feature = "python")]
use pyo3::pyclass;
use std::{
    fmt,
    sync::atomic::{AtomicUsize, Ordering},
};

// Useful balatro docs: https://balatrogame.fandom.com/wiki/Card_Ranks

pub use balatro_types::{Edition, Enhancement, Seal, Suit, Value};

// Each card gets a unique id. Not sure this is strictly
// necessary but it makes identifying otherwise identical cards
// possible (i.e. for trashing, reordering, etc)
static CARD_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

// Deserializing a Game doesn't allocate ids, so the counter has no idea
// which ones are already in use in a loaded save. Call this with the
// highest id found in the loaded state before minting any new cards.
pub(crate) fn ensure_id_counter_past(max_seen: usize) {
    CARD_ID_COUNTER.fetch_max(max_seen + 1, Ordering::SeqCst);
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass)]
#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Hash)]
pub struct Card {
    pub value: Value,
    pub suit: Suit,
    pub id: usize,
    pub edition: Edition,
    pub enhancement: Option<Enhancement>,
    pub seal: Option<Seal>,
}

impl Card {
    pub fn new(value: Value, suit: Suit) -> Self {
        let id = CARD_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        Self {
            value,
            suit,
            id,
            edition: Edition::Base,
            enhancement: None,
            seal: None,
        }
    }

    pub fn is_face_card(&self) -> bool {
        matches!(self.value, Value::Jack | Value::Queen | Value::King)
    }

    pub(crate) fn is_even_impl(&self, is_face: bool) -> bool {
        self.value != Value::Ace && !is_face && (self.value as u16).is_multiple_of(2)
    }

    pub(crate) fn is_odd_impl(&self, is_face: bool) -> bool {
        self.value == Value::Ace || (!is_face && !(self.value as u16).is_multiple_of(2))
    }

    pub fn is_even(&self) -> bool {
        self.is_even_impl(self.is_face_card())
    }

    pub fn is_odd(&self) -> bool {
        self.is_odd_impl(self.is_face_card())
    }

    pub fn matches_suit(&self, suit: Suit) -> bool {
        self.enhancement == Some(Enhancement::Wild) || self.suit == suit
    }

    pub fn chips(&self) -> usize {
        match self.value {
            Value::Two => 2,
            Value::Three => 3,
            Value::Four => 4,
            Value::Five => 5,
            Value::Six => 6,
            Value::Seven => 7,
            Value::Eight => 8,
            Value::Nine => 9,
            Value::Ten => 10,
            Value::Jack => 10,
            Value::Queen => 10,
            Value::King => 10,
            Value::Ace => 11,
        }
    }
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[cfg(feature = "colored")]
        let suit = match self.suit {
            Suit::Spade => self.suit.unicode().bold(),
            Suit::Club => self.suit.unicode().green().bold(),
            Suit::Heart => self.suit.unicode().red().bold(),
            Suit::Diamond => self.suit.unicode().blue().bold(),
        };
        #[cfg(not(feature = "colored"))]
        let suit = self.suit.unicode();
        write!(f, "Card({}{})", char::from(self.value), suit)
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[cfg(feature = "colored")]
        let suit = match self.suit {
            Suit::Spade => self.suit.unicode().bold(),
            Suit::Club => self.suit.unicode().green().bold(),
            Suit::Heart => self.suit.unicode().red().bold(),
            Suit::Diamond => self.suit.unicode().blue().bold(),
        };
        #[cfg(not(feature = "colored"))]
        let suit = self.suit.unicode();
        write!(f, "{}{}", char::from(self.value), suit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_suit() {
        let diamond = Card::new(Value::Ace, Suit::Diamond);
        assert!(diamond.matches_suit(Suit::Diamond));
        assert!(!diamond.matches_suit(Suit::Heart));

        let mut wild = Card::new(Value::Two, Suit::Club);
        wild.enhancement = Some(Enhancement::Wild);
        assert!(wild.matches_suit(Suit::Club));
        assert!(wild.matches_suit(Suit::Spade));
        assert!(wild.matches_suit(Suit::Heart));
        assert!(wild.matches_suit(Suit::Diamond));
    }

    #[test]
    fn test_constructor() {
        let c = Card::new(Value::King, Suit::Heart);
        assert_eq!(Value::King, c.value);
        assert_eq!(Suit::Heart, c.suit);
    }

    #[test]
    fn test_face() {
        let king = Card::new(Value::King, Suit::Heart);
        assert!(king.is_face_card());
        let jack = Card::new(Value::Jack, Suit::Spade);
        assert!(jack.is_face_card());
        let queen = Card::new(Value::Queen, Suit::Club);
        assert!(queen.is_face_card());
        let ten = Card::new(Value::Ten, Suit::Heart);
        assert!(!ten.is_face_card());
        let two = Card::new(Value::Two, Suit::Diamond);
        assert!(!two.is_face_card());
    }

    #[test]
    fn test_even_odd() {
        // ace is odd
        let ace = Card::new(Value::Ace, Suit::Spade);
        assert!(!ace.is_even());
        assert!(ace.is_odd());

        // two is even
        let two = Card::new(Value::Two, Suit::Diamond);
        assert!(two.is_even());
        assert!(!two.is_odd());

        // three is odd
        let three = Card::new(Value::Three, Suit::Heart);
        assert!(!three.is_even());
        assert!(three.is_odd());

        // ten is even
        let ten = Card::new(Value::Ten, Suit::Heart);
        assert!(ten.is_even());
        assert!(!ten.is_odd());

        //king is neither odd nor even
        let king = Card::new(Value::King, Suit::Club);
        assert!(!king.is_even());
        assert!(!king.is_odd());
    }

    // Card's `id` is part of its derived Eq/Ord/Hash
    // so two structurally-identical cards with different ids are NOT equal.
    #[test]
    fn test_card_equality_is_id_sensitive() {
        let a = Card::new(Value::Ace, Suit::Spade);
        let b = Card::new(Value::Ace, Suit::Spade);
        assert_ne!(a.id, b.id, "Card::new must assign distinct ids");
        assert_eq!(a.value, b.value);
        assert_eq!(a.suit, b.suit);
        assert_ne!(
            a, b,
            "cards with identical value/suit but different ids must not be equal"
        );
        assert_eq!(a, a, "a card must equal itself");
    }
}
