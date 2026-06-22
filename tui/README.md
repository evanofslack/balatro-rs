# balatro-tui

Terminal UI for the balatro-rs game engine, built with ratatui.

## Running

```bash
# from workspace root
cargo run -p balatro-tui

# or from tui/
cargo run
```

## Options

| Flag            | Description                              |
| --------------- | ---------------------------------------- |
| `--load <FILE>` | Load a saved game state from a JSON file |

```bash
cargo run -p balatro-tui -- --load game.json
```

## Saving and loading

Press `s` during any stage to open the save dialog. The filename defaults to `game_<timestamp>.json` and can be edited before confirming.

Reload a saved game with `--load`:

```bash
cargo run -p balatro-tui -- --load game_1781391549.json
```

## Layout

```
tui/src/
  main.rs          entry point, terminal setup/teardown
  app.rs           AppState, focus zones, overlay types
  input.rs         keyboard and mouse event handling
  ui/
    mod.rs         top-level render dispatch
    blind.rs       playing a hand
    preblind.rs    blind selection screen
    postblind.rs   cash-out screen
    shop.rs        shop screen
    tarot.rs       tarot card application screen
    end.rs         win/lose screen
    cards.rs       shared card rendering
    joker_strip.rs joker and consumable strip
    sidebar.rs     score/stats sidebar
    overlay/       modal overlays (inspect, run info, save, controls)
```
