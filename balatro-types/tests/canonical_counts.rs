use balatro_types::*;
use strum::IntoEnumIterator;

#[test]
fn canonical_roster_counts() {
    assert_eq!(Jokers::iter().count(), 150, "Jokers");
    assert_eq!(Tarot::iter().count(), 22, "Tarot");
    assert_eq!(Planets::iter().count(), 12, "Planets");
    assert_eq!(Spectral::iter().count(), 18, "Spectral");
    assert_eq!(Voucher::iter().count(), 32, "Voucher");
    assert_eq!(Tag::iter().count(), 24, "Tag");
    assert_eq!(DeckVariant::iter().count(), 15, "DeckVariant");
    assert_eq!(Stake::iter().count(), 8, "Stake");
    assert_eq!(BossBlind::iter().count(), 28, "BossBlind");
    assert_eq!(HandRank::iter().count(), 13, "HandRank");
    assert_eq!(Value::iter().count(), 13, "Value");
    assert_eq!(Suit::iter().count(), 4, "Suit");
    assert_eq!(Enhancement::iter().count(), 8, "Enhancement");
    assert_eq!(Seal::iter().count(), 4, "Seal");
    assert_eq!(Edition::iter().count(), 5, "Edition");
    assert_eq!(Blind::iter().count(), 3, "Blind");
}
