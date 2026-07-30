"""Validates (or refutes) the Stage 0 regression's leading hypothesis from
docs/mcts.md: that the GBT model was trained on the wrong population of
states.

`run_episode_with_logging()` only ever logs *real* decision points — states
snapshotted before `search()` is called, always with `is_over == False` (the
loop's own condition). But `MctsAgent._rollout()` (the model's actual
inference call site) walks up to `ROLLOUT_HORIZON` semi-random steps away
from a tree node and evaluates whatever it lands on — which, per this
project's own earlier finding that random play reliably busts out within
~6-12 steps, is quite plausibly *terminal* game-over states most of the
time. If so, the model has literally zero terminal-state training coverage
(no `Stage::End` row ever appeared in its ~13k training samples) despite
terminal states being a large fraction of what it's asked to score in
practice — a much more concrete, checkable version of "distribution
mismatch" than a vague appeal to it.

Two checks, cheapest first:

1. **Terminal-state coverage.** For a sample of real decision points, walk
   the *exact* `_rollout()` mechanics (same `search_actions()` candidate
   selection, same `ROLLOUT_HORIZON`) and record what fraction of resulting
   leaves are terminal. Contrasted against 0% by construction on the real
   side. If this fraction is large, that alone is a near-complete
   explanation for the regression — model_value() would be extrapolating far
   outside its training data on the majority of real search-time calls.

2. **Prediction quality by population.** For both real decision-point states
   and rollout-leaf states, estimate a fair "ground truth" value via several
   independent Monte-Carlo continuations (same rollout policy, played to
   termination rather than stopping at ROLLOUT_HORIZON), then compare both
   model_value() and heuristic_value() against that ground truth —
   model_value() directly (same log10(final_score+1) scale as the MC
   estimate), heuristic_value() via rank correlation (different scale, but
   rank agreement is still meaningful). If the model's error/correlation is
   much worse on rollout-leaf states than real ones, while heuristic_value()
   stays comparatively stable across both, that's the distribution-mismatch
   signature. If both degrade similarly, the mismatch isn't the (main)
   story and the other candidates in docs/mcts.md (target shape, data
   volume, class imbalance) deserve more weight instead.

    python gym/diagnose_value_model.py results/value_model_v1.joblib
"""

import argparse
import json
import random
import statistics
from typing import List, Optional

from agents.mcts_agent import DEFAULT_HEURISTIC_PARAMS, MctsAgent, heuristic_value
from agents.model_value import model_value
from env import BalatroEnv
from rollout_value import mc_rollout_value, rollout_leaf

# Diagnostic-only seeds — disjoint from EVAL_SEEDS (1000-1099), TUNING_SEEDS
# (2000-2199), and COLLECTION_SEEDS (3000-3999), so this never overlaps
# either the training data or the held-out comparison set.
DIAGNOSTIC_SEEDS = list(range(9_000, 9_050))


def collect_real_decision_states(seeds, sims, max_steps, agent_seed=0):
    """Real decision-point states, via the actual v10 agent — the same
    population run_episode_with_logging() trains on."""
    states = []
    for i, seed in enumerate(seeds):
        env = BalatroEnv(max_steps=max_steps)
        agent = MctsAgent(n_simulations=sims, agent_seed=agent_seed + i)
        env.reset(seed=seed)
        terminated = truncated = False
        steps = 0
        while not (terminated or truncated) and steps < max_steps:
            states.append((env._game.clone(), env._config))
            action = agent.search(env._game)
            _, _, terminated, truncated, _ = env.step_action(action)
            steps += 1
    return states


def spearman(a: List[float], b: List[float]) -> Optional[float]:
    n = len(a)
    if n < 2:
        return None
    ra = _ranks(a)
    rb = _ranks(b)
    mean_ra, mean_rb = statistics.mean(ra), statistics.mean(rb)
    cov = sum((x - mean_ra) * (y - mean_rb) for x, y in zip(ra, rb))
    var_a = sum((x - mean_ra) ** 2 for x in ra)
    var_b = sum((y - mean_rb) ** 2 for y in rb)
    if var_a == 0 or var_b == 0:
        return None
    return cov / (var_a * var_b) ** 0.5


