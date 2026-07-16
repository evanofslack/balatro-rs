use strum::EnumIter;

/// One of the 15 selectable starting decks, each with its own starting
/// modifier.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, EnumIter)]
pub enum DeckVariant {
    Red,
    Blue,
    Yellow,
    Green,
    Black,
    Magic,
    Nebula,
    Ghost,
    Abandoned,
    Checkered,
    Zodiac,
    Painted,
    Anaglyph,
    Plasma,
    Erratic,
}

impl DeckVariant {
    pub fn name(&self) -> &str {
        match self {
            Self::Red => "Red Deck",
            Self::Blue => "Blue Deck",
            Self::Yellow => "Yellow Deck",
            Self::Green => "Green Deck",
            Self::Black => "Black Deck",
            Self::Magic => "Magic Deck",
            Self::Nebula => "Nebula Deck",
            Self::Ghost => "Ghost Deck",
            Self::Abandoned => "Abandoned Deck",
            Self::Checkered => "Checkered Deck",
            Self::Zodiac => "Zodiac Deck",
            Self::Painted => "Painted Deck",
            Self::Anaglyph => "Anaglyph Deck",
            Self::Plasma => "Plasma Deck",
            Self::Erratic => "Erratic Deck",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::Red => "+1 discard every round",
            Self::Blue => "+1 hand every round",
            Self::Yellow => "Start with an extra $10",
            Self::Green => {
                "At end of round: gain $2 per remaining hand and $1 per remaining discard, earn no interest"
            }
            Self::Black => "+1 Joker slot, -1 hand every round",
            Self::Magic => "Start with the Crystal Ball voucher and 2 copies of The Fool",
            Self::Nebula => "Start with the Telescope voucher, -1 consumable slot",
            Self::Ghost => "Start with a Hex card, Spectral cards may appear in the shop",
            Self::Abandoned => "Start with no face cards in the deck",
            Self::Checkered => "Start with only Spades and Hearts in the deck",
            Self::Zodiac => {
                "Start with the Tarot Merchant, Planet Merchant, and Overstock vouchers"
            }
            Self::Painted => "+2 hand size, -1 Joker slot",
            Self::Anaglyph => "Gain a Double Tag after defeating each Boss Blind",
            Self::Plasma => "Balances Chips and Mult when scoring, Blind requirements are doubled",
            Self::Erratic => "Every card in the deck has a random rank and suit",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn test_deck_variant_count() {
        assert_eq!(DeckVariant::iter().count(), 15);
    }
}
