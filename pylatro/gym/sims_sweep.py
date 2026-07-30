"""Sweep n_simulations at the current default HeuristicParams to test
whether MCTS's search depth is a real lever now that the value function is
stable (v10, see docs/mcts.md) — before the first tuning round, more sims
mostly meant "sample the same miscalibrated value function more times";
that argument no longer applies now that a validated, tuned value function
exists. Deliberately does not touch heuristic_value()/HeuristicParams — the
only thing varied here is n_simulations.

Reuses gym/tune.py's run_batch() (same clone+apply/ProcessPoolExecutor
machinery already used for tuning and eval) and the project's established
5-agent-seed x 100-episode-per-seed methodology (matching every prior
version-vs-version comparison this session, e.g. `just mcts` x5 +
`just aggregate` + `just compare`) — just varying `sims` instead of code
version, at a fixed, unvaried heuristic config.

    python gym/sims_sweep.py --sims 100,250,500 --agent-seeds 0,1,2,3,69 --workers 10
"""

import argparse
import time

import aggregate
import compare
from agents.mcts_agent import (
    AGENT_VERSION,
    DEFAULT_EXPLORATION,
    DEFAULT_HEURISTIC_PARAMS,
    ROLLOUT_HORIZON,
)
from eval_seeds import EVAL_SEEDS
from logging_utils import save_results
from tune import run_batch


def run_sweep(
    sims_values,
    agent_seeds,
    episodes: int,
    max_steps: int,
    workers: int,
    out_prefix: str = "results/sims_sweep",
):
    """For each value in sims_values: run every agent_seed as its own
    5-episode-set pass (same seeded-A/B discipline as every prior version
    comparison this session), save each, aggregate.aggregate() the
    per-seed runs into one avg summary per sims value, then compare.compare()
    all sims levels side by side in one table."""
    seeds = EVAL_SEEDS[:episodes]
    avg_paths = []
    for sims in sims_values:
        per_seed_paths = []
        for agent_seed in agent_seeds:
            start = time.perf_counter()
            logs, summary = run_batch(
                DEFAULT_HEURISTIC_PARAMS,
                DEFAULT_EXPLORATION,
                ROLLOUT_HORIZON,
                seeds,
                sims,
                max_steps,
                workers,
                agent_seed=agent_seed,
            )
            elapsed = time.perf_counter() - start
            path = f"{out_prefix}_{sims}_s{agent_seed}.json"
            save_results(
                path,
                logs,
                summary,
                {
                    "agent": "mcts",
                    "agent_version": f"{AGENT_VERSION}-sims{sims}",
                    "sims": sims,
                    "agent_seed": agent_seed,
                    "episodes": episodes,
                    "workers": workers,
                    "elapsed_sec": elapsed,
                },
            )
            print(
                f"sims={sims} seed={agent_seed}: {elapsed:.1f}s, "
                f"win_rate={summary['win_rate']:.1%}, "
                f"avg_ante_reached={summary['avg_ante_reached']:.2f}, "
                f"avg_final_score={summary['avg_final_score']:.1f}"
            )
            per_seed_paths.append(path)
        avg_path = f"{out_prefix}_{sims}_avg.json"
        aggregate.aggregate(per_seed_paths, avg_path)
        avg_paths.append(avg_path)

    print()
    compare.compare(avg_paths)
    return avg_paths


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--sims", default="100,250,500", help="comma-separated n_simulations values to compare"
    )
    parser.add_argument(
        "--agent-seeds",
        default="0,1,2,3,69",
        help="comma-separated MctsAgent(agent_seed=...) values, averaged per sims value "
        "(matches the fixed 5-seed set used for every prior version comparison)",
    )
    parser.add_argument("--episodes", type=int, default=len(EVAL_SEEDS))
    parser.add_argument("--max-steps", type=int, default=300)
    parser.add_argument("--workers", type=int, default=10)
    parser.add_argument("--out-prefix", default="results/sims_sweep")
    args = parser.parse_args()

    sims_values = [int(s) for s in args.sims.split(",")]
    agent_seeds = [int(s) for s in args.agent_seeds.split(",")]
    run_sweep(
        sims_values,
        agent_seeds,
        args.episodes,
        args.max_steps,
        args.workers,
        args.out_prefix,
    )
