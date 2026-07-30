"""Tests for logging_utils.summarize()'s min/median/max/stdev and
best_episode stats — added so a win rate/average alone can't hide whether a
result is "9 solid wins" or "9 near-misses". Uses synthetic EpisodeLog
instances directly; no real engine needed for this pure data-shape logic.
"""

from logging_utils import EpisodeLog, summarize


def _log(seed, ante_reached, final_score, won=False):
    return EpisodeLog(
        seed=seed, won=won, ante_reached=ante_reached, steps=1, final_score=final_score
    )


def test_summarize_score_and_ante_stats():
    logs = [
        _log(1, ante_reached=0, final_score=100),
        _log(2, ante_reached=1, final_score=300),
        _log(3, ante_reached=2, final_score=500),
    ]
    summary = summarize(logs)

    assert summary["final_score_min"] == 100
    assert summary["final_score_median"] == 300
    assert summary["final_score_max"] == 500
    assert summary["final_score_stdev"] > 0.0

    assert summary["ante_reached_min"] == 0
    assert summary["ante_reached_median"] == 1
    assert summary["ante_reached_max"] == 2
    assert summary["ante_reached_stdev"] > 0.0


def test_summarize_best_episode_prefers_ante_reached_then_score():
    # seed 2 has a lower final_score than seed 1 but a higher ante_reached
    # (the actual win-progress metric) — must win regardless of score.
    logs = [
        _log(1, ante_reached=0, final_score=999),
        _log(2, ante_reached=3, final_score=10),
        _log(3, ante_reached=3, final_score=50),  # same ante as seed 2, higher score
    ]
    summary = summarize(logs)
    assert summary["best_episode"] == {"seed": 3, "ante_reached": 3, "final_score": 50}


def test_summarize_single_episode_stdev_does_not_raise():
    # statistics.stdev() requires n > 1 — must be guarded, not crash on the
    # single-episode case (e.g. test_tune.py's tiny smoke-scale runs).
    summary = summarize([_log(1, ante_reached=0, final_score=42)])
    assert summary["final_score_stdev"] == 0.0
    assert summary["ante_reached_stdev"] == 0.0
    assert summary["best_episode"] == {"seed": 1, "ante_reached": 0, "final_score": 42}


def test_summarize_empty_logs_does_not_raise():
    summary = summarize([])
    assert summary["final_score_min"] == 0
    assert summary["final_score_stdev"] == 0.0
    assert summary["best_episode"] is None
