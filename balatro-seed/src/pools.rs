//! Raw draw-order pool tables, transcribed verbatim from
//! `TheSoul/include/items.hpp`. Order **is** the draw-index space — do not
//! reorder, dedupe, or alphabetize. Each table has a `_POOL` sibling
//! pairing it with its `resolve.rs` resolver for typed draws.

use crate::pool::Pool;
use crate::resolve;
use balatro_types::{BossBlind, Card, Enhancement, Jokers, Planets, Spectral, Tag, Tarot, Voucher};

pub const TAROTS: &[&str] = &[
    "The Fool",
    "The Magician",
    "The High Priestess",
    "The Empress",
    "The Emperor",
    "The Hierophant",
    "The Lovers",
    "The Chariot",
    "Justice",
    "The Hermit",
    "The Wheel of Fortune",
    "Strength",
    "The Hanged Man",
    "Death",
    "Temperance",
    "The Devil",
    "The Tower",
    "The Star",
    "The Moon",
    "The Sun",
    "Judgement",
    "The World",
];

pub(crate) const TAROTS_POOL: Pool<Tarot> = Pool::new(TAROTS, resolve::resolve_tarot);

pub const PLANETS: &[&str] = &[
    "Mercury", "Venus", "Earth", "Mars", "Jupiter", "Saturn", "Uranus", "Neptune", "Pluto",
    "Planet X", "Ceres", "Eris",
];

pub(crate) const PLANETS_POOL: Pool<Planets> = Pool::new(PLANETS, resolve::resolve_planet);

/// Includes two "RETRY" sentinels, never actually returned — they just
/// lower the effective odds of a real spectral.
pub const SPECTRALS: &[&str] = &[
    "Familiar",
    "Grim",
    "Incantation",
    "Talisman",
    "Aura",
    "Wraith",
    "Sigil",
    "Ouija",
    "Ectoplasm",
    "Immolate",
    "Ankh",
    "Deja Vu",
    "Hex",
    "Trance",
    "Medium",
    "Cryptid",
    "RETRY",
    "RETRY",
];

pub(crate) const SPECTRALS_POOL: Pool<Spectral> = Pool::new(SPECTRALS, resolve::resolve_spectral);

pub const COMMON_JOKERS: &[&str] = &[
    "Joker",
    "Greedy Joker",
    "Lusty Joker",
    "Wrathful Joker",
    "Gluttonous Joker",
    "Jolly Joker",
    "Zany Joker",
    "Mad Joker",
    "Crazy Joker",
    "Droll Joker",
    "Sly Joker",
    "Wily Joker",
    "Clever Joker",
    "Devious Joker",
    "Crafty Joker",
    "Half Joker",
    "Credit Card",
    "Banner",
    "Mystic Summit",
    "8 Ball",
    "Misprint",
    "Raised Fist",
    "Chaos the Clown",
    "Scary Face",
    "Abstract Joker",
    "Delayed Gratification",
    "Gros Michel",
    "Even Steven",
    "Odd Todd",
    "Scholar",
    "Business Card",
    "Supernova",
    "Ride the Bus",
    "Egg",
    "Runner",
    "Ice Cream",
    "Splash",
    "Blue Joker",
    "Faceless Joker",
    "Green Joker",
    "Superposition",
    "To Do List",
    "Cavendish",
    "Red Card",
    "Square Joker",
    "Riff-raff",
    "Photograph",
    "Reserved Parking",
    "Mail-In Rebate",
    "Hallucination",
    "Fortune Teller",
    "Juggler",
    "Drunkard",
    "Golden Joker",
    "Popcorn",
    "Walkie Talkie",
    "Smiley Face",
    "Golden Ticket",
    "Swashbuckler",
    "Hanging Chad",
    "Shoot the Moon",
];

pub(crate) const COMMON_JOKERS_POOL: Pool<Jokers> = Pool::new(COMMON_JOKERS, resolve::resolve_joker);

