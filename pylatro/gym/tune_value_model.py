"""Auto-tune the Stage 0 GBT value model's (rescale, exploration) pair with
Optuna, instead of hand-guessing them — mirrors gym/tune.py's structure and
methodology exactly, but tunes MctsAgent's leaf-value scale/exploration
constant against a fixed trained model instead of HeuristicParams.

Why this exists: Stage 0 v2 (docs/mcts.md) diagnosed the GBT model's real-play
regression as a value-scale/exploration-constant mismatch, not a bad model —
mc_log_score spans only ~1.7 units vs. heuristic_value()'s tuned ~165-unit
range that DEFAULT_EXPLORATION (2.0246) was calibrated against. Same class of
problem tune.py already solved once for the heuristic (three hand-guessed
attempts failed, one Optuna run found a 10-30x win) — this applies the same
auto-tuning approach here instead of hand-guessing a rescale constant.

Searches (rescale, exploration) against TUNING_SEEDS (disjoint from
EVAL_SEEDS), then validates the best-found pair against the real held-out
EVAL_SEEDS set, tuned-value-model agent vs. today's hardcoded heuristic
defaults (v10) — that validation result, not the small-scale in-study
objective, is what decides whether this is worth adopting.

    python gym/tune_value_model.py --model-path results/value_model_v2.joblib \
        --n-trials 50 --episodes-per-trial 20 --workers 10 \
        --out results/tune_value_best.json --validate
    python gym/tune_value_model.py --model-path results/value_model_v2.joblib \
        --validate-only results/tune_value_best.json --workers 10

Deliberately does not modify eval.py's tuning-unrelated code paths further
than the --exploration/--value-rescale flags it already gained for the cheap
manual directional check (see docs/mcts.md) — this script owns the actual
search, same "keep the heavily-used CLI simple" precedent tune.py set for
itself.
"""

import argparse
import functools
import json
import time
from concurrent.futures import ProcessPoolExecutor
from typing import Optional

import optuna

import compare
import tune
from agents.mcts_agent import AGENT_VERSION, MctsAgent
from agents.model_value import model_value
from env import BalatroEnv
from eval_seeds import EVAL_SEEDS
from logging_utils import record_episode, save_results, summarize
from tuning_seeds import TUNING_SEEDS


def _run_one_value(seed, sims, max_steps, agent_seed, model_path, rescale, exploration):
    env = BalatroEnv(max_steps=max_steps)
    value_fn = functools.partial(
        model_value, model_path=model_path, config=env._config, rescale=rescale
    )
    agent = MctsAgent(
        n_simulations=sims, exploration=exploration, agent_seed=agent_seed, value_fn=value_fn
    )
    agent.run_episode(env, seed, max_steps)
    return record_episode(env, seed)


def run_batch_value(
    model_path: str,
    rescale: float,
    exploration: float,
    seeds,
    sims: int,
    max_steps: int,
    workers: int,
    agent_seed: Optional[int] = None,
):
    """Runs `seeds` through one (rescale, exploration) configuration of the
    GBT-value-model agent, returns (logs, summary). Mirrors tune.py's
    run_batch exactly (per-episode seed derivation, --workers pattern)."""
    n = len(seeds)
    per_episode_seeds = [
        None if agent_seed is None else agent_seed + i for i in range(n)
    ]
    if workers <= 1:
        logs = [
            _run_one_value(s, sims, max_steps, a, model_path, rescale, exploration)
            for s, a in zip(seeds, per_episode_seeds)
        ]
    else:
        with ProcessPoolExecutor(max_workers=workers) as pool:
            logs = list(
                pool.map(
                    _run_one_value,
                    seeds,
                    [sims] * n,
                    [max_steps] * n,
                    per_episode_seeds,
                    [model_path] * n,
                    [rescale] * n,
                    [exploration] * n,
                )
            )
    return logs, summarize(logs)


def build_value_params(trial: "optuna.Trial"):
    """Optuna search space: (rescale, exploration). Both wide/log-uniform —
    the right order of magnitude isn't known a priori (Stage 0 v2's ~97x
    compression estimate is a rough diagnosis, not a target to search
    around)."""
    rescale = trial.suggest_float("rescale", 1.0, 300.0, log=True)
    exploration = trial.suggest_float("exploration", 0.01, 4.0, log=True)
    return rescale, exploration


