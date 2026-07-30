"""Wiring smoke test for gym/sims_sweep.py — proves the sweep loop, saving,
aggregate.aggregate(), and compare.compare() all connect correctly at a tiny
scale. Not a test that any particular sims value performs better; see
docs/mcts.md for real sweep results.
"""

import json
import os

from sims_sweep import run_sweep


def test_sims_sweep_smoke(tmp_path):
    avg_paths = run_sweep(
        sims_values=[5, 10],
        agent_seeds=[0, 1],
        episodes=2,
        max_steps=20,
        workers=1,
        out_prefix=str(tmp_path / "sweep"),
    )
    assert len(avg_paths) == 2
    for path in avg_paths:
        assert os.path.exists(path)
        summary = json.load(open(path))["summary"]
        assert summary["episodes"] == 2
