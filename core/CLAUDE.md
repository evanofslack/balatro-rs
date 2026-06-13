# core/

Main game engine library. All game logic lives here.

## Key modules

| Module | Purpose |
|---|---|
| `game.rs` | `Game` struct — central state and orchestration |
| `action.rs` | `Action` enum — every possible player move |
| `joker.rs` | `Joker` trait, `Jokers` enum, all joker implementations |
| `hand.rs` | Hand evaluation: `SelectHand`, `MadeHand`, poker detection |
| `generator.rs` | `gen_actions_*` functions — generate valid moves per stage |
| `space.rs` | `ActionSpace` — fixed-size binary mask for RL environments |
| `stage.rs` | `Stage` enum (PreBlind, Blind, PostBlind, Shop, End) |
| `rank.rs` | `HandRank` enum with chip/mult values per level |
| `effect.rs` | `EffectRegistry` — joker effect callbacks |
| `shop.rs` | `Shop`, `JokerGenerator` — shop inventory and buying |
| `card.rs` | `Card` with `Value`, `Suit`, `Enhancement`, `Edition`, `Seal` |
| `available.rs` | `Available` — hand of cards with selection state |
| `config.rs` | `Config` — all game parameters |

## Joker system

Jokers use enum dispatch, not `Box<dyn Joker>`. This is intentional — trait objects don't compose well with serde and pyo3.

The `make_jokers!` macro at the top of `joker.rs` generates the `Jokers` enum and blanket `Joker` impls by matching each variant. To add a new joker:

1. Define a struct (e.g., `pub struct MyJoker;`) implementing the `Joker` trait — provide `name`, `rarity`, `cost`, `desc`, `categories`, and `effects`.
2. Add the struct name to the `make_jokers!(...)` invocation.
3. Register it in `JokerGenerator` (in `shop.rs`) so it can appear in the shop.

`effects(&self, game: &Game) -> Vec<Effects>` returns closures wrapped in `Arc<Mutex<dyn Fn>>` via the `Effects` enum variants. See existing jokers for patterns.

## Effect system

`EffectRegistry` collects hooks from all active jokers at the start of each scoring pass. Hooks fire at:

- `on_play` — when a hand is played
- `on_discard` — when cards are discarded  
- `on_score` — per scored card
- `on_handrank` — after hand rank determined
- `on_modify_hand` — to modify the select hand before scoring

Effects are registered by calling `game.effect_registry.register(effects)` during joker activation. The registry is rebuilt each scoring round.

## Stage transitions

```
PreBlind -> Blind(Small) -> PostBlind -> Shop
         -> Blind(Big)   -> PostBlind -> Shop
         -> Blind(Boss)  -> PostBlind -> Shop -> (next ante or End)
```

`handle_action(Action::SelectBlind)` advances from PreBlind to Blind. `handle_action(Action::CashOut)` advances from PostBlind to Shop. `handle_action(Action::NextRound)` advances from Shop back to PreBlind (or to End if ante complete).

## Card identity

Cards have a `uuid` ID (behind the `serde` feature) to disambiguate identical cards during selection and reordering. `CARD_ID_COUNTER` is a global atomic for non-uuid builds.

## Features

- `serde` (default) — enables serialization of all game types, activates uuid IDs
- `python` (default) — enables PyO3 `#[pyclass]` on `Game`, `Config`, `Stage`, `Jokers`
- `colored` — enables colored terminal output in `card.rs`/`hand.rs`

When adding new public types that should be usable from Python, gate `#[pyclass]` / `#[pymethods]` behind `#[cfg(feature = "python")]` and mirror in `pylatro/src/lib.rs` if needed.

## Tests

Full-game integration tests live in `src/lib.rs` (`test_game_gen_actions`, `test_game_action_space`) and `tests/test.rs`. Unit tests are colocated with their module in `#[cfg(test)] mod tests` blocks.

The integration tests run a complete game with random valid moves — they validate that no `handle_action` call returns an error and the game reaches `Stage::End`.
