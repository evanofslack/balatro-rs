# balatro-seed

Byte-accurate port of Balatro's real seed/RNG algorithm, written in Rust.

## Overview

Balatro's "seeded" randomness isn't a single advancing RNG stream — for every
decision (a shop slot, a pack card, a boss, ...) the game reseeds a fresh
generator from a hash of a decision-specific node ID plus the run's seed.
This crate ports that scheme (`pseudohash`, a Lua-5.4-derived `LuaRandom`,
and the per-node cache that makes it all reproducible) directly from
[`TheSoul`/Immolate](https://spectralpack.github.io/TheSoul/), a
community-trusted WASM reimplementation, so that a seed string here and the
same seed typed into the real game produce identical shops, packs, tags,
vouchers, and bosses.

Given the same seed, you can predict what a real run will contain before
starting it — useful for searching for seeds with specific properties, then
actually playing them in Balatro.

## Example

```rust
use balatro_seed::Instance;

let mut inst = Instance::new("TESTSEED");
inst.init_locks(1);

let boss = inst.next_boss(1);
let voucher = inst.next_voucher(1);
println!("Boss: {}, Voucher: {}", boss.name(), voucher.name());
```

## CLI

An `explore` binary prints a seed's ante-by-ante contents in the same text
format as TheSoul's website output, so you can diff the two directly:

```
cargo run -p balatro-seed --bin explore -- SEED [--ante N] [--cards-per-ante 15,50,50,50,50,50,50,50]
```

Example:

```
$ cargo run -q -p balatro-seed --bin explore -- TEST --ante 1
==ANTE 1==
Boss: The Goad
Voucher: Tarot Merchant
Tags: Investment Tag, Speed Tag
Shop Queue: 
1) Ice Cream
2) Strength
...
Packs: 
Buffoon Pack - Baseball Card, Drunkard
...
```

To diff against the reference: visit
<https://spectralpack.github.io/TheSoul/>, leave every setting at its
default (Red Deck, White Stake, version 1.0.1f, all unlocks on), enter the
same seed, and compare the `Output` box line by line against
`explore`'s stdout — everything should match except Standard Pack contents
(see Features below).

## Features

- [x] Jokers, Tarots, Planets, Spectrals (including Soul/Black Hole pulls),
      Vouchers, Tags, Bosses
- [x] Buffoon, Arcana, Celestial, Spectral packs
- [x] Shop item type selection (Joker/Tarot/Planet/Spectral), joker
      rarity + edition rolls
- [x] Ante-gated locks/unlocks (`init_locks`/`init_unlocks`)
- [x] `explore` CLI for diffing against TheSoul's website output

Not yet implemented:

- [ ] Joker stickers (eternal/perishable/rental) — stake-gated
- [ ] Standard Packs / playing-card shop items — needs a Card/Suit/Value
      resolver, a different shape than the name-lookup resolvers used
      elsewhere (see `resolve.rs`)
- [ ] `initLocks`/`initUnlocks`'s `freshProfile`/`freshRun` blocks — a real
      caller should seed the lock table from an actual unlocked-item set
      instead
- [ ] Pre-10099-version joker pool variants (only the current tables are
      ported; see `pools.rs`)
