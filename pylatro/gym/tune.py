"""Auto-tune MctsAgent's heuristic weights (HeuristicParams) with Optuna,
instead of hand-guessing them — three consecutive manual attempts
(heuristic_value()'s original weights, an abandoned static discard
heuristic, v8's real-scored discard ranking) all failed to move win/
ante-reached rate, see docs/mcts.md. Searches against TUNING_SEEDS
(disjoint from EVAL_SEEDS, tuning_seeds.py), then validates the best-found
config against the real held-out EVAL_SEEDS set — that validation result,
not the small-scale in-study objective, is what decides whether a tuned
config is worth adopting.

    python gym/tune.py --n-trials 100 --episodes-per-trial 30 --workers 10 \
        --out results/tune_best.json --validate
    python gym/tune.py --validate-only results/tune_best.json --workers 10

Deliberately does not modify eval.py: its _build_agent/_run_one have no path
for the extra per-trial params, and it's the most heavily-used CLI in gym/ —
safer to keep untouched. run_batch()/_run_one_tuned() below mirror its
--workers pattern exactly (ProcessPoolExecutor over episodes, not threads —
pyo3 never releases the GIL around clone()/handle_action()).
"""

import argparse
import json
import time
from concurrent.futures import ProcessPoolExecutor
from typing import Optional

import optuna

import compare
from agents.mcts_agent import (
    DEFAULT_EXPLORATION,
    DEFAULT_HEURISTIC_PARAMS,
    ROLLOUT_HORIZON,
    HeuristicParams,
    MctsAgent,
)
from env import BalatroEnv
from eval_seeds import EVAL_SEEDS
from logging_utils import record_episode, save_results, summarize
from tuning_seeds import TUNING_SEEDS


def _run_one_tuned(
    seed, sims, max_steps, agent_seed, heuristic_params, exploration, rollout_horizon
):
    env = BalatroEnv(max_steps=max_steps)
    agent = MctsAgent(
        n_simulations=sims,
        exploration=exploration,
        agent_seed=agent_seed,
        heuristic_params=heuristic_params,
        rollout_horizon=rollout_horizon,
    )
    agent.run_episode(env, seed, max_steps)
    return record_episode(env, seed)


def run_batch(
    heuristic_params: HeuristicParams,
    exploration: float,
    rollout_horizon: int,
    seeds,
    sims: int,
    max_steps: int,
    workers: int,
    agent_seed: Optional[int] = None,
):
    """Runs `seeds` through one HeuristicParams/exploration/rollout_horizon
    configuration, returns (logs, summary)."""
    n = len(seeds)
    per_episode_seeds = [
        None if agent_seed is None else agent_seed + i for i in range(n)
    ]
    if workers <= 1:
        logs = [
            _run_one_tuned(
                s, sims, max_steps, a, heuristic_params, exploration, rollout_horizon
            )
            for s, a in zip(seeds, per_episode_seeds)
        ]
    else:
        with ProcessPoolExecutor(max_workers=workers) as pool:
            logs = list(
                pool.map(
                    _run_one_tuned,
                    seeds,
                    [sims] * n,
                    [max_steps] * n,
                    per_episode_seeds,
                    [heuristic_params] * n,
                    [exploration] * n,
                    [rollout_horizon] * n,
                )
            )
    return logs, summarize(logs)


