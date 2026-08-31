# balatro-spectator

Terminal spectator for Balatro AI runs — replay library, recorded JSON episodes, or a live agent sidecar.

Built on the same ratatui UI as `balatro-tui`, with replay transport controls and gym action-index playback.

## Running

```bash
# from workspace root
cargo run -p balatro-spectator -- --library

# or from spectator/
cargo run -- --library
```

## Watch replays

Replays are stored in a `replays/` directory (walked up from cwd, or set `BALATRO_REPLAYS_DIR`).

```bash
# Browse saved replays
./spectator/run_library.sh
# or: cargo run -p balatro-spectator -- --library

# Play one file directly
cargo run -p balatro-spectator -- \
  --replay replays/10037_expert_ante3_1730000000.json
```

### Library keys

| Key | Action |
|-----|--------|
| j / k | Move selection |
| [ / ] | Previous / next page (25 per page) |
| PgUp / PgDn | Previous / next page |
| Enter | Watch selected replay |
| R | Rename selected replay |
| r | Refresh list |
| d | Delete (confirm with Enter) |
| Esc | Cancel delete / rename |
| q | Quit |

Replay **display names** default to `{seed}_{agent}_ante{N}` (e.g. `10037_expert_ante3`). Custom names are stored in the replay JSON `name` field — press **R** in the library to rename.

### JSON replay transport bar

When watching a recorded replay (`--library` or `--replay`), the UI shows:

- **Top banner:** `Model Replay`
- **Bottom bar:** YouTube-style controls with speed label (`1x` / `2x` / `4x` forward; `.5` / `.25` when rewinding)

| Control | Action |
|---------|--------|
| `❚❚` / `▶` | Pause / resume |
| `◀` | Step back one action |
| `◀◀` | Cycle rewind speed (1x→2x→4x), continuous rewind |
| `▶` | Step forward one action (while paused) |
| `▶▶` | Cycle forward speed (1x→2x→4x) |
| `✕` (red) | Exit to replay library |
| `←` / `→` | Keyboard step back / forward |
| `Esc` / `b` | Exit to library |

When a replay finishes, the spectator returns to the library automatically after ~1.5s.

On narrow terminals (<40 cols), the bar shows compact controls: `◀◀` `❚❚` `▶▶` `✕`.

### Live agent keys (sidecar only)

| Key | Action |
|-----|--------|
| a | Pause / resume |
| . | Step while paused |
| + / - | Adjust speed |
| q | Quit |

## Live agent sidecar

Run a model (or any policy) as a subprocess without pre-recording:

```bash
cargo run -p balatro-spectator -- \
  --seed 42 --ante-end 3 \
  --agent "python my_agent.py --expert"
```

With a policy checkpoint (passed through to the agent command):

```bash
cargo run -p balatro-spectator -- \
  --seed 42 --ante-end 3 \
  --checkpoint pylatro/gym/checkpoints/dagger_policy_028.pt \
  --agent "python my_agent.py"
```

### Agent wire protocol

The sidecar speaks line-delimited JSON over stdin/stdout:

1. **Spectator → agent:** one line of serialized game state (`Game::to_json()`).
2. **Agent → spectator:** one line reply: `{"action": <index>}` or `{"action": null}` to stop.

Action indices are the same **0–108** fixed space as `pylatro/gym` and `Game::handle_action_index()` — see [`core/src/space.rs`](../core/src/space.rs) for the full index table. Deselect is not a gym index.

On error, the agent may reply with `{"error": "..."}` — the spectator will surface this and exit.

## Replay JSON format (v2)

```json
{
  "format_version": 2,
  "seed": 10037,
  "config": { "ante_end": 1 },
  "agent": { "type": "expert" },
  "actions": [78, 1, 2, 70, 72],
  "outcome": { "won": true, "score": 144, "steps": 89 }
}
```

Optional fields: `name`, `seed_str`, `recorded_at`, `agent.checkpoint`.

**Breaking change:** `format_version: 1` replays used a pre-core action layout and are not supported. Re-record episodes against current `core` to produce v2 files.

Regenerate the test fixture with:

```bash
cargo test -p balatro-spectator gen_minimal_fixture -- --ignored --nocapture
```

## Layout

```
spectator/src/
  main.rs              CLI, event loop, library / replay / live modes
  replay.rs            JSON load, ReplayDriver, library scan
  agent.rs             Sidecar + highlight/apply helpers
  library_input.rs     Library keyboard/mouse
  app.rs               AppState, AppMode, LibraryState
  replay_transport.rs  Transport bar input + rewind/step
  ui/replay_chrome.rs  Top banner + bottom transport UI
```

Game UI modules (`ui/shop.rs`, `ui/preblind.rs`, etc.) are synced from `balatro-tui`.
