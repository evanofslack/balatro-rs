"""Stage 0 v2 data collection: same real v10 agent, same real-decision-point
state population as gym/collect_data.py, but each state is labeled with a
Monte-Carlo estimate of continued rollout-policy play
(MctsAgent.run_episode_with_mc_labeling / gym/rollout_value.py) instead of
the real episode's eventual outcome — the corrected target from
docs/mcts.md's Stage 0 diagnosis (v1's label reflected continued
full-strength-agent play, a mismatch with model_value()'s actual call site).
Also picks up bonus terminal-state samples for free (v1's other diagnosed
gap: 0% terminal-state training coverage).

A new script, not a modification of collect_data.py — keeps v1's collector
runnable as-is for reference/reproducibility. Same ProcessPoolExecutor
structure (--workers mirrors eval.py's own flag; real processes, not
threads, since pyo3 never releases the GIL around clone()/handle_action()).

    python gym/collect_data_v2.py --episodes 1000 --sims 100 --mc-k 6 \
        --workers 10 --out results/value_data_v2.json
"""

import argparse
import itertools
import time
from concurrent.futures import ProcessPoolExecutor
from typing import List, Optional

from agents.mcts_agent import MctsAgent
from collection_seeds import COLLECTION_SEEDS
from data_sample import DecisionSample, save_samples
from env import BalatroEnv


def _collect_one(seed, sims, max_steps, mc_k, mc_horizon, agent_seed) -> List[DecisionSample]:
    env = BalatroEnv(max_steps=max_steps)
    agent = MctsAgent(n_simulations=sims, agent_seed=agent_seed)
    _, samples = agent.run_episode_with_mc_labeling(
        env, seed, max_steps, mc_k=mc_k, mc_horizon=mc_horizon
    )
    return samples


def run_collection_batch(
    seeds,
    sims: int,
    max_steps: int,
    mc_k: int,
    mc_horizon: int,
    workers: int,
    agent_seed: Optional[int] = None,
) -> List[DecisionSample]:
    n = len(seeds)
    per_episode_seeds = [
        None if agent_seed is None else agent_seed + i for i in range(n)
    ]
    if workers <= 1:
        per_episode = [
            _collect_one(s, sims, max_steps, mc_k, mc_horizon, a)
            for s, a in zip(seeds, per_episode_seeds)
        ]
    else:
        with ProcessPoolExecutor(max_workers=workers) as pool:
            per_episode = list(
                pool.map(
                    _collect_one,
                    seeds,
                    [sims] * n,
                    [max_steps] * n,
                    [mc_k] * n,
                    [mc_horizon] * n,
                    per_episode_seeds,
                )
            )
    return list(itertools.chain.from_iterable(per_episode))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--episodes", type=int, default=200)
    parser.add_argument("--sims", type=int, default=100)
    parser.add_argument("--max-steps", type=int, default=300)
    parser.add_argument("--mc-k", type=int, default=6)
    parser.add_argument("--mc-horizon", type=int, default=80)
    parser.add_argument("--workers", type=int, default=10)
    parser.add_argument("--agent-seed", type=int, default=0)
    parser.add_argument("--out", default="results/value_data_v2.json")
    args = parser.parse_args()

    seeds = COLLECTION_SEEDS[: args.episodes]
    print(
        f"collecting (v2, mc-labeled): {len(seeds)} episodes, sims={args.sims}, "
        f"mc_k={args.mc_k}, mc_horizon={args.mc_horizon}, workers={args.workers}"
    )
    start = time.perf_counter()
    samples = run_collection_batch(
        seeds,
        args.sims,
        args.max_steps,
        args.mc_k,
        args.mc_horizon,
        args.workers,
        args.agent_seed,
    )
    elapsed = time.perf_counter() - start
    print(
        f"collected {len(samples)} samples from {len(seeds)} episodes "
        f"in {elapsed:.1f}s"
    )
    save_samples(args.out, samples)
    print(f"saved to {args.out}")
