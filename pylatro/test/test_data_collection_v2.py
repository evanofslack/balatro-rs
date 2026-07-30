"""Wiring smoke test for gym/collect_data_v2.py and
MctsAgent.run_episode_with_mc_labeling — the Stage 0 v2 collector (corrected
Monte-Carlo rollout-policy label, see docs/mcts.md's Stage 0 diagnosis).
Mirrors test_data_collection.py's non-goal: proves the pipeline runs
end-to-end correctly, not that the collected labels are good training data.
"""

import os
import tempfile

from collect_data_v2 import run_collection_batch
from collection_seeds import COLLECTION_SEEDS
from data_sample import load_samples, save_samples


def test_run_collection_batch_v2_sample_count_matches_one_plus_mc_k():
    seeds = COLLECTION_SEEDS[:2]
    mc_k = 3
    samples = run_collection_batch(
        seeds, sims=10, max_steps=20, mc_k=mc_k, mc_horizon=30, workers=1, agent_seed=0
    )
    assert len(samples) > 0

    by_episode_step = {}
    for s in samples:
        by_episode_step.setdefault((s.seed, s.step_index), []).append(s)
    # Every real decision point should have produced exactly 1 (starting
    # state) + mc_k (terminal replicate) samples sharing its (seed, step_index).
    for key, group in by_episode_step.items():
        assert len(group) == 1 + mc_k, f"{key} had {len(group)} samples, expected {1 + mc_k}"


def test_run_collection_batch_v2_every_sample_has_mc_log_score():
    seeds = COLLECTION_SEEDS[2:4]
    samples = run_collection_batch(
        seeds, sims=10, max_steps=20, mc_k=2, mc_horizon=30, workers=1, agent_seed=0
    )
    assert len(samples) > 0
    assert all(s.mc_log_score is not None for s in samples)


def test_run_collection_batch_v2_workers_matches_serial_sample_count():
    seeds = COLLECTION_SEEDS[4:8]
    serial = run_collection_batch(
        seeds, sims=10, max_steps=20, mc_k=2, mc_horizon=30, workers=1, agent_seed=0
    )
    parallel = run_collection_batch(
        seeds, sims=10, max_steps=20, mc_k=2, mc_horizon=30, workers=2, agent_seed=0
    )
    assert len({s.seed for s in serial}) == len({s.seed for s in parallel})


def test_save_and_load_v2_samples_round_trip_mc_log_score():
    seeds = COLLECTION_SEEDS[8:9]
    samples = run_collection_batch(
        seeds, sims=10, max_steps=20, mc_k=2, mc_horizon=30, workers=1, agent_seed=0
    )
    assert len(samples) > 0

    with tempfile.TemporaryDirectory() as d:
        path = os.path.join(d, "samples_v2.json")
        save_samples(path, samples)
        loaded = load_samples(path)

    assert len(loaded) == len(samples)
    for original, restored in zip(samples, loaded):
        assert original.mc_log_score == restored.mc_log_score
        assert original.features == restored.features