def build_params(trial: "optuna.Trial", tune_rollout_horizon: bool):
    """Optuna search space. Returns (HeuristicParams, exploration, rollout_horizon)."""
    lose_floor = trial.suggest_float("terminal_lose_floor", -30.0, -1.0)
    # Sample a strictly-positive gap instead of the ceiling directly, so
    # terminal_lose_ceiling > terminal_lose_floor holds by construction, no
    # rejection sampling needed.
    lose_margin = trial.suggest_float("lose_margin_bonus", 0.5, 15.0)
    params = HeuristicParams(
        terminal_win_value=trial.suggest_float("terminal_win_value", 20.0, 150.0),
        terminal_lose_floor=lose_floor,
        terminal_lose_ceiling=lose_floor + lose_margin,
        money_weight=trial.suggest_float("money_weight", 1e-4, 0.5, log=True),
        joker_weight=trial.suggest_float("joker_weight", 0.01, 2.0, log=True),
        round_weight=trial.suggest_float("round_weight", 0.01, 2.0, log=True),
    )
    # exploration is a tree-search parameter, not a leaf-value weight, so it
    # stays outside HeuristicParams — sampled alongside but passed as
    # MctsAgent's separate kwarg.
    exploration = trial.suggest_float("exploration", 0.1, 4.0)
    if tune_rollout_horizon:
        rollout_horizon = trial.suggest_int("rollout_horizon", 5, 40)
    else:
        rollout_horizon = ROLLOUT_HORIZON
    return params, exploration, rollout_horizon


def heuristic_params_from_dict(d: dict) -> HeuristicParams:
    return HeuristicParams(
        terminal_win_value=d["terminal_win_value"],
        terminal_lose_floor=d["terminal_lose_floor"],
        terminal_lose_ceiling=d["terminal_lose_floor"] + d["lose_margin_bonus"],
        money_weight=d["money_weight"],
        joker_weight=d["joker_weight"],
        round_weight=d["round_weight"],
    )


# win_rate (~0-1% today) is too sparse for an optimizer to get gradient from
# at realistic per-trial episode counts; avg_ante_reached is coarser and
# mostly clustered near the floor given the near-zero win rate.
# avg_final_score is continuous, has real per-trial variance at these
# episode counts, and is the metric this whole project has used for every
# prior A/B comparison (compare.py, every saved results/*.json) — a tuned
# config stays directly comparable to prior versions.
OBJECTIVES = {
    "avg_final_score": lambda summary: summary["avg_final_score"],
    "ante_weighted": lambda summary: summary["avg_final_score"]
    + 50.0 * summary["avg_ante_reached"],
}


def make_objective(args):
    seeds = TUNING_SEEDS[: args.episodes_per_trial]
    objective_fn = OBJECTIVES[args.objective]

    def objective(trial: "optuna.Trial") -> float:
        params, exploration, rollout_horizon = build_params(
            trial, args.tune_rollout_horizon
        )
        _, summary = run_batch(
            params,
            exploration,
            rollout_horizon,
            seeds,
            args.sims,
            args.max_steps,
            args.workers,
        )
        # Diagnostics only — not optimized on, see OBJECTIVES comment above.
        trial.set_user_attr("win_rate", summary["win_rate"])
        trial.set_user_attr("avg_ante_reached", summary["avg_ante_reached"])
        trial.set_user_attr("avg_steps", summary["avg_steps"])
        trial.set_user_attr("discard_rate", summary["discard_rate"])
        return objective_fn(summary)

    return objective


