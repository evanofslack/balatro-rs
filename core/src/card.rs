#[cfg(feature = "colored")]
use colored::Colorize;
#[cfg(feature = "python")]
use pyo3::pyclass;
use std::{
    fmt,
    sync::atomic::{AtomicUsize, Ordering},
};

// Useful balatro docs: https://balatrogame.fandom.com/wiki/Card_Ranks

/// Card rank or value.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Copy, Hash)]
pub enum Value {
    Two = 0,
    Three = 1,
    Four = 2,
    Five = 3,
    Six = 4,
    Seven = 5,
    Eight = 6,
    Nine = 7,
    Ten = 8,
    Jack = 9,
    Queen = 10,
    King = 11,
    Ace = 12,
}

/// Constant of all the values.
/// This is what `Value::values()` returns
const VALUES: [Value; 13] = [
    Value::Two,
    Value::Three,
    Value::Four,
    Value::Five,
    Value::Six,
    Value::Seven,
    Value::Eight,
    Value::Nine,
    Value::Ten,
    Value::Jack,
    Value::Queen,
    Value::King,
    Value::Ace,
];

impl Value {
    pub const fn values() -> [Self; 13] {
        VALUES
    }

    pub fn next(self) -> Self {
        match self {
            Value::Two => Value::Three,
            Value::Three => Value::Four,
            Value::Four => Value::Five,
            Value::Five => Value::Six,
            Value::Six => Value::Seven,
            Value::Seven => Value::Eight,
            Value::Eight => Value::Nine,
            Value::Nine => Value::Ten,
            Value::Ten => Value::Jack,
            Value::Jack => Value::Queen,
            Value::Queen => Value::King,
            Value::King => Value::Ace,
            Value::Ace => Value::Two,
        }
    }
}

impl From<Value> for char {
    fn from(value: Value) -> Self {
        match value {
            Value::Two => '2',
            Value::Three => '3',
            Value::Four => '4',
            Value::Five => '5',
            Value::Six => '6',
            Value::Seven => '7',
            Value::Eight => '8',
            Value::Nine => '9',
            Value::Ten => 'T',
            Value::Jack => 'J',
            Value::Queen => 'Q',
            Value::King => 'K',
            Value::Ace => 'A',
        }
    }
}

/// Enum for the four different suits.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Copy, Hash)]
pub enum Suit {
    Spade = 0,
    Club = 1,
    Heart = 2,
    Diamond = 3,
}

/// All of the `Suit`'s. This is what `Suit::suits()` returns.
const SUITS: [Suit; 4] = [Suit::Spade, Suit::Club, Suit::Heart, Suit::Diamond];

impl Suit {
    pub const fn suits() -> [Self; 4] {
        SUITS
    }
    pub fn unicode(&self) -> &str {
        match self {
            Self::Spade => "♤",
            Self::Club => "♧",
            Self::Heart => "♡",
            Self::Diamond => "♢",
        }
    }
}

impl From<Suit> for char {
    fn from(value: Suit) -> Self {
        match value {
            Suit::Spade => 's',
            Suit::Club => 'c',
            Suit::Heart => 'h',
            Suit::Diamond => 'd',
        }
    }
}

/// Enum for card  enhancements
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Copy, Hash)]
pub enum Enhancement {
    Bonus,
    Mult,
    Wild,
    Glass,
    Steel,
    Stone,
    Gold,
    Lucky,
}

/// Enum for card  editions
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Copy, Hash)]
pub enum Edition {
    Base,
    Foil,
    Holographic,
    Polychrome,
    Negative,
}

/// Enum for card seals
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Copy, Hash)]
pub enum Seal {
    Gold,
    Red,
    Blue,
    Purple,
}

// Each card gets a unique id. Not sure this is strictly
// necessary but it makes identifying otherwise identical cards
// possible (i.e. for trashing, reordering, etc)
static CARD_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

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
    pub is_face_card: bool,
}

impl Card {
    pub fn new(value: Value, suit: Suit) -> Self {
        let id = CARD_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let is_face_card = value == Value::Jack || value == Value::Queen || value == Value::King;
        Self {
            value,
            suit,
            id,
            edition: Edition::Base,
            enhancement: None,
            seal: None,
            is_face_card,
        }
    }

    pub fn is_even(&self) -> bool {
        self.value != Value::Ace && !self.is_face_card && (self.value as u16).is_multiple_of(2)
    }

    pub fn is_odd(&self) -> bool {
        self.value == Value::Ace || !self.is_face_card && !(self.value as u16).is_multiple_of(2)
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
    fn test_constructor() {
        let c = Card::new(Value::King, Suit::Heart);
        assert_eq!(Value::King, c.value);
        assert_eq!(Suit::Heart, c.suit);
    }

    #[test]
    fn test_face() {
        let king = Card::new(Value::King, Suit::Heart);
        assert!(king.is_face_card);
        let two = Card::new(Value::Two, Suit::Diamond);
        assert!(!two.is_face_card);
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
}
