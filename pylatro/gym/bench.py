"""Throughput benchmark: steps/sec, episodes/sec, and isolated clone cost.

For the pure-Rust (no-FFI) baseline, see `cargo bench -p balatro-rs`
(`core/benches/benchmark.rs` already times a random-rollout loop in the
same shape this script uses over pyo3).

The isolated `GameEngine.clone()` timing is the number that should decide
whether an MCTS/self-play hot loop needs to move to Rust before adding a
neural net, rather than assuming it.
"""

import random
import time

import pylatro
from joker_pool import apply_to_config


def random_rollout_steps_per_sec(n_steps: int = 5_000, seed: int = 0) -> float:
    config = pylatro.Config()
    config.ante_end = 1
    apply_to_config(config)
    config.seed = seed
    game = pylatro.GameEngine(config)

    rng = random.Random(seed)
    start = time.perf_counter()
    steps = 0
    while steps < n_steps:
        if game.is_over:
            game = pylatro.GameEngine(config)
        mask = game.gen_action_space()
        legal = [i for i, m in enumerate(mask) if m == 1]
        game.handle_action_index(rng.choice(legal))
        steps += 1
    elapsed = time.perf_counter() - start
    return steps / elapsed


def episodes_per_sec(n_episodes: int = 100, seed: int = 0) -> float:
    config = pylatro.Config()
    config.ante_end = 1
    apply_to_config(config)

    rng = random.Random(seed)
    start = time.perf_counter()
    for i in range(n_episodes):
        config.seed = seed + i
        game = pylatro.GameEngine(config)
        while not game.is_over:
            mask = game.gen_action_space()
            legal = [i for i, m in enumerate(mask) if m == 1]
            game.handle_action_index(rng.choice(legal))
    elapsed = time.perf_counter() - start
    return n_episodes / elapsed


def clone_cost_per_sec(n_clones: int = 20_000, seed: int = 0) -> float:
    config = pylatro.Config()
    config.ante_end = 1
    apply_to_config(config)
    config.seed = seed
    game = pylatro.GameEngine(config)

    start = time.perf_counter()
    for _ in range(n_clones):
        game.clone()
    elapsed = time.perf_counter() - start
    return n_clones / elapsed


if __name__ == "__main__":
    sps = random_rollout_steps_per_sec()
    print(f"Python-driven random-rollout: {sps:,.0f} steps/sec")

    eps = episodes_per_sec()
    print(f"Python-driven random-rollout: {eps:,.1f} episodes/sec")

    cps = clone_cost_per_sec()
    print(f"GameEngine.clone(): {cps:,.0f} clones/sec ({1e6 / cps:.2f} us/clone)")
