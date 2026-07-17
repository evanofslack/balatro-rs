//! Prints a seed's expected ante-by-ante contents in the same text format
//! as `TheSoul`'s website output, for direct manual diffing against it.
//!
//! Usage: `explore SEED [--ante N] [--cards-per-ante 15,50,50,50,50,50,50,50]`
//!
//! Deliberately mirrors quirks of the website's own reference JS rather
//! than "fixing" them, since the goal is a byte-diffable match:
//! - `init_locks` is called once for ante 1 before the loop, `init_unlocks`
//!   once per ante inside it — not symmetric per-ante calls.
//! - A drawn voucher is locked (and its upgrade tier unlocked) but never
//!   *activated* — so voucher-driven rate effects (Hone, Tarot Tycoon, ...)
//!   never kick in, matching the site's own demo loop, not a live run.

use balatro_seed::{Instance, ShopItem, pack_info, voucher_upgrade};
use balatro_types::Edition;

fn edition_prefix(e: Edition) -> &'static str {
    match e {
        Edition::Base => "",
        Edition::Foil => "Foil ",
        Edition::Holographic => "Holographic ",
        Edition::Polychrome => "Polychrome ",
        Edition::Negative => "Negative ",
    }
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
        "Standard Pack" => "[standard pack: not implemented]".to_string(),
        other => format!("[unknown pack category: {other}]"),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut seed: Option<String> = None;
    let mut max_ante: i32 = 8;
    let mut cards_per_ante: Vec<i32> = vec![15, 50, 50, 50, 50, 50, 50, 50];

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
            other if seed.is_none() => seed = Some(other.to_string()),
            other => panic!("unrecognized argument: {other}"),
        }
        i += 1;
    }

    let seed = seed
        .unwrap_or_else(|| {
            eprintln!("usage: explore SEED [--ante N] [--cards-per-ante 15,50,...]");
            std::process::exit(1);
        })
        .to_uppercase()
        .replace('0', "O");

    let mut inst = Instance::new(&seed);
    inst.init_locks(1);
    // Every voucher upgrade tier starts locked until its base voucher is
    // owned — the site hardcodes this list in `index.html` unconditionally
    // (separate from Instance::init_locks/the freshProfile block), since a
    // fresh analysis has no way to know which vouchers a real profile has
    // actually purchased.
    for upgrade in [
        "Overstock Plus",
        "Liquidation",
        "Glow Up",
        "Reroll Glut",
        "Omen Globe",
        "Observatory",
        "Nacho Tong",
        "Recyclomancy",
        "Tarot Tycoon",
        "Planet Tycoon",
        "Money Tree",
        "Antimatter",
        "Illusion",
        "Petroglyph",
        "Retcon",
        "Palette",
    ] {
        inst.lock(upgrade);
    }

    for ante in 1..=max_ante {
        inst.init_unlocks(ante, false);

        println!("==ANTE {ante}==");

        let boss = inst.next_boss(ante);
        println!("Boss: {}", boss.name());

        let voucher = inst.next_voucher(ante);
        println!("Voucher: {}", voucher.name());
        inst.lock(voucher.name());
        if let Some(upgrade) = voucher_upgrade(voucher) {
            inst.unlock(upgrade.name());
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
