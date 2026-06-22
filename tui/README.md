# balatro-tui

Terminal UI for balatro-rs, built with [ratatui](https://github.com/ratatui-org/ratatui).

## Run

```bash
# new game
cargo run -p balatro-tui

# load a saved game
cargo run -p balatro-tui -- --load game.json
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
