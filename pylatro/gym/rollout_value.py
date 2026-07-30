"""Shared rollout-policy helpers, used by both gym/diagnose_value_model.py
(to measure ground truth) and gym/agents/mcts_agent.py's
run_episode_with_mc_labeling (to generate Stage 0 v2 training labels) — both
need the *exact same* estimator, or a diagnosis of a v2-trained model
wouldn't be measuring it against the yardstick it was actually trained on.
See docs/mcts.md's Stage 0 diagnosis: v1's training label (each episode's
real final outcome) reflected continued play by the full-strength v10 agent,
not the weak rollout policy the model is actually queried under at
MctsAgent._rollout()'s call site — these helpers produce the corrected
label.
"""

import statistics

from agents.mcts_agent import ROLLOUT_HORIZON, search_actions
from agents.model_value import score_to_log10


def rollout_leaf(game, rng, horizon=ROLLOUT_HORIZON):
    """Exactly mirrors MctsAgent._rollout()'s walk (same search_actions()
    candidate selection, same horizon), returning the resulting leaf. This
    is the actual population model_value()/heuristic_value() are queried
    against at the real search-time call site."""
    leaf = game.clone()
    for _ in range(horizon):
        if leaf.is_over:
            break
        actions = search_actions(leaf, rng)
        if not actions:
            break
        action = rng.choice(actions)
        try:
            leaf.handle_action(action)
        except Exception:
            break
    return leaf


def mc_rollout_value(game, rng, k=6, max_extra_steps=80):
    """Returns (mean_log_score, terminal_clones): mean_log_score is a
    Monte-Carlo estimate of continued rollout-policy play from `game`
    (averaging score_to_log10 over k independent continuations, played all
    the way to termination rather than stopping at ROLLOUT_HORIZON) —
    the corrected Stage 0 v2 training label, replacing v1's mistaken use of
    each episode's real final outcome. terminal_clones is a list of
    (GameEngine, log_score) pairs, one per replicate — a free byproduct of
    computing the mean, each carrying its own exact (not estimated,
    already-computed) outcome — useful as extra terminal-state training
    rows, a population real per-decision-point states never include (see
    docs/mcts.md's Stage 0 diagnosis: 0% terminal coverage in v1's training
    data vs. 96.2% terminal at the actual _rollout() call site)."""
    terminal_clones = []
    log_scores = []
    for _ in range(k):
        sim = game.clone()
        for _ in range(max_extra_steps):
            if sim.is_over:
                break
            actions = search_actions(sim, rng)
            if not actions:
                break
            action = rng.choice(actions)
            try:
                sim.handle_action(action)
            except Exception:
                break
        log_score = score_to_log10(sim.state.score)
        terminal_clones.append((sim, log_score))
        log_scores.append(log_score)
    return statistics.mean(log_scores), terminal_clones
