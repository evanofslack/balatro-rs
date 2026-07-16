use crate::card::Edition;
#[cfg(feature = "python")]
use pyo3::pyclass;
use strum::EnumIter;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
}

impl std::fmt::Display for Rarity {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Self::Common => "Common",
            Self::Uncommon => "Uncommon",
            Self::Rare => "Rare",
            Self::Legendary => "Legendary",
        };
        write!(f, "{s}")
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Categories {
    Chips,
    AdditiveMult,
    MultiplicativeMult,
    ChipsAndAdditiveMult,
    Effect,
    Retrigger,
    Economy,
}

/// Stake-gated per-instance debuff flags, as tracked in real save files.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub struct Stickers {
    pub eternal: bool,
    pub perishable: bool,
    pub rental: bool,
}

/// `Jokers` is the definitive static repr of all jokers in the game.
// We could pass around `Box<dyn Joker>` but it doesn't work so nice with pyo3 and serde.
// Since we know all variants (one for each joker), we define an enum that implements
// our `Joker` trait. This macro just reduces the amount of boilerplate we have to copy
// to match each joker and call its methods. It ends up creating an enum `Jokers` that
// contains each joker struct (where each struct impl `Joker`), and we impl `Joker` for
// `Jokers` enum by matching each case and calling underlying methods.
// https://stackoverflow.com/questions/63848427/using-enums-for-dynamic-polymorphism-in-rust/63849405#63849405
macro_rules! make_jokers {
    ($($x:ident),* $(,)?) => {
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "python", pyclass(eq))]
        #[derive(Debug, Clone, Eq, PartialEq, Hash, EnumIter)]
        pub enum Jokers {
            $($x($x),)*
        }

        $(
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "python", pyclass(eq))]
        #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
        pub struct $x {
            pub edition: Edition,
            pub stickers: Stickers,
        }
        )*

        impl Jokers {
            fn inner_ref(&self) -> (&Edition, &Stickers) {
                match self {
                    $(Self::$x(j) => (&j.edition, &j.stickers),)*
                }
            }

            fn inner_mut(&mut self) -> (&mut Edition, &mut Stickers) {
                match self {
                    $(Self::$x(j) => (&mut j.edition, &mut j.stickers),)*
                }
            }

            pub fn edition(&self) -> Edition { *self.inner_ref().0 }
            pub fn stickers(&self) -> Stickers { *self.inner_ref().1 }

            pub fn set_edition(&mut self, edition: Edition) { *self.inner_mut().0 = edition; }
            pub fn set_stickers(&mut self, stickers: Stickers) { *self.inner_mut().1 = stickers; }
        }

        impl std::str::FromStr for Jokers {
            type Err = strum::ParseError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                $(if s.eq_ignore_ascii_case(stringify!($x)) {
                    return Ok(Self::$x($x::default()));
                })*
                Err(strum::ParseError::VariantNotFound)
            }
        }
    };
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
    FourFingers,
    Mime,
    CreditCard,
    CeremonialDagger,
    Banner,
    MysticSummit,
    MarbleJoker,
    LoyaltyCard,
    EightBall,
    Misprint,
    Dusk,
    RaisedFist,
    ChaosTheClown,
    Fibonacci,
    SteelJoker,
    ScaryFace,
    AbstractJoker,
    DelayedGratification,
    Hack,
    Pareidolia,
    GrosMichel,
    EvenSteven,
    OddTodd,
    Scholar,
    BusinessCard,
    Supernova,
    RideTheBus,
    SpaceJoker,
    Egg,
    Burglar,
    Blackboard,
    Runner,
    IceCream,
    Dna,
    Splash,
    BlueJoker,
    SixthSense,
    Constellation,
    Hiker,
    FacelessJoker,
    GreenJoker,
    Superposition,
    ToDoList,
    Cavendish,
    CardSharp,
    RedCard,
    Madness,
    SquareJoker,
    Seance,
    RiffRaff,
    Vampire,
    Shortcut,
    Hologram,
    Vagabond,
    Baron,
    Cloud9,
    Rocket,
    Obelisk,
    MidasMask,
    Luchador,
    Photograph,
    GiftCard,
    TurtleBean,
    Erosion,
    ReservedParking,
    MailInRebate,
    ToTheMoon,
    Hallucination,
    FortuneTeller,
    Juggler,
    Drunkard,
    StoneJoker,
    GoldenJoker,
    LuckyCat,
    BaseballCard,
    Bull,
    DietCola,
    TradingCard,
    FlashCard,
    Popcorn,
    SpareTrousers,
    AncientJoker,
    Ramen,
    WalkieTalkie,
    Seltzer,
    Castle,
    SmileyFace,
    Campfire,
    GoldenTicket,
    MrBones,
    Acrobat,
    SockAndBuskin,
    Swashbuckler,
    Troubadour,
    Certificate,
    SmearedJoker,
    Throwback,
    HangingChad,
    RoughGem,
    Bloodstone,
    Arrowhead,
    OnyxAgate,
    GlassJoker,
    Showman,
    FlowerPot,
    Blueprint,
    WeeJoker,
    MerryAndy,
    OopsAllSixes,
    TheIdol,
    SeeingDouble,
    Matador,
    HitTheRoad,
    TheDuo,
    TheTrio,
    TheFamily,
    TheOrder,
    TheTribe,
    Stuntman,
    InvisibleJoker,
    Brainstorm,
    Satellite,
    ShootTheMoon,
    DriversLicense,
    Cartomancer,
    Astronomer,
    BurntJoker,
    Bootstraps,
    Canio,
    Triboulet,
    Yorick,
    Chicot,
    Perkeo,
);

