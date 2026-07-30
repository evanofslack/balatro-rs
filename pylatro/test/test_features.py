"""features.state_features()'s core invariant: a fixed-length vector for a
given Config, regardless of which state/stage it's called against — this is
what makes stacking samples into a training matrix (gym/train_value_model.py)
safe. Would catch silent shape drift if a later Config-driven padding size
ever varies per-call.
"""

import random

import numpy as np

from env import BalatroEnv
from features import state_features


def test_state_features_fixed_length_across_random_play():
    env = BalatroEnv(max_steps=40)
    env.reset(seed=7)
    rng = random.Random(0)

    lengths = set()
    for _ in range(40):
        vec = state_features(env._game.state, env._config)
        assert isinstance(vec, np.ndarray)
        assert vec.ndim == 1
        lengths.add(vec.shape[0])

        actions = list(env.legal_actions())
        if not actions:
            break
        env.step_action(rng.choice(actions))
        if env._game.is_over:
            break

    assert len(lengths) == 1, f"feature vector length varied: {lengths}"


def test_state_features_fresh_game_is_finite():
    env = BalatroEnv()
    env.reset(seed=1)
    vec = state_features(env._game.state, env._config)
    assert np.all(np.isfinite(vec))
