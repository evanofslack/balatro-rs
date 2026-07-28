"""Evaluate an agent against the fixed held-out seed set and print a
strategy report. This is the actual "run it and get real numbers" entry
point the medium-term plan is aimed at.

    python eval.py --agent random
    python eval.py --agent mcts --sims 100
    python eval.py --agent mcts --sims 100 --out results/mcts_100.json
"""

import argparse
import time
from typing import Optional

from env import BalatroEnv
from eval_seeds import EVAL_SEEDS
from logging_utils import print_report, record_episode, save_results


def run_agent(
    agent_name: str,
    sims: int,
    max_steps: int,
    episodes: int,
    out: Optional[str] = None,
    agent_seed: Optional[int] = None,
):
    print(
        f"starting {episodes} episodes with {sims} sims for agent {agent_name}, out={out}"
    )

    env = BalatroEnv(max_steps=max_steps)

    if agent_name == "random":
        from agents.random_agent import AGENT_VERSION, RandomAgent

        agent = RandomAgent()
        run_episode = lambda seed: agent.run_episode(env, seed, max_steps)
    elif agent_name == "mcts":
        from agents.mcts_agent import AGENT_VERSION, MctsAgent

        agent = MctsAgent(n_simulations=sims, agent_seed=agent_seed)
        run_episode = lambda seed: agent.run_episode(env, seed, max_steps)
    else:
        raise ValueError(f"unknown agent: {agent_name}")

    logs = []
    seeds = EVAL_SEEDS[:episodes]
    start = time.perf_counter()
    for seed in seeds:
        run_episode(seed)
        logs.append(record_episode(env, seed))
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
    args = parser.parse_args()
    run_agent(
        args.agent,
        args.sims,
        args.max_steps,
        args.episodes,
        args.out,
        args.agent_seed,
    )
