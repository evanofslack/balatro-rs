"""Wiring smoke test for gym/tune.py — proves the Optuna pipeline runs
end-to-end (imports resolve, param names round-trip, pickling works across
ProcessPoolExecutor) at a tiny scale. Not a test that tuning finds anything
good;
"""

import optuna

from agents.mcts_agent import HeuristicParams
from tune import build_params, heuristic_params_from_dict, make_objective
from tuning_seeds import TUNING_SEEDS


class _Args:
    episodes_per_trial = 2
    sims = 5
    max_steps = 20
    workers = 1
    objective = "avg_final_score"
    tune_rollout_horizon = False


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
