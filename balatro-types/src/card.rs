#[cfg(feature = "python")]
use pyo3::pyclass;
use std::fmt;
use strum::{EnumIter, EnumString};

/// Card rank or value.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Copy, Hash, EnumIter, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Value {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Value {
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

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Value::Two => "Two",
            Value::Three => "Three",
            Value::Four => "Four",
            Value::Five => "Five",
            Value::Six => "Six",
            Value::Seven => "Seven",
            Value::Eight => "Eight",
            Value::Nine => "Nine",
            Value::Ten => "Ten",
            Value::Jack => "Jack",
            Value::Queen => "Queen",
            Value::King => "King",
            Value::Ace => "Ace",
        };
        write!(f, "{s}")
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
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Copy, Hash, EnumIter, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Suit {
    Spade,
    Club,
    Heart,
    Diamond,
}

impl Suit {
    pub fn unicode(&self) -> &str {
        match self {
            Self::Spade => "\u{2660}",
            Self::Club => "\u{2663}",
            Self::Heart => "\u{2665}",
            Self::Diamond => "\u{2666}",
        }
    }
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Suit::Spade => "Spades",
            Suit::Club => "Clubs",
            Suit::Heart => "Hearts",
            Suit::Diamond => "Diamonds",
        };
        write!(f, "{s}")
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

/// Enum for card enhancements.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Copy, Hash, EnumIter, EnumString)]
#[strum(ascii_case_insensitive)]
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

/// Enum for card seals.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Copy, Hash, EnumIter, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Seal {
    Gold,
    Red,
    Blue,
    Purple,
}

/// Enum for card/joker/consumable editions.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(
    PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Copy, Hash, Default, EnumIter, EnumString,
)]
#[strum(ascii_case_insensitive)]
pub enum Edition {
    #[default]
    Base,
    Foil,
    Holographic,
    Polychrome,
    Negative,
}

impl Edition {
    /// Playing cards can be Foil/Holographic/Polychrome, but never Negative.
    pub fn valid_for_card(&self) -> bool {
        !matches!(self, Edition::Negative)
    }

    /// Jokers can carry any edition, including Negative (+1 Joker slot).
    pub fn valid_for_joker(&self) -> bool {
        true
    }

    /// Tarot/Planet/Spectral consumables can only be Base or Negative
    /// (+1 consumable slot), not Foil/Holographic/Polychrome.
    pub fn valid_for_consumable(&self) -> bool {
        matches!(self, Edition::Base | Edition::Negative)
    }

    /// Save-file id for this edition.
    pub fn id(&self) -> &'static str {
        match self {
            Self::Base => "e_base",
            Self::Foil => "e_foil",
            Self::Holographic => "e_holo",
            Self::Polychrome => "e_polychrome",
            Self::Negative => "e_negative",
        }
    }

    /// Parses a save-file id into an `Edition`. `e_negative_consumable`
    /// also maps to `Negative`.
    pub fn from_id(s: &str) -> Option<Self> {
        match s {
            "e_base" => Some(Self::Base),
            "e_foil" => Some(Self::Foil),
            "e_holo" => Some(Self::Holographic),
            "e_polychrome" => Some(Self::Polychrome),
            "e_negative" | "e_negative_consumable" => Some(Self::Negative),
            _ => None,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Hash, Debug)]
pub struct Card {
    pub value: Value,
    pub suit: Suit,
    pub enhancement: Option<Enhancement>,
    pub edition: Edition,
    pub seal: Option<Seal>,
}

impl Card {
    pub fn new(value: Value, suit: Suit) -> Self {
        Self {
            value,
            suit,
            enhancement: None,
            edition: Edition::Base,
            seal: None,
        }
    }

    pub fn is_face_card(&self) -> bool {
        matches!(self.value, Value::Jack | Value::Queen | Value::King)
    }

    pub fn is_even(&self) -> bool {
        self.value != Value::Ace && !self.is_face_card() && (self.value as u16).is_multiple_of(2)
    }

    pub fn is_odd(&self) -> bool {
        self.value == Value::Ace || (!self.is_face_card() && !(self.value as u16).is_multiple_of(2))
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

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} of {}", self.value, self.suit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn test_value_count() {
        assert_eq!(Value::iter().count(), 13);
    }

    #[test]
    fn test_suit_count() {
        assert_eq!(Suit::iter().count(), 4);
    }

    #[test]
    fn test_enhancement_count() {
        assert_eq!(Enhancement::iter().count(), 8);
    }

    #[test]
    fn test_seal_count() {
        assert_eq!(Seal::iter().count(), 4);
    }

    #[test]
    fn test_edition_count() {
        assert_eq!(Edition::iter().count(), 5);
    }

    #[test]
    fn test_edition_id_round_trip() {
        for e in Edition::iter() {
            assert_eq!(Edition::from_id(e.id()), Some(e));
        }
    }

    #[test]
    fn test_edition_negative_consumable_alias() {
        assert_eq!(
            Edition::from_id("e_negative_consumable"),
            Some(Edition::Negative)
        );
    }

    #[test]
    fn test_edition_gating() {
        assert!(!Edition::Negative.valid_for_card());
        assert!(Edition::Foil.valid_for_card());
        assert!(Edition::Negative.valid_for_joker());
        assert!(Edition::Negative.valid_for_consumable());
        assert!(!Edition::Foil.valid_for_consumable());
    }

    #[test]
    fn test_card_chips() {
        let ace = Card::new(Value::Ace, Suit::Spade);
        assert_eq!(ace.chips(), 11);
        let king = Card::new(Value::King, Suit::Heart);
        assert_eq!(king.chips(), 10);
    }

    #[test]
    fn test_card_is_face_card() {
        let king = Card::new(Value::King, Suit::Spade);
        assert!(king.is_face_card());
        let two = Card::new(Value::Two, Suit::Spade);
        assert!(!two.is_face_card());
    }

    #[test]
    fn test_card_wild_matches_suit() {
        let mut wild = Card::new(Value::Two, Suit::Spade);
        wild.enhancement = Some(Enhancement::Wild);
        assert!(wild.matches_suit(Suit::Diamond));
    }

    #[test]
    fn test_value_from_str_round_trip() {
        for value in Value::iter() {
            assert_eq!(format!("{value:?}").parse::<Value>(), Ok(value));
        }
    }

    #[test]
    fn test_suit_from_str_case_insensitive() {
        assert_eq!("spade".parse::<Suit>(), Ok(Suit::Spade));
        assert_eq!("SPADE".parse::<Suit>(), Ok(Suit::Spade));
    }

    #[test]
    fn test_enhancement_seal_edition_from_str() {
        assert_eq!("wild".parse::<Enhancement>(), Ok(Enhancement::Wild));
        assert_eq!("gold".parse::<Seal>(), Ok(Seal::Gold));
        assert_eq!("negative".parse::<Edition>(), Ok(Edition::Negative));
    }

    #[test]
    fn test_value_from_str_invalid() {
        assert!("NotAValue".parse::<Value>().is_err());
    }
}