/// Declares all per-variant constant data and generates the corresponding
/// accessor methods on `Jokers` in one pass, keeping each joker's data
/// on a single line.
///
/// Columns: variant, name, rarity, cost, blueprint, perishable, eternal,
///          category, desc
macro_rules! joker_data {
    ($(
        $variant:ident,
        $name:literal,
        $rarity:expr,
        $cost:expr,
        $blueprint:expr,
        $perishable:expr,
        $eternal:expr,
        $category:expr,
        $desc:literal
    );* $(;)?) => {
        impl Jokers {
            pub fn name(&self) -> &'static str {
                match self { $(Self::$variant(_) => $name,)* }
            }
            pub fn rarity(&self) -> Rarity {
                match self { $(Self::$variant(_) => $rarity,)* }
            }
            pub fn cost(&self) -> usize {
                match self { $(Self::$variant(_) => $cost,)* }
            }
            pub fn blueprint_compatible(&self) -> bool {
                match self { $(Self::$variant(_) => $blueprint,)* }
            }
            pub fn perishable_compatible(&self) -> bool {
                match self { $(Self::$variant(_) => $perishable,)* }
            }
            pub fn eternal_compatible(&self) -> bool {
                match self { $(Self::$variant(_) => $eternal,)* }
            }
            pub fn category(&self) -> Categories {
                match self { $(Self::$variant(_) => $category,)* }
            }
            pub fn desc(&self) -> &'static str {
                match self { $(Self::$variant(_) => $desc,)* }
            }
            pub fn sell_value(&self) -> usize {
                std::cmp::max(1, self.cost() / 2)
            }
        }
    };
}

use Categories::*;
use Rarity::*;

