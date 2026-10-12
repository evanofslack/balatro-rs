# balatro-tui

Terminal UI for the balatro-rs game engine, built with ratatui.

### Warning

I have no idea how to use ratatui, all work in this package is all claude, sorry...

## Running

```bash
# from workspace root — opens the title screen (Play / Library)
cargo run -p balatro-tui

# or from tui/
cargo run
```

## Title screen

With no flags, `balatro-tui` opens a Balatro-style title layout with Play and Library.

| Button    | Action |
| --------- | ------ |
| **PLAY**  | Start an interactive run and record it by default. Esc/`q` returns to the title. |
| **LIBRARY** | Open the replay library for recorded runs. |

←/→ (or h/l) move focus; Enter selects; `r` toggles automatic recording; `q` quits from the title. The setting is saved in `replays/.tui-recording` (or the configured replays directory) and applies to new TUI games, including `--seed`. `--record [PATH]` explicitly records even when the setting is off. Loaded save files are not recorded because replay logs begin from a new game.

Agent policies run separately with `balatro-agent-run`; see the CLI README. Training remains external.

## Options

| Flag                 | Description                                                                 |
| -------------------- | --------------------------------------------------------------------------- |
| `--load <FILE>`      | Load a saved game state from a JSON file (skips title)                      |
| `--seed <SEED>`      | Start a new game from a numeric seed or a seed string (skips title)         |
| `--record [PATH]`    | Explicitly record this run; omit PATH to write under `replays/`             |
| `--replay <FILE>`    | Watch a replay log, including one that is still being written               |
| `--library`          | Open the replay library (skips title)                                       |
| `--replays-dir <DIR>`| Directory for `--library` and default `--record` (or `BALATRO_REPLAYS_DIR`) |

`--record` and `--replay` cannot be used together. `--library` conflicts with play/record/replay shortcuts. `--replay` cannot be combined with `--load` or `--seed`.

```bash
cargo run -p balatro-tui -- --load game.json
cargo run -p balatro-tui -- --record
cargo run -p balatro-tui -- --record run.log
cargo run -p balatro-tui -- --replay run.log
cargo run -p balatro-tui -- --library
```

## Replay library

Lists `*.log` files in the replays directory (newest first), labeled PLAYER or MODEL from the log header. Enter watches with the same `--replay` reader. Esc/`q` from the list returns to the title (or quits if you used `--library`). Esc/`q` during a library-launched replay returns to the list.

| Key            | Action                          |
| -------------- | ------------------------------- |
| j/k or arrows  | Move                            |
| Enter          | Watch selected                  |
| R              | Rename file                     |
| d              | Delete (confirm with y)         |
| r              | Refresh                         |
| Esc / q        | Back to title (or quit)         |

## Replay

Recorded runs write one JSON object per line: a header (format version, seed, config, and `source: "player"` or `"model"`), one gym action index per successful move (`{"action": N}`), hand-slot deselects when you toggle a card off (`{"deselect": N}`, not in the gym space), and an outcome line when the episode ends. Wins and losses have `won`, `score`, and `steps`. Leaving a TUI game before it ends deletes its log. An agent failure writes `result: "error"`. A default filename is `{seed}_{unix_nanos}.log` under `replays/` (or `BALATRO_REPLAYS_DIR` / `--replays-dir`). `balatro-cli --record` writes the same format. The CLI menu number is not the gym index stored in the file.

`--replay` is the viewer. Forward reads the next complete line. If the file has not grown yet, nothing happens. After the outcome line, or when an abandoned run's file is deleted, playback stops. Back rebuilds the game from the header up to the previous action.

While replaying, game keys do nothing.

| Key            | Action                          |
| -------------- | ------------------------------- |
| space          | Pause or resume autoplay        |
| l / right / .  | Forward one action              |
| h / left       | Back one action                 |
| q / esc        | Quit (or return to library)     |

The board is the normal game UI, plus one status line (step, paused, playing, waiting, win, loss, abandoned, or error).

## Saving and loading

Press `e` during any stage to export (save) the current game state. The filename defaults to `game_<timestamp>.json` and can be edited before confirming.

Reload a saved game with `--load`:

```bash
cargo run -p balatro-tui -- --load game.json
```

## Screenshot

<img width="1504" height="862" alt="balatro-rs tui" src="https://github.com/user-attachments/assets/3a281099-9f49-43d0-8ae0-cbb53b2190f8" />
