"""Trains the Stage 0 gradient-boosted-tree value model on data from
collect_data_v2.py — a swappable candidate for MctsAgent's heuristic_value()
(see docs/mcts.md's "Stage 0" plan).

Regression target: `mc_log_score`, computed at *collection* time
(gym/rollout_value.py's mc_rollout_value(), via
MctsAgent.run_episode_with_mc_labeling()) as a Monte-Carlo estimate of
continued rollout-policy play from each state — the corrected v2 target.
v1 used `log10(final_score + 1)` computed here at train time from each
sample's real episode outcome; that was diagnosed as the root cause of v1's
regression (`gym/diagnose_value_model.py`, `docs/mcts.md`'s Stage 0
diagnosis) — the real-episode outcome reflects continued play by the
full-strength agent, not the weak rollout policy the model is actually
queried under at `_rollout()`'s call site. There's no train-time formula
left to vary here: `mc_log_score` already *is* the corrected label, computed
once at collection time, not derived from a raw stored score.

    python gym/train_value_model.py results/value_data_v2.json \
        --out results/value_model_v2
"""

import argparse
import time

import joblib
import numpy as np
import sklearn
from sklearn.ensemble import GradientBoostingRegressor
from sklearn.metrics import mean_absolute_error, r2_score
from sklearn.model_selection import train_test_split

from agents.mcts_agent import AGENT_VERSION
from data_sample import load_samples


def build_xy(samples):
    x = np.stack([s.features for s in samples])
    y = np.array([s.mc_log_score for s in samples])
    return x, y


def train(samples, random_state: int = 0):
    x, y = build_xy(samples)
    x_train, x_val, y_train, y_val = train_test_split(
        x, y, test_size=0.2, random_state=random_state
    )
    model = GradientBoostingRegressor(random_state=random_state)
    model.fit(x_train, y_train)

    train_pred = model.predict(x_train)
    val_pred = model.predict(x_val)
    metrics = {
        "train_r2": r2_score(y_train, train_pred),
        "val_r2": r2_score(y_val, val_pred),
        "val_mae": mean_absolute_error(y_val, val_pred),
    }
    return model, metrics


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("data_path", help="path to a collect_data_v2.py --out JSON file")
    parser.add_argument("--out", default="results/value_model_v2")
    args = parser.parse_args()

    samples = load_samples(args.data_path)
    n_episodes = len({s.seed for s in samples})
    print(f"loaded {len(samples)} samples from {n_episodes} episodes")

    start = time.perf_counter()
    model, metrics = train(samples)
    elapsed = time.perf_counter() - start
    print(f"trained in {elapsed:.1f}s: {metrics}")

    model_path = f"{args.out}.joblib"
    joblib.dump(model, model_path)

    meta = {
        "feature_vector_length": len(samples[0].features),
        "regression_target": "mc_log_score (Monte-Carlo rollout-policy estimate, see rollout_value.py)",
        "n_training_samples": len(samples),
        "n_episodes_collected": n_episodes,
        "data_path": args.data_path,
        "collector_agent_version": AGENT_VERSION,
        "sklearn_version": sklearn.__version__,
        "timestamp": time.time(),
        **metrics,
    }
    meta_path = f"{args.out}.meta.json"
    import json

    with open(meta_path, "w") as f:
        json.dump(meta, f, indent=2)

    print(f"saved model to {model_path}")
    print(f"saved metadata to {meta_path}")