pub const UNCOMMON_JOKERS: &[&str] = &[
    "Joker Stencil",
    "Four Fingers",
    "Mime",
    "Ceremonial Dagger",
    "Marble Joker",
    "Loyalty Card",
    "Dusk",
    "Fibonacci",
    "Steel Joker",
    "Hack",
    "Pareidolia",
    "Space Joker",
    "Burglar",
    "Blackboard",
    "Sixth Sense",
    "Constellation",
    "Hiker",
    "Card Sharp",
    "Madness",
    "Seance",
    "Vampire",
    "Shortcut",
    "Hologram",
    "Cloud 9",
    "Rocket",
    "Midas Mask",
    "Luchador",
    "Gift Card",
    "Turtle Bean",
    "Erosion",
    "To the Moon",
    "Stone Joker",
    "Lucky Cat",
    "Bull",
    "Diet Cola",
    "Trading Card",
    "Flash Card",
    "Spare Trousers",
    "Ramen",
    "Seltzer",
    "Castle",
    "Mr. Bones",
    "Acrobat",
    "Sock and Buskin",
    "Troubadour",
    "Certificate",
    "Smeared Joker",
    "Throwback",
    "Rough Gem",
    "Bloodstone",
    "Arrowhead",
    "Onyx Agate",
    "Glass Joker",
    "Showman",
    "Flower Pot",
    "Merry Andy",
    "Oops! All 6s",
    "The Idol",
    "Seeing Double",
    "Matador",
    "Satellite",
    "Cartomancer",
    "Astronomer",
    "Bootstraps",
];

pub(crate) const UNCOMMON_JOKERS_POOL: Pool<Jokers> =
    Pool::new(UNCOMMON_JOKERS, resolve::resolve_joker);

pub const RARE_JOKERS: &[&str] = &[
    "DNA",
    "Vagabond",
    "Baron",
    "Obelisk",
    "Baseball Card",
    "Ancient Joker",
    "Campfire",
    "Blueprint",
    "Wee Joker",
    "Hit the Road",
    "The Duo",
    "The Trio",
    "The Family",
    "The Order",
    "The Tribe",
    "Stuntman",
    "Invisible Joker",
    "Brainstorm",
    "Driver's License",
    "Burnt Joker",
];

pub(crate) const RARE_JOKERS_POOL: Pool<Jokers> = Pool::new(RARE_JOKERS, resolve::resolve_joker);

pub const LEGENDARY_JOKERS: &[&str] = &["Canio", "Triboulet", "Yorick", "Chicot", "Perkeo"];

pub(crate) const LEGENDARY_JOKERS_POOL: Pool<Jokers> =
    Pool::new(LEGENDARY_JOKERS, resolve::resolve_joker);

pub const ENHANCEMENTS: &[&str] = &[
    "Bonus", "Mult", "Wild", "Glass", "Steel", "Stone", "Gold", "Lucky",
];

pub(crate) const ENHANCEMENTS_POOL: Pool<Enhancement> =
    Pool::new(ENHANCEMENTS, resolve::resolve_enhancement);

/// `{suit}_{rank}`. Order within each suit is 2-9, Ace, Jack, King, Queen,
/// Ten — not the usual 2..10,J,Q,K,A ordering.
pub const CARDS: &[&str] = &[
    "C_2", "C_3", "C_4", "C_5", "C_6", "C_7", "C_8", "C_9", "C_A", "C_J", "C_K", "C_Q", "C_T",
    "D_2", "D_3", "D_4", "D_5", "D_6", "D_7", "D_8", "D_9", "D_A", "D_J", "D_K", "D_Q", "D_T",
    "H_2", "H_3", "H_4", "H_5", "H_6", "H_7", "H_8", "H_9", "H_A", "H_J", "H_K", "H_Q", "H_T",
    "S_2", "S_3", "S_4", "S_5", "S_6", "S_7", "S_8", "S_9", "S_A", "S_J", "S_K", "S_Q", "S_T",
];

pub(crate) const CARDS_POOL: Pool<Card> = Pool::new(CARDS, resolve::resolve_card_base);

pub const VOUCHERS: &[&str] = &[
    "Overstock",
    "Overstock Plus",
    "Clearance Sale",
    "Liquidation",
    "Hone",
    "Glow Up",
    "Reroll Surplus",
    "Reroll Glut",
    "Crystal Ball",
    "Omen Globe",
    "Telescope",
    "Observatory",
    "Grabber",
    "Nacho Tong",
    "Wasteful",
    "Recyclomancy",
    "Tarot Merchant",
    "Tarot Tycoon",
    "Planet Merchant",
    "Planet Tycoon",
    "Seed Money",
    "Money Tree",
    "Blank",
    "Antimatter",
    "Magic Trick",
    "Illusion",
    "Hieroglyph",
    "Petroglyph",
    "Director's Cut",
    "Retcon",
    "Paint Brush",
    "Palette",
];

