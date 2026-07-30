"""Wiring smoke test for gym/tune.py — proves the Optuna pipeline runs
end-to-end (imports resolve, param names round-trip, pickling works across
ProcessPoolExecutor) at a tiny scale. Not a test that tuning finds anything
good;
"""

import optuna

from agents.mcts_agent import HeuristicParams
from tune import _narrowed_bounds, build_params, heuristic_params_from_dict, make_objective
from tuning_seeds import TUNING_SEEDS


class _Args:
    episodes_per_trial = 2
    sims = 5
    max_steps = 20
    workers = 1
    objective = "avg_final_score"
    tune_rollout_horizon = False
    narrow_window_frac = 0.2


def test_tune_smoke_study_completes_and_best_params_round_trip():
    assert len(TUNING_SEEDS) >= _Args.episodes_per_trial

    study = optuna.create_study(
        direction="maximize", sampler=optuna.samplers.TPESampler(seed=0)
    )
    study.optimize(make_objective(_Args), n_trials=2, n_jobs=1)

    best = study.best_trial
    assert isinstance(best.value, float)
    for key in (
        "win_rate",
        "avg_ante_reached",
        "avg_steps",
        "discard_rate",
    ):
        assert key in best.user_attrs

    params = heuristic_params_from_dict(best.params)
    assert isinstance(params, HeuristicParams)
    assert params.terminal_lose_ceiling > params.terminal_lose_floor


def test_build_params_ceiling_always_above_floor():
    # The ordering invariant is meant to hold by construction (lose_margin
    # sampled strictly positive), not by luck — check across several trials.
    study = optuna.create_study(sampler=optuna.samplers.TPESampler(seed=1))
    for _ in range(20):
        trial = study.ask()
        params, exploration, rollout_horizon = build_params(
            trial, tune_rollout_horizon=False
        )
        assert params.terminal_lose_ceiling > params.terminal_lose_floor
        assert rollout_horizon == 15  # ROLLOUT_HORIZON, unchanged when not tuned
        study.tell(trial, 0.0)


def test_narrowed_bounds_centers_and_clips():
    # Centered window of the requested width, clipped back to [low, high].
    lo, hi = _narrowed_bounds(center=0.0, low=-10.0, high=10.0, window_frac=0.2)
    assert (lo, hi) == (-2.0, 2.0)

    # Center near the edge: the naive window would spill past `high`, so the
    # narrowed range must clip back to it, not exceed it.
    lo, hi = _narrowed_bounds(center=9.5, low=-10.0, high=10.0, window_frac=0.5)
    assert hi == 10.0
    assert lo < hi

    # Degenerate case (center pinned exactly at `high`, zero-width window):
    # falls back to the full original range rather than returning an
    # unusable zero-width range.
    lo, hi = _narrowed_bounds(center=10.0, low=-10.0, high=10.0, window_frac=0.0)
    assert (lo, hi) == (-10.0, 10.0)


def test_build_params_with_warm_start_narrows_around_center():
    warm_start = {
        "terminal_lose_floor": -28.058568720074668,
        "lose_margin_bonus": 1.3118273753863896,
        "terminal_win_value": 137.09321642258507,
        "money_weight": 0.0044455600572427196,
        "joker_weight": 0.12321152103741428,
        "round_weight": 0.2076322995162519,
        "exploration": 2.0246187402022633,
        # rollout_horizon deliberately absent, as if stage 1 didn't tune it.
    }
    narrow_window_frac = 0.2
    # Ground truth: the exact narrowed range build_params() should be
    # sampling terminal_lose_floor/terminal_win_value from, computed the
    # same way _narrowed_bounds() does directly — proves real narrowing
    # happened, not just that the sample stayed within the full original
    # (99-wide / 380-wide) bounds, which would be true either way.
    floor_lo, floor_hi = _narrowed_bounds(
        warm_start["terminal_lose_floor"], -100.0, -1.0, narrow_window_frac
    )
    win_lo, win_hi = _narrowed_bounds(
        warm_start["terminal_win_value"], 20.0, 400.0, narrow_window_frac
    )
    assert (floor_hi - floor_lo) < (100.0 - 1.0)  # narrower than the full range
    assert (win_hi - win_lo) < (400.0 - 20.0)

    study = optuna.create_study(sampler=optuna.samplers.TPESampler(seed=2))
    for _ in range(10):
        trial = study.ask()
        params, exploration, rollout_horizon = build_params(
            trial,
            tune_rollout_horizon=True,  # exercises the "absent from warm_start" path
            warm_start=warm_start,
            narrow_window_frac=narrow_window_frac,
        )
        assert floor_lo <= params.terminal_lose_floor <= floor_hi
        assert win_lo <= params.terminal_win_value <= win_hi
        # rollout_horizon wasn't in warm_start -> falls back to the full
        # original [5, 40] range, not narrowed around anything.
        assert 5 <= rollout_horizon <= 40
        study.tell(trial, 0.0)
