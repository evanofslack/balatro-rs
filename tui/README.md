# balatro-tui

Terminal UI for the balatro-rs game engine, built with ratatui.

### Warning

I have no idea how to use ratatui, all work in this package is all claude, sorry...

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

Press `e` during any stage to export (save) the current game state. The filename defaults to `game_<timestamp>.json` and can be edited before confirming.

Reload a saved game with `--load`:

```bash
cargo run -p balatro-tui -- --load game.json
```

## Screenshot

<img width="1504" height="862" alt="balatro-rs tui" src="https://github.com/user-attachments/assets/3a281099-9f49-43d0-8ae0-cbb53b2190f8" />
