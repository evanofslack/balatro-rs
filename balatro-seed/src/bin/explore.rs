//! Prints a seed's expected ante-by-ante contents in the same text format
//! as `TheSoul`'s website output, for direct manual diffing against it.
//!
//! Usage: `explore SEED [--ante N] [--cards-per-ante 15,50,50,50,50,50,50,50]
//! [--vouchers bought|offered] [--no-activate-vouchers] [--fresh-profile] [--ante-0]`
//!
//! `--vouchers bought` (default) locks and activates each drawn voucher,
//! matching a live run; `--no-activate-vouchers` reproduces the site's own
//! demo (locks but never activates), needed to byte-diff against it.
//! `--ante-0` previews the shop reachable via Hieroglyph/Petroglyph —
//! computed last (after the full ante loop, so lock state matches) but
//! displayed first, reusing Ante 1's boss since boss RNG has no ante suffix.

use balatro_seed::{Instance, ShopItem, pack_card_count, voucher_upgrade};
use balatro_types::{BossBlind, Card, Edition, Enhancement, PackCategory, PackSize, Seal};

fn pack_display_name(category: PackCategory, size: PackSize) -> String {
    let cat = match category {
        PackCategory::Arcana => "Arcana",
        PackCategory::Celestial => "Celestial",
        PackCategory::Buffoon => "Buffoon",
        PackCategory::Standard => "Standard",
        PackCategory::Spectral => "Spectral",
    };
    match size {
        PackSize::Normal => format!("{cat} Pack"),
        PackSize::Jumbo => format!("Jumbo {cat} Pack"),
        PackSize::Mega => format!("Mega {cat} Pack"),
    }
}

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
        ShopItem::Consumable(c) => c.name().to_string(),
        ShopItem::PlayingCard => "[playing card: not implemented]".to_string(),
    }
}

fn render_pack_contents(inst: &mut Instance, category: PackCategory, count: i32, ante: i32) -> String {
    match category {
        PackCategory::Celestial => inst
            .next_celestial_pack(count, ante)
            .iter()
            .map(|c| c.name())
            .collect::<Vec<_>>()
            .join(", "),
        PackCategory::Arcana => inst
            .next_arcana_pack(count, ante)
            .iter()
            .map(|c| c.name())
            .collect::<Vec<_>>()
            .join(", "),
        PackCategory::Spectral => inst
            .next_spectral_pack(count, ante)
            .iter()
            .map(|c| c.name())
            .collect::<Vec<_>>()
            .join(", "),
        PackCategory::Buffoon => inst
            .next_buffoon_pack(count, ante)
            .iter()
            .map(|j| format!("{}{}", edition_prefix(j.edition()), j.name()))
            .collect::<Vec<_>>()
            .join(", "),
        PackCategory::Standard => inst
            .next_standard_pack(count, ante)
            .iter()
            .map(render_card)
            .collect::<Vec<_>>()
            .join(", "),
    }
}

/// Renders one ante's section to a string so `main` can compute sections
/// out of order but print in display order. `draw_ante` is 0 for the
/// Ante-0 preview, else equal to `label`.
fn render_ante(
    inst: &mut Instance,
    label: i32,
    draw_ante: i32,
    n_cards: i32,
    vouchers_bought: bool,
    activate_vouchers: bool,
    boss_override: Option<BossBlind>,
) -> (String, BossBlind) {
    use std::fmt::Write;
    let mut out = String::new();

    let _ = writeln!(out, "==ANTE {label}==");

    let boss = boss_override.unwrap_or_else(|| inst.next_boss(draw_ante));
    let _ = writeln!(out, "Boss: {}", boss.name());

    let voucher = inst.next_voucher(draw_ante);
    let _ = writeln!(out, "Voucher: {}", voucher.name());
    if vouchers_bought {
        inst.lock(voucher.name());
        if let Some(upgrade) = voucher_upgrade(voucher) {
            inst.unlock(upgrade.name());
        }
        if activate_vouchers {
            inst.activate_voucher(voucher.name());
        }
    }

    let tag1 = inst.next_tag(draw_ante);
    let tag2 = inst.next_tag(draw_ante);
    let _ = writeln!(out, "Tags: {}, {}", tag1.name(), tag2.name());

    let _ = writeln!(out, "Shop Queue: ");
    for q in 1..=n_cards {
        let item = inst.next_shop_item(draw_ante);
        let _ = writeln!(out, "{q}) {}", render_shop_item(item));
    }

    let _ = writeln!(out);
    let _ = writeln!(out, "Packs: ");
    let num_packs = if label <= 1 { 4 } else { 6 };
    for _ in 1..=num_packs {
        let (category, size) = inst.next_pack(draw_ante);
        let count = pack_card_count(category, size);
        let contents = render_pack_contents(inst, category, count, draw_ante);
        let pack = pack_display_name(category, size);
        let _ = writeln!(out, "{pack} - {contents}");
    }

    let _ = writeln!(out);

    (out, boss)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut seed: Option<String> = None;
    let mut max_ante: i32 = 8;
    let mut cards_per_ante: Vec<i32> = vec![15, 50, 50, 50, 50, 50, 50, 50];
    let mut vouchers_bought = true;
    let mut activate_vouchers = true;
    let mut fresh_profile = false;
    let mut ante_0 = false;

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
            "--no-activate-vouchers" => activate_vouchers = false,
            "--fresh-profile" => fresh_profile = true,
            "--ante-0" => ante_0 = true,
            other if seed.is_none() => seed = Some(other.to_string()),
            other => panic!("unrecognized argument: {other}"),
        }
        i += 1;
    }

    let seed = seed
        .unwrap_or_else(|| {
            eprintln!(
                "usage: explore SEED [--ante N] [--cards-per-ante 15,50,...] \
                 [--vouchers bought|offered] [--no-activate-vouchers] \
                 [--fresh-profile] [--ante-0]"
            );
            std::process::exit(1);
        })
        .to_uppercase()
        .replace('0', "O");

    let mut inst = Instance::new(&seed);
    // fresh_run locks reset every run regardless of profile, so always true here.
    inst.init_locks(1, fresh_profile, true);

    let mut ante_1_boss = None;
    let mut sections: Vec<String> = Vec::new();
    for ante in 1..=max_ante {
        inst.init_unlocks(ante, false);

        let n_cards = cards_per_ante
            .get((ante - 1) as usize)
            .copied()
            .unwrap_or(0);
        let (section, boss) = render_ante(
            &mut inst,
            ante,
            ante,
            n_cards,
            vouchers_bought,
            activate_vouchers,
            None,
        );
        if ante == 1 {
            ante_1_boss = Some(boss);
        }
        sections.push(section);
    }

    // Computed after the ante loop but displayed first — see module doc.
    if ante_0 {
        let n_cards = cards_per_ante.first().copied().unwrap_or(0);
        let (section, _) = render_ante(
            &mut inst,
            0,
            0,
            n_cards,
            vouchers_bought,
            activate_vouchers,
            ante_1_boss,
        );
        print!("{section}");
    }

    for section in sections {
        print!("{section}");
    }
}
