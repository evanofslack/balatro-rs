import numpy as np
import pytest
from env import BalatroEnv


def test_reset_and_random_legal_steps():
    env = BalatroEnv(max_steps=50)
    obs, info = env.reset(seed=1)
    assert env.observation_space.contains(obs)

    rng = np.random.default_rng(1)
    for _ in range(50):
        mask = env.action_mask()
        legal = np.flatnonzero(mask)
        assert legal.size > 0
        action = int(rng.choice(legal))
        obs, reward, terminated, truncated, info = env.step(action)
        assert env.observation_space.contains(obs)
        assert isinstance(reward, float)
        if terminated or truncated:
            break


def test_action_mask_legality():
    env = BalatroEnv()
    env.reset(seed=2)
    mask = env.action_mask()
    space = env._game.gen_action_space()
    for index in np.flatnonzero(mask):
        assert space[int(index)] == 1

    # An explicitly masked-off index should be rejected by step() as illegal.
    off = np.flatnonzero(mask == 0)
    if off.size > 0:
        _, reward, *_ = env.step(int(off[0]))
        assert reward == -1.0


def test_seed_determinism():
    env = BalatroEnv()
    env.reset(seed=42)
    select_blind = [
        a for a in env.legal_actions() if "SelectBlind" in type(a).__name__
    ][0]
    obs_a, *_ = env.step_action(select_blind)

    env.reset(seed=42)
    select_blind_b = [
        a for a in env.legal_actions() if "SelectBlind" in type(a).__name__
    ][0]
    obs_b, *_ = env.step_action(select_blind_b)

    for key in obs_a:
        assert np.array_equal(obs_a[key], obs_b[key]), f"mismatch in {key}"


def test_legal_actions_includes_atomic_play_hand():
    env = BalatroEnv()
    env.reset(seed=3)
    select_blind = [
        a for a in env.legal_actions() if "SelectBlind" in type(a).__name__
    ][0]
    env.step_action(select_blind)
    kinds = {type(a).__name__ for a in env.legal_actions()}
    assert "Action_PlayHand" in kinds
    assert "Action_DiscardHand" in kinds


def test_truncation_independent_of_termination():
    env = BalatroEnv(max_steps=1)
    env.reset(seed=5)
    mask = env.action_mask()
    legal = np.flatnonzero(mask)
    obs, reward, terminated, truncated, info = env.step(int(legal[0]))
    if not terminated:
        assert truncated
