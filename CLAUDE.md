# balatro-rs

Game engine and move generator for a simplified version of Balatro, written in Rust with Python bindings.

## Workspace

Cargo workspace with three members:

- `core/` — main game engine library (`balatro-rs` crate), where almost all logic lives
- `cli/` — interactive terminal game (`balatro-cli`), depends on core
- `pylatro/` — PyO3 Python bindings, depends on core

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
```

Core has optional features; default is `["serde", "python"]`. To build without Python bindings:

```bash
cargo build -p balatro-rs --no-default-features --features serde
```

## What is and isn't implemented

Implemented: poker hand scoring, playing/discarding/reordering cards, blind progression (small/big/boss), ante progression (1-8), money/interest, buying/selling jokers (partial).

Not implemented: tarots, planets, spectrals, boss blind modifiers, skip blind/tags, card enhancements/foils/seals in effect (structs exist but unused), alternative decks/stakes.

## Two public APIs

`Game` exposes two ways to get and execute moves:

1. `gen_actions() -> impl Iterator<Item = Action>` — variable-length, ergonomic
2. `gen_action_space() -> ActionSpace` — fixed-size binary mask vector for RL environments

Both call `handle_action(action)` to execute moves.
