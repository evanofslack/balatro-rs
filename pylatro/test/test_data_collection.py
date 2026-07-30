"""Wiring smoke test for gym/collect_data.py and gym/data_sample.py — proves
the collection pipeline runs end-to-end (agents.mcts_agent's logging hook,
ProcessPoolExecutor, JSON round-trip) at a tiny scale. Not a test that the
collected data is good training data, same non-goal as test_tune.py's smoke
test for the Optuna pipeline.
"""

import os
import tempfile

from collect_data import run_collection_batch
from collection_seeds import COLLECTION_SEEDS
from data_sample import load_samples, save_samples


def test_run_collection_batch_returns_samples_with_fixed_feature_length():
    seeds = COLLECTION_SEEDS[:2]
    samples = run_collection_batch(
        seeds, sims=5, max_steps=20, workers=1, agent_seed=0
    )
    assert len(samples) > 0

    lengths = {len(s.features) for s in samples}
    assert len(lengths) == 1, f"feature vector length varied: {lengths}"

    seen_seeds = {s.seed for s in samples}
    assert seen_seeds <= set(seeds)


def test_run_collection_batch_workers_matches_serial_sample_count():
    seeds = COLLECTION_SEEDS[2:6]
    serial = run_collection_batch(seeds, sims=5, max_steps=20, workers=1, agent_seed=0)
    parallel = run_collection_batch(
        seeds, sims=5, max_steps=20, workers=2, agent_seed=0
    )
    # Same per-episode-seed derivation as eval.py/tune.py's --workers path
    # (agent_seed + position), so episode count (not necessarily identical
    # trajectories) should match between workers=1 and workers=2.
    assert len({s.seed for s in serial}) == len({s.seed for s in parallel})


def test_every_sample_within_an_episode_shares_its_final_outcome():
    seeds = COLLECTION_SEEDS[6:8]
    samples = run_collection_batch(seeds, sims=5, max_steps=20, workers=1, agent_seed=0)
    by_seed = {}
    for s in samples:
        by_seed.setdefault(s.seed, []).append(s)

    for seed, seed_samples in by_seed.items():
        outcomes = {(s.final_score, s.won, s.ante_reached) for s in seed_samples}
        assert len(outcomes) == 1, (
            f"seed {seed}'s samples disagree on episode outcome: {outcomes}"
        )
        # step_index should be a real, gap-free 0..n-1 sequence.
        assert sorted(s.step_index for s in seed_samples) == list(
            range(len(seed_samples))
        )


def test_save_and_load_samples_round_trip():
    seeds = COLLECTION_SEEDS[8:9]
    samples = run_collection_batch(seeds, sims=5, max_steps=20, workers=1, agent_seed=0)
    assert len(samples) > 0

    with tempfile.TemporaryDirectory() as d:
        path = os.path.join(d, "samples.json")
        save_samples(path, samples)
        loaded = load_samples(path)

    assert len(loaded) == len(samples)
    for original, restored in zip(samples, loaded):
        assert original.features == restored.features
        assert original.final_score == restored.final_score
        assert original.won == restored.won
        assert original.ante_reached == restored.ante_reached
        assert original.seed == restored.seed
        assert original.step_index == restored.step_index
