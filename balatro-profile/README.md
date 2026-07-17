# balatro-profile

Typed Balatro save data, written in Rust.

## Overview

`balatro-jkr` decodes `.jkr` files into a generic Lua-table AST with no game
vocabulary attached. This crate turns that into typed data expressed in
`balatro-types` vocabulary:

- `Profile` — from `meta.jkr` + `profile.jkr`: unlocks, discovery status,
  usage stats, career stats, high scores.
- `SaveSnapshot` — from `save.jkr`: an in-progress run's stake, deck,
  dollars, jokers, and cards.

Import only — this crate does not write `.jkr` files back out.

## Example

```rust
use balatro_profile::Profile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let meta = balatro_jkr::decode(&std::fs::read("meta.jkr")?)?;
    let profile = balatro_jkr::decode(&std::fs::read("profile.jkr")?)?;

    let p = Profile::from_lua(&meta, &profile)?;
    println!("{}", p.summary());
    Ok(())
}
```

## CLI

A `profile` binary is included for inspecting save data from the terminal:

```
cargo run --bin profile -- meta.jkr profile.jkr        # full dump
cargo run --bin profile -- meta.jkr profile.jkr -s      # short summary
cargo run --bin profile -- save.jkr                     # in-progress run
```

Pass one path (a `save.jkr`) or two (`meta.jkr` + `profile.jkr`, either
order) — the file kind is detected by content shape, not filename.

Example `-s` output:

```
PLAYER1
Last played: Red Deck, Red Stake

Best hand score: 128,198,110,833 (128.2B)
Furthest ante: 13
Wins: 27 / Rounds: 1,275 (31%)
Best win streak: 4
Hands played: 2,867
Top hands: OnePair (804x), TwoPair (502x), Flush (478x)
Most money: $307 (earned $18,460, spent $16,393)

Jokers: 143 unlocked, 134 played, 5,250 bought, 266 sold
Top jokers: Blueprint (190x), Hanging Chad (188x), Smiley Face (179x)
Consumables: 52/52 discovered, 2,517 used

Vouchers unlocked: 30/32
Decks unlocked: 14/15
```

## Features

- [x] `Profile` from `meta.jkr` + `profile.jkr`
- [x] `SaveSnapshot` from `save.jkr` (stake, deck, dollars, jokers, cards)
- [x] `id()`/`from_id()` on the relevant `balatro-types` enums, hand-declared
      against the game's own save-file ids
- [x] `profile` CLI (full dump and short summary)

Not yet implemented:

- [ ] `deck_stakes` — shape unresolved, kept as raw `LuaValue`
- [ ] `SaveSnapshot` ante/round progress
- [ ] stateful joker counters (live `ability`-block state)
- [ ] writing `.jkr` files back out
