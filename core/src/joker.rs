use crate::card::{Card, Enhancement, Suit, Value};
use crate::effect::Effects;
use crate::game::Game;
use crate::hand::{MadeHand, SelectHand};
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
// to match each joker and call its methods. It ends up creating an enum `Jokers` that
// contains each joker struct (where each struct impl `Joker`), and we impl `Joker` for
// `Jokers` enum by matching each case and calling underlying methods.
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
    //DelayedGratification,
    //Hack,
    Pareidolia,
    //GrosMichel,
    EvenSteven,
    OddTodd,
    Scholar,
    BusinessCard,
    //Supernova,
    //RideTheBus,
    //SpaceJoker,
    //Egg,
    //Burglar,
    //Blackboard,
    //Runner,
    //IceCream,
    //DNA,
    //Splash,
    //BlueJoker,
    //SixthSense,
    //Constellation,
    //Hiker,
    FacelessJoker,
    //GreenJoker,
    //Superposition,
    //ToDoList,
    //Cavendish,
    //CardSharp,
    //RedCard,
    //Madness,
    //SquareJoker,
    //Seance,
    //Riff-raff,
    //Vampire,
    //Shortcut,
    //Hologram,
    //Vagabond,
    Baron,
    //Cloud9,
    //Rocket,
    //Obelisk,
    MidasMask,
    //Luchador,
    Photograph,
    //GiftCard,
    //TurtleBean,
    //Erosion,
    ReservedParking,
    //Mail-InRebate,
    //ToTheMoon,
    //Hallucination,
    //FortuneTeller,
    //Juggler,
    //Drunkard,
    //StoneJoker,
    //GoldenJoker,
    //LuckyCat,
    BaseballCard,
    Bull,
    //DietCola,
    //TradingCard,
    //FlashCard,
    //Popcorn,
    //SpareTrousers,
    //AncientJoker,
    //Ramen,
    WalkieTalkie,
    //Seltzer,
    //Castle,
    SmileyFace,
    //Campfire,
    GoldenTicket,
    //MrBones,
    Acrobat,
    //SockAndBuskin,
    //Swashbuckler,
    //Troubadour,
    //Certificate,
    //SmearedJoker,
    //Throwback,
    //HangingChad,
    RoughGem,
    Bloodstone
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
pub struct Pareidolia {}

