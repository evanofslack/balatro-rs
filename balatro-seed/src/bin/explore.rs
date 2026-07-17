//! Prints a seed's expected ante-by-ante contents in the same text format
//! as `TheSoul`'s website output, for direct manual diffing against it.
//!
//! Usage: `explore SEED [--ante N] [--cards-per-ante 15,50,50,50,50,50,50,50]
//! [--vouchers bought|offered] [--fresh-profile]`
//!
//! Deliberately mirrors quirks of the website's own reference JS rather
//! than "fixing" them, since the goal is a byte-diffable match:
//! - `init_locks` is called once for ante 1 before the loop, `init_unlocks`
//!   once per ante inside it — not symmetric per-ante calls.
//! - `--vouchers bought` (default) locks a drawn voucher (and unlocks its
//!   upgrade tier) but never *activates* it — so voucher-driven rate effects
//!   (Hone, Tarot Tycoon, ...) never kick in, matching the site's own demo
//!   loop, not a live run. `--vouchers offered` skips the lock entirely, so
//!   an unbought voucher can resurface in a later ante.

use balatro_seed::{Instance, ShopItem, pack_info, voucher_upgrade};
use balatro_types::{Card, Edition, Enhancement, Seal};

fn edition_prefix(e: Edition) -> &'static str {
    match e {
        Edition::Base => "",
        Edition::Foil => "Foil ",
        Edition::Holographic => "Holographic ",
        Edition::Polychrome => "Polychrome ",
        Edition::Negative => "Negative ",
    }
}

fn enhancement_name(e: Enhancement) -> &'static str {
    match e {
        Enhancement::Bonus => "Bonus",
        Enhancement::Mult => "Mult",
        Enhancement::Wild => "Wild",
        Enhancement::Glass => "Glass",
        Enhancement::Steel => "Steel",
        Enhancement::Stone => "Stone",
        Enhancement::Gold => "Gold",
        Enhancement::Lucky => "Lucky",
    }
}

fn seal_name(s: Seal) -> &'static str {
    match s {
        Seal::Gold => "Gold",
        Seal::Red => "Red",
        Seal::Blue => "Blue",
        Seal::Purple => "Purple",
    }
}

fn value_text(v: balatro_types::Value) -> String {
    match char::from(v) {
        'T' => "10".to_string(),
        'J' => "Jack".to_string(),
        'Q' => "Queen".to_string(),
        'K' => "King".to_string(),
        'A' => "Ace".to_string(),
        c => c.to_string(),
    }
}

fn render_card(card: &Card) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(seal) = card.seal {
        parts.push(format!("{} Seal", seal_name(seal)));
    }
    let edition = edition_prefix(card.edition);
    if !edition.is_empty() {
        parts.push(edition.trim().to_string());
    }
    if let Some(enh) = card.enhancement {
        parts.push(enhancement_name(enh).to_string());
    }
    parts.push(format!("{} of {}", value_text(card.value), card.suit));
    parts.join(" ")
}

fn render_shop_item(item: ShopItem) -> String {
    match item {
        ShopItem::Joker(j) => format!("{}{}", edition_prefix(j.edition()), j.name()),
        ShopItem::Consumable(c) => c.name(),
        ShopItem::PlayingCard => "[playing card: not implemented]".to_string(),
    }
}

