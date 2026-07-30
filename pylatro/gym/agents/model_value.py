"""Stage 0 learned value function: wraps a trained gradient-boosted-tree
regressor (gym/train_value_model.py) as a leaf evaluator, swappable in for
MctsAgent's hand-tuned heuristic_value() (see docs/mcts.md's "Stage 0" plan).

Kept out of agents/mcts_agent.py so the default heuristic-only path doesn't
need joblib/features.py — this module is only imported when a value model is
actually requested (eval.py's --value-model flag).

The model was trained on real, terminal-inclusive outcomes, so it scores
terminal states directly too, rather than special-casing win/loss the way
heuristic_value()'s hardcoded TERMINAL_WIN_VALUE/floor/ceiling do — worth an
ablation later if results look off, but this is the simpler default to try
first.
"""

import math

import joblib

from features import state_features

# Per-process cache, keyed by path: a fitted model gets `joblib.load`ed
# lazily on first use inside whichever process calls it, not passed as a live
# object across ProcessPoolExecutor's process boundary (sidesteps pickling a
# fitted sklearn model through the pool).
_MODEL_CACHE = {}


def _load_model(model_path: str):
    model = _MODEL_CACHE.get(model_path)
    if model is None:
        model = joblib.load(model_path)
        _MODEL_CACHE[model_path] = model
    return model


def model_value(game, model_path: str, config, feature_fn=state_features) -> float:
    """Leaf-evaluation value for `game`, using the model at `model_path`.
    `config` supplies the padding-size constants state_features() needs
    (available_max/selected_max/joker_slots_max/consumable_slots) — game/
    GameEngine clones inside MctsAgent's search tree don't carry a Config
    reference of their own, so callers must thread the same Config the env
    was built with (see eval.py's --value-model wiring)."""
    model = _load_model(model_path)
    feat = feature_fn(game.state, config).reshape(1, -1)
    log_score = model.predict(feat)[0]
    return log_score


def score_to_log10(final_score: int) -> float:
    """Inverse-friendly helper matching train_value_model.py's regression
    target formula, exposed for tests/analysis that want to compare a raw
    score against the model's predicted scale directly."""
    return math.log10(final_score + 1)
