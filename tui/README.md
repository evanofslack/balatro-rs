# balatro-tui

Terminal UI for the balatro-rs game engine, built with ratatui.

### Warning

I have no idea how to use ratatui, all work in this package is all claude, sorry...

## Running

```bash
# from workspace root — starts a game
cargo run -p balatro-tui

# or from tui/
cargo run
```

## Recording

Use `--record` to record a game under `data/replays/`, or pass a path. `--library` browses saved replays. Leaving a recorded game writes an `abandoned` outcome so live viewers finish and the run stays in the Library. Gym recordings are available through `BalatroEnv(record_dir=...)`.

## Options

| Flag | Description |
| --- | --- |
| `--load <FILE>` | Load a saved game state from a JSON file |
| `--seed <SEED>` | Start a new game from a numeric seed or seed string |
| `--record [PATH]` | Record to a new replay log; the default is `<data-dir>/replays/` |
| `--replay <FILE>` | Watch a replay log, including one still being written |
| `--library` | Browse replay logs |
| `--data-dir <DIR>` | Directory containing `saves/` and `replays/` (default: `data`) |

`--record` and `--replay` cannot be used together. `--library` conflicts with play/record/replay shortcuts. `--replay` cannot be combined with `--load` or `--seed`.

```bash
cargo run -p balatro-tui -- --load game.json
cargo run -p balatro-tui -- --record
cargo run -p balatro-tui -- --record run.log
cargo run -p balatro-tui -- --replay run.log
cargo run -p balatro-tui -- --library
```

## Replay library

Lists `*.log` files in the replays directory (newest first), labeled PLAYER or MODEL from the log header. Enter watches with the same `--replay` reader. Esc/`q` from the list quits. Esc/`q` during a library-launched replay returns to the list.

| Key            | Action                          |
| -------------- | ------------------------------- |
| j/k or arrows  | Move                            |
| Enter          | Watch selected                  |
| R              | Rename file                     |
| d              | Delete (confirm with y)         |
| r              | Refresh                         |
| Esc / q        | Quit         |

## Replay

Recorded runs write one JSON object per line: a header with source and seed, one move per successful action, and a terminal outcome with `result`, `score`, and `steps`. Leaving early records `result: "abandoned"`. Files go under `<data-dir>/replays/` by default. `balatro-cli --record` and `BalatroEnv(record_dir=...)` write the same format.

`--replay` is the viewer. Forward reads the next complete line. If the file has not grown yet, nothing happens. After the outcome line, playback stops. Back rebuilds the game from the header up to the previous action.

While replaying, game keys do nothing.

| Key            | Action                          |
| -------------- | ------------------------------- |
| space          | Pause or resume autoplay        |
| l / right / .  | Forward one action              |
| h / left       | Back one action                 |
| q / esc        | Quit (or return to library)     |

The board is the normal game UI, plus one status line (step, paused, playing, waiting, win, loss, abandoned, or error).

## Saving and loading

Press `e` during any stage to save the current game state under `<data-dir>/saves/`. The filename defaults to `game_<timestamp>.json` and can be edited before confirming.

Reload a saved game with `--load`:

```bash
cargo run -p balatro-tui -- --load game.json
```

## Screenshot

<img width="1504" height="862" alt="balatro-rs tui" src="https://github.com/user-attachments/assets/3a281099-9f49-43d0-8ae0-cbb53b2190f8" />
