"""Structural smoke tests for gym/train_value_model.py and
gym/agents/model_value.py — not a test that the trained model predicts well
(same non-goal as test_tune.py's Optuna wiring test), just that training,
persistence, and the leaf-evaluator wrapper are wired correctly.
"""

import json
import os
import random
import tempfile

import joblib
import numpy as np
import pytest

import train_value_model
from agents.model_value import model_value
from data_sample import DecisionSample
from env import BalatroEnv
from features import state_features


def _synthetic_samples(n=40, feature_len=12, seed=0):
    rng = random.Random(seed)
    samples = []
    for i in range(n):
        features = [rng.uniform(-1, 1) for _ in range(feature_len)]
        samples.append(
            DecisionSample(
                features=features,
                final_score=rng.randint(50, 500),
                won=rng.random() < 0.1,
                ante_reached=rng.randint(0, 2),
                seed=1000 + (i // 5),
                step_index=i % 5,
                mc_log_score=rng.uniform(0.0, 3.0),
            )
        )
    return samples


def test_build_xy_shapes_and_mc_log_score_target():
    samples = _synthetic_samples(n=10, feature_len=5)
    x, y = train_value_model.build_xy(samples)
    assert x.shape == (10, 5)
    assert y.shape == (10,)
    for s, target in zip(samples, y):
        assert target == pytest.approx(s.mc_log_score)


def test_train_returns_model_and_sane_metrics():
    samples = _synthetic_samples(n=60, feature_len=8)
    model, metrics = train_value_model.train(samples)
    assert hasattr(model, "predict")
    for key in ("train_r2", "val_r2", "val_mae"):
        assert key in metrics
        assert np.isfinite(metrics[key])


def test_joblib_round_trip_preserves_predictions():
    samples = _synthetic_samples(n=60, feature_len=8)
    model, _ = train_value_model.train(samples)
    x, _ = train_value_model.build_xy(samples)
    before = model.predict(x)

    with tempfile.TemporaryDirectory() as d:
        path = os.path.join(d, "model.joblib")
        joblib.dump(model, path)
        reloaded = joblib.load(path)

    after = reloaded.predict(x)
    assert np.allclose(before, after)


def test_metadata_json_has_required_fields(tmp_path):
    samples = _synthetic_samples(n=60, feature_len=8)
    data_path = tmp_path / "samples.json"
    from data_sample import save_samples

    save_samples(str(data_path), samples)

    out_prefix = str(tmp_path / "model_v1")
    import subprocess
    import sys

    subprocess.run(
        [sys.executable, "train_value_model.py", str(data_path), "--out", out_prefix],
        cwd=os.path.dirname(os.path.abspath(train_value_model.__file__)),
        check=True,
    )

    with open(f"{out_prefix}.meta.json") as f:
        meta = json.load(f)
    for key in (
        "feature_vector_length",
        "regression_target",
        "n_training_samples",
        "n_episodes_collected",
        "collector_agent_version",
        "sklearn_version",
        "train_r2",
        "val_r2",
        "val_mae",
        "timestamp",
    ):
        assert key in meta
    assert meta["feature_vector_length"] == 8
    assert meta["n_training_samples"] == 60


def test_model_value_returns_float_for_live_and_terminal_states():
    samples = _synthetic_samples(n=60, feature_len=245)
    model, _ = train_value_model.train(samples)

    with tempfile.TemporaryDirectory() as d:
        model_path = os.path.join(d, "model.joblib")
        joblib.dump(model, model_path)

        env = BalatroEnv(max_steps=200)
        env.reset(seed=0)
        live_value = model_value(env._game, model_path=model_path, config=env._config)
        assert isinstance(live_value, float) or isinstance(live_value, np.floating)

        # Random play reliably reaches a terminal (loss) state within a
        # handful of steps — unlike reliably *clearing* a blind (see
        # docs/mcts.md's testing caveat), reaching *any* terminal state via
        # random play is fast and deterministic-enough for a unit test.
        rng = random.Random(1)
        for _ in range(200):
            if env._game.is_over:
                break
            actions = list(env.legal_actions())
            if not actions:
                break
            env.step_action(rng.choice(actions))
        assert env._game.is_over

        terminal_value = model_value(
            env._game, model_path=model_path, config=env._config
        )
        assert isinstance(terminal_value, float) or isinstance(
            terminal_value, np.floating
        )


def test_model_value_uses_real_feature_length():
    """Confirms model_value's default feature_fn is features.state_features
    (not just any 245-length stand-in) by training on the actual real length
    and checking prediction doesn't raise a shape mismatch."""
    env = BalatroEnv()
    env.reset(seed=2)
    real_len = state_features(env._game.state, env._config).shape[0]

    samples = _synthetic_samples(n=30, feature_len=real_len)
    model, _ = train_value_model.train(samples)
    with tempfile.TemporaryDirectory() as d:
        model_path = os.path.join(d, "model.joblib")
        joblib.dump(model, model_path)
        value = model_value(env._game, model_path=model_path, config=env._config)
        assert np.isfinite(value)
