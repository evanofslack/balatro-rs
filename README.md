# balatro-rs

A toolbox of Rust crates for [Balatro](https://www.playbalatro.com/)

## Features

- rules engine and move generator for simulation/RL (with python bindings)
- real save-file (`.jkr`) parsing
- byte-accurate port of the game's actual seed/RNG algorithm
- TUI for playing core game, CLIs for various other functionlity

## Crates

| Crate                                 | What it does                                                                                                                                                                                                                                                          |
| ------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [`core`](core) (package `balatro-rs`) | The rules engine — scoring, playing/discarding, blinds, shop, jokers/tarots/planets. Exposes an exhaustive move generator, meant as a base for simulation and reinforcement learning. Ships a `calc` CLI that scores an existing hand without needing a running game. |
| [`balatro-types`](balatro-types)      | Neutral vocabulary shared by everything — `Card`, `Jokers`, `Tarot`, `Voucher`, etc. Pure data, no behavior.                                                                                                                                                          |
| [`balatro-jkr`](balatro-jkr)          | Codec for Balatro's `.jkr` save-file container format (raw-DEFLATE + Lua table literals). No game vocabulary. Ships a `jkr` CLI that dumps a `.jkr` file's decoded contents.                                                                                          |
| [`balatro-profile`](balatro-profile)  | Reads real save/profile files into typed data (`Profile`, `SaveSnapshot`) using `balatro-jkr` + `balatro-types`. Read-only; ships a `profile` CLI.                                                                                                                    |
| [`balatro-seed`](balatro-seed)        | A byte-accurate port of Balatro's actual seed/RNG algorithm (pseudohash + per-node `LuaRandom`), verified against a reference implementation ante-by-ante. Ships an `explore` CLI to print a seed's full expected contents.                                           |
| [`cli`](cli) (`balatro-cli`)          | Interactive terminal game over `core`. Also ships `balatro-edit`, a CLI for editing a `core::Game`'s JSON state.                                                                                                                                                      |
| [`tui`](tui) (`balatro-tui`)          | Full ratatui-based terminal UI over `core::Game`.                                                                                                                                                                                                                     |
| [`pylatro`](pylatro)                  | PyO3 Python bindings over `core`, for scripting and RL experimentation.                                                                                                                                                                                               |

## Example

```rust
use balatro_rs::{action::Action, game::Game};
use rand::Rng;

fn main() {
    let mut g = Game::default();
    g.start();
    while !g.is_over() {
        // Get all available moves
        let actions: Vec<Action> = g.gen_actions().collect();
        if actions.is_empty() {
            break;
        }

        // Pick a random move and execute it
        let i = rand::thread_rng().gen_range(0..actions.len());
        let action = actions[i].clone();
        g.handle_action(action);
    }
    let result = g.result();
}
```

`Game` also exposes `gen_action_space()`, a fixed-size binary action mask,
for RL environments that want a stable action-space shape instead of a
variable-length iterator.

## `core` feature coverage

`core` does not implement all of Balatro and likely never will. Implemented
(including move generation):

- [x] identification and scoring of poker hands
- [x] playing/discarding/reordering of cards
- [x] blind pass/fail and game win/lose conditions
- [x] money/interest generation
- [x] ante progression (1-8) and blind progression (small/big/boss)
- [x] buying/selling/using jokers, tarots, and planets
- [x] card enhancements, editions, and seals in effect
- [x] skip blind / tags (partialy)
- [x] optional real-seed-accurate shop/pack generation via `balatro-seed` (partially wired)

The following are missing and may or may not be added:

- [ ] spectral card use-effects
- [ ] boss blind modifiers
- [ ] vouchers
- [ ] alternative decks and stakes as wired starting config
- [ ] remaining unimplemented jokers

## Building & running

```bash
# run all tests across the workspace
cargo test

# core library only
cargo test -p balatro-rs

# interactive terminal game
cargo run -p balatro-cli

# interactive terminal UI
cargo run -p balatro-tui

# explore a seed's full expected contents, byte-accurate vs. real Balatro
cargo run -p balatro-seed --bin explore -- SEED --ante 1

# inspect a real save/profile file
cargo run -p balatro-profile --bin profile -- save.jkr

# dump a raw .jkr file's decoded contents
cargo run -p balatro-jkr --bin jkr -- meta.jkr

# score an existing game-state JSON (e.g. from balatro-edit) without a running game
cargo run -p balatro-rs --bin calc -- score state.json

# build the Python bindings (requires maturin)
cd pylatro && maturin develop
```

Each interface (`cli`, `tui`, `pylatro`, and the various CLI binaries) has
its own usage docs in its directory.
