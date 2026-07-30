"""Structural tests for gym/rollout_value.py — the shared Monte-Carlo
rollout-policy estimator used by both gym/diagnose_value_model.py and
MctsAgent.run_episode_with_mc_labeling (Stage 0 v2's corrected training
label, see docs/mcts.md's Stage 0 diagnosis). Not a test that the estimate
is a *good* value signal — just that the mechanics (replicate count,
terminality, mean-vs-replicates consistency) are correct.
"""

import random

import pylatro

from env import BalatroEnv
from rollout_value import mc_rollout_value, rollout_leaf


def test_mc_rollout_value_returns_k_terminal_replicates():
    env = BalatroEnv()
    env.reset(seed=3)
    rng = random.Random(0)

    mean_log_score, terminal_clones = mc_rollout_value(
        env._game, rng, k=5, max_extra_steps=60
    )
    assert isinstance(mean_log_score, float)
    assert len(terminal_clones) == 5
    for clone, log_score in terminal_clones:
        assert isinstance(clone, pylatro.GameEngine)
        assert clone.is_over
        assert isinstance(log_score, float)


def test_mc_rollout_value_mean_is_bounded_by_replicates():
    env = BalatroEnv()
    env.reset(seed=4)
    rng = random.Random(1)

    mean_log_score, terminal_clones = mc_rollout_value(
        env._game, rng, k=8, max_extra_steps=60
    )
    log_scores = [log_score for _, log_score in terminal_clones]
    assert min(log_scores) - 1e-9 <= mean_log_score <= max(log_scores) + 1e-9


def test_mc_rollout_value_does_not_mutate_input_game():
    env = BalatroEnv()
    env.reset(seed=5)
    rng = random.Random(2)

    before_score = env._game.state.score
    before_stage = env._game.state.stage.int()
    mc_rollout_value(env._game, rng, k=3, max_extra_steps=60)
    assert env._game.state.score == before_score
    assert env._game.state.stage.int() == before_stage


def test_rollout_leaf_mirrors_rollout_horizon_and_does_not_mutate_input():
    env = BalatroEnv()
    env.reset(seed=6)
    rng = random.Random(3)

    before_score = env._game.state.score
    leaf = rollout_leaf(env._game, rng, horizon=15)
    assert isinstance(leaf, pylatro.GameEngine)
    # Either terminated early (random play reliably busts out within a
    # handful of steps, see docs/mcts.md) or ran the full horizon — either
    # way it must be a genuinely different object from the input.
    assert env._game.state.score == before_score