impl Joker for Pareidolia {
    fn name(&self) -> String {
        "Pareidolia".to_string()
    }
    fn blueprint_compatible(&self) -> bool {
        false
    }
    fn desc(&self) -> String {
        "All cards are considered face cards".to_string()
    }
    fn cost(&self) -> usize {
        5
    }
    fn rarity(&self) -> Rarity {
        Rarity::Uncommon
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::Effect]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(_g: &mut Game, hand: &mut MadeHand) {
            for card in &mut hand.all {
                card.is_face_card = true;
            }
            let cards: Vec<Card> = hand
                .hand
                .cards()
                .into_iter()
                .map(|mut c| {
                    c.is_face_card = true;
                    c
                })
                .collect();
            hand.hand = SelectHand::new(cards);
        }
        vec![Effects::OnModifyHand(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct EvenSteven {}

impl Joker for EvenSteven {
    fn name(&self) -> String {
        "Even Steven".to_string()
    }
    fn desc(&self) -> String {
        "Played cards with even rank give +4 Mult when scored".to_string()
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
        fn apply(g: &mut Game, _hand: MadeHand) {
            for card in _hand.hand.cards() {
                if card.is_even() {
                    g.mult += 4;
                }
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct OddTodd {}

impl Joker for OddTodd {
    fn name(&self) -> String {
        "Odd Todd".to_string()
    }
    fn desc(&self) -> String {
        "Played cards with odd rank give +31 Chips when scored".to_string()
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
                if card.is_odd() {
                    g.chips += 31;
                }
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct Scholar {}

impl Joker for Scholar {
    fn name(&self) -> String {
        "Scholar".to_string()
    }
    fn desc(&self) -> String {
        "Played Aces give +20 Chips and +4 Mult when scored".to_string()
    }
    fn cost(&self) -> usize {
        4
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::Chips, Categories::MultPlus]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, _hand: MadeHand) {
            for card in _hand.hand.cards() {
                if card.value == Value::Ace {
                    g.chips += 20;
                    g.mult += 4;
                }
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct BusinessCard {}

impl Joker for BusinessCard {
    fn name(&self) -> String {
        "Business Card".to_string()
    }
    fn desc(&self) -> String {
        "Played face cards have a 1 in 2 chance to give $2 when scored".to_string()
    }
    fn cost(&self) -> usize {
        4
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::Economy]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, _hand: MadeHand) {
            for card in _hand.hand.cards() {
                if card.is_face_card && g.prob_roll(1, 2) {
                    g.money += 2;
                }
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct FacelessJoker {}

impl Joker for FacelessJoker {
    fn name(&self) -> String {
        "Faceless Joker".to_string()
    }
    fn desc(&self) -> String {
        "If you discard at least 5 cards in a single discard, gain $5".to_string()
    }
    fn cost(&self) -> usize {
        4
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::Economy]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, _hand: MadeHand) {
            if _hand.all.len() >= 5 {
                g.money += 5;
            }
        }
        vec![Effects::OnDiscard(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct Baron {}

impl Joker for Baron {
    fn name(&self) -> String {
        "Baron".to_string()
    }
    fn desc(&self) -> String {
        "Each King held in hand gives X1.5 Mult".to_string()
    }
    fn cost(&self) -> usize {
        8
    }
    fn rarity(&self) -> Rarity {
        Rarity::Rare
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::MultMult]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, _hand: MadeHand) {
            let kings = g.held.iter().filter(|c| c.value == Value::King).count();
            for _ in 0..kings {
                g.mult = (g.mult as f64 * 1.5) as usize;
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct MidasMask {}

impl Joker for MidasMask {
    fn name(&self) -> String {
        "Midas Mask".to_string()
    }
    fn blueprint_compatible(&self) -> bool {
        false
    }
    fn desc(&self) -> String {
        "All played cards are converted to Gold cards when scored".to_string()
    }
    fn cost(&self) -> usize {
        7
    }
    fn rarity(&self) -> Rarity {
        Rarity::Uncommon
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::Effect]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(_g: &mut Game, hand: &mut MadeHand) {
            for card in &mut hand.all {
                card.enhancement = Some(Enhancement::Gold);
            }
            let cards: Vec<Card> = hand
                .hand
                .cards()
                .into_iter()
                .map(|mut c| {
                    c.enhancement = Some(Enhancement::Gold);
                    c
                })
                .collect();
            hand.hand = SelectHand::new(cards);
        }
        vec![Effects::OnModifyHand(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct Photograph {}

impl Joker for Photograph {
    fn name(&self) -> String {
        "Photograph".to_string()
    }
    fn desc(&self) -> String {
        "First face card scored gives X2 Mult".to_string()
    }
    fn cost(&self) -> usize {
        5
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::MultMult]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, _hand: MadeHand) {
            for card in _hand.hand.cards() {
                if card.is_face_card {
                    g.mult *= 2;
                    break;
                }
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct ReservedParking {}

impl Joker for ReservedParking {
    fn name(&self) -> String {
        "Reserved Parking".to_string()
    }
    fn desc(&self) -> String {
        "Each face card held in hand has a 1 in 2 chance to give $1".to_string()
    }
    fn cost(&self) -> usize {
        6
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::Economy]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, _hand: MadeHand) {
            for card in &g.held.clone() {
                if card.is_face_card && g.prob_roll(1, 2) {
                    g.money += 1;
                }
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct BaseballCard {}

impl Joker for BaseballCard {
    fn name(&self) -> String {
        "Baseball Card".to_string()
    }
    fn desc(&self) -> String {
        "Each Uncommon joker gives X1.5 Mult".to_string()
    }
    fn cost(&self) -> usize {
        8
    }
    fn rarity(&self) -> Rarity {
        Rarity::Rare
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::MultMult]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, _hand: MadeHand) {
            let uncommon = g
                .jokers
                .iter()
                .filter(|j| j.rarity() == Rarity::Uncommon)
                .count();
            for _ in 0..uncommon {
                g.mult = (g.mult as f64 * 1.5) as usize;
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct Bull {}

impl Joker for Bull {
    fn name(&self) -> String {
        "Bull".to_string()
    }
    fn desc(&self) -> String {
        "+2 Chips for each dollar you have".to_string()
    }
    fn cost(&self) -> usize {
        6
    }
    fn rarity(&self) -> Rarity {
        Rarity::Uncommon
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::Chips]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, _hand: MadeHand) {
            g.chips += g.money * 2;
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct WalkieTalkie {}

impl Joker for WalkieTalkie {
    fn name(&self) -> String {
        "Walkie Talkie".to_string()
    }
    fn desc(&self) -> String {
        "Each played 10 or 4 gives +10 Chips and +4 Mult when scored".to_string()
    }
    fn cost(&self) -> usize {
        4
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::Chips, Categories::MultPlus]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, _hand: MadeHand) {
            for card in _hand.hand.cards() {
                if card.value == Value::Ten || card.value == Value::Four {
                    g.chips += 10;
                    g.mult += 4;
                }
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct SmileyFace {}

impl Joker for SmileyFace {
    fn name(&self) -> String {
        "Smiley Face".to_string()
    }
    fn desc(&self) -> String {
        "+5 Mult for each scored face card".to_string()
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
        fn apply(g: &mut Game, _hand: MadeHand) {
            for card in _hand.hand.cards() {
                if card.is_face_card {
                    g.mult += 5;
                }
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct GoldenTicket {}

impl Joker for GoldenTicket {
    fn name(&self) -> String {
        "Golden Ticket".to_string()
    }
    fn desc(&self) -> String {
        "Played Gold cards give $4 when scored".to_string()
    }
    fn cost(&self) -> usize {
        5
    }
    fn rarity(&self) -> Rarity {
        Rarity::Common
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::Economy]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, _hand: MadeHand) {
            for card in _hand.hand.cards() {
                if card.enhancement == Some(Enhancement::Gold) {
                    g.money += 4;
                }
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct Acrobat {}

impl Joker for Acrobat {
    fn name(&self) -> String {
        "Acrobat".to_string()
    }
    fn desc(&self) -> String {
        "X3 Mult on final hand of round".to_string()
    }
    fn cost(&self) -> usize {
        6
    }
    fn rarity(&self) -> Rarity {
        Rarity::Uncommon
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::MultMult]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, _hand: MadeHand) {
            if g.plays == 0 {
                g.mult *= 3;
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct RoughGem {}

impl Joker for RoughGem {
    fn name(&self) -> String {
        "Rough Gem".to_string()
    }
    fn desc(&self) -> String {
        "Each played Diamond earns $1".to_string()
    }
    fn cost(&self) -> usize {
        7
    }
    fn rarity(&self) -> Rarity {
        Rarity::Uncommon
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::Economy]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, _hand: MadeHand) {
            for card in _hand.hand.cards() {
                if card.suit == Suit::Diamond {
                    g.money += 1;
                }
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq))]
pub struct Bloodstone {}

impl Joker for Bloodstone {
    fn name(&self) -> String {
        "Bloodstone".to_string()
    }
    fn desc(&self) -> String {
        "1 in 2 chance for each scored Heart card to give X1.5 Mult".to_string()
    }
    fn cost(&self) -> usize {
        7
    }
    fn rarity(&self) -> Rarity {
        Rarity::Uncommon
    }
    fn categories(&self) -> Vec<Categories> {
        vec![Categories::MultMult]
    }
    fn effects(&self, _in: &Game) -> Vec<Effects> {
        fn apply(g: &mut Game, _hand: MadeHand) {
            for card in _hand.hand.cards() {
                if card.suit == Suit::Heart && g.prob_roll(1, 2) {
                    g.mult += g.mult / 2;
                }
            }
        }
        vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
    }
}

#[cfg(test)]
mod tests {
    use crate::card::{Card, Enhancement, Suit, Value};
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

    #[test]
    fn test_pareidolia_scary_face() {
        // Ace low straight: no natural face cards
        let ace = Card::new(Value::Ace, Suit::Club);
        let two = Card::new(Value::Two, Suit::Heart);
        let three = Card::new(Value::Three, Suit::Spade);
        let four = Card::new(Value::Four, Suit::Diamond);
        let five = Card::new(Value::Five, Suit::Club);
        let hand = SelectHand::new(vec![ace, two, three, four, five]);
        let best = hand.best_hand().unwrap();

        // Straight (level 1): 30 chips, 4 mult
        // Card chips: 11 + 2 + 3 + 4 + 5 = 25
        // (30 + 25) * 4 = 220
        let mut g = Game::default();
        g.stage = Stage::Blind(Blind::Small);
        assert_eq!(g.calc_score(best.clone()), 220);

        // Add Scary Face: still no face cards, so still 220
        g.money += 1000;
        g.stage = Stage::Shop();
        let sf = Jokers::ScaryFace(ScaryFace {});
        g.shop.jokers.push(sf.clone());
        g.buy_joker(sf).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        assert_eq!(g.calc_score(best.clone()), 220);

        // Add Pareidolia: now all cards are face cards
        // Scary Face gives +30 chips × 5 = +150
        // (30 + 25 + 150) * 4 = 820
        g.money += 1000;
        g.stage = Stage::Shop();
        let p = Jokers::Pareidolia(Pareidolia {});
        g.shop.jokers.push(p.clone());
        g.buy_joker(p).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        assert_eq!(g.calc_score(best.clone()), 820);
    }

    #[test]
    fn test_even_steven() {
        let two = Card::new(Value::Two, Suit::Club);
        let four = Card::new(Value::Four, Suit::Club);
        let six = Card::new(Value::Six, Suit::Club);
        let eight = Card::new(Value::Eight, Suit::Club);
        let ten = Card::new(Value::Ten, Suit::Club);
        let hand = SelectHand::new(vec![two, four, six, eight, ten]);
        let j = Jokers::EvenSteven(EvenSteven {});

        // Flush (level 1): 35 chips, 4 mult
        // Played (5 cards): 2 + 4 + 6 + 8 + 10 = 30 chips
        // (35 + 30) * 4 = 260
        let before = 260;

        // Even Steven: 5 even cards * +4 mult = +20 mult
        // (35 + 30) * (4 + 20) = 65 * 24 = 1560
        let after = 1560;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_even_steven_odd_cards() {
        let three = Card::new(Value::Three, Suit::Club);
        let five = Card::new(Value::Five, Suit::Club);
        let seven = Card::new(Value::Seven, Suit::Club);
        let nine = Card::new(Value::Nine, Suit::Club);
        let ace = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![three, five, seven, nine, ace]);
        let j = Jokers::EvenSteven(EvenSteven {});

        // Flush (level 1): 35 chips, 4 mult
        // Played (5 cards): 3 + 5 + 7 + 9 + 11 = 35 chips
        // (35 + 35) * 4 = 280
        let before = 280;

        // Even Steven: 0 even cards -> no bonus
        let after = 280;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_odd_todd() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let three = Card::new(Value::Three, Suit::Heart);
        let five = Card::new(Value::Five, Suit::Heart);
        let seven = Card::new(Value::Seven, Suit::Heart);
        let nine = Card::new(Value::Nine, Suit::Heart);
        let hand = SelectHand::new(vec![ace, three, five, seven, nine]);
        let j = Jokers::OddTodd(OddTodd {});

        // Flush (level 1): 35 chips, 4 mult
        // Played (5 cards): 11 + 3 + 5 + 7 + 9 = 35 chips
        // (35 + 35) * 4 = 280
        let before = 280;

        // Odd Todd: 5 odd cards * +31 chips = +155 chips
        // (35 + 35 + 155) * 4 = 225 * 4 = 900
        let after = 900;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_odd_todd_even_cards() {
        let two = Card::new(Value::Two, Suit::Club);
        let four = Card::new(Value::Four, Suit::Club);
        let six = Card::new(Value::Six, Suit::Club);
        let eight = Card::new(Value::Eight, Suit::Club);
        let ten = Card::new(Value::Ten, Suit::Club);
        let hand = SelectHand::new(vec![two, four, six, eight, ten]);
        let j = Jokers::OddTodd(OddTodd {});

        // Flush (level 1): 35 chips, 4 mult
        // Played (5 cards): 2 + 4 + 6 + 8 + 10 = 30 chips
        // (35 + 30) * 4 = 260
        let before = 260;

        // Odd Todd: 0 odd cards -> no bonus
        let after = 260;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_scholar() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace, ace]);
        let j = Jokers::Scholar(Scholar {});

        // Pair (level 1): 10 chips, 2 mult
        // Played (2 aces): 2 * 11 = 22 chips
        // (10 + 22) * 2 = 64
        let before = 64;

        // Scholar: 2 aces * (+20 chips, +4 mult) = +40 chips, +8 mult
        // (10 + 22 + 40) * (2 + 8) = 72 * 10 = 720
        let after = 720;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_scholar_no_aces() {
        let king = Card::new(Value::King, Suit::Club);
        let hand = SelectHand::new(vec![king]);
        let j = Jokers::Scholar(Scholar {});

        // High card (level 1): 5 chips, 1 mult
        // Played (1 king): 10 chips
        // (5 + 10) * 1 = 15
        let before = 15;

        // Scholar: 0 aces -> no bonus
        let after = 15;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_business_card_no_face_cards() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace, ace]);
        let j = Jokers::BusinessCard(BusinessCard {});

        let mut g = Game::default();
        g.stage = Stage::Blind(Blind::Small);
        let score = g.calc_score(hand.best_hand().unwrap());
        assert_eq!(score, 64);

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        let money_before = g.money;
        g.calc_score(hand.best_hand().unwrap());
        assert_eq!(g.money, money_before);
    }

    #[test]
    fn test_business_card_face_cards() {
        let king = Card::new(Value::King, Suit::Heart);
        let queen = Card::new(Value::Queen, Suit::Heart);
        let jack = Card::new(Value::Jack, Suit::Heart);
        let hand = SelectHand::new(vec![king, queen, jack]);
        let j = Jokers::BusinessCard(BusinessCard {});

        let mut g = Game::default();
        g.stage = Stage::Blind(Blind::Small);
        g.calc_score(hand.best_hand().unwrap());

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        let mut saw_increase = false;
        for _ in 0..100 {
            g.money = 1000;
            g.calc_score(hand.best_hand().unwrap());
            if g.money > 1000 {
                saw_increase = true;
                break;
            }
        }
        assert!(saw_increase, "Business Card should sometimes give money");
    }

    #[test]
    fn test_faceless_joker() {
        let j = Jokers::FacelessJoker(FacelessJoker {});

        let mut g = Game::default();
        g.start();
        g.stage = Stage::Blind(Blind::Small);
        g.blind = Some(Blind::Small);

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        let cards: Vec<Card> = g.available.cards().iter().take(5).copied().collect();
        for card in &cards {
            g.available.select_card(*card).expect("can select");
        }
        assert_eq!(g.available.selected().len(), 5);

        g.discard_selected().expect("can discard");
        assert_eq!(g.money, 1001);
    }

    #[test]
    fn test_faceless_joker_few_cards() {
        let j = Jokers::FacelessJoker(FacelessJoker {});

        let mut g = Game::default();
        g.start();
        g.stage = Stage::Blind(Blind::Small);
        g.blind = Some(Blind::Small);

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        let cards: Vec<Card> = g.available.cards().iter().take(3).copied().collect();
        for card in &cards {
            g.available.select_card(*card).expect("can select");
        }
        assert_eq!(g.available.selected().len(), 3);

        g.discard_selected().expect("can discard");
        assert_eq!(g.money, 996);
    }

    #[test]
    fn test_baron() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace, ace]);
        let j = Jokers::Baron(Baron {});

        let mut g = Game::default();
        g.stage = Stage::Blind(Blind::Small);
        g.held = vec![
            Card::new(Value::King, Suit::Club),
            Card::new(Value::King, Suit::Spade),
        ];
        let best = hand.best_hand().unwrap();

        // Pair (level 1): 10 chips, 2 mult
        // Played (2 aces): 22 chips
        // (10 + 22) * 2 = 64
        assert_eq!(g.calc_score(best.clone()), 64);

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // Baron: 2 kings -> 2 * 1.5 = 3, then 3 * 1.5 = 4 (truncated)
        // (10 + 22) * (2 * 1.5 * 1.5) = 32 * 4 = 128
        assert_eq!(g.calc_score(best.clone()), 128);
    }

    #[test]
    fn test_baron_no_kings() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace, ace]);
        let j = Jokers::Baron(Baron {});

        let mut g = Game::default();
        g.stage = Stage::Blind(Blind::Small);
        g.held = vec![
            Card::new(Value::Queen, Suit::Club),
            Card::new(Value::Jack, Suit::Spade),
        ];
        let best = hand.best_hand().unwrap();

        // Pair (level 1): 10 chips, 2 mult
        // Played (2 aces): 22 chips
        // (10 + 22) * 2 = 64
        assert_eq!(g.calc_score(best.clone()), 64);

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // Baron: 0 kings -> no bonus
        assert_eq!(g.calc_score(best.clone()), 64);
    }

    #[test]
    fn test_midas_mask() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace, ace]);
        let j = Jokers::MidasMask(MidasMask {});

        // Pair (level 1): 10 chips, 2 mult
        // Played (2 aces): 22 chips
        // (10 + 22) * 2 = 64
        let before = 64;

        // Midas Mask converts to Gold but Gold has no game logic yet -> same score
        let after = 64;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_photograph() {
        let king = Card::new(Value::King, Suit::Heart);
        let hand = SelectHand::new(vec![king]);
        let j = Jokers::Photograph(Photograph {});

        // High card (level 1): 5 chips, 1 mult
        // Played (1 king): 10 chips
        // (5 + 10) * 1 = 15
        let before = 15;

        // Photograph: 1 face card -> X2 mult
        // (5 + 10) * (1 * 2) = 30
        let after = 30;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_photograph_no_face() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::Photograph(Photograph {});

        // High card (level 1): 5 chips, 1 mult
        // Played (1 ace): 11 chips
        // (5 + 11) * 1 = 16
        let before = 16;

        // Photograph: 0 face cards -> no bonus
        let after = 16;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_reserved_parking() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::ReservedParking(ReservedParking {});

        let mut g = Game::default();
        g.stage = Stage::Blind(Blind::Small);
        g.held = vec![
            Card::new(Value::King, Suit::Club),
            Card::new(Value::Queen, Suit::Spade),
            Card::new(Value::Jack, Suit::Heart),
        ];
        let best = hand.best_hand().unwrap();
        g.calc_score(best.clone());

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        let mut saw_increase = false;
        for _ in 0..100 {
            g.money = 994;
            g.calc_score(best.clone());
            if g.money > 994 {
                saw_increase = true;
                break;
            }
        }
        assert!(saw_increase, "Reserved Parking should sometimes give money");
    }

    #[test]
    fn test_reserved_parking_no_face() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::ReservedParking(ReservedParking {});

        let mut g = Game::default();
        g.stage = Stage::Blind(Blind::Small);
        g.held = vec![
            Card::new(Value::Ace, Suit::Club),
            Card::new(Value::Two, Suit::Spade),
        ];
        let best = hand.best_hand().unwrap();
        g.calc_score(best.clone());

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        let money_before = g.money;
        g.calc_score(best.clone());
        assert_eq!(g.money, money_before);
    }

    #[test]
    fn test_baseball_card() {
        let ten = Card::new(Value::Ten, Suit::Heart);
        let hand = SelectHand::new(vec![ten, ten]);
        let best = hand.best_hand().unwrap();

        let mut g = Game::default();
        g.stage = Stage::Blind(Blind::Small);

        // Pair (level 1): 10 chips, 2 mult
        // Played (2 tens): 20 chips
        // (10 + 20) * 2 = 60
        assert_eq!(g.calc_score(best.clone()), 60);

        // Buy 2 uncommon jokers (MidasMask, Pareidolia) and BaseballCard
        let midas = Jokers::MidasMask(MidasMask {});
        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(midas.clone());
        g.buy_joker(midas).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        g.calc_score(best.clone());

        let pareidolia = Jokers::Pareidolia(Pareidolia {});
        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(pareidolia.clone());
        g.buy_joker(pareidolia).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        g.calc_score(best.clone());

        let bb = Jokers::BaseballCard(BaseballCard {});
        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(bb.clone());
        g.buy_joker(bb).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // BaseballCard: 2 uncommon * X1.5
        // (10 + 20) * (2 * 1.5 * 1.5) = 30 * 4 = 120
        assert_eq!(g.calc_score(best.clone()), 120);
    }

    #[test]
    fn test_baseball_card_no_uncommon() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let best = hand.best_hand().unwrap();

        let mut g = Game::default();
        g.stage = Stage::Blind(Blind::Small);

        // Buy BaseballCard with no uncommon jokers
        let bb = Jokers::BaseballCard(BaseballCard {});
        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(bb.clone());
        g.buy_joker(bb).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // High card (level 1): 5 chips, 1 mult
        // Played (1 ace): 11 chips
        // (5 + 11) * 1 = 16
        assert_eq!(g.calc_score(best.clone()), 16);
    }

    #[test]
    fn test_bull() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::Bull(Bull {});

        let mut g = Game::default();
        g.stage = Stage::Blind(Blind::Small);
        g.money = 100;
        let best = hand.best_hand().unwrap();

        // High card (level 1): 5 chips, 1 mult
        // Played (1 ace): 11 chips
        // (5 + 11) * 1 = 16
        assert_eq!(g.calc_score(best.clone()), 16);

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // Bull: (100 + 1000 - 6) * 2 = 1094 * 2 = 2188 chips
        // (5 + 11 + 2188) * 1 = 2204
        assert_eq!(g.calc_score(best.clone()), 2204);
    }

    #[test]
    fn test_bull_no_money() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::Bull(Bull {});

        let mut g = Game::default();
        g.stage = Stage::Blind(Blind::Small);
        let best = hand.best_hand().unwrap();

        // High card (level 1): 5 chips, 1 mult
        // Played (1 ace): 11 chips
        // (5 + 11) * 1 = 16
        assert_eq!(g.calc_score(best.clone()), 16);

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);
        g.money = 0;

        // Bull: 0 * 2 = 0 chips
        // (5 + 11) * 1 = 16
        assert_eq!(g.calc_score(best.clone()), 16);
    }

    #[test]
    fn test_walkie_talkie() {
        let ten = Card::new(Value::Ten, Suit::Heart);
        let hand = SelectHand::new(vec![ten, ten]);
        let j = Jokers::WalkieTalkie(WalkieTalkie {});

        // Pair (level 1): 10 chips, 2 mult
        // Played (2 tens): 20 chips
        // (10 + 20) * 2 = 60
        let before = 60;

        // WalkieTalkie: 2 tens * (+10 chips, +4 mult) = +20 chips, +8 mult
        // (10 + 20 + 20) * (2 + 8) = 50 * 10 = 500
        let after = 500;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_walkie_talkie_other_cards() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::WalkieTalkie(WalkieTalkie {});

        // High card (level 1): 5 chips, 1 mult
        // Played (1 ace): 11 chips
        // (5 + 11) * 1 = 16
        let before = 16;

        // WalkieTalkie: 0 tens or fours -> no bonus
        let after = 16;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_smiley_face() {
        let king = Card::new(Value::King, Suit::Heart);
        let king2 = Card::new(Value::King, Suit::Diamond);
        let hand = SelectHand::new(vec![king, king2]);
        let j = Jokers::SmileyFace(SmileyFace {});

        // Pair (level 1): 10 chips, 2 mult
        // Played (2 kings): 10 + 10 = 20 chips
        // (10 + 20) * 2 = 60
        let before = 60;

        // Smiley Face: 2 face cards * +5 mult = +10 mult
        // (10 + 20) * (2 + 10) = 30 * 12 = 360
        let after = 360;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_smiley_face_no_face() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::SmileyFace(SmileyFace {});

        // High card (level 1): 5 chips, 1 mult
        // Played (1 ace): 11 chips
        // (5 + 11) * 1 = 16
        let before = 16;

        // Smiley Face: 0 face cards -> no bonus
        let after = 16;
        score_before_after_joker(j, hand, before, after);
    }

    #[test]
    fn test_golden_ticket_no_gold() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace, ace]);
        let j = Jokers::GoldenTicket(GoldenTicket {});

        let mut g = Game::default();
        g.stage = Stage::Blind(Blind::Small);
        let score = g.calc_score(hand.best_hand().unwrap());
        assert_eq!(score, 64);

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        let money_before = g.money;
        g.calc_score(hand.best_hand().unwrap());
        assert_eq!(g.money, money_before);
    }

    #[test]
    fn test_golden_ticket_with_gold() {
        let mut ace = Card::new(Value::Ace, Suit::Heart);
        ace.enhancement = Some(Enhancement::Gold);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::GoldenTicket(GoldenTicket {});

        let mut g = Game::default();
        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        let money_before = g.money;
        g.calc_score(hand.best_hand().unwrap());
        assert_eq!(g.money, money_before + 4);
    }

    #[test]
    fn test_acrobat_final_hand() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::Acrobat(Acrobat {});

        let mut g = Game::default();
        g.stage = Stage::Blind(Blind::Small);
        g.calc_score(hand.best_hand().unwrap());

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // Final hand: plays == 0
        g.plays = 0;
        // High card ace: (5 + 11) * 1 = 16
        // Acrobat X3: 16 * 3 = 48
        assert_eq!(g.calc_score(hand.best_hand().unwrap()), 48);
    }

    #[test]
    fn test_acrobat_not_final_hand() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::Acrobat(Acrobat {});

        let mut g = Game::default();
        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // Not final hand: plays > 0
        g.plays = 1;
        // High card ace: (5 + 11) * 1 = 16
        // Acrobat: no bonus
        assert_eq!(g.calc_score(hand.best_hand().unwrap()), 16);
    }

    #[test]
    fn test_rough_gem_no_diamonds() {
        let ace = Card::new(Value::Ace, Suit::Heart);
        let hand = SelectHand::new(vec![ace, ace]);
        let j = Jokers::RoughGem(RoughGem {});

        let mut g = Game::default();
        g.stage = Stage::Blind(Blind::Small);
        g.calc_score(hand.best_hand().unwrap());

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        let money_before = g.money;
        g.calc_score(hand.best_hand().unwrap());
        assert_eq!(g.money, money_before);
    }

    #[test]
    fn test_rough_gem_with_diamonds() {
        let dia1 = Card::new(Value::Ace, Suit::Diamond);
        let dia2 = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![dia1, dia2]);
        let j = Jokers::RoughGem(RoughGem {});

        let mut g = Game::default();
        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        let money_before = g.money;
        g.calc_score(hand.best_hand().unwrap());
        assert_eq!(g.money, money_before + 1);
    }

    #[test]
    fn test_bloodstone_no_hearts() {
        let ace = Card::new(Value::Ace, Suit::Club);
        let hand = SelectHand::new(vec![ace]);
        let j = Jokers::Bloodstone(Bloodstone {});

        let mut g = Game::default();
        g.stage = Stage::Blind(Blind::Small);
        g.calc_score(hand.best_hand().unwrap());

        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // High card club: (5 + 11) * 1 = 16, no hearts -> no Xmult
        assert_eq!(g.calc_score(hand.best_hand().unwrap()), 16);
    }

    #[test]
    fn test_bloodstone_with_hearts() {
        let heart = Card::new(Value::Ace, Suit::Heart);
        let heart2 = Card::new(Value::Ace, Suit::Diamond);
        let hand = SelectHand::new(vec![heart, heart2]);
        let j = Jokers::Bloodstone(Bloodstone {});

        let mut g = Game::default();
        g.money += 1000;
        g.stage = Stage::Shop();
        g.shop.jokers.push(j.clone());
        g.buy_joker(j).unwrap();
        g.stage = Stage::Blind(Blind::Small);

        // Pair of aces (level 1): 10 chips, 2 mult
        // Played: Ace heart (11 chips) + Ace diamond (11 chips) = 22 chips
        // (10 + 22) * 2 = 64
        // With Bloodstone: 1 heart card, 50% chance X1.5
        // Expected: sometimes 64, sometimes 96
        let mut saw_increase = false;
        let mut saw_no_increase = false;
        for _ in 0..50 {
            let score = g.calc_score(hand.best_hand().unwrap());
            if score == 96 {
                saw_increase = true;
            } else if score == 64 {
                saw_no_increase = true;
            }
        }
        assert!(saw_increase, "Bloodstone should sometimes Xmult");
        assert!(saw_no_increase, "Bloodstone should sometimes not Xmult");
    }
}