def make_objective(args):
    seeds = TUNING_SEEDS[: args.episodes_per_trial]

    def objective(trial: "optuna.Trial") -> float:
        rescale, exploration = build_value_params(trial)
        _, summary = run_batch_value(
            args.model_path, rescale, exploration, seeds, args.sims, args.max_steps, args.workers
        )
        # Diagnostics only — not optimized on, same reasoning as tune.py's
        # OBJECTIVES comment (win_rate too sparse for gradient at realistic
        # per-trial episode counts).
        trial.set_user_attr("win_rate", summary["win_rate"])
        trial.set_user_attr("avg_ante_reached", summary["avg_ante_reached"])
        trial.set_user_attr("avg_steps", summary["avg_steps"])
        trial.set_user_attr("discard_rate", summary["discard_rate"])
        return summary["avg_final_score"]

    return objective


def run_validation(
    model_path: str,
    rescale: float,
    exploration: float,
    workers: int,
    episodes: int,
    trial_number,
    out_prefix: str = "results/tune_value_validate",
):
    """Full-scale comparison against the real held-out EVAL_SEEDS set —
    tuned (rescale, exploration) GBT agent vs. today's hardcoded heuristic
    defaults (v10) — via the same compare.py reporting every other A/B in
    this project has used. The "default" side reuses tune.py's own run_batch
    with DEFAULT_HEURISTIC_PARAMS/DEFAULT_EXPLORATION/ROLLOUT_HORIZON,
    exactly what eval.py's --agent mcts (no --value-model) produces."""
    seeds = EVAL_SEEDS[:episodes]

    tuned_logs, tuned_summary = run_batch_value(
        model_path, rescale, exploration, seeds, 100, 300, workers
    )
    tuned_path = f"{out_prefix}_tuned.json"
    save_results(
        tuned_path,
        tuned_logs,
        tuned_summary,
        {
            "agent": "mcts",
            "agent_version": f"{AGENT_VERSION}-gbt-rescale-trial{trial_number}",
            "sims": 100,
            "episodes": episodes,
            "workers": workers,
            "value_model": model_path,
            "value_rescale": rescale,
            "exploration": exploration,
        },
    )

    default_logs, default_summary = tune.run_batch(
        tune.DEFAULT_HEURISTIC_PARAMS,
        tune.DEFAULT_EXPLORATION,
        tune.ROLLOUT_HORIZON,
        seeds,
        100,
        300,
        workers,
    )
    default_path = f"{out_prefix}_default.json"
    save_results(
        default_path,
        default_logs,
        default_summary,
        {
            "agent": "mcts",
            "agent_version": AGENT_VERSION,
            "sims": 100,
            "episodes": episodes,
            "workers": workers,
        },
    )

    compare.compare([default_path, tuned_path])
    return tuned_path, default_path


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--model-path", required=True, help="path to a train_value_model.py --out .joblib file"
    )
    parser.add_argument("--n-trials", type=int, default=50)
    parser.add_argument("--episodes-per-trial", type=int, default=20)
    parser.add_argument("--sims", type=int, default=100)
    parser.add_argument("--max-steps", type=int, default=300)
    parser.add_argument("--workers", type=int, default=10)
    parser.add_argument("--sampler-seed", type=int, default=None)
    parser.add_argument("--study-name", default="mcts-value-tune")
    parser.add_argument(
        "--storage", default=None, help="e.g. sqlite:///results/tune_value.db, for resumable studies"
    )
    parser.add_argument("--out", default="results/tune_value_best.json")
    parser.add_argument(
        "--validate",
        action="store_true",
        help="after tuning, run a full EVAL_SEEDS pass comparing the tuned GBT agent vs. "
        "v10 heuristic defaults",
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
        run_validation(
            args.model_path,
            best["params"]["rescale"],
            best["params"]["exploration"],
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
            f"model={args.model_path}, workers={args.workers}"
        )
        start = time.perf_counter()
        study.optimize(make_objective(args), n_trials=args.n_trials, n_jobs=1)
        elapsed = time.perf_counter() - start

        best_trial = study.best_trial
        print(f"tuning done in {elapsed:.1f}s over {args.n_trials} trials")
        print(f"best value (avg_final_score): {best_trial.value:.2f}")
        print(f"best params: {best_trial.params}")
        print(f"best user_attrs: {best_trial.user_attrs}")

        out_payload = {
            "meta": {
                "model_path": args.model_path,
                "n_trials": args.n_trials,
                "episodes_per_trial": args.episodes_per_trial,
                "sims": args.sims,
                "tuning_seeds": f"{TUNING_SEEDS[0]}-{TUNING_SEEDS[args.episodes_per_trial - 1]}",
                "objective": "avg_final_score",
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
            run_validation(
                args.model_path,
                best_trial.params["rescale"],
                best_trial.params["exploration"],
                args.workers,
                args.validate_episodes,
                best_trial.number,
            )