// variant, name, rarity, cost, blueprint, perishable, eternal, category, desc
joker_data!(
    TheJoker,              "Joker",                Common,    2,  true,  true,  true,  AdditiveMult,       "+4 Mult";
    GreedyJoker,           "Greedy Joker",         Common,    5,  true,  true,  true,  AdditiveMult,       "Played cards with Diamond suit icon Diamond suit give +3 Mult when scored";
    LustyJoker,            "Lusty Joker",          Common,    5,  true,  true,  true,  AdditiveMult,       "Played cards with Heart suit icon Heart suit give +3 Mult when scored";
    WrathfulJoker,         "Wrathful Joker",       Common,    5,  true,  true,  true,  AdditiveMult,       "Played cards with Spade suit icon Spade suit give +3 Mult when scored";
    GluttonousJoker,       "Gluttonous Joker",     Common,    5,  true,  true,  true,  AdditiveMult,       "Played cards with Club suit icon Club suit give +3 Mult when scored";
    JollyJoker,            "Jolly Joker",          Common,    3,  true,  true,  true,  AdditiveMult,       "+8 Mult if played hand contains a Pair";
    ZanyJoker,             "Zany Joker",           Common,    4,  true,  true,  true,  AdditiveMult,       "+12 Mult if played hand contains a Three of a Kind";
    MadJoker,              "Mad Joker",            Common,    4,  true,  true,  true,  AdditiveMult,       "+10 Mult if played hand contains a Two Pair";
    CrazyJoker,            "Crazy Joker",          Common,    4,  true,  true,  true,  AdditiveMult,       "+12 Mult if played hand contains a Straight";
    DrollJoker,            "Droll Joker",          Common,    4,  true,  true,  true,  AdditiveMult,       "+10 Mult if played hand contains a Flush";
    SlyJoker,              "Sly Joker",            Common,    3,  true,  true,  true,  Chips,              "+50 Chips if played hand contains a Pair";
    WilyJoker,             "Wily Joker",           Common,    4,  true,  true,  true,  Chips,              "+100 Chips if played hand contains a Three of a Kind";
    CleverJoker,           "Clever Joker",         Common,    4,  true,  true,  true,  Chips,              "+80 Chips if played hand contains a Two Pair";
    DeviousJoker,          "Devious Joker",        Common,    4,  true,  true,  true,  Chips,              "+100 Chips if played hand contains a Straight";
    CraftyJoker,           "Crafty Joker",         Common,    4,  true,  true,  true,  Chips,              "+80 Chips if played hand contains a Flush";
    HalfJoker,             "Half Joker",           Common,    5,  true,  true,  true,  AdditiveMult,       "+20 Mult if played hand contains 3 or fewer cards";
    JokerStencil,          "Joker Stencil",        Uncommon,  8,  true,  true,  true,  MultiplicativeMult, "X1 Mult for each empty Joker slot. Joker Stencil included";
    FourFingers,           "Four Fingers",         Uncommon,  7,  false, true,  true,  Effect,             "All Flushes and Straights can be made with 4 cards";
    Mime,                  "Mime",                 Uncommon,  5,  true,  true,  true,  Retrigger,          "Retrigger all card held in hand abilities";
    CreditCard,            "Credit Card",          Common,    1,  false, true,  true,  Economy,            "Go up to -$20 in debt";
    CeremonialDagger,      "Ceremonial Dagger",    Uncommon,  6,  true,  false, true,  AdditiveMult,       "When Blind is selected, destroy Joker to the right and permanently add double its sell value to it's Mult";
    Banner,                "Banner",               Common,    5,  true,  true,  true,  Chips,              "+30 Chips for each remaining discard";
    MysticSummit,          "Mystic Summit",        Common,    5,  true,  true,  true,  AdditiveMult,       "+15 Mult when 0 discards remaining";
    MarbleJoker,           "Marble Joker",         Uncommon,  6,  true,  true,  true,  Effect,             "Adds one Stone card to the deck when Blind is selected";
    LoyaltyCard,           "Loyalty Card",         Uncommon,  5,  true,  true,  true,  MultiplicativeMult, "X4 Mult every 6 hands played";
    EightBall,             "8 Ball",               Common,    5,  true,  true,  true,  Effect,             "1 in 4 chance for each played 8 to create a Tarot card when scored";
    Misprint,              "Misprint",             Common,    4,  true,  true,  true,  AdditiveMult,       "+0-23 Mult";
    Dusk,                  "Dusk",                 Uncommon,  5,  true,  true,  true,  Retrigger,          "Retrigger all played cards in final hand of the round";
    RaisedFist,            "Raised Fist",          Common,    5,  true,  true,  true,  AdditiveMult,       "Adds double the rank of lowest ranked card held in hand to Mult";
    ChaosTheClown,         "Chaos the Clown",      Common,    4,  false, true,  true,  Effect,             "1 free Reroll per shop";
    Fibonacci,             "Fibonacci",            Uncommon,  8,  true,  true,  true,  AdditiveMult,       "Each played Ace, 2, 3, 5, or 8 gives +8 Mult when scored";
    SteelJoker,            "Steel Joker",          Uncommon,  7,  true,  true,  true,  MultiplicativeMult, "Gives X0.2 Mult for each Steel Card in your full deck";
    ScaryFace,             "Scary Face",           Common,    4,  true,  true,  true,  Chips,              "Played face cards give +30 Chips when scored";
    AbstractJoker,         "Abstract Joker",       Common,    4,  true,  true,  true,  AdditiveMult,       "+3 Mult for each Joker card";
    DelayedGratification,  "Delayed Gratification",Common,    4,  false, true,  true,  Economy,            "Earn $2 per discard if no discards are used by end of the round";
    Hack,                  "Hack",                 Uncommon,  6,  true,  true,  true,  Retrigger,          "Retrigger each played 2, 3, 4, or 5";
    Pareidolia,            "Pareidolia",           Uncommon,  5,  false, true,  true,  Effect,             "All cards are considered face cards";
    GrosMichel,            "Gros Michel",          Common,    5,  true,  true,  false, AdditiveMult,       "+15 Mult. 1 in 6 chance this card is destroyed at the end of round.";
    EvenSteven,            "Even Steven",          Common,    4,  true,  true,  true,  AdditiveMult,       "Played cards with even rank give +4 Mult when scored (10, 8, 6, 4, 2)";
    OddTodd,               "Odd Todd",             Common,    4,  true,  true,  true,  Chips,              "Played cards with odd rank give +31 Chips when scored (A, 9, 7, 5, 3)";
    Scholar,               "Scholar",              Common,    4,  true,  true,  true,  ChipsAndAdditiveMult,"Played Aces give +20 Chips and +4 Mult when scored";
    BusinessCard,          "Business Card",        Common,    4,  true,  true,  true,  Economy,            "Played face cards have a 1 in 2 chance to give $2 when scored";
    Supernova,             "Supernova",            Common,    5,  true,  true,  true,  AdditiveMult,       "Adds the number of times poker hand has been played this run to Mult";
    RideTheBus,            "Ride the Bus",         Common,    6,  true,  false, true,  AdditiveMult,       "This Joker gains +1 Mult per consecutive hand played without a scoring face card";
    SpaceJoker,            "Space Joker",          Uncommon,  5,  true,  true,  true,  Effect,             "1 in 4 chance to upgrade level of played poker hand";
    Egg,                   "Egg",                  Common,    4,  false, true,  true,  Economy,            "Gains $3 of sell value at end of round";
    Burglar,               "Burglar",              Uncommon,  6,  true,  true,  true,  Effect,             "When Blind is selected, gain +3 Hands and lose all discards";
    Blackboard,            "Blackboard",           Uncommon,  6,  true,  true,  true,  MultiplicativeMult, "X3 Mult if all cards held in hand are Spades or Clubs";
    Runner,                "Runner",               Common,    5,  true,  false, true,  Chips,              "Gains +15 Chips if played hand contains a Straight";
    IceCream,              "Ice Cream",            Common,    5,  true,  true,  false, Chips,              "+100 Chips. -5 Chips for every hand played";
    Dna,                   "DNA",                  Rare,      8,  true,  true,  true,  Effect,             "If first hand of round has only 1 card, add a permanent copy to deck and draw it to hand";
    Splash,                "Splash",               Common,    3,  false, true,  true,  Effect,             "Every played card counts in scoring";
    BlueJoker,             "Blue Joker",           Common,    5,  true,  true,  true,  Chips,              "+2 Chips for each remaining card in deck";
    SixthSense,            "Sixth Sense",          Uncommon,  6,  false, true,  true,  Effect,             "If first hand of round is a single 6, destroy it and create a Spectral card";
    Constellation,         "Constellation",        Uncommon,  6,  true,  false, true,  MultiplicativeMult, "This Joker gains X0.1 Mult every time a Planet card is used";
    Hiker,                 "Hiker",                Uncommon,  5,  true,  true,  true,  Chips,              "Every played card permanently gains +5 Chips when scored";
    FacelessJoker,         "Faceless Joker",       Common,    4,  true,  true,  true,  Economy,            "Earn $5 if 3 or more face cards are discarded at the same time";
    GreenJoker,            "Green Joker",          Common,    4,  true,  false, true,  AdditiveMult,       "+1 Mult per hand played. -1 Mult per discard";
    Superposition,         "Superposition",        Common,    4,  true,  true,  true,  Effect,             "Create a Tarot card if poker hand contains an Ace and a Straight";
    ToDoList,              "To Do List",           Common,    4,  true,  true,  true,  Economy,            "Earn $4 if poker hand is a [Poker Hand], poker hand changes at end of round";
    Cavendish,             "Cavendish",            Common,    4,  true,  true,  false, MultiplicativeMult, "X3 Mult. 1 in 1000 chance this card is destroyed at the end of round";
    CardSharp,             "Card Sharp",           Uncommon,  6,  true,  true,  true,  MultiplicativeMult, "X3 Mult if played poker hand has already been played this round";
    RedCard,               "Red Card",             Common,    5,  true,  false, true,  AdditiveMult,       "This Joker gains +3 Mult when any Booster Pack is skipped";
    Madness,               "Madness",              Uncommon,  7,  true,  false, true,  MultiplicativeMult, "When Small Blind or Big Blind is selected, gain X0.5 Mult and destroy a random Joker";
    SquareJoker,           "Square Joker",         Common,    4,  true,  false, true,  Chips,              "This Joker gains +4 Chips if played hand has exactly 4 cards";
    Seance,                "Seance",               Uncommon,  6,  true,  true,  true,  Effect,             "If poker hand is a Straight Flush, create a random Spectral card";
    RiffRaff,              "Riff-raff",            Common,    6,  true,  true,  true,  Effect,             "When Blind is selected, create 2 Common Jokers";
    Vampire,               "Vampire",              Uncommon,  7,  true,  false, true,  MultiplicativeMult, "This Joker gains X0.1 Mult per scoring Enhanced card played, removes card Enhancement";
    Shortcut,              "Shortcut",             Uncommon,  7,  false, true,  true,  Effect,             "Allows Straights to be made with gaps of 1 rank (ex: 10 8 6 5 3)";
    Hologram,              "Hologram",             Uncommon,  7,  true,  false, true,  MultiplicativeMult, "This Joker gains X0.25 Mult every time a playing card is added to your deck";
    Vagabond,              "Vagabond",             Rare,      8,  true,  true,  true,  Effect,             "Create a Tarot card if hand is played with $4 or less";
    Baron,                 "Baron",                Rare,      8,  true,  true,  true,  MultiplicativeMult, "Each King held in hand gives X1.5 Mult";
    Cloud9,                "Cloud 9",              Uncommon,  7,  false, true,  true,  Economy,            "Earn $1 for each 9 in your full deck at end of round";
    Rocket,                "Rocket",               Uncommon,  6,  false, false, true,  Economy,            "Earn $1 at end of round. Payout increases by $2 when Boss Blind is defeated";
    Obelisk,               "Obelisk",              Rare,      8,  true,  false, true,  MultiplicativeMult, "This Joker gains X0.2 Mult per consecutive hand played without playing your most played poker hand";
    MidasMask,             "Midas Mask",           Uncommon,  7,  false, true,  true,  Effect,             "All played face cards become Gold cards when scored";
    Luchador,              "Luchador",             Uncommon,  5,  true,  true,  false, Effect,             "Sell this card to disable the current Boss Blind";
    Photograph,            "Photograph",           Common,    5,  true,  true,  true,  MultiplicativeMult, "First played face card gives X2 Mult when scored";
    GiftCard,              "Gift Card",            Uncommon,  6,  false, true,  true,  Economy,            "Add $1 of sell value to every Joker and Consumable card at end of round";
    TurtleBean,            "Turtle Bean",          Uncommon,  6,  false, true,  false, Effect,             "+5 hand size, reduces by 1 each round";
    Erosion,               "Erosion",              Uncommon,  6,  true,  true,  true,  AdditiveMult,       "+4 Mult for each card below the deck's starting size in your full deck";
    ReservedParking,       "Reserved Parking",     Common,    6,  true,  true,  true,  Economy,            "Each face card held in hand has a 1 in 2 chance to give $1";
    MailInRebate,          "Mail-In Rebate",       Common,    4,  true,  true,  true,  Economy,            "Earn $5 for each discarded [rank], rank changes every round";
    ToTheMoon,             "To the Moon",          Uncommon,  5,  false, true,  true,  Economy,            "Earn an extra $1 of interest for every $5 you have at end of round";
    Hallucination,         "Hallucination",        Common,    4,  true,  true,  true,  Effect,             "1 in 2 chance to create a Tarot card when any Booster Pack is opened";
    FortuneTeller,         "Fortune Teller",       Common,    6,  true,  true,  true,  AdditiveMult,       "+1 Mult per Tarot card used this run";
    Juggler,               "Juggler",              Common,    4,  false, true,  true,  Effect,             "+1 hand size";
    Drunkard,              "Drunkard",             Common,    4,  false, true,  true,  Effect,             "+1 discard each round";
    StoneJoker,            "Stone Joker",          Uncommon,  6,  true,  true,  true,  Chips,              "Gives +25 Chips for each Stone Card in your full deck";
    GoldenJoker,           "Golden Joker",         Common,    6,  false, true,  true,  Economy,            "Earn $4 at end of round";
    LuckyCat,              "Lucky Cat",            Uncommon,  6,  true,  false, true,  MultiplicativeMult, "This Joker gains X0.25 Mult every time a Lucky card successfully triggers";
    BaseballCard,          "Baseball Card",        Rare,      8,  true,  true,  true,  MultiplicativeMult, "Uncommon Jokers each give X1.5 Mult";
    Bull,                  "Bull",                 Uncommon,  6,  true,  true,  true,  Chips,              "+2 Chips for each $1 you have";
    DietCola,              "Diet Cola",            Uncommon,  6,  true,  true,  false, Effect,             "Sell this card to create a free Double Tag";
    TradingCard,           "Trading Card",         Uncommon,  6,  false, true,  true,  Economy,            "If first discard of round has only 1 card, destroy it and earn $3";
    FlashCard,             "Flash Card",           Uncommon,  5,  true,  false, true,  AdditiveMult,       "This Joker gains +2 Mult per reroll in the shop";
    Popcorn,               "Popcorn",              Common,    5,  true,  true,  false, AdditiveMult,       "+20 Mult. -4 Mult per round played";
    SpareTrousers,         "Spare Trousers",       Uncommon,  6,  true,  false, true,  AdditiveMult,       "This Joker gains +2 Mult if played hand contains a Two Pair";
    AncientJoker,          "Ancient Joker",        Rare,      8,  true,  true,  true,  MultiplicativeMult, "Each played card with [suit] gives X1.5 Mult when scored, suit changes at end of round";
    Ramen,                 "Ramen",                Uncommon,  6,  true,  true,  false, MultiplicativeMult, "X2 Mult, loses X0.01 Mult per card discarded";
    WalkieTalkie,          "Walkie Talkie",        Common,    4,  true,  true,  true,  ChipsAndAdditiveMult,"Each played 10 or 4 gives +10 Chips and +4 Mult when scored";
    Seltzer,               "Seltzer",              Uncommon,  6,  true,  true,  false, Retrigger,          "Retrigger all cards played for the next 10 hands";
    Castle,                "Castle",               Uncommon,  6,  true,  false, true,  Chips,              "This Joker gains +3 Chips per discarded [suit] card, suit changes every round";
    SmileyFace,            "Smiley Face",          Common,    4,  true,  true,  true,  AdditiveMult,       "Played face cards give +5 Mult when scored";
    Campfire,              "Campfire",             Rare,      9,  true,  true,  true,  MultiplicativeMult, "This Joker gains X0.25 Mult for each card sold, resets when Boss Blind is defeated";
    GoldenTicket,          "Golden Ticket",        Common,    5,  true,  true,  true,  Economy,            "Played Gold cards earn $4 when scored";
    MrBones,               "Mr. Bones",            Uncommon,  5,  false, true,  false, Effect,             "Prevents Death if chips scored are at least 25% of required chips. self destructs";
    Acrobat,               "Acrobat",              Uncommon,  6,  true,  true,  true,  MultiplicativeMult, "X3 Mult on final hand of round";
    SockAndBuskin,         "Sock and Buskin",      Uncommon,  6,  true,  true,  true,  Retrigger,          "Retrigger all played face cards";
    Swashbuckler,          "Swashbuckler",         Common,    4,  true,  true,  true,  AdditiveMult,       "Adds the sell value of all other owned Jokers to Mult";
    Troubadour,            "Troubadour",           Uncommon,  6,  false, true,  true,  Effect,             "+2 hand size, -1 hand per round";
    Certificate,           "Certificate",          Uncommon,  6,  true,  true,  true,  Effect,             "When round begins, add a random playing card with a random seal to your hand";
    SmearedJoker,          "Smeared Joker",        Uncommon,  7,  false, true,  true,  Effect,             "Hearts and Diamonds count as the same suit, Spades and Clubs count as the same suit";
    Throwback,             "Throwback",            Uncommon,  6,  true,  true,  true,  MultiplicativeMult, "X0.25 Mult for each Blind skipped this run";
    HangingChad,           "Hanging Chad",         Common,    4,  true,  true,  true,  Retrigger,          "Retrigger first played card used in scoring 2 additional times";
    RoughGem,              "Rough Gem",            Uncommon,  7,  true,  true,  true,  Economy,            "Played cards with Diamond suit earn $1 when scored";
    Bloodstone,            "Bloodstone",           Uncommon,  7,  true,  true,  true,  MultiplicativeMult, "1 in 2 chance for played cards with Heart suit to give X1.5 Mult when scored";
    Arrowhead,             "Arrowhead",            Uncommon,  7,  true,  true,  true,  Chips,              "Played cards with Spade suit give +50 Chips when scored";
    OnyxAgate,             "Onyx Agate",           Uncommon,  7,  true,  true,  true,  AdditiveMult,       "Played cards with Club suit give +7 Mult when scored";
    GlassJoker,            "Glass Joker",          Uncommon,  6,  true,  false, true,  MultiplicativeMult, "This Joker gains X0.75 Mult for every Glass Card that is destroyed";
    Showman,               "Showman",              Uncommon,  5,  false, true,  true,  Effect,             "Joker, Tarot, Planet, and Spectral cards may appear multiple times";
    FlowerPot,             "Flower Pot",           Uncommon,  6,  true,  true,  true,  MultiplicativeMult, "X3 Mult if poker hand contains a Diamond card, Club card, Heart card, and Spade card";
    Blueprint,             "Blueprint",            Rare,      10, true,  true,  true,  Effect,             "Copies ability of Joker to the right";
    WeeJoker,              "Wee Joker",            Rare,      8,  true,  false, true,  Chips,              "This Joker gains +8 Chips when each played 2 is scored";
    MerryAndy,             "Merry Andy",           Uncommon,  7,  false, true,  true,  Effect,             "+3 discards each round, -1 hand size";
    OopsAllSixes,          "Oops! All 6s",         Uncommon,  4,  false, true,  true,  Effect,             "Doubles all listed probabilities (ex: 1 in 3 -> 2 in 3)";
    TheIdol,               "The Idol",             Uncommon,  6,  true,  true,  true,  MultiplicativeMult, "Each played [rank] of [suit] gives X2 Mult when scored, card changes every round";
    SeeingDouble,          "Seeing Double",        Uncommon,  6,  true,  true,  true,  MultiplicativeMult, "X2 Mult if played hand has a scoring Club card and a scoring card of any other suit";
    Matador,               "Matador",              Uncommon,  7,  true,  true,  true,  Economy,            "Earn $8 if played hand triggers the Boss Blind ability";
    HitTheRoad,            "Hit the Road",         Rare,      8,  true,  true,  true,  MultiplicativeMult, "This Joker gains X0.5 Mult for every Jack discarded this round";
    TheDuo,                "The Duo",              Rare,      8,  true,  true,  true,  MultiplicativeMult, "X2 Mult if played hand contains a Pair";
    TheTrio,               "The Trio",             Rare,      8,  true,  true,  true,  MultiplicativeMult, "X3 Mult if played hand contains a Three of a Kind";
    TheFamily,             "The Family",           Rare,      8,  true,  true,  true,  MultiplicativeMult, "X4 Mult if played hand contains a Four of a Kind";
    TheOrder,              "The Order",            Rare,      8,  true,  true,  true,  MultiplicativeMult, "X3 Mult if played hand contains a Straight";
    TheTribe,              "The Tribe",            Rare,      8,  true,  true,  true,  MultiplicativeMult, "X2 Mult if played hand contains a Flush";
    Stuntman,              "Stuntman",             Rare,      7,  true,  true,  true,  Chips,              "+250 Chips, -2 hand size";
    InvisibleJoker,        "Invisible Joker",      Rare,      8,  false, true,  false, Effect,             "After 2 rounds, sell this card to Duplicate a random Joker (Removes Negative from copy)";
    Brainstorm,            "Brainstorm",           Rare,      10, true,  true,  true,  Effect,             "Copies the ability of leftmost Joker";
    Satellite,             "Satellite",            Uncommon,  6,  false, true,  true,  Economy,            "Earn $1 at end of round per unique Planet card used this run";
    ShootTheMoon,          "Shoot the Moon",       Common,    5,  true,  true,  true,  AdditiveMult,       "Each Queen held in hand gives +13 Mult";
    DriversLicense,        "Driver's License",     Rare,      7,  true,  true,  true,  MultiplicativeMult, "X3 Mult if you have at least 16 Enhanced cards in your full deck";
    Cartomancer,           "Cartomancer",          Uncommon,  6,  true,  true,  true,  Effect,             "Create a Tarot card when Blind is selected";
    Astronomer,            "Astronomer",           Uncommon,  8,  false, true,  true,  Effect,             "All Planet cards and Celestial Packs in the shop are free";
    BurntJoker,            "Burnt Joker",          Rare,      8,  true,  true,  true,  Effect,             "Upgrade the level of the first discarded poker hand each round";
    Bootstraps,            "Bootstraps",           Uncommon,  7,  true,  true,  true,  AdditiveMult,       "+2 Mult for every $5 you have";
    Canio,                 "Canio",                Legendary, 20, true,  true,  true,  MultiplicativeMult, "This Joker gains X1 Mult when a face card is destroyed";
    Triboulet,             "Triboulet",            Legendary, 20, true,  true,  true,  MultiplicativeMult, "Played Kings and Queens each give X2 Mult when scored";
    Yorick,                "Yorick",               Legendary, 20, true,  true,  true,  MultiplicativeMult, "This Joker gains X1 Mult every 23 cards discarded";
    Chicot,                "Chicot",               Legendary, 20, false, true,  true,  Effect,             "Disables effect of every Boss Blind";
    Perkeo,                "Perkeo",               Legendary, 20, true,  true,  true,  Effect,             "Creates a Negative copy of 1 random consumable card in your possession at the end of the shop";
);

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn test_jokers_from_str_round_trip() {
        for joker in Jokers::iter() {
            let debug = format!("{joker:?}");
            let variant_name = debug.split('(').next().unwrap();
            assert_eq!(variant_name.parse::<Jokers>().unwrap(), joker);
        }
    }

    #[test]
    fn test_jokers_from_str_case_insensitive() {
        let expected = Jokers::HalfJoker(HalfJoker::default());
        assert_eq!("halfjoker".parse::<Jokers>(), Ok(expected.clone()));
        assert_eq!("HALFJOKER".parse::<Jokers>(), Ok(expected));
    }

    #[test]
    fn test_jokers_from_str_invalid() {
        assert!("NotARealJoker".parse::<Jokers>().is_err());
    }
}