fn render_pack_contents(inst: &mut Instance, category: &str, size: i32, ante: i32) -> String {
    match category {
        "Celestial Pack" => inst
            .next_celestial_pack(size, ante)
            .iter()
            .map(|c| c.name())
            .collect::<Vec<_>>()
            .join(", "),
        "Arcana Pack" => inst
            .next_arcana_pack(size, ante)
            .iter()
            .map(|c| c.name())
            .collect::<Vec<_>>()
            .join(", "),
        "Spectral Pack" => inst
            .next_spectral_pack(size, ante)
            .iter()
            .map(|c| c.name())
            .collect::<Vec<_>>()
            .join(", "),
        "Buffoon Pack" => inst
            .next_buffoon_pack(size, ante)
            .iter()
            .map(|j| format!("{}{}", edition_prefix(j.edition()), j.name()))
            .collect::<Vec<_>>()
            .join(", "),
        "Standard Pack" => inst
            .next_standard_pack(size, ante)
            .iter()
            .map(render_card)
            .collect::<Vec<_>>()
            .join(", "),
        other => format!("[unknown pack category: {other}]"),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut seed: Option<String> = None;
    let mut max_ante: i32 = 8;
    let mut cards_per_ante: Vec<i32> = vec![15, 50, 50, 50, 50, 50, 50, 50];
    let mut vouchers_bought = true;
    let mut fresh_profile = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--ante" => {
                i += 1;
                max_ante = args[i].parse().expect("--ante expects an integer");
            }
            "--cards-per-ante" => {
                i += 1;
                cards_per_ante = args[i]
                    .split(',')
                    .map(|n| n.trim().parse().expect("cards-per-ante expects integers"))
                    .collect();
            }
            "--vouchers" => {
                i += 1;
                vouchers_bought = match args[i].as_str() {
                    "bought" => true,
                    "offered" => false,
                    other => panic!("--vouchers expects bought|offered, got {other}"),
                };
            }
            "--fresh-profile" => fresh_profile = true,
            other if seed.is_none() => seed = Some(other.to_string()),
            other => panic!("unrecognized argument: {other}"),
        }
        i += 1;
    }

    let seed = seed
        .unwrap_or_else(|| {
            eprintln!(
                "usage: explore SEED [--ante N] [--cards-per-ante 15,50,...] \
                 [--vouchers bought|offered] [--fresh-profile]"
            );
            std::process::exit(1);
        })
        .to_uppercase()
        .replace('0', "O");

    let mut inst = Instance::new(&seed);
    // fresh_run is always true: Cavendish, Planet X/Ceres/Eris, Stone/Steel/
    // Glass Joker and the voucher upgrade tiers reset every run regardless
    // of profile, so there's no static (non-simulated) analysis where
    // treating them as available makes sense. fresh_profile defaults to
    // false (an experienced profile — Swashbuckler, Onyx Agate, Cartomancer,
    // Astronomer, etc. already unlocked), matching the site's own default
    // demo; pass --fresh-profile to simulate a brand-new save instead.
    inst.init_locks(1, fresh_profile, true);

    for ante in 1..=max_ante {
        inst.init_unlocks(ante, false);

        println!("==ANTE {ante}==");

        let boss = inst.next_boss(ante);
        println!("Boss: {}", boss.name());

        let voucher = inst.next_voucher(ante);
        println!("Voucher: {}", voucher.name());
        // --vouchers bought (default): assume every offered voucher gets
        // bought, matching the site's own analysis — an unbought voucher
        // can't resurface in a later ante. --vouchers offered skips the
        // lock, so a voucher you haven't confirmed as purchased stays
        // eligible to reappear (closer to how the real game actually
        // gates vouchers: on purchase, not on mere appearance).
        if vouchers_bought {
            inst.lock(voucher.name());
            if let Some(upgrade) = voucher_upgrade(voucher) {
                inst.unlock(upgrade.name());
            }
        }

        let tag1 = inst.next_tag(ante);
        let tag2 = inst.next_tag(ante);
        println!("Tags: {}, {}", tag1.name(), tag2.name());

        println!("Shop Queue: ");
        let n_cards = cards_per_ante
            .get((ante - 1) as usize)
            .copied()
            .unwrap_or(0);
        for q in 1..=n_cards {
            let item = inst.next_shop_item(ante);
            println!("{q}) {}", render_shop_item(item));
        }

        println!();
        println!("Packs: ");
        let num_packs = if ante == 1 { 4 } else { 6 };
        for _ in 1..=num_packs {
            let pack = inst.next_pack(ante);
            let (category, size) = pack_info(pack);
            let contents = render_pack_contents(&mut inst, category, size, ante);
            println!("{pack} - {contents}");
        }

        println!();
    }
}
