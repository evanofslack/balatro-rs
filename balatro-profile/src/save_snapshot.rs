use std::fmt;

use balatro_jkr::LuaValue;
use balatro_types::{Card, Consumable, DeckVariant, Edition, Jokers, Stake, Suit, Value};

use crate::error::ProfileError;
use crate::fmt_num::format_number;
use crate::lua_ext::{as_num, as_str, get, require};

/// A typed view of an in-progress run, parsed from `save.jkr`. First pass —
/// no export, no stateful joker counters.
#[derive(Debug, Clone, PartialEq)]
pub struct SaveSnapshot {
    pub stake: Stake,
    pub deck: DeckVariant,
    pub dollars: i64,
    /// Always `None` — no confirmed field to parse it from yet.
    pub jokers: Vec<(Jokers, Option<Edition>)>,
    pub deck_cards: Vec<Card>,
    pub hand: Vec<Card>,
    pub discard: Vec<Card>,
    pub consumables: Vec<Consumable>,
}

impl SaveSnapshot {
    pub fn from_lua(save: &LuaValue) -> Result<SaveSnapshot, ProfileError> {
        let game = require(save, "GAME")?;
        let back = require(save, "BACK")?;
        let card_areas = require(save, "cardAreas")?;

        let stake_n = as_num(require(game, "stake")?).unwrap_or(0.0) as u8;
        let stake = Stake::from_id(stake_n)
            .ok_or_else(|| ProfileError::UnknownId(format!("stake {stake_n}")))?;

        let deck_key = as_str(require(back, "key")?).unwrap_or("");
        let deck = DeckVariant::from_id(deck_key)
            .ok_or_else(|| ProfileError::UnknownId(deck_key.to_string()))?;

        let dollars = as_num(require(game, "dollars")?).unwrap_or(0.0) as i64;

        Ok(SaveSnapshot {
            stake,
            deck,
            dollars,
            jokers: pile_cards(card_areas, "jokers")
                .filter_map(Jokers::from_id)
                .map(|j| (j, None))
                .collect(),
            deck_cards: pile_playing_cards(card_areas, "deck"),
            hand: pile_playing_cards(card_areas, "hand"),
            discard: pile_playing_cards(card_areas, "discard"),
            consumables: pile_cards(card_areas, "consumeables")
                .filter_map(Consumable::from_id)
                .collect(),
        })
    }

    /// A short overview — see [`SaveSnapshotSummary`].
    pub fn summary(&self) -> SaveSnapshotSummary<'_> {
        SaveSnapshotSummary(self)
    }
}

impl fmt::Display for SaveSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}, {:?} Stake", self.deck.name(), self.stake)?;
        writeln!(f, "Dollars: {}", format_number(self.dollars))?;

        writeln!(f, "\nJokers ({}):", self.jokers.len())?;
        for (j, edition) in &self.jokers {
            match edition {
                Some(e) => writeln!(f, "  {} ({e:?})", j.name())?,
                None => writeln!(f, "  {}", j.name())?,
            }
        }

        writeln!(f, "\nConsumables ({}):", self.consumables.len())?;
        for c in &self.consumables {
            writeln!(f, "  {}", c.name())?;
        }

        writeln!(f, "\nHand ({}):", self.hand.len())?;
        for c in &self.hand {
            writeln!(f, "  {c}")?;
        }

        writeln!(f, "\nDiscard ({}):", self.discard.len())?;
        for c in &self.discard {
            writeln!(f, "  {c}")?;
        }

        write!(f, "\nDeck ({} cards remaining)", self.deck_cards.len())?;

        Ok(())
    }
}

/// Short overview of a [`SaveSnapshot`].
pub struct SaveSnapshotSummary<'a>(pub &'a SaveSnapshot);

impl fmt::Display for SaveSnapshotSummary<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.0;
        writeln!(f, "{}, {:?} Stake", s.deck.name(), s.stake)?;
        writeln!(f, "Dollars: {}", format_number(s.dollars))?;
        writeln!(f, "Jokers: {}", s.jokers.len())?;
        writeln!(f, "Consumables: {}", s.consumables.len())?;
        writeln!(f, "Hand: {} cards", s.hand.len())?;
        writeln!(f, "Discard pile: {} cards", s.discard.len())?;
        write!(f, "Remaining deck: {} cards", s.deck_cards.len())?;
        Ok(())
    }
}

