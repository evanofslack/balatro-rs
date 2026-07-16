#![cfg(feature = "python")]

use balatro_types::card::{Card, Suit, Value};
use balatro_types::joker::Jokers;
use balatro_types::pack::PackContent;
use balatro_types::planet::Planets;
use pyo3::prelude::*;

#[test]
fn simple_enum_roundtrips() {
    Python::with_gil(|py| {
        let suit = Suit::Heart;
        let obj = suit.into_pyobject(py).unwrap().into_any();
        let back: Suit = obj.extract().unwrap();
        assert_eq!(back, Suit::Heart);
    });
}

#[test]
fn card_struct_roundtrips() {
    Python::with_gil(|py| {
        let card = Card::new(Value::Ace, Suit::Spade);
        let obj = card.into_pyobject(py).unwrap().into_any();
        let back: Card = obj.extract().unwrap();
        assert_eq!(back, card);
    });
}

#[test]
fn planets_roundtrip() {
    Python::with_gil(|py| {
        let obj = Planets::Eris.into_pyobject(py).unwrap().into_any();
        let back: Planets = obj.extract().unwrap();
        assert_eq!(back, Planets::Eris);
    });
}

/// The risky case: `Jokers` is a 150-variant enum-of-structs. Confirms
/// pyo3's complex-enum pyclass support actually round-trips at runtime,
/// not just compiles.
#[test]
fn jokers_complex_enum_roundtrips() {
    Python::with_gil(|py| {
        let joker = Jokers::Perkeo(Default::default());
        let obj = joker.clone().into_pyobject(py).unwrap().into_any();
        let back: Jokers = obj.extract().unwrap();
        assert_eq!(back, joker);
        assert_eq!(back.name(), "Perkeo");

        let the_joker = Jokers::TheJoker(Default::default());
        let obj2 = the_joker.clone().into_pyobject(py).unwrap().into_any();
        let back2: Jokers = obj2.extract().unwrap();
        assert_eq!(back2, the_joker);
        assert_eq!(back2.cost(), 2);
    });
}

/// A complex enum whose variant wraps another complex enum (`PackContent`
/// wrapping `Jokers`) - one level deeper than the above.
#[test]
fn pack_content_wrapping_jokers_roundtrips() {
    Python::with_gil(|py| {
        let content = PackContent::Joker(Jokers::Triboulet(Default::default()));
        let obj = content.clone().into_pyobject(py).unwrap().into_any();
        let back: PackContent = obj.extract().unwrap();
        assert_eq!(back, content);
    });
}
