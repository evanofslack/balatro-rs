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
/// Columns: variant, id, name, rarity, cost, blueprint, perishable, eternal,
///          category, desc
macro_rules! joker_data {
    ($(
        $variant:ident,
        $id:literal,
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
            /// Save-file id for this joker.
            pub fn id(&self) -> &'static str {
                match self { $(Self::$variant(_) => $id,)* }
            }
            /// Parses a save-file id into a `Jokers`.
            pub fn from_id(s: &str) -> Option<Self> {
                match s {
                    $($id => Some(Self::$variant($variant::default())),)*
                    _ => None,
                }
            }
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

// variant, id, name, rarity, cost, blueprint, perishable, eternal, category, desc
joker_data!(
    TheJoker,              "j_joker",                 "Joker",                Common,    2,  true,  true,  true,  AdditiveMult,       "+4 Mult";
    GreedyJoker,           "j_greedy_joker",          "Greedy Joker",         Common,    5,  true,  true,  true,  AdditiveMult,       "Played cards with Diamond suit icon Diamond suit give +3 Mult when scored";
    LustyJoker,            "j_lusty_joker",           "Lusty Joker",          Common,    5,  true,  true,  true,  AdditiveMult,       "Played cards with Heart suit icon Heart suit give +3 Mult when scored";
    WrathfulJoker,         "j_wrathful_joker",        "Wrathful Joker",       Common,    5,  true,  true,  true,  AdditiveMult,       "Played cards with Spade suit icon Spade suit give +3 Mult when scored";
    GluttonousJoker,       "j_gluttenous_joker",      "Gluttonous Joker",     Common,    5,  true,  true,  true,  AdditiveMult,       "Played cards with Club suit icon Club suit give +3 Mult when scored";
    JollyJoker,            "j_jolly",                 "Jolly Joker",          Common,    3,  true,  true,  true,  AdditiveMult,       "+8 Mult if played hand contains a Pair";
    ZanyJoker,             "j_zany",                  "Zany Joker",           Common,    4,  true,  true,  true,  AdditiveMult,       "+12 Mult if played hand contains a Three of a Kind";
    MadJoker,              "j_mad",                   "Mad Joker",            Common,    4,  true,  true,  true,  AdditiveMult,       "+10 Mult if played hand contains a Two Pair";
    CrazyJoker,            "j_crazy",                 "Crazy Joker",          Common,    4,  true,  true,  true,  AdditiveMult,       "+12 Mult if played hand contains a Straight";
    DrollJoker,            "j_droll",                 "Droll Joker",          Common,    4,  true,  true,  true,  AdditiveMult,       "+10 Mult if played hand contains a Flush";
    SlyJoker,              "j_sly",                   "Sly Joker",            Common,    3,  true,  true,  true,  Chips,              "+50 Chips if played hand contains a Pair";
    WilyJoker,             "j_wily",                  "Wily Joker",           Common,    4,  true,  true,  true,  Chips,              "+100 Chips if played hand contains a Three of a Kind";
    CleverJoker,           "j_clever",                "Clever Joker",         Common,    4,  true,  true,  true,  Chips,              "+80 Chips if played hand contains a Two Pair";
    DeviousJoker,          "j_devious",               "Devious Joker",        Common,    4,  true,  true,  true,  Chips,              "+100 Chips if played hand contains a Straight";
    CraftyJoker,           "j_crafty",                "Crafty Joker",         Common,    4,  true,  true,  true,  Chips,              "+80 Chips if played hand contains a Flush";
    HalfJoker,             "j_half",                  "Half Joker",           Common,    5,  true,  true,  true,  AdditiveMult,       "+20 Mult if played hand contains 3 or fewer cards";
    JokerStencil,          "j_stencil",               "Joker Stencil",        Uncommon,  8,  true,  true,  true,  MultiplicativeMult, "X1 Mult for each empty Joker slot. Joker Stencil included";
    FourFingers,           "j_four_fingers",          "Four Fingers",         Uncommon,  7,  false, true,  true,  Effect,             "All Flushes and Straights can be made with 4 cards";
    Mime,                  "j_mime",                  "Mime",                 Uncommon,  5,  true,  true,  true,  Retrigger,          "Retrigger all card held in hand abilities";
    CreditCard,            "j_credit_card",           "Credit Card",          Common,    1,  false, true,  true,  Economy,            "Go up to -$20 in debt";
    CeremonialDagger,      "j_ceremonial",            "Ceremonial Dagger",    Uncommon,  6,  true,  false, true,  AdditiveMult,       "When Blind is selected, destroy Joker to the right and permanently add double its sell value to it's Mult";
    Banner,                "j_banner",                "Banner",               Common,    5,  true,  true,  true,  Chips,              "+30 Chips for each remaining discard";
    MysticSummit,          "j_mystic_summit",         "Mystic Summit",        Common,    5,  true,  true,  true,  AdditiveMult,       "+15 Mult when 0 discards remaining";
    MarbleJoker,           "j_marble",                "Marble Joker",         Uncommon,  6,  true,  true,  true,  Effect,             "Adds one Stone card to the deck when Blind is selected";
    LoyaltyCard,           "j_loyalty_card",          "Loyalty Card",         Uncommon,  5,  true,  true,  true,  MultiplicativeMult, "X4 Mult every 6 hands played";
    EightBall,             "j_8_ball",                "8 Ball",               Common,    5,  true,  true,  true,  Effect,             "1 in 4 chance for each played 8 to create a Tarot card when scored";
    Misprint,              "j_misprint",              "Misprint",             Common,    4,  true,  true,  true,  AdditiveMult,       "+0-23 Mult";
    Dusk,                  "j_dusk",                  "Dusk",                 Uncommon,  5,  true,  true,  true,  Retrigger,          "Retrigger all played cards in final hand of the round";
    RaisedFist,            "j_raised_fist",           "Raised Fist",          Common,    5,  true,  true,  true,  AdditiveMult,       "Adds double the rank of lowest ranked card held in hand to Mult";
    ChaosTheClown,         "j_chaos",                 "Chaos the Clown",      Common,    4,  false, true,  true,  Effect,             "1 free Reroll per shop";
    Fibonacci,             "j_fibonacci",             "Fibonacci",            Uncommon,  8,  true,  true,  true,  AdditiveMult,       "Each played Ace, 2, 3, 5, or 8 gives +8 Mult when scored";
    SteelJoker,            "j_steel_joker",           "Steel Joker",          Uncommon,  7,  true,  true,  true,  MultiplicativeMult, "Gives X0.2 Mult for each Steel Card in your full deck";
    ScaryFace,             "j_scary_face",            "Scary Face",           Common,    4,  true,  true,  true,  Chips,              "Played face cards give +30 Chips when scored";
    AbstractJoker,         "j_abstract",              "Abstract Joker",       Common,    4,  true,  true,  true,  AdditiveMult,       "+3 Mult for each Joker card";
    DelayedGratification,  "j_delayed_grat",          "Delayed Gratification",Common,    4,  false, true,  true,  Economy,            "Earn $2 per discard if no discards are used by end of the round";
    Hack,                  "j_hack",                  "Hack",                 Uncommon,  6,  true,  true,  true,  Retrigger,          "Retrigger each played 2, 3, 4, or 5";
    Pareidolia,            "j_pareidolia",            "Pareidolia",           Uncommon,  5,  false, true,  true,  Effect,             "All cards are considered face cards";
    GrosMichel,            "j_gros_michel",           "Gros Michel",          Common,    5,  true,  true,  false, AdditiveMult,       "+15 Mult. 1 in 6 chance this card is destroyed at the end of round.";
    EvenSteven,            "j_even_steven",           "Even Steven",          Common,    4,  true,  true,  true,  AdditiveMult,       "Played cards with even rank give +4 Mult when scored (10, 8, 6, 4, 2)";
    OddTodd,               "j_odd_todd",              "Odd Todd",             Common,    4,  true,  true,  true,  Chips,              "Played cards with odd rank give +31 Chips when scored (A, 9, 7, 5, 3)";
    Scholar,               "j_scholar",               "Scholar",              Common,    4,  true,  true,  true,  ChipsAndAdditiveMult,"Played Aces give +20 Chips and +4 Mult when scored";
    BusinessCard,          "j_business",              "Business Card",        Common,    4,  true,  true,  true,  Economy,            "Played face cards have a 1 in 2 chance to give $2 when scored";
    Supernova,             "j_supernova",             "Supernova",            Common,    5,  true,  true,  true,  AdditiveMult,       "Adds the number of times poker hand has been played this run to Mult";
    RideTheBus,            "j_ride_the_bus",          "Ride the Bus",         Common,    6,  true,  false, true,  AdditiveMult,       "This Joker gains +1 Mult per consecutive hand played without a scoring face card";
    SpaceJoker,            "j_space",                 "Space Joker",          Uncommon,  5,  true,  true,  true,  Effect,             "1 in 4 chance to upgrade level of played poker hand";
    Egg,                   "j_egg",                   "Egg",                  Common,    4,  false, true,  true,  Economy,            "Gains $3 of sell value at end of round";
    Burglar,               "j_burglar",               "Burglar",              Uncommon,  6,  true,  true,  true,  Effect,             "When Blind is selected, gain +3 Hands and lose all discards";
    Blackboard,            "j_blackboard",            "Blackboard",           Uncommon,  6,  true,  true,  true,  MultiplicativeMult, "X3 Mult if all cards held in hand are Spades or Clubs";
    Runner,                "j_runner",                "Runner",               Common,    5,  true,  false, true,  Chips,              "Gains +15 Chips if played hand contains a Straight";
    IceCream,              "j_ice_cream",             "Ice Cream",            Common,    5,  true,  true,  false, Chips,              "+100 Chips. -5 Chips for every hand played";
    Dna,                   "j_dna",                   "DNA",                  Rare,      8,  true,  true,  true,  Effect,             "If first hand of round has only 1 card, add a permanent copy to deck and draw it to hand";
    Splash,                "j_splash",                "Splash",               Common,    3,  false, true,  true,  Effect,             "Every played card counts in scoring";
    BlueJoker,             "j_blue_joker",            "Blue Joker",           Common,    5,  true,  true,  true,  Chips,              "+2 Chips for each remaining card in deck";
    SixthSense,            "j_sixth_sense",           "Sixth Sense",          Uncommon,  6,  false, true,  true,  Effect,             "If first hand of round is a single 6, destroy it and create a Spectral card";
    Constellation,         "j_constellation",         "Constellation",        Uncommon,  6,  true,  false, true,  MultiplicativeMult, "This Joker gains X0.1 Mult every time a Planet card is used";
    Hiker,                 "j_hiker",                 "Hiker",                Uncommon,  5,  true,  true,  true,  Chips,              "Every played card permanently gains +5 Chips when scored";
    FacelessJoker,         "j_faceless",              "Faceless Joker",       Common,    4,  true,  true,  true,  Economy,            "Earn $5 if 3 or more face cards are discarded at the same time";
    GreenJoker,            "j_green_joker",           "Green Joker",          Common,    4,  true,  false, true,  AdditiveMult,       "+1 Mult per hand played. -1 Mult per discard";
    Superposition,         "j_superposition",         "Superposition",        Common,    4,  true,  true,  true,  Effect,             "Create a Tarot card if poker hand contains an Ace and a Straight";
    ToDoList,              "j_todo_list",             "To Do List",           Common,    4,  true,  true,  true,  Economy,            "Earn $4 if poker hand is a [Poker Hand], poker hand changes at end of round";
    Cavendish,             "j_cavendish",             "Cavendish",            Common,    4,  true,  true,  false, MultiplicativeMult, "X3 Mult. 1 in 1000 chance this card is destroyed at the end of round";
    CardSharp,             "j_card_sharp",            "Card Sharp",           Uncommon,  6,  true,  true,  true,  MultiplicativeMult, "X3 Mult if played poker hand has already been played this round";
    RedCard,               "j_red_card",              "Red Card",             Common,    5,  true,  false, true,  AdditiveMult,       "This Joker gains +3 Mult when any Booster Pack is skipped";
    Madness,               "j_madness",               "Madness",              Uncommon,  7,  true,  false, true,  MultiplicativeMult, "When Small Blind or Big Blind is selected, gain X0.5 Mult and destroy a random Joker";
    SquareJoker,           "j_square",                "Square Joker",         Common,    4,  true,  false, true,  Chips,              "This Joker gains +4 Chips if played hand has exactly 4 cards";
    Seance,                "j_seance",                "Seance",               Uncommon,  6,  true,  true,  true,  Effect,             "If poker hand is a Straight Flush, create a random Spectral card";
    RiffRaff,              "j_riff_raff",             "Riff-raff",            Common,    6,  true,  true,  true,  Effect,             "When Blind is selected, create 2 Common Jokers";
    Vampire,               "j_vampire",               "Vampire",              Uncommon,  7,  true,  false, true,  MultiplicativeMult, "This Joker gains X0.1 Mult per scoring Enhanced card played, removes card Enhancement";
    Shortcut,              "j_shortcut",              "Shortcut",             Uncommon,  7,  false, true,  true,  Effect,             "Allows Straights to be made with gaps of 1 rank (ex: 10 8 6 5 3)";
    Hologram,              "j_hologram",              "Hologram",             Uncommon,  7,  true,  false, true,  MultiplicativeMult, "This Joker gains X0.25 Mult every time a playing card is added to your deck";
    Vagabond,              "j_vagabond",              "Vagabond",             Rare,      8,  true,  true,  true,  Effect,             "Create a Tarot card if hand is played with $4 or less";
    Baron,                 "j_baron",                 "Baron",                Rare,      8,  true,  true,  true,  MultiplicativeMult, "Each King held in hand gives X1.5 Mult";
    Cloud9,                "j_cloud_9",               "Cloud 9",              Uncommon,  7,  false, true,  true,  Economy,            "Earn $1 for each 9 in your full deck at end of round";
    Rocket,                "j_rocket",                "Rocket",               Uncommon,  6,  false, false, true,  Economy,            "Earn $1 at end of round. Payout increases by $2 when Boss Blind is defeated";
    Obelisk,               "j_obelisk",               "Obelisk",              Rare,      8,  true,  false, true,  MultiplicativeMult, "This Joker gains X0.2 Mult per consecutive hand played without playing your most played poker hand";
    MidasMask,             "j_midas_mask",            "Midas Mask",           Uncommon,  7,  false, true,  true,  Effect,             "All played face cards become Gold cards when scored";
    Luchador,              "j_luchador",              "Luchador",             Uncommon,  5,  true,  true,  false, Effect,             "Sell this card to disable the current Boss Blind";
    Photograph,            "j_photograph",            "Photograph",           Common,    5,  true,  true,  true,  MultiplicativeMult, "First played face card gives X2 Mult when scored";
    GiftCard,              "j_gift",                  "Gift Card",            Uncommon,  6,  false, true,  true,  Economy,            "Add $1 of sell value to every Joker and Consumable card at end of round";
    TurtleBean,            "j_turtle_bean",           "Turtle Bean",          Uncommon,  6,  false, true,  false, Effect,             "+5 hand size, reduces by 1 each round";
    Erosion,               "j_erosion",               "Erosion",              Uncommon,  6,  true,  true,  true,  AdditiveMult,       "+4 Mult for each card below the deck's starting size in your full deck";
    ReservedParking,       "j_reserved_parking",      "Reserved Parking",     Common,    6,  true,  true,  true,  Economy,            "Each face card held in hand has a 1 in 2 chance to give $1";
    MailInRebate,          "j_mail",                  "Mail-In Rebate",       Common,    4,  true,  true,  true,  Economy,            "Earn $5 for each discarded [rank], rank changes every round";
    ToTheMoon,             "j_to_the_moon",           "To the Moon",          Uncommon,  5,  false, true,  true,  Economy,            "Earn an extra $1 of interest for every $5 you have at end of round";
    Hallucination,         "j_hallucination",         "Hallucination",        Common,    4,  true,  true,  true,  Effect,             "1 in 2 chance to create a Tarot card when any Booster Pack is opened";
    FortuneTeller,         "j_fortune_teller",        "Fortune Teller",       Common,    6,  true,  true,  true,  AdditiveMult,       "+1 Mult per Tarot card used this run";
    Juggler,               "j_juggler",               "Juggler",              Common,    4,  false, true,  true,  Effect,             "+1 hand size";
    Drunkard,              "j_drunkard",              "Drunkard",             Common,    4,  false, true,  true,  Effect,             "+1 discard each round";
    StoneJoker,            "j_stone",                 "Stone Joker",          Uncommon,  6,  true,  true,  true,  Chips,              "Gives +25 Chips for each Stone Card in your full deck";
    GoldenJoker,           "j_golden",                "Golden Joker",         Common,    6,  false, true,  true,  Economy,            "Earn $4 at end of round";
    LuckyCat,              "j_lucky_cat",             "Lucky Cat",            Uncommon,  6,  true,  false, true,  MultiplicativeMult, "This Joker gains X0.25 Mult every time a Lucky card successfully triggers";
    BaseballCard,          "j_baseball",              "Baseball Card",        Rare,      8,  true,  true,  true,  MultiplicativeMult, "Uncommon Jokers each give X1.5 Mult";
    Bull,                  "j_bull",                  "Bull",                 Uncommon,  6,  true,  true,  true,  Chips,              "+2 Chips for each $1 you have";
    DietCola,              "j_diet_cola",             "Diet Cola",            Uncommon,  6,  true,  true,  false, Effect,             "Sell this card to create a free Double Tag";
    TradingCard,           "j_trading",               "Trading Card",         Uncommon,  6,  false, true,  true,  Economy,            "If first discard of round has only 1 card, destroy it and earn $3";
    FlashCard,             "j_flash",                 "Flash Card",           Uncommon,  5,  true,  false, true,  AdditiveMult,       "This Joker gains +2 Mult per reroll in the shop";
    Popcorn,               "j_popcorn",               "Popcorn",              Common,    5,  true,  true,  false, AdditiveMult,       "+20 Mult. -4 Mult per round played";
    SpareTrousers,         "j_trousers",              "Spare Trousers",       Uncommon,  6,  true,  false, true,  AdditiveMult,       "This Joker gains +2 Mult if played hand contains a Two Pair";
    AncientJoker,          "j_ancient",               "Ancient Joker",        Rare,      8,  true,  true,  true,  MultiplicativeMult, "Each played card with [suit] gives X1.5 Mult when scored, suit changes at end of round";
    Ramen,                 "j_ramen",                 "Ramen",                Uncommon,  6,  true,  true,  false, MultiplicativeMult, "X2 Mult, loses X0.01 Mult per card discarded";
    WalkieTalkie,          "j_walkie_talkie",         "Walkie Talkie",        Common,    4,  true,  true,  true,  ChipsAndAdditiveMult,"Each played 10 or 4 gives +10 Chips and +4 Mult when scored";
    Seltzer,               "j_selzer",                "Seltzer",              Uncommon,  6,  true,  true,  false, Retrigger,          "Retrigger all cards played for the next 10 hands";
    Castle,                "j_castle",                "Castle",               Uncommon,  6,  true,  false, true,  Chips,              "This Joker gains +3 Chips per discarded [suit] card, suit changes every round";
    SmileyFace,            "j_smiley",                "Smiley Face",          Common,    4,  true,  true,  true,  AdditiveMult,       "Played face cards give +5 Mult when scored";
    Campfire,              "j_campfire",              "Campfire",             Rare,      9,  true,  true,  true,  MultiplicativeMult, "This Joker gains X0.25 Mult for each card sold, resets when Boss Blind is defeated";
    GoldenTicket,          "j_ticket",                "Golden Ticket",        Common,    5,  true,  true,  true,  Economy,            "Played Gold cards earn $4 when scored";
    MrBones,               "j_mr_bones",              "Mr. Bones",            Uncommon,  5,  false, true,  false, Effect,             "Prevents Death if chips scored are at least 25% of required chips. self destructs";
    Acrobat,               "j_acrobat",               "Acrobat",              Uncommon,  6,  true,  true,  true,  MultiplicativeMult, "X3 Mult on final hand of round";
    SockAndBuskin,         "j_sock_and_buskin",       "Sock and Buskin",      Uncommon,  6,  true,  true,  true,  Retrigger,          "Retrigger all played face cards";
    Swashbuckler,          "j_swashbuckler",          "Swashbuckler",         Common,    4,  true,  true,  true,  AdditiveMult,       "Adds the sell value of all other owned Jokers to Mult";
    Troubadour,            "j_troubadour",            "Troubadour",           Uncommon,  6,  false, true,  true,  Effect,             "+2 hand size, -1 hand per round";
    Certificate,           "j_certificate",           "Certificate",          Uncommon,  6,  true,  true,  true,  Effect,             "When round begins, add a random playing card with a random seal to your hand";
    SmearedJoker,          "j_smeared",               "Smeared Joker",        Uncommon,  7,  false, true,  true,  Effect,             "Hearts and Diamonds count as the same suit, Spades and Clubs count as the same suit";
    Throwback,             "j_throwback",             "Throwback",            Uncommon,  6,  true,  true,  true,  MultiplicativeMult, "X0.25 Mult for each Blind skipped this run";
    HangingChad,           "j_hanging_chad",          "Hanging Chad",         Common,    4,  true,  true,  true,  Retrigger,          "Retrigger first played card used in scoring 2 additional times";
    RoughGem,              "j_rough_gem",             "Rough Gem",            Uncommon,  7,  true,  true,  true,  Economy,            "Played cards with Diamond suit earn $1 when scored";
    Bloodstone,            "j_bloodstone",            "Bloodstone",           Uncommon,  7,  true,  true,  true,  MultiplicativeMult, "1 in 2 chance for played cards with Heart suit to give X1.5 Mult when scored";
    Arrowhead,             "j_arrowhead",             "Arrowhead",            Uncommon,  7,  true,  true,  true,  Chips,              "Played cards with Spade suit give +50 Chips when scored";
    OnyxAgate,             "j_onyx_agate",            "Onyx Agate",           Uncommon,  7,  true,  true,  true,  AdditiveMult,       "Played cards with Club suit give +7 Mult when scored";
    GlassJoker,            "j_glass",                 "Glass Joker",          Uncommon,  6,  true,  false, true,  MultiplicativeMult, "This Joker gains X0.75 Mult for every Glass Card that is destroyed";
    Showman,               "j_ring_master",           "Showman",              Uncommon,  5,  false, true,  true,  Effect,             "Joker, Tarot, Planet, and Spectral cards may appear multiple times";
    FlowerPot,             "j_flower_pot",            "Flower Pot",           Uncommon,  6,  true,  true,  true,  MultiplicativeMult, "X3 Mult if poker hand contains a Diamond card, Club card, Heart card, and Spade card";
    Blueprint,             "j_blueprint",             "Blueprint",            Rare,      10, true,  true,  true,  Effect,             "Copies ability of Joker to the right";
    WeeJoker,              "j_wee",                   "Wee Joker",            Rare,      8,  true,  false, true,  Chips,              "This Joker gains +8 Chips when each played 2 is scored";
    MerryAndy,             "j_merry_andy",            "Merry Andy",           Uncommon,  7,  false, true,  true,  Effect,             "+3 discards each round, -1 hand size";
    OopsAllSixes,          "j_oops",                  "Oops! All 6s",         Uncommon,  4,  false, true,  true,  Effect,             "Doubles all listed probabilities (ex: 1 in 3 -> 2 in 3)";
    TheIdol,               "j_idol",                  "The Idol",             Uncommon,  6,  true,  true,  true,  MultiplicativeMult, "Each played [rank] of [suit] gives X2 Mult when scored, card changes every round";
    SeeingDouble,          "j_seeing_double",         "Seeing Double",        Uncommon,  6,  true,  true,  true,  MultiplicativeMult, "X2 Mult if played hand has a scoring Club card and a scoring card of any other suit";
    Matador,               "j_matador",               "Matador",              Uncommon,  7,  true,  true,  true,  Economy,            "Earn $8 if played hand triggers the Boss Blind ability";
    HitTheRoad,            "j_hit_the_road",          "Hit the Road",         Rare,      8,  true,  true,  true,  MultiplicativeMult, "This Joker gains X0.5 Mult for every Jack discarded this round";
    TheDuo,                "j_duo",                   "The Duo",              Rare,      8,  true,  true,  true,  MultiplicativeMult, "X2 Mult if played hand contains a Pair";
    TheTrio,               "j_trio",                  "The Trio",             Rare,      8,  true,  true,  true,  MultiplicativeMult, "X3 Mult if played hand contains a Three of a Kind";
    TheFamily,             "j_family",                "The Family",           Rare,      8,  true,  true,  true,  MultiplicativeMult, "X4 Mult if played hand contains a Four of a Kind";
    TheOrder,              "j_order",                 "The Order",            Rare,      8,  true,  true,  true,  MultiplicativeMult, "X3 Mult if played hand contains a Straight";
    TheTribe,              "j_tribe",                 "The Tribe",            Rare,      8,  true,  true,  true,  MultiplicativeMult, "X2 Mult if played hand contains a Flush";
    Stuntman,              "j_stuntman",              "Stuntman",             Rare,      7,  true,  true,  true,  Chips,              "+250 Chips, -2 hand size";
    InvisibleJoker,        "j_invisible",             "Invisible Joker",      Rare,      8,  false, true,  false, Effect,             "After 2 rounds, sell this card to Duplicate a random Joker (Removes Negative from copy)";
    Brainstorm,            "j_brainstorm",            "Brainstorm",           Rare,      10, true,  true,  true,  Effect,             "Copies the ability of leftmost Joker";
    Satellite,             "j_satellite",             "Satellite",            Uncommon,  6,  false, true,  true,  Economy,            "Earn $1 at end of round per unique Planet card used this run";
    ShootTheMoon,          "j_shoot_the_moon",        "Shoot the Moon",       Common,    5,  true,  true,  true,  AdditiveMult,       "Each Queen held in hand gives +13 Mult";
    DriversLicense,        "j_drivers_license",       "Driver's License",     Rare,      7,  true,  true,  true,  MultiplicativeMult, "X3 Mult if you have at least 16 Enhanced cards in your full deck";
    Cartomancer,           "j_cartomancer",           "Cartomancer",          Uncommon,  6,  true,  true,  true,  Effect,             "Create a Tarot card when Blind is selected";
    Astronomer,            "j_astronomer",            "Astronomer",           Uncommon,  8,  false, true,  true,  Effect,             "All Planet cards and Celestial Packs in the shop are free";
    BurntJoker,            "j_burnt",                 "Burnt Joker",          Rare,      8,  true,  true,  true,  Effect,             "Upgrade the level of the first discarded poker hand each round";
    Bootstraps,            "j_bootstraps",            "Bootstraps",           Uncommon,  7,  true,  true,  true,  AdditiveMult,       "+2 Mult for every $5 you have";
    Canio,                 "j_caino",                 "Canio",                Legendary, 20, true,  true,  true,  MultiplicativeMult, "This Joker gains X1 Mult when a face card is destroyed";
    Triboulet,             "j_triboulet",             "Triboulet",            Legendary, 20, true,  true,  true,  MultiplicativeMult, "Played Kings and Queens each give X2 Mult when scored";
    Yorick,                "j_yorick",                "Yorick",               Legendary, 20, true,  true,  true,  MultiplicativeMult, "This Joker gains X1 Mult every 23 cards discarded";
    Chicot,                "j_chicot",                "Chicot",               Legendary, 20, false, true,  true,  Effect,             "Disables effect of every Boss Blind";
    Perkeo,                "j_perkeo",                "Perkeo",               Legendary, 20, true,  true,  true,  Effect,             "Creates a Negative copy of 1 random consumable card in your possession at the end of the shop";
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
    fn test_jokers_id_round_trip() {
        for joker in Jokers::iter() {
            let id = joker.id();
            assert_eq!(Jokers::from_id(id).map(|j| j.id()), Some(id));
        }
    }

    #[test]
    fn test_jokers_known_ids() {
        assert_eq!(Jokers::TheJoker(TheJoker::default()).id(), "j_joker");
        assert_eq!(
            Jokers::GluttonousJoker(GluttonousJoker::default()).id(),
            "j_gluttenous_joker"
        );
        assert_eq!(
            Jokers::from_id("j_scary_face"),
            Some(Jokers::ScaryFace(ScaryFace::default()))
        );
        assert_eq!(Jokers::from_id("j_not_a_real_joker"), None);
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