/// `save_fields.center` ids for each card in a pile.
fn pile_cards<'a>(card_areas: &'a LuaValue, pile: &str) -> impl Iterator<Item = &'a str> {
    get(card_areas, pile)
        .and_then(|p| get(p, "cards"))
        .and_then(balatro_jkr_table_values)
        .into_iter()
        .flatten()
        .filter_map(|card| get(card, "save_fields"))
        .filter_map(|sf| get(sf, "center"))
        .filter_map(as_str)
}

fn pile_playing_cards(card_areas: &LuaValue, pile: &str) -> Vec<Card> {
    let cards = get(card_areas, pile).and_then(|p| get(p, "cards"));
    let Some(cards) = cards.and_then(balatro_jkr_table_values) else {
        return vec![];
    };
    cards
        .filter_map(|card| get(card, "save_fields"))
        .filter_map(|sf| get(sf, "card"))
        .filter_map(as_str)
        .filter_map(parse_playing_card_code)
        .collect()
}

/// `LuaValue::Table` values, ignoring keys.
fn balatro_jkr_table_values(v: &LuaValue) -> Option<impl Iterator<Item = &LuaValue>> {
    match v {
        LuaValue::Table(entries) => Some(entries.iter().map(|(_, v)| v)),
        _ => None,
    }
}

/// Parses a playing-card code like `"D_7"` or `"C_T"`. No enhancement/seal
/// support yet.
fn parse_playing_card_code(code: &str) -> Option<Card> {
    let (suit_ch, rank) = code.split_once('_')?;
    let suit = match suit_ch {
        "S" => Suit::Spade,
        "H" => Suit::Heart,
        "D" => Suit::Diamond,
        "C" => Suit::Club,
        _ => return None,
    };
    let value = match rank {
        "2" => Value::Two,
        "3" => Value::Three,
        "4" => Value::Four,
        "5" => Value::Five,
        "6" => Value::Six,
        "7" => Value::Seven,
        "8" => Value::Eight,
        "9" => Value::Nine,
        "T" => Value::Ten,
        "J" => Value::Jack,
        "Q" => Value::Queen,
        "K" => Value::King,
        "A" => Value::Ace,
        _ => return None,
    };
    Some(Card::new(value, suit))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fixture(name: &str) -> LuaValue {
        let bytes = fs::read(format!("tests/fixtures/real/{name}")).expect("fixture file present");
        balatro_jkr::decode(&bytes).expect("valid jkr")
    }

    #[test]
    fn test_parses_real_1_save_white_red() {
        let save = fixture("1-save.jkr");
        let s = SaveSnapshot::from_lua(&save).expect("parses");
        assert_eq!(s.stake, Stake::White);
        assert_eq!(s.deck, DeckVariant::Red);
        assert_eq!(s.deck_cards.len() + s.hand.len() + s.discard.len(), 52);
        assert!(s.jokers.iter().any(|(j, _)| j.id() == "j_scary_face"));
    }

    #[test]
    fn test_parses_real_yellow_deck_red_stake() {
        let save = fixture("save-yellowdeck-redstake.jkr");
        let s = SaveSnapshot::from_lua(&save).expect("parses");
        assert_eq!(s.stake, Stake::Red);
        assert_eq!(s.deck, DeckVariant::Yellow);
    }

    #[test]
    fn test_parses_real_red_deck_orange_stake() {
        let save = fixture("save-reddeck-orangestake.jkr");
        let s = SaveSnapshot::from_lua(&save).expect("parses");
        assert_eq!(s.stake, Stake::Orange);
        assert_eq!(s.deck, DeckVariant::Red);
    }

    #[test]
    fn test_parse_playing_card_code() {
        assert_eq!(
            parse_playing_card_code("D_7"),
            Some(Card::new(Value::Seven, Suit::Diamond))
        );
        assert_eq!(
            parse_playing_card_code("C_T"),
            Some(Card::new(Value::Ten, Suit::Club))
        );
        assert_eq!(parse_playing_card_code("not_a_code"), None);
    }
}
