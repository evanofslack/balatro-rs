"""Tests for aggregate.py's handling of the min/median/stdev/max/best_episode
stats added to logging_utils.summarize() — specifically that _max fields and
best_episode take the true value across inputs, not an average (averaging a
max would understate the real best-ever result, the one bug this file is
meant to catch before it ships silently).
"""

import json

from aggregate import aggregate


def _write_run(path, summary, meta=None):
    with open(path, "w") as f:
        json.dump({"meta": meta or {"agent": "mcts", "agent_seed": 0}, "summary": summary}, f)


def _summary(final_score_max, ante_reached_max, best_episode, **overrides):
    base = {
        "win_rate": 0.0,
        "avg_ante_reached": 0.0,
        "avg_steps": 0.0,
        "avg_final_score": 0.0,
        "discard_rate": 0.0,
        "final_score_min": 0,
        "final_score_median": 0,
        "final_score_max": final_score_max,
        "final_score_stdev": 0.0,
        "ante_reached_min": 0,
        "ante_reached_median": 0,
        "ante_reached_max": ante_reached_max,
        "ante_reached_stdev": 0.0,
        "best_episode": best_episode,
        "episodes": 1,
        "action_kind_histogram": {},
        "top_jokers_bought": [],
    }
    base.update(overrides)
    return base


def test_aggregate_max_fields_take_true_max_not_average(tmp_path):
    a = tmp_path / "a.json"
    b = tmp_path / "b.json"
    _write_run(
        a,
        _summary(
            final_score_max=500,
            ante_reached_max=2,
            best_episode={"seed": 1, "ante_reached": 2, "final_score": 500},
        ),
    )
    _write_run(
        b,
        _summary(
            final_score_max=1200,
            ante_reached_max=1,
            best_episode={"seed": 2, "ante_reached": 1, "final_score": 1200},
        ),
    )

    out = tmp_path / "avg.json"
    aggregate([str(a), str(b)], str(out))
    result = json.loads(out.read_text())["summary"]

    # True max across inputs (1200), NOT the average of 500 and 1200 (850).
    assert result["final_score_max"] == 1200
    assert result["ante_reached_max"] == 2
    # best_episode picked by (ante_reached, final_score) — seed 1's ante=2
    # beats seed 2's ante=1 despite seed 2's higher raw score.
    assert result["best_episode"] == {"seed": 1, "ante_reached": 2, "final_score": 500}


def test_aggregate_numeric_fields_are_averaged(tmp_path):
    a = tmp_path / "a.json"
    b = tmp_path / "b.json"
    _write_run(
        a,
        _summary(
            final_score_max=0,
            ante_reached_max=0,
            best_episode=None,
            final_score_median=100,
        ),
    )
    _write_run(
        b,
        _summary(
            final_score_max=0,
            ante_reached_max=0,
            best_episode=None,
            final_score_median=300,
        ),
    )

    out = tmp_path / "avg.json"
    aggregate([str(a), str(b)], str(out))
    result = json.loads(out.read_text())["summary"]

    assert result["final_score_median"] == 200  # plain average, unlike the max fields
