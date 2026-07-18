//! Prints a seed's expected ante-by-ante contents in the same text format
//! as `TheSoul`'s website output, for direct manual diffing against it.
//!
//! Usage: `explore SEED [--ante N] [--cards-per-ante 15,50,50,50,50,50,50,50]
//! [--vouchers bought|offered] [--no-activate-vouchers] [--fresh-profile] [--ante-0]`
//!
//! Deliberately mirrors quirks of the website's own reference JS rather
//! than "fixing" them, since the goal is a byte-diffable match:
//! - `init_locks` is called once for ante 1 before the loop, `init_unlocks`
//!   once per ante inside it — not symmetric per-ante calls.
//! - `--vouchers bought` (default) locks a drawn voucher (and unlocks its
//!   upgrade tier) and, by default, also activates it, so voucher-driven
//!   rate effects (Hone, Tarot Tycoon, ...) apply to later antes exactly as
//!   they would in a live run. Pass `--no-activate-vouchers` to suppress
//!   that and reproduce the reference site's own demo loop instead (which
//!   locks but never activates) — needed to byte-diff against the site.
//!   `--vouchers offered` skips the lock (and activation) entirely, so an
//!   unbought voucher can resurface in a later ante.
//! - `--ante-0` (default off, so the tool's default output shape stays
//!   diffable against the site, which never shows this) prints an extra
//!   section for the Ante 0 you'd reach by buying Hieroglyph/Petroglyph
//!   (`-1 Ante`) out of the Ante 1 shop. Shown unconditionally when passed,
//!   regardless of whether Ante 1's actual drawn voucher is one of those —
//!   same "what's available," not "what you'll see" spirit as the rest of
//!   this tool. It *displays* first but is *computed* last, after the whole
//!   Ante 1..max_ante loop: its Tags/Shop/Packs draw against whatever lock
//!   state that loop has progressed to (e.g. Garbage Tag, unlocked at Ante
//!   2), matching the reference site's own analysis, which does the same
//!   thing (computes it last, displays it first via numeric key order).
//!   Computing it before the loop instead draws against Ante 1's
//!   still-locked-down state and gives a different, non-matching result.
//!   Its boss is copied from Ante 1's rather than drawn fresh:
//!   the boss RNG is a single continuously-mutating node with no ante
//!   suffix (`functions.hpp::nextBoss` locks/draws from plain `"boss"`), so
//!   dropping to Ante 0 before fighting a boss doesn't reroll it — you still
//!   face whatever boss Ante 1 already rolled.

use balatro_seed::{Instance, ShopItem, pack_info, voucher_upgrade};
use balatro_types::{BossBlind, Card, Edition, Enhancement, Seal};

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

/// Renders one ante's section to a string (rather than printing it directly)
/// so callers can compute sections out of order but still print them in
/// display order — see the Ante-0 handling in `main` for why that split
/// matters. `label` is what gets displayed and used for pack-count/shop-count
/// shape (Ante 0 mirrors Ante 1's); `draw_ante` is what actually gets passed
/// to the draw functions (0 for the Ante-0 preview, otherwise equal to
/// `label`). `boss_override` skips drawing a boss and uses this one instead
/// — see the Ante-0 module doc note for why.
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
    // --vouchers bought (default): assume every offered voucher gets
    // bought, matching the real game's own gating (on purchase, not mere
    // appearance) rather than the site demo's lock-without-activate. Pass
    // --no-activate-vouchers to reproduce the site's own demo loop exactly.
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
    // Ante 0 gets the same pack count as Ante 1 (it's the "still early game"
    // shop you'd see instead of Ante 1's, not an ante of its own).
    let num_packs = if label <= 1 { 4 } else { 6 };
    for _ in 1..=num_packs {
        let pack = inst.next_pack(draw_ante);
        let (category, size) = pack_info(pack);
        let contents = render_pack_contents(inst, category, size, draw_ante);
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
    // fresh_run is always true: Cavendish, Planet X/Ceres/Eris, Stone/Steel/
    // Glass Joker and the voucher upgrade tiers reset every run regardless
    // of profile, so there's no static (non-simulated) analysis where
    // treating them as available makes sense. fresh_profile defaults to
    // false (an experienced profile — Swashbuckler, Onyx Agate, Cartomancer,
    // Astronomer, etc. already unlocked), matching the site's own default
    // demo; pass --fresh-profile to simulate a brand-new save instead.
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

    // Computed *after* the ante 1..max_ante loop above (not before it), even
    // though it prints *first*: its Tags/Shop/Packs draw from whatever lock
    // state that loop has progressed to by now (e.g. Garbage Tag, unlocked
    // partway through a real run's ante progression), matching what the
    // reference site's own analysis does — it likewise computes Ante 0 last
    // but displays it first by numeric key order. Computing it early instead
    // would draw against Ante 1's still-locked-down state and diverge.
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
