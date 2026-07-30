"""Evaluate an agent against the fixed held-out seed set and print a
strategy report. This is the actual "run it and get real numbers" entry
point the medium-term plan is aimed at.

    python eval.py --agent random
    python eval.py --agent mcts --sims 100
    python eval.py --agent mcts --sims 100 --out results/mcts_100.json
    python eval.py --agent mcts --sims 100 --workers 10 --out results/mcts_100.json

--workers >1 runs episodes across a process pool, not threads — pyo3 never
releases the GIL around clone()/handle_action() here, so threads wouldn't
overlap at all (see docs/mcts.md's Rust-vs-Python check-in). Each episode is
fully independent (own env, own agent), so this is a pure wall-clock win with
no change to search behavior — see --workers' help text for the one caveat
(per-episode RNG derivation differs slightly from the --workers=1 path).
"""

import argparse
import functools
import time
from concurrent.futures import ProcessPoolExecutor
from typing import Optional

from env import BalatroEnv
from eval_seeds import EVAL_SEEDS
from logging_utils import EpisodeLog, print_report, record_episode, save_results


def _build_agent(
    agent_name: str,
    sims: int,
    agent_seed: Optional[int],
    value_model: Optional[str] = None,
    config=None,
):
    if agent_name == "random":
        from agents.random_agent import RandomAgent

        return RandomAgent()
    elif agent_name == "mcts":
        from agents.mcts_agent import MctsAgent

        value_fn = None
        if value_model is not None:
            # Stage 0 (docs/mcts.md): swap in a trained value model instead
            # of the hand-tuned heuristic. `config` supplies the padding-size
            # constants the model's feature encoder needs — see
            # agents/model_value.py's docstring for why it can't be read off
            # `game`/`GameEngine` clones inside the search tree directly.
            from agents.model_value import model_value

            value_fn = functools.partial(
                model_value, model_path=value_model, config=config
            )
        return MctsAgent(n_simulations=sims, agent_seed=agent_seed, value_fn=value_fn)
    else:
        raise ValueError(f"unknown agent: {agent_name}")


def _run_one(
    agent_name: str,
    seed: int,
    sims: int,
    max_steps: int,
    agent_seed: Optional[int],
    value_model: Optional[str] = None,
) -> EpisodeLog:
    """Runs one episode in isolation — own env, own agent. This is what makes
    an episode a valid unit of work for a process pool: no state is shared
    with any other episode, so workers can't step on each other."""
    env = BalatroEnv(max_steps=max_steps)
    agent = _build_agent(agent_name, sims, agent_seed, value_model, env._config)
    agent.run_episode(env, seed, max_steps)
    return record_episode(env, seed)


def run_agent(
    agent_name: str,
    sims: int,
    max_steps: int,
    episodes: int,
    out: Optional[str] = None,
    agent_seed: Optional[int] = None,
    workers: int = 1,
    value_model: Optional[str] = None,
):
    print(
        f"starting {episodes} episodes with {sims} sims for agent {agent_name}, "
        f"out={out}, workers={workers}"
    )

    if agent_name == "random":
        from agents.random_agent import AGENT_VERSION
    elif agent_name == "mcts":
        from agents.mcts_agent import AGENT_VERSION
    else:
        raise ValueError(f"unknown agent: {agent_name}")

    seeds = EVAL_SEEDS[:episodes]
    start = time.perf_counter()

    if workers <= 1:
        # Unchanged from before parallelism was added: one agent instance
        # (and its RNG stream) reused across all episodes in sequence. Kept
        # bit-identical so `--agent-seed` still reproduces every saved
        # results/*.json generated before this flag existed.
        env = BalatroEnv(max_steps=max_steps)
        agent = _build_agent(agent_name, sims, agent_seed, value_model, env._config)
        logs = []
        for seed in seeds:
            agent.run_episode(env, seed, max_steps)
            logs.append(record_episode(env, seed))
    else:
        # Each worker process needs its own agent (a Python object, and its
        # RNG, can't be shared across a process boundary) — so a single
        # continuous RNG stream across all episodes isn't possible here.
        # Each episode instead gets its own derived-but-deterministic seed
        # (agent_seed + position in the eval set), so --agent-seed is still
        # fully reproducible run-to-run, just not identical to the
        # --workers=1 stream for the same --agent-seed value.
        per_episode_seeds = [
            None if agent_seed is None else agent_seed + i for i in range(len(seeds))
        ]
        with ProcessPoolExecutor(max_workers=workers) as pool:
            logs = list(
                pool.map(
                    _run_one,
                    [agent_name] * len(seeds),
                    seeds,
                    [sims] * len(seeds),
                    [max_steps] * len(seeds),
                    per_episode_seeds,
                    [value_model] * len(seeds),
                )
            )
    elapsed = time.perf_counter() - start

    print(
        f"ran {len(logs)} episodes in {elapsed:.1f}s ({len(logs) / elapsed:.2f} ep/s)"
    )
    summary = print_report(logs, label=agent_name)

    if out:
        meta = {
            "agent": agent_name,
            "agent_version": AGENT_VERSION,
            "sims": sims,
            "max_steps": max_steps,
            "episodes": episodes,
            "elapsed_sec": elapsed,
            "agent_seed": agent_seed,
            "workers": workers,
            "value_model": value_model,
        }
        save_results(out, logs, summary, meta)
        print(f"saved to {out}")

    return summary


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--agent", choices=["random", "mcts"], default="mcts")
    parser.add_argument("--sims", type=int, default=100)
    parser.add_argument("--max-steps", type=int, default=300)
    parser.add_argument("--episodes", type=int, default=len(EVAL_SEEDS))
    parser.add_argument(
        "--out", default=None, help="write per-episode + summary JSON to this path"
    )
    parser.add_argument(
        "--agent-seed",
        type=int,
        default=None,
        help="seed the mcts agent's own rng for reproducible A/B comparisons "
        "(default: unseeded)",
    )
    parser.add_argument(
        "--workers",
        type=int,
        default=1,
        help="run episodes across N processes instead of serially (default: 1, "
        "unchanged/bit-identical behavior). With --agent-seed set, results "
        "are still fully reproducible for a given --workers value, but a "
        "--workers=1 run and a --workers>1 run of the same --agent-seed will "
        "not produce identical per-episode outcomes (see module docstring).",
    )
    parser.add_argument(
        "--value-model",
        default=None,
        help="path to a gym/train_value_model.py --out .joblib file (--agent mcts "
        "only) — swaps in the Stage 0 learned value function for "
        "heuristic_value() as MctsAgent's leaf evaluator. See docs/mcts.md.",
    )
    args = parser.parse_args()
    run_agent(
        args.agent,
        args.sims,
        args.max_steps,
        args.episodes,
        args.out,
        args.agent_seed,
        args.workers,
        args.value_model,
    )
