"""Stage 0 data collection: run the current (v10) MCTS+heuristic agent over
many episodes, logging every real decision point's state (feature-encoded)
labeled with that episode's eventual outcome — training data for a
gradient-boosted-tree value model (see docs/mcts.md's "Stage 0" plan).

This is a bootstrap, not self-play: the data-generating policy is today's
existing tuned heuristic agent, not the model being trained. Structured like
gym/tune.py's run_batch() (ProcessPoolExecutor over episodes, --workers
mirrors eval.py's own flag) — real processes, not threads, since pyo3 never
releases the GIL around clone()/handle_action().

    python gym/collect_data.py --episodes 500 --sims 100 --workers 10 \
        --out results/value_data.json
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


def _collect_one(seed, sims, max_steps, agent_seed) -> List[DecisionSample]:
    env = BalatroEnv(max_steps=max_steps)
    agent = MctsAgent(n_simulations=sims, agent_seed=agent_seed)
    _, samples = agent.run_episode_with_logging(env, seed, max_steps)
    return samples


def run_collection_batch(
    seeds,
    sims: int,
    max_steps: int,
    workers: int,
    agent_seed: Optional[int] = None,
) -> List[DecisionSample]:
    n = len(seeds)
    per_episode_seeds = [
        None if agent_seed is None else agent_seed + i for i in range(n)
    ]
    if workers <= 1:
        per_episode = [
            _collect_one(s, sims, max_steps, a)
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
                    per_episode_seeds,
                )
            )
    return list(itertools.chain.from_iterable(per_episode))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--episodes", type=int, default=200)
    parser.add_argument("--sims", type=int, default=100)
    parser.add_argument("--max-steps", type=int, default=300)
    parser.add_argument("--workers", type=int, default=10)
    parser.add_argument("--agent-seed", type=int, default=0)
    parser.add_argument("--out", default="results/value_data.json")
    args = parser.parse_args()

    seeds = COLLECTION_SEEDS[: args.episodes]
    print(
        f"collecting: {len(seeds)} episodes, sims={args.sims}, "
        f"workers={args.workers}"
    )
    start = time.perf_counter()
    samples = run_collection_batch(
        seeds, args.sims, args.max_steps, args.workers, args.agent_seed
    )
    elapsed = time.perf_counter() - start
    print(
        f"collected {len(samples)} samples from {len(seeds)} episodes "
        f"in {elapsed:.1f}s"
    )
    save_samples(args.out, samples)
    print(f"saved to {args.out}")
