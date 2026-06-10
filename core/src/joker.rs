use crate::card::{Suit, Value};
use crate::effect::Effects;
use crate::game::Game;
use crate::hand::MadeHand;
#[cfg(feature = "python")]
use pyo3::pyclass;
use std::fmt;
use std::sync::{Arc, Mutex};
use strum::{EnumIter, IntoEnumIterator};

pub trait Joker: std::fmt::Debug + Clone {
    fn name(&self) -> String;
    fn blueprint_compatible(&self) -> bool {
        true
    }
    fn perishable_compatible(&self) -> bool {
        true
    }
    fn eternal_compatible(&self) -> bool {
        true
    }
    fn rarity(&self) -> Rarity;
    fn cost(&self) -> usize;
    fn sell_value(&self) -> usize {
        std::cmp::max(1, self.cost() / 2)
    }
    fn desc(&self) -> String;
    fn categories(&self) -> Vec<Categories>;
    fn effects(&self, game: &Game) -> Vec<Effects>;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Categories {
    MultPlus,
    MultMult,
    Chips,
    Economy,
    Retrigger,
    Effect,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
}

impl fmt::Display for Rarity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Common => {
                write!(f, "Common")
            }
            Self::Uncommon => {
                write!(f, "Uncommon")
            }
            Self::Rare => {
                write!(f, "Rare")
            }
            Self::Legendary => {
                write!(f, "Legendary")
            }
        }
    }
}

// We could pass around `Box<dyn Joker>` but it doesn't work so nice with pyo3 and serde.
// Since we know all variants (one for each joker), we define an enum that implements
// our `Joker` trait. This macro just reduces the amount of boilerplate we have to copy
// to match each joker and call its methods.
// It ends up creating an enum `Jokers` that contains each joker struct (where each struct impl `Joker`), and we impl `Joker`
// for `Jokers` enum by matching each case and calling underlying methods.
// https://stackoverflow.com/questions/63848427/using-enums-for-dynamic-polymorphism-in-rust/63849405#63849405
macro_rules! make_jokers {
    ($($x:ident), *) => {
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "python", pyclass(eq))]
        #[derive(Debug, Clone, EnumIter, Eq, PartialEq, Hash)]
        pub enum Jokers {
            $(
                $x($x),
            )*
        }

        impl Joker for Jokers {
            fn name(&self) -> String {
                match self {
                    $(
                        Jokers::$x(joker) => joker.name(),
                    )*
                }
            }
            fn blueprint_compatible(&self) -> bool {
                match self {
                    $(
                        Jokers::$x(joker) => joker.blueprint_compatible(),
                    )*
                }
            }
            fn perishable_compatible(&self) -> bool {
                match self {
                    $(
                        Jokers::$x(joker) => joker.perishable_compatible(),
                    )*
                }
            }
            fn eternal_compatible(&self) -> bool {
                match self {
                    $(
                        Jokers::$x(joker) => joker.eternal_compatible(),
                    )*
                }
            }
            fn rarity(&self) -> Rarity {
                match self {
                    $(
                        Jokers::$x(joker) => joker.rarity(),
                    )*
                }
            }
            fn cost(&self) -> usize {
                match self {
                    $(
                        Jokers::$x(joker) => joker.cost(),
                    )*
                }
            }
            fn sell_value(&self) -> usize {
                match self {
                    $(
                        Jokers::$x(joker) => joker.sell_value(),
                    )*
                }
            }
            fn desc(&self) -> String {
                match self {
                    $(
                        Jokers::$x(joker) => joker.desc(),
                    )*
                }
            }
            fn categories(&self) -> Vec<Categories> {
                match self {
                    $(
                        Jokers::$x(joker) => joker.categories(),
                    )*
                }
            }
            fn effects(&self, game: &Game) -> Vec<Effects> {
                match self {
                    $(
                        Jokers::$x(joker) => joker.effects(game),
                    )*
                }
            }
        }
    }
}

