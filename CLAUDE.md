# balatro-rs

Game engine and move generator for a simplified version of Balatro, written in Rust with Python bindings.

## Workspace

Cargo workspace with eight members — see `ARCHITECTURE.md` for the full crate graph and design rationale:

- `core/` — main game engine library (`balatro-rs` crate), where almost all logic lives
- `cli/` — interactive terminal game (`balatro-cli`), depends on core
- `tui/` — interactive terminal UI (ratatui-based), depends on core
- `pylatro/` — PyO3 Python bindings, depends on core
- `balatro-types/` — neutral vocabulary (`Card`, `Jokers`, `Tarot`, etc.), no behavior, no file-format knowledge
- `balatro-jkr/` — codec for Balatro's `.jkr` save-file format (Lua table literals)
- `balatro-profile/` — reads real save/profile files into typed data via `balatro-jkr` + `balatro-types`; read-only, ships a `profile` CLI binary
- `balatro-seed/` — byte-accurate port of Balatro's actual seed/RNG algorithm (pseudohash + per-node LuaRandom reseeding, not the simplified `rand_chacha` stream `core` uses); depends on `balatro-types` only, ships an `explore` CLI binary

## Build & Test

```bash
# run all tests (all workspace members)
cargo test

# core library only
cargo test -p balatro-rs

# run benchmarks
cargo bench -p balatro-rs

# build Python bindings (requires maturin)
cd pylatro && maturin develop

# explore a seed's full expected contents (byte-accurate vs. real Balatro),
# same text format as the TheSoul/Immolate reference site for direct diffing
cargo run -p balatro-seed --bin explore -- SEED [--ante N]
```

Core has optional features; default is `["serde", "python"]`. To build without Python bindings:

```bash
cargo build -p balatro-rs --no-default-features --features serde
```

## What `core`'s game engine does and doesn't implement

Implemented: poker hand scoring, playing/discarding/reordering cards, blind progression (small/big/boss), ante progression (1-8), money/interest, buying/selling jokers, tarots, planets, card enhancements/editions in effect. 54/150 jokers have real scoring behavior — see `jokers.md` for per-joker status and the implementation roadmap for the rest.

Not implemented: spectrals, skip blind/tags, boss blind modifiers, seals in effect (struct exists, unused), alternative decks/stakes (selecting one doesn't currently change starting deck composition or add restrictions), the remaining 96 jokers.

This section is about `core`'s gameplay engine. It generates shops/packs with its own
simplified, non-Balatro-accurate RNG — reproducing exactly what a real seed generates
(shop contents, packs, vouchers, etc., matching the real game byte-for-byte) is
`balatro-seed`'s job, not `core`'s; the two aren't wired together yet (see
`ARCHITECTURE.md`'s "Open / deferred decisions").

## Two public APIs

`Game` exposes two ways to get and execute moves:

1. `gen_actions() -> impl Iterator<Item = Action>` — variable-length, ergonomic
2. `gen_action_space() -> ActionSpace` — fixed-size binary mask vector for RL environments

Both call `handle_action(action)` to execute moves.
