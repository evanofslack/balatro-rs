# pylatro

PyO3 Python bindings for the Rust `balatro-rs` game engine, plus a
Gymnasium-based training/eval harness for search and RL agents.

DISCLAIMER: I really do not know what I am doing here :)

## Setup

```bash
./setup.sh          # creates .env/, installs maturin, builds+installs the extension
source .env/bin/activate
```

Or manually: `python -m venv .env && source .env/bin/activate && pip install maturin pytest && maturin develop`.

Re-run `maturin develop` after any change to `core`/`balatro-types`/`pylatro/src`.

## Layout

- `src/lib.rs` — the pyo3 bindings themselves (`GameEngine`, `GameState`, `Config`, ...).
- `test/` — pytest suite (`pytest test/`). `test_env.py` covers the gym env: reset/step
  smoke test, action-mask legality, seed determinism, truncation.
- `gym/` — training/eval tooling:
  - `env.py` — `BalatroEnv`, a `gymnasium.Env` with identity-encoded observations
    (per-card/per-joker, not just counts) and goal-relative log-scale reward.
  - `joker_pool.py` — curated ~24-joker allow-list used for training/eval (keeps the
    search space small; expand as more of `core`'s jokers get scoring logic).
  - `eval_seeds.py` — the fixed 100-seed held-out set every agent is compared against.
  - `agents/random_agent.py` — floor baseline, samples uniformly from legal actions.
  - `agents/mcts_agent.py` — vanilla UCT MCTS, no neural net. Re-plans from scratch
    at every real decision.
  - `bench.py` — throughput numbers (steps/sec, episodes/sec, isolated `clone()` cost).
  - `eval.py` — runs an agent over the eval seed set, prints a strategy report,
    optionally saves it to JSON.
  - `logging_utils.py` — per-episode logging + aggregate summary (win rate with a
    Wilson confidence interval, avg ante reached, discard rate, top jokers bought,
    action-kind histogram).
  - `compare.py` — diffs two saved `eval.py --out` runs side by side.

## Running the agent

```bash
source .env/bin/activate

# random baseline (floor)
python gym/eval.py --agent random

# vanilla MCTS
python gym/eval.py --agent mcts --sims 100

# save results for later comparison
python gym/eval.py --agent mcts --sims 200 --out results/mcts_200.json
```

Flags: `--sims` (MCTS simulations per decision), `--max-steps` (episode truncation
cap), `--episodes` (how many of the 100 eval seeds to use — full set by default).

Throughput numbers:

```bash
python gym/bench.py
```

## Analyzing training / agent performance

`eval.py` always prints a strategy report: win rate (with a 95% CI, since 30-100
episodes is a small sample), average ante reached, average episode length, discard
rate, most-bought jokers, and the action-kind mix. That's the first thing to read
after any change — win rate alone is noisy at this sample size, ante-reached and
episode length move first.

To compare across runs (different sim budgets, before/after a heuristic tweak,
agent vs. agent), save each run and diff them:

```bash
python gym/eval.py --agent random --out results/random.json
python gym/eval.py --agent mcts --sims 100 --out results/mcts_100.json
python gym/compare.py results/random.json results/mcts_100.json
```

The saved JSON also has a per-episode `episodes` array (seed, win/loss, ante
reached, steps, final score, full action-kind and jokers-bought counts) if you want
to dig in further (e.g. in a notebook/pandas) rather than just the aggregate.

`results/` is gitignored — treat these as local scratch artifacts, not committed history.

## Rust-side benchmarks and tests

```bash
cargo test --workspace
cargo bench -p balatro-rs   # pure-Rust, no-FFI baseline to compare gym/bench.py against
```