pub(crate) const VOUCHERS_POOL: Pool<Voucher> = Pool::new(VOUCHERS, resolve::resolve_voucher);

pub const TAGS: &[&str] = &[
    "Uncommon Tag",
    "Rare Tag",
    "Negative Tag",
    "Foil Tag",
    "Holographic Tag",
    "Polychrome Tag",
    "Investment Tag",
    "Voucher Tag",
    "Boss Tag",
    "Standard Tag",
    "Charm Tag",
    "Meteor Tag",
    "Buffoon Tag",
    "Handy Tag",
    "Garbage Tag",
    "Ethereal Tag",
    "Coupon Tag",
    "Double Tag",
    "Juggle Tag",
    "D6 Tag",
    "Top-up Tag",
    "Speed Tag",
    "Orbital Tag",
    "Economy Tag",
];

pub(crate) const TAGS_POOL: Pool<Tag> = Pool::new(TAGS, resolve::resolve_tag);

pub const BOSSES: &[&str] = &[
    "The Arm",
    "The Club",
    "The Eye",
    "Amber Acorn",
    "Cerulean Bell",
    "Crimson Heart",
    "Verdant Leaf",
    "Violet Vessel",
    "The Fish",
    "The Flint",
    "The Goad",
    "The Head",
    "The Hook",
    "The House",
    "The Manacle",
    "The Mark",
    "The Mouth",
    "The Needle",
    "The Ox",
    "The Pillar",
    "The Plant",
    "The Psychic",
    "The Serpent",
    "The Tooth",
    "The Wall",
    "The Water",
    "The Wheel",
    "The Window",
];

pub(crate) const BOSSES_POOL: Pool<BossBlind> = Pool::new(BOSSES, resolve::resolve_boss);

/// `(name, weight)`. Index 0 is a sentinel — see `Instance::randweightedchoice`.
pub const PACKS: &[(&str, f64)] = &[
    ("RETRY", 22.42),
    ("Arcana Pack", 4.0),
    ("Jumbo Arcana Pack", 2.0),
    ("Mega Arcana Pack", 0.5),
    ("Celestial Pack", 4.0),
    ("Jumbo Celestial Pack", 2.0),
    ("Mega Celestial Pack", 0.5),
    ("Standard Pack", 4.0),
    ("Jumbo Standard Pack", 2.0),
    ("Mega Standard Pack", 0.5),
    ("Buffoon Pack", 1.2),
    ("Jumbo Buffoon Pack", 0.6),
    ("Mega Buffoon Pack", 0.15),
    ("Spectral Pack", 0.6),
    ("Jumbo Spectral Pack", 0.3),
    ("Mega Spectral Pack", 0.07),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_sentinel_weight_equals_sum_of_real_entries() {
        let sum: f64 = PACKS[1..].iter().map(|(_, w)| w).sum();
        assert!(
            (sum - PACKS[0].1).abs() < 1e-9,
            "sentinel weight {} != sum of real entries {}",
            PACKS[0].1,
            sum
        );
    }

    #[test]
    fn all_pool_names_resolve() {
        let mut failures: Vec<String> = Vec::new();
        for (label, unresolved) in [
            ("tarot", TAROTS_POOL.unresolved()),
            ("planet", PLANETS_POOL.unresolved()),
            ("spectral", SPECTRALS_POOL.unresolved()),
            ("common joker", COMMON_JOKERS_POOL.unresolved()),
            ("uncommon joker", UNCOMMON_JOKERS_POOL.unresolved()),
            ("rare joker", RARE_JOKERS_POOL.unresolved()),
            ("legendary joker", LEGENDARY_JOKERS_POOL.unresolved()),
            ("voucher", VOUCHERS_POOL.unresolved()),
            ("tag", TAGS_POOL.unresolved()),
            ("boss", BOSSES_POOL.unresolved()),
            ("enhancement", ENHANCEMENTS_POOL.unresolved()),
            ("card base", CARDS_POOL.unresolved()),
        ] {
            for name in unresolved {
                failures.push(format!("{label}: {name:?}"));
            }
        }
        assert!(
            failures.is_empty(),
            "{} pool name(s) failed to resolve against balatro_types:\n{}",
            failures.len(),
            failures.join("\n")
        );
    }
}