def _ranks(values: List[float]) -> List[float]:
    order = sorted(range(len(values)), key=lambda i: values[i])
    ranks = [0.0] * len(values)
    for rank, i in enumerate(order):
        ranks[i] = rank
    return ranks


def analyze_population(label, games_and_configs, model_path, rng, mc_k):
    n_terminal = sum(1 for g, _ in games_and_configs if g.is_over)
    model_preds, heur_preds, ground_truths = [], [], []
    for game, config in games_and_configs:
        model_preds.append(model_value(game, model_path=model_path, config=config))
        heur_preds.append(heuristic_value(game, DEFAULT_HEURISTIC_PARAMS))
        mean_log_score, _ = mc_rollout_value(game, rng, k=mc_k)
        ground_truths.append(mean_log_score)

    n = len(games_and_configs)
    model_mae = statistics.mean(
        abs(p - g) for p, g in zip(model_preds, ground_truths)
    )
    model_corr = spearman(model_preds, ground_truths)
    heur_corr = spearman(heur_preds, ground_truths)

    print(f"--- {label} (n={n}) ---")
    print(f"  terminal fraction:        {n_terminal / n:.1%}")
    print(f"  model_value MAE vs MC gt: {model_mae:.3f}")
    print(f"  model_value spearman:    {model_corr if model_corr is not None else 'n/a'}")
    print(f"  heuristic_value spearman:{heur_corr if heur_corr is not None else 'n/a'}")
    return {
        "n": n,
        "terminal_fraction": n_terminal / n,
        "model_mae_vs_mc_ground_truth": model_mae,
        "model_spearman_vs_mc_ground_truth": model_corr,
        "heuristic_spearman_vs_mc_ground_truth": heur_corr,
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("model_path", help="path to a train_value_model.py --out .joblib file")
    parser.add_argument("--episodes", type=int, default=15)
    parser.add_argument("--sims", type=int, default=50)
    parser.add_argument("--max-steps", type=int, default=40)
    parser.add_argument("--mc-k", type=int, default=6)
    parser.add_argument("--agent-seed", type=int, default=0)
    parser.add_argument("--rng-seed", type=int, default=0)
    parser.add_argument("--out", default=None)
    args = parser.parse_args()

    seeds = DIAGNOSTIC_SEEDS[: args.episodes]
    print(f"collecting real decision states from {len(seeds)} episodes...")
    real_states = collect_real_decision_states(
        seeds, args.sims, args.max_steps, args.agent_seed
    )
    print(f"  got {len(real_states)} real decision-point states")

    rng = random.Random(args.rng_seed)
    print("generating matched rollout-leaf states (mirrors _rollout() exactly)...")
    rollout_states = [
        (rollout_leaf(g, rng), config) for g, config in real_states
    ]

    print()
    real_report = analyze_population(
        "real decision points (training distribution)",
        real_states,
        args.model_path,
        rng,
        args.mc_k,
    )
    print()
    rollout_report = analyze_population(
        "rollout-leaf states (actual _rollout() inference distribution)",
        rollout_states,
        args.model_path,
        rng,
        args.mc_k,
    )

    print()
    print("=== verdict ===")
    gap = rollout_report["terminal_fraction"] - real_report["terminal_fraction"]
    print(f"terminal-fraction gap (rollout - real): {gap:+.1%}")
    if rollout_report["model_mae_vs_mc_ground_truth"] > 1.5 * max(
        real_report["model_mae_vs_mc_ground_truth"], 1e-6
    ):
        print(
            "model_value's error is much larger on rollout-leaf states than real "
            "decision states -> distribution-mismatch hypothesis SUPPORTED."
        )
    else:
        print(
            "model_value's error is comparable across both populations -> "
            "distribution mismatch is NOT the dominant explanation; look at "
            "target shape / data volume / class imbalance instead."
        )

    if args.out:
        with open(args.out, "w") as f:
            json.dump(
                {"real_decision_points": real_report, "rollout_leaf": rollout_report},
                f,
                indent=2,
            )
        print(f"saved to {args.out}")