make_jokers!(
    TheJoker,
    GreedyJoker,
    LustyJoker,
    WrathfulJoker,
    GluttonousJoker,
    JollyJoker,
    ZanyJoker,
    MadJoker,
    CrazyJoker,
    DrollJoker,
    SlyJoker,
    WilyJoker,
    CleverJoker,
    DeviousJoker,
    CraftyJoker,
    HalfJoker,
    JokerStencil,
    //FourFingers,
    //Mime,
    //CreditCard,
    //CeremonialDagger,
    Banner,
    MysticSummit,
    //MarbleJoker,
    //LoyaltyCard,
    //8Ball,
    //Misprint,
    //Dusk,
    //RaisedFist,
    //ChaosTheClown,
    Fibonacci,
    //SteelJoker,
    ScaryFace,
    AbstractJoker,
    ShootTheMoon,
    GreenJoker
);

impl Jokers {
    pub(crate) fn by_rarity(rarirty: Rarity) -> Vec<Self> {
        return Self::iter().filter(|j| j.rarity() == rarirty).collect();
    }
}

impl fmt::Display for Jokers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} [${}, {}] {}",
            self.name(),
            self.cost(),
            self.rarity(),
            self.desc()
        )
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct TheJoker {}

impl Joker for TheJoker {
    fn name(&self) -> String {
        "Joker".to_string()
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn cost(&self) -> usize {
        2
    }
    fn desc(&self) -> String {
        "+4 Mult".to_string()
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::MultPlus]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, _hand: MadeHand) {
            g.mult += 4;
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct GreedyJoker {}

impl Joker for GreedyJoker {
    fn name(&self) -> String {
        "Greedy Joker".to_string()
    }
    fn desc(&self) -> String {
        "Played cards with diamond suit give +3 mult when scored ".to_string()
    }
    fn cost(&self) -> usize {
        5
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::MultPlus]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, hand: MadeHand) {
            let diamonds = hand
                .hand
                .suits()
                .iter()
                .filter(|s| **s == Suit::Diamond)
                .count();
            g.mult += diamonds * 3
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct LustyJoker {}

impl Joker for LustyJoker {
    fn name(&self) -> String {
        "Lusty Joker".to_string()
    }
    fn desc(&self) -> String {
        "Played cards with heart suit give +3 mult when scored ".to_string()
    }
    fn cost(&self) -> usize {
        5
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::MultPlus]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, hand: MadeHand) {
            let hearts = hand
                .hand
                .suits()
                .iter()
                .filter(|s| **s == Suit::Heart)
                .count();
            g.mult += hearts * 3
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct WrathfulJoker {}

impl Joker for WrathfulJoker {
    fn name(&self) -> String {
        "Wrathful Joker".to_string()
    }
    fn desc(&self) -> String {
        "Played cards with spade suit give +3 mult when scored ".to_string()
    }
    fn cost(&self) -> usize {
        5
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::MultPlus]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, hand: MadeHand) {
            let spades = hand
                .hand
                .suits()
                .iter()
                .filter(|s| **s == Suit::Spade)
                .count();
            g.mult += spades * 3
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct GluttonousJoker {}

impl Joker for GluttonousJoker {
    fn name(&self) -> String {
        "Gluttonous Joker".to_string()
    }
    fn desc(&self) -> String {
        "Played cards with club suit give +3 mult when scored ".to_string()
    }
    fn cost(&self) -> usize {
        5
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::MultPlus]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, hand: MadeHand) {
            let clubs = hand
                .hand
                .suits()
                .iter()
                .filter(|s| **s == Suit::Club)
                .count();
            g.mult += clubs * 3
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct JollyJoker {}

impl Joker for JollyJoker {
    fn name(&self) -> String {
        "Jolly Joker".to_string()
    }
    fn desc(&self) -> String {
        "+8 mult if played hand contains a pair".to_string()
    }
    fn cost(&self) -> usize {
        3
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::MultPlus]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, hand: MadeHand) {
            if hand.hand.is_pair().is_some() {
                g.mult += 8
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct ZanyJoker {}

impl Joker for ZanyJoker {
    fn name(&self) -> String {
        "Zany Joker".to_string()
    }
    fn desc(&self) -> String {
        "+12 mult if played hand contains a three of a kind".to_string()
    }
    fn cost(&self) -> usize {
        4
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::MultPlus]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, hand: MadeHand) {
            if hand.hand.is_three_of_kind().is_some() {
                g.mult += 12
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct MadJoker {}

impl Joker for MadJoker {
    fn name(&self) -> String {
        "Mad Joker".to_string()
    }
    fn desc(&self) -> String {
        "+10 mult if played hand contains a two pair".to_string()
    }
    fn cost(&self) -> usize {
        4
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::MultPlus]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, hand: MadeHand) {
            if hand.hand.is_two_pair().is_some() {
                g.mult += 10
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct CrazyJoker {}

impl Joker for CrazyJoker {
    fn name(&self) -> String {
        "Crazy Joker".to_string()
    }
    fn desc(&self) -> String {
        "+12 mult if played hand contains a straight".to_string()
    }
    fn cost(&self) -> usize {
        4
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::MultPlus]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, hand: MadeHand) {
            if hand.hand.is_straight().is_some() {
                g.mult += 12
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct DrollJoker {}

impl Joker for DrollJoker {
    fn name(&self) -> String {
        "Droll Joker".to_string()
    }
    fn desc(&self) -> String {
        "+10 mult if played hand contains a flush".to_string()
    }
    fn cost(&self) -> usize {
        4
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::MultPlus]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, hand: MadeHand) {
            if hand.hand.is_flush().is_some() {
                g.mult += 10
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct SlyJoker {}

impl Joker for SlyJoker {
    fn name(&self) -> String {
        "Sly Joker".to_string()
    }
    fn desc(&self) -> String {
        "+50 chips if played hand contains a pair".to_string()
    }
    fn cost(&self) -> usize {
        3
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::Chips]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, hand: MadeHand) {
            if hand.hand.is_pair().is_some() {
                g.chips += 50
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct WilyJoker {}

impl Joker for WilyJoker {
    fn name(&self) -> String {
        "Wily Joker".to_string()
    }
    fn desc(&self) -> String {
        "+100 chips if played hand contains a three of a kind".to_string()
    }
    fn cost(&self) -> usize {
        4
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::Chips]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, hand: MadeHand) {
            if hand.hand.is_three_of_kind().is_some() {
                g.chips += 100
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct CleverJoker {}

impl Joker for CleverJoker {
    fn name(&self) -> String {
        "Clever Joker".to_string()
    }
    fn desc(&self) -> String {
        "+80 chips if played hand contains a two pair".to_string()
    }
    fn cost(&self) -> usize {
        4
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::Chips]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, hand: MadeHand) {
            if hand.hand.is_two_pair().is_some() {
                g.chips += 80
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct DeviousJoker {}

impl Joker for DeviousJoker {
    fn name(&self) -> String {
        "Devious Joker".to_string()
    }
    fn desc(&self) -> String {
        "+100 chips if played hand contains a straight".to_string()
    }
    fn cost(&self) -> usize {
        4
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::Chips]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, hand: MadeHand) {
            if hand.hand.is_straight().is_some() {
                g.chips += 100
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct CraftyJoker {}

impl Joker for CraftyJoker {
    fn name(&self) -> String {
        "Crafty Joker".to_string()
    }
    fn desc(&self) -> String {
        "+80 chips if played hand contains a flush".to_string()
    }
    fn cost(&self) -> usize {
        4
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::Chips]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, hand: MadeHand) {
            if hand.hand.is_flush().is_some() {
                g.chips += 80
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct HalfJoker {}

impl Joker for HalfJoker {
    fn name(&self) -> String {
        "Half Joker".to_string()
    }
    fn desc(&self) -> String {
        "+20 Mult if played hand contains 3 or fewer cards".to_string()
    }
    fn cost(&self) -> usize {
        5
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::MultPlus]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, hand: MadeHand) {
            if hand.hand.len() <= 3 {
                g.mult += 20;
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct JokerStencil {}

impl Joker for JokerStencil {
    fn name(&self) -> String {
        "Joker Stencil".to_string()
    }
    fn desc(&self) -> String {
        "X1 Mult for each empty Joker Slot Joker Stencil included".to_string()
    }
    fn cost(&self) -> usize {
        8
    }
    fn rarity(&self) -> Rarity {
        Rarity::Uncommon
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::MultMult]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, _hand: MadeHand) {
            let empty = g.config.joker_slots.saturating_sub(g.jokers.len());
            if empty > 0 {
                g.mult *= empty;
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct Banner {}

impl Joker for Banner {
    fn name(&self) -> String {
        "Banner".to_string()
    }
    fn desc(&self) -> String {
        "+30 Chips for each remaining discard".to_string()
    }
    fn cost(&self) -> usize {
        5
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::Chips]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, _hand: MadeHand) {
            g.chips += 30 * g.discards;
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct MysticSummit {}

impl Joker for MysticSummit {
    fn name(&self) -> String {
        "Mystic Summit".to_string()
    }
    fn desc(&self) -> String {
        "+15 Mult when 0 discards remaining".to_string()
    }
    fn cost(&self) -> usize {
        5
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::MultPlus]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, _hand: MadeHand) {
            if g.discards == 0 {
                g.mult += 15;
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct Fibonacci {}

impl Joker for Fibonacci {
    fn name(&self) -> String {
        "Fibonacci".to_string()
    }
    fn desc(&self) -> String {
        "Each played Ace, 2, 3, 5, and 8 give +8 Mult when scored".to_string()
    }
    fn cost(&self) -> usize {
        8
    }
    fn rarity(&self) -> Rarity {
        Rarity::Uncommon
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::MultPlus]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, _hand: MadeHand) {
            for card in _hand.hand.cards() {
                if card.value == Value::Ace
                    || card.value == Value::Two
                    || card.value == Value::Three
                    || card.value == Value::Five
                    || card.value == Value::Eight
                {
                    g.mult += 8;
                }
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct ScaryFace {}

impl Joker for ScaryFace {
    fn name(&self) -> String {
        "Scary Face".to_string()
    }
    fn desc(&self) -> String {
        "Played face cards give +30 chips when scored".to_string()
    }
    fn cost(&self) -> usize {
        4
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::Chips]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, _hand: MadeHand) {
            for card in _hand.hand.cards() {
                if card.is_face_card {
                    g.chips += 30;
                }
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct AbstractJoker {}

impl Joker for AbstractJoker {
    fn name(&self) -> String {
        "Abstract Joker".to_string()
    }
    fn desc(&self) -> String {
        "+3 Mult for each Joker card".to_string()
    }
    fn cost(&self) -> usize {
        4
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::MultPlus]
    }
    fn effects(&self, _game: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, _hand: MadeHand) {
            g.mult += g.jokers.len() * 3;
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct ShootTheMoon {}

impl Joker for ShootTheMoon {
    fn name(&self) -> String {
        "Shoot the Moon".to_string()
    }
    fn desc(&self) -> String {
        "Each Queen held in hand gives +13 Mult".to_string()
    }
    fn cost(&self) -> usize {
        7
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::MultPlus]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, _hand: MadeHand) {
            let queens = g.held.iter().filter(|c| c.value == Value::Queen).count();
            g.mult += queens * 13;
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct GreenJoker {}

impl Joker for GreenJoker {
    fn name(&self) -> String {
        "Green Joker".to_string()
    }
    fn desc(&self) -> String {
        "+1 Mult per hand played this round, -1 Mult per discard".to_string()
    }
    fn cost(&self) -> usize {
        5
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::MultPlus]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, _hand: MadeHand) {
            let hands_used = g.config.plays.saturating_sub(g.plays);
            let discards_used = g.config.discards.saturating_sub(g.discards);
            let net = (hands_used as isize) - (discards_used as isize);
            if net > 0 {
                g.mult = g.mult.saturating_add(net as usize);
            } else if net < 0 {
                g.mult = g.mult.saturating_sub((-net) as usize);
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[cfg(test)]
mod tests {
    use crate::card::{Card, Suit, Value};
    use crate::hand::SelectHand;
    use crate::stage::{Blind, Stage};

    use super::*;

    fn score_before_after_joker(joker: Jokers, hand: SelectHand, before: usize, after: usize) {
        let mut g = Game::default();
        g.stage = Stage::Blind(Blind::Small);

        // First score without joker
        let score = g.calc_score(hand.best_hand().unwrap());
        assert_eq!(score, before);

        // Buy (and apply) the joker
        g.money += 1000; // Give adequate money to buy
        g.stage = Stage::Shop();
        g.shop.jokers.push(joker.clone());
        g.buy_joker(joker).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        // Second score with joker applied
        let score = g.calc_score(hand.best_hand().unwrap());
        assert_eq!(score, after);
    }

    #[test]
    fn test_the_joker() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);

        // Score Ace high without joker
        // High card (level 1) -> 5 chips, 1 mult
        // Played cards (1 ace) -> 11 chips
        // (5 + 11) * (1) = 16
        let before = 16;
        // Score Ace high with the Joker
        // High card (level 1) -> 5 chips, 1 mult
        // Played cards (1 ace) -> 11 chips
        // Joker (The Joker) -> 4 mult
        // (5 + 11) * (1 + 4) = 80
        let after = 80;

        let j = Jokers::TheJoker(TheJoker {});
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_lusty_joker() {
        let ah = Card::new(Value::Ace, Suit::Heart);
        let ac = Card::new(Value::Ace, Suit::Club);
        let ad = Card::new(Value::Ace, Suit::Diamond);
        let hand = SelectHand::new(vec![ah, ah, ac, ad]);

        // Score 4ok without joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // (60 + 44) * (7) = 728
        let before = 728;
        // Score 4ok (2 hearts) with joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // joker w/ 2 hearts = +6 mult
        // (60 + 44) * (7 + 6) = 1352
        let after = 1352;

        let j = Jokers::LustyJoker(LustyJoker {});
        score_before_after_joker(j, hand, before, after)
    }

    #[test]
    fn test_greedy_joker() {
        let ah = Card::new(Value::Ace, Suit::Heart);
        let ad = Card::new(Value::Ace, Suit::Diamond);
        let hand = SelectHand::new(vec![ad, ad, ad, ah]);

        // Score 4ok without joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // (60 + 44) * (7) = 728
        let before = 728;
        // Score 4ok (3 diamonds) with joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // joker w/ 3 diamonds = +9 mult
        // (60 + 44) * (7 + 9) = 1664
        let after = 1664;

        let j = Jokers::GreedyJoker(GreedyJoker {});
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_wrathful_joker() {
        let asp = Card::new(Value::Ace, Suit::Spade);
        let ad = Card::new(Value::Ace, Suit::Diamond);
        let hand = SelectHand::new(vec![asp, ad, ad, ad]);

        // Score 4ok without joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // (60 + 44) * (7) = 728
        let before = 728;
        // Score 4ok (1 spade) with joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // joker w/ 1 spade = +3 mult
        // (60 + 44) * (7 + 3) = 1040
        let after = 1040;

        let j = Jokers::WrathfulJoker(WrathfulJoker {});
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_gluttonous_joker() {
        let ac = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![ac, ac, ac, ac]);

        // Score 4ok without joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // (60 + 44) * (7) = 728
        let before = 728;
        // Score 4ok (4 clubs) with joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // joker w/ 4 clubs = +12 mult
        // (60 + 44) * (7 + 12) = 1976
        let after = 1976;

        let j = Jokers::GluttonousJoker(GluttonousJoker {});
        score_before_after_joker(j, hand, before, after)
    }

    #[test]
    fn test_jolly_joker() {
        let ac = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![ac, ac, ac, ac]);

        // Score 4ok without joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // (60 + 44) * (7) = 728
        let before = 728;
        // Score 4ok with joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // joker w/ pair = +8 mult
        // (60 + 44) * (7 + 8) = 1560
        let after = 1560;

        let j = Jokers::JollyJoker(JollyJoker {});
        score_before_after_joker(j, hand, before, after)
    }

    #[test]
    fn test_zany_joker() {
        let ac = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![ac, ac, ac, ac]);

        // Score 4ok without joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // (60 + 44) * (7) = 728
        let before = 728;
        // Score 4ok with joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // joker w/ 3ok = +12 mult
        // (60 + 44) * (7 + 12) = 1976
        let after = 1976;

        let j = Jokers::ZanyJoker(ZanyJoker {});
        score_before_after_joker(j, hand, before, after)
    }

    #[test]
    fn test_mad_joker() {
        let ac = Card::new(Value::Ace, Suit::Club);
        let kc = Card::new(Value::King, Suit::Club);
        let hand = SelectHand::new(vec![ac, ac, kc, kc]);

        // Score two pair without joker
        // two pair (level 1) -> 20 chips, 2 mult
        // Played cards (2 ace, 2 king) -> 42 chips
        // (20 + 42) * (2) = 124
        let before = 124;
        let j = Jokers::MadJoker(MadJoker {});
        // Score two pair with joker
        // two pair (level 1) -> 20 chips, 2 mult
        // Played cards (2 ace, 2 king) -> 42 chips
        // joker w/ two pair = +10 mult
        // (20 + 42) * (2 + 10) = 744
        let after = 744;

        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_crazy_joker() {
        let two = Card::new(Value::Two, Suit::Club);
        let three = Card::new(Value::Three, Suit::Club);
        let four = Card::new(Value::Four, Suit::Club);
        let five = Card::new(Value::Five, Suit::Club);
        let six = Card::new(Value::Six, Suit::Heart);
        let hand = SelectHand::new(vec![two, three, four, five, six]);

        // Score straight without joker
        // straight (level 1) -> 30 chips, 4 mult
        // Played cards (2, 3, 4, 5, 6) -> 20 chips
        // (20 + 30) * (4) = 200
        let before = 200;
        // Score straight with joker
        // straight (level 1) -> 30 chips, 4 mult
        // Played cards (2, 3, 4, 5, 6) -> 20 chips
        // joker w/ straight = +12 mult
        // (20 + 30) * (4 + 12) = 800
        let after = 800;

        let j = Jokers::CrazyJoker(CrazyJoker {});
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_droll_joker() {
        let two = Card::new(Value::Two, Suit::Club);
        let three = Card::new(Value::Three, Suit::Club);
        let four = Card::new(Value::Four, Suit::Club);
        let five = Card::new(Value::Five, Suit::Club);
        let ten = Card::new(Value::Ten, Suit::Club);
        let hand = SelectHand::new(vec![two, three, four, five, ten]);

        // Score flush without joker
        // flush (level 1) -> 35 chips, 4 mult
        // Played cards (2, 3, 4, 5, 10) -> 24 chips
        // (24 + 35) * (4) = 236
        let before = 236;
        // Score flush with joker
        // flush (level 1) -> 35 chips, 4 mult
        // Played cards (2, 3, 4, 5, 10) -> 24 chips
        // joker w/ flush = +10 mult
        // (24 + 35) * (4 + 10) = 826
        let after = 826;

        let j = Jokers::DrollJoker(DrollJoker {});
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_sly_joker() {
        let ac = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![ac, ac, ac, ac]);

        // Score 4ok without joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // (60 + 44) * (7) = 728
        let before = 728;
        // Score 4ok with joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // joker w/ pair = +50 chips
        // (60 + 44 + 50) * (7) = 1078
        let after = 1078;

        let j = Jokers::SlyJoker(SlyJoker {});
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_wily_joker() {
        let ac = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![ac, ac, ac, ac]);

        // Score 4ok without joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // (60 + 44) * (7) = 728
        let before = 728;
        // Score 4ok with joker
        // 4ok (level 1) -> 60 chips, 7 mult
        // Played cards (4 ace) -> 44 chips
        // joker w/ 3ok = +100 chips
        // (60 + 44 + 100) * (7) = 1428
        let after = 1428;

        let j = Jokers::WilyJoker(WilyJoker {});
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_clever_joker() {
        let ac = Card::new(Value::Ace, Suit::Club);
        let kc = Card::new(Value::King, Suit::Club);
        let hand = SelectHand::new(vec![ac, ac, kc, kc]);

        // Score two pair without joker
        // two pair (level 1) -> 20 chips, 2 mult
        // Played cards (2 ace, 2 king) -> 42 chips
        // (20 + 42) * (2) = 124
        let before = 124;
        // Score two pair with joker
        // two pair (level 1) -> 20 chips, 2 mult
        // Played cards (2 ace, 2 king) -> 42 chips
        // joker w/ two pair = +80 chips
        // (20 + 42 + 80) * (2) = 284
        let after = 284;

        let j = Jokers::CleverJoker(CleverJoker {});
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_devious_joker() {
        let two = Card::new(Value::Two, Suit::Club);
        let three = Card::new(Value::Three, Suit::Club);
        let four = Card::new(Value::Four, Suit::Club);
        let five = Card::new(Value::Five, Suit::Club);
        let six = Card::new(Value::Six, Suit::Heart);
        let hand = SelectHand::new(vec![two, three, four, five, six]);

        // Score straight without joker
        // straight (level 1) -> 30 chips, 4 mult
        // Played cards (2, 3, 4, 5, 6) -> 20 chips
        // (20 + 30) * (4) = 200
        let before = 200;
        // Score straight with joker
        // straight (level 1) -> 30 chips, 4 mult
        // Played cards (2, 3, 4, 5, 6) -> 20 chips
        // joker w/ straight = +100 chips
        // (20 + 30 + 100) * (4) = 600
        let after = 600;

        let j = Jokers::DeviousJoker(DeviousJoker {});
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_crafty_joker() {
        let two = Card::new(Value::Two, Suit::Club);
        let three = Card::new(Value::Three, Suit::Club);
        let four = Card::new(Value::Four, Suit::Club);
        let five = Card::new(Value::Five, Suit::Club);
        let ten = Card::new(Value::Ten, Suit::Club);
        let hand = SelectHand::new(vec![two, three, four, five, ten]);

        // Score flush without joker
        // flush (level 1) -> 35 chips, 4 mult
        // Played cards (2, 3, 4, 5, 10) -> 24 chips
        // (24 + 35) * (4) = 236
        let before = 236;
        // Score flush with joker
        // flush (level 1) -> 35 chips, 4 mult
        // Played cards (2, 3, 4, 5, 10) -> 24 chips
        // joker w/ flush = +80 chips
        // (24 + 35 + 80) * (4) = 556
        let after = 556;
        let j = Jokers::CraftyJoker(CraftyJoker {});
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_half_joker() {
        let ac = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![ac, ac, ac]);

        // Score 3ok without joker
        // 3ok (level 1) -> 30 chips, 3 mult
        // Played cards (3 ace) -> 33 chips
        // (30 + 33) * 3 = 189
        let before = 189;
        // Score 3ok with joker
        // 3ok (level 1) -> 30 chips, 3 mult
        // Played cards (3 ace) -> 33 chips
        // joker w/ <=3 cards = +20 mult
        // (30 + 33) * (3 + 20) = 1449
        let after = 1449;

        let j = Jokers::HalfJoker(HalfJoker {});
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_joker_stencil() {
        let ace = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![ace]);
        let best = hand.best_hand().unwrap();

        // High card (level 1) -> 5 chips, 1 mult
        // Played cards (1 ace) -> 11 chips
        // (5 + 11) * 1 = 16
        let mut g = Game::default();
        g.stage = Stage::Blind(Blind::Small);
        assert_eq!(g.calc_score(best.clone()), 16);

        // Stencil alone in 5 slots = 4 empty -> X4
        // (5 + 11) * (1 * 4) = 64
        let j = Jokers::JokerStencil(JokerStencil {});
        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        assert_eq!(g.calc_score(best.clone()), 64);

        // Add another joker -> 3 empty -> X3
        let j2 = Jokers::Banner(Banner {});
        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j2.clone());
        g.buy_joker(j2).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        // (5 + 11 + 4*30) * (1 * 3) = 136 * 3 = 408
        assert_eq!(g.calc_score(best.clone()), 408);
    }

    #[test]
    fn test_banner_joker() {
        let ace = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![ace]);
        let best = hand.best_hand().unwrap();
        let j = Jokers::Banner(Banner {});

        // High card (level 1) -> 5 chips, 1 mult
        // Played cards (1 ace) -> 11 chips
        // (5 + 11) * (1) = 16
        let mut g = Game::default();
        g.stage = Stage::Blind(Blind::Small);
        assert_eq!(g.calc_score(best.clone()), 16);

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // Banner: 4 discards * 30 chips = +120
        // (5 + 11 + 120) * 1 = 136
        assert_eq!(g.calc_score(best.clone()), 136);

        g.discards = 0;
        // Banner: +0 chips
        // (5 + 11 + 0) * 1 = 16
        assert_eq!(g.calc_score(best.clone()), 16);
    }

    #[test]
    fn test_mystic_summit() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let best = hand.best_hand().unwrap();
        let j = Jokers::MysticSummit(MysticSummit {});

        // High card (level 1): 5 chips, 1 mult
        // Played (1 ace): 11 chips
        // (5 + 11) * 1 = 16
        let mut g = Game::default();
        g.stage = Stage::Blind(Blind::Small);
        assert_eq!(g.calc_score(best.clone()), 16);

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // discards = 4 (default), so Mystic Summit does NOT fire
        // (5 + 11) * 1 = 16
        assert_eq!(g.calc_score(best.clone()), 16);

        // Now set discards to 0 -> +15 mult
        g.discards = 0;
        // (5 + 11) * (1 + 15) = 16 * 16 = 256
        assert_eq!(g.calc_score(best.clone()), 256);
    }

    #[test]
    fn test_fibonacci() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let two = Card::new(Value::Two, Suit::Heart);
        let three = Card::new(Value::Three, Suit::Heart);
        let five = Card::new(Value::Five, Suit::Heart);
        let eight = Card::new(Value::Eight, Suit::Heart);
        let hand = SelectHand::new(vec![ace, two, three, five, eight]);
        let j = Jokers::Fibonacci(Fibonacci {});

        // Flush (level 1): 35 chips, 4 mult
        // Played (5 cards): 11 + 2 + 3 + 5 + 8 = 29 chips
        // (35 + 29) * 4 = 256
        let before = 256;

        // Fibonacci: 1 ace, 1 two, 1 three, 1 five, 1 eight -> +8 mult each
        // (35 + 29) * (4 + 40) = 64 * 44 = 2816
        let after = 2816;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_scary_face() {
        let ace = Card::new(Value::Ace, Suit::Club);
        let king = Card::new(Value::King, Suit::Club);
        let queen = Card::new(Value::Queen, Suit::Spade);
        let jack = Card::new(Value::Jack, Suit::Heart);
        let ten = Card::new(Value::Ten, Suit::Diamond);
        let hand = SelectHand::new(vec![ace, jack, queen, king, ten]);
        let j = Jokers::ScaryFace(ScaryFace {});

        // Straight (level 1): 30 chips, 4 mult
        // Played (5 cards): 11 + 10 + 10 + 10 + 10 = 51 chips
        // (30 + 51) * 4 = 324
        let before = 324;

        // Scary Face: jack, queen, king -> +30 chips each
        // (30 + 51 + 90) * 4 = 684
        let after = 684;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_abstract_joker() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let best = hand.best_hand().unwrap();

        // High card (level 1): 5 chips, 1 mult
        // Played (1 ace): 11 chips
        // (5 + 11) * 1 = 16
        let mut g = Game::default();
        g.stage = Stage::Blind(Blind::Small);
        assert_eq!(g.calc_score(best.clone()), 16);

        // Buy Abstract Joker -> 1 joker, +3 mult
        // (5 + 11) * (1 + 3) = 64
        g.money += 1000;
        g.stage = Stage::Shop();
        let aj = Jokers::AbstractJoker(AbstractJoker {});
        g.shop.jokers.push(aj.clone());
        g.buy_joker(aj).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        assert_eq!(g.calc_score(best.clone()), 64);

        // Buy Scary Face -> 2 jokers, +6 mult
        // (5 + 11) * (1 + 6) = 112
        g.money += 1000;
        g.stage = Stage::Shop();
        let sf = Jokers::ScaryFace(ScaryFace {});
        g.shop.jokers.push(sf.clone());
        g.buy_joker(sf).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        assert_eq!(g.calc_score(best.clone()), 112);
    }
}