def run_validation(
    params: HeuristicParams,
    exploration: float,
    rollout_horizon: int,
    workers: int,
    episodes: int,
    trial_number,
    out_prefix: str = "results/tune_validate",
):
    """Full-scale comparison against the real held-out EVAL_SEEDS set —
    tuned params vs. today's hardcoded defaults — via the same compare.py
    reporting every other A/B in this project has used. This is the number
    that decides whether a tuned config is worth adopting as the new
    default, not the small-scale in-study objective."""
    seeds = EVAL_SEEDS[:episodes]

    tuned_logs, tuned_summary = run_batch(
        params, exploration, rollout_horizon, seeds, 100, 300, workers
    )
    tuned_path = f"{out_prefix}_tuned.json"
    save_results(
        tuned_path,
        tuned_logs,
        tuned_summary,
        {
            "agent": "mcts",
            "agent_version": f"9-tuned-trial{trial_number}",
            "sims": 100,
            "episodes": episodes,
            "workers": workers,
        },
    )

    default_logs, default_summary = run_batch(
        DEFAULT_HEURISTIC_PARAMS, DEFAULT_EXPLORATION, ROLLOUT_HORIZON, seeds, 100, 300, workers
    )
    default_path = f"{out_prefix}_default.json"
    save_results(
        default_path,
        default_logs,
        default_summary,
        {
            "agent": "mcts",
            "agent_version": "9",
            "sims": 100,
            "episodes": episodes,
            "workers": workers,
        },
    )

    compare.compare([default_path, tuned_path])
    return tuned_path, default_path


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--n-trials", type=int, default=100)
    parser.add_argument("--episodes-per-trial", type=int, default=30)
    parser.add_argument("--sims", type=int, default=100)
    parser.add_argument("--max-steps", type=int, default=300)
    parser.add_argument("--workers", type=int, default=10)
    parser.add_argument(
        "--tune-rollout-horizon",
        action="store_true",
        help="also tune ROLLOUT_HORIZON (changes per-trial cost, off by default)",
    )
    parser.add_argument("--objective", choices=list(OBJECTIVES), default="avg_final_score")
    parser.add_argument("--sampler-seed", type=int, default=None)
    parser.add_argument("--study-name", default="mcts-tune")
    parser.add_argument(
        "--storage", default=None, help="e.g. sqlite:///results/tune.db, for resumable studies"
    )
    parser.add_argument("--out", default="results/tune_best.json")
    parser.add_argument(
        "--validate",
        action="store_true",
        help="after tuning, run a full EVAL_SEEDS pass comparing tuned vs default params",
    )
    parser.add_argument("--validate-episodes", type=int, default=len(EVAL_SEEDS))
    parser.add_argument(
        "--validate-only",
        default=None,
        help="skip the search; load a saved --out JSON and just run --validate",
    )
    args = parser.parse_args()

    if args.validate_only:
        with open(args.validate_only) as f:
            best = json.load(f)
        params = heuristic_params_from_dict(best["params"])
        exploration = best["params"]["exploration"]
        rollout_horizon = best["params"].get("rollout_horizon", ROLLOUT_HORIZON)
        run_validation(
            params,
            exploration,
            rollout_horizon,
            args.workers,
            args.validate_episodes,
            best["best_trial_number"],
        )
    else:
        sampler = optuna.samplers.TPESampler(seed=args.sampler_seed)
        study = optuna.create_study(
            direction="maximize",
            sampler=sampler,
            study_name=args.study_name,
            storage=args.storage,
            load_if_exists=args.storage is not None,
        )
        print(
            f"tuning: {args.n_trials} trials x {args.episodes_per_trial} episodes, "
            f"objective={args.objective}, workers={args.workers}"
        )
        start = time.perf_counter()
        study.optimize(make_objective(args), n_trials=args.n_trials, n_jobs=1)
        elapsed = time.perf_counter() - start

        best_trial = study.best_trial
        print(f"tuning done in {elapsed:.1f}s over {args.n_trials} trials")
        print(f"best value ({args.objective}): {best_trial.value:.2f}")
        print(f"best params: {best_trial.params}")
        print(f"best user_attrs: {best_trial.user_attrs}")

        out_payload = {
            "meta": {
                "n_trials": args.n_trials,
                "episodes_per_trial": args.episodes_per_trial,
                "sims": args.sims,
                "tuning_seeds": f"{TUNING_SEEDS[0]}-{TUNING_SEEDS[args.episodes_per_trial - 1]}",
                "objective": args.objective,
                "study_name": args.study_name,
                "timestamp": time.time(),
            },
            "best_trial_number": best_trial.number,
            "best_value": best_trial.value,
            "params": best_trial.params,
            "user_attrs": best_trial.user_attrs,
        }
        with open(args.out, "w") as f:
            json.dump(out_payload, f, indent=2)
        print(f"saved best params to {args.out}")

        if args.validate:
            params = heuristic_params_from_dict(best_trial.params)
            exploration = best_trial.params["exploration"]
            rollout_horizon = best_trial.params.get("rollout_horizon", ROLLOUT_HORIZON)
            run_validation(
                params,
                exploration,
                rollout_horizon,
                args.workers,
                args.validate_episodes,
                best_trial.number,
            )
