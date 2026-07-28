"""Vanilla UCT MCTS agent — no neural net.

Re-plans from scratch at every real decision (clone the current state,
build a fresh tree, act on the most-visited root child). Bump AGENT_VERSION
whenever search/heuristic behavior changes, so saved eval results
(eval.py --out, which records it in meta.agent_version) can be attributed to
the code that produced them.

Deliberate simplifications, documented as they were added:

- Rollouts are bounded-horizon with a heuristic leaf evaluation, not
  uniform-random-to-terminal — a uniform-random policy essentially never
  clears a blind, so a full random rollout gives near-zero signal at any
  search depth worth exploring on a laptop-scale simulation budget.
- The granular SelectCard/DeselectCard/Play/Discard actions are excluded
  from the search tree in favor of the atomic PlayHand/DiscardHand actions
  they're strictly dominated by (same reachable outcomes, fewer nodes).
  PlayHand/DiscardHand's own branching factor (all C(n,<=5) subsets) is
  still large, so candidates are randomly sampled down to a bounded count
  per node rather than enumerated exhaustively.
- SkipBlind is excluded from the search tree too, but for a different
  reason: skip_blind()'s tag effect is currently a no-op for every Tag
  variant (core/src/game.rs), and skipping forfeits the money a cleared
  blind would pay out — but within ROLLOUT_HORIZON, a rollout that skips
  reliably avoids ever risking the terminal-loss value, while a rollout
  that actually tries to play a blind risks it. Left unfiltered, UCB1
  learns to skip every blind (empirically: 81 SkipBlind vs 112 SelectBlind
  picks, 0% win rate over 100 eval episodes at 100 sims/decision).

  This filter is local to this agent's own search — core, env.legal_actions(),
  and other agents/human play still see and can choose SkipBlind normally.
- MoveCard/SortHand are excluded from the search tree everywhere: hand
  order has no effect on score in the current curated pool (no seals or
  order-dependent enhancements like Glass are in play yet), so they're
  pure search overhead — confirmed empirically as the dominant real-action
  cost (60-70% of an episode's actual moves) once SkipBlind stopped
  masking it. Revisit this exclusion once seals/enhancements that care
  about hand position are introduced.

  This also fixes a real dead end, not just a speed cost: during
  Stage::TarotHand (targeting a tarot), SelectCard/DeselectCard are the
  *only* way to satisfy the tarot's min_targets() before ApplyTarot()
  becomes legal (core/src/generator.rs's gen_actions_tarot_hand) — there's
  no atomic equivalent there the way there is in the Blind stage. Since
  MoveCard was unbounded specifically in TarotHand (core/src/game.rs,
  deliberately exempted from the blind-phase reorder cap) and
  SelectCard/DeselectCard were excluded unconditionally everywhere, the
  search had no way to ever reach ApplyTarot() and fell back to spamming
  MoveCard until max_steps truncated the episode (confirmed: 4/100 eval
  episodes did exactly this, ~290 MoveCard actions each, ~40% of that
  run's entire action budget). SelectCard/DeselectCard are therefore only
  excluded during the Blind stage now — see _excluded_kinds().
"""

import math
import random
from typing import Optional

import pylatro
from joker_pool import apply_to_config

AGENT_VERSION = "6"  # 1 = original baseline (results/mcts_*.json, pre-versioning)
# 2 = exclude SkipBlind from search tree
# 3 = margin-aware terminal loss value (heuristic_value)
# 4 = rank PlayHand candidates by one-ply lookahead score
# 5 = exclude MoveCard/SortHand everywhere; stage-aware
#     SelectCard/DeselectCard exclusion (Blind-only),
#     fixing the TarotHand dead end
# 6 = score every legal PlayHand mask at tree expansion instead of a random
#     60-of-218 pre-sample, so a flush/straight/full-house completion can't
#     be missed by sampling luck before it's even ranked

# Stage::TarotHand's Stage.int() encoding (core/src/stage.rs) — the one
# stage where SelectCard/DeselectCard aren't dominated by anything atomic.
TAROT_HAND_STAGE = 8

# Excluded in every stage: Play()/Discard() are dominated by atomic
# PlayHand/DiscardHand; SkipBlind and MoveCard/SortHand per the module
# docstring above.
ALWAYS_EXCLUDED_KINDS = {"Play", "Discard", "SkipBlind", "MoveCard", "SortHand"}
# Excluded only during the Blind stage, where atomic PlayHand/DiscardHand
# dominate them. In Stage::TarotHand they're the only way to select tarot
# targets, so they must stay legal there.
BLIND_ONLY_EXCLUDED_KINDS = {"SelectCard", "DeselectCard"}

MAX_PLAY_HAND_CANDIDATES = 20
MAX_DISCARD_HAND_CANDIDATES = 20
ROLLOUT_HORIZON = 15


def _action_kind(action) -> str:
    name = type(action).__name__
    prefix = "Action_"
    return name[len(prefix) :] if name.startswith(prefix) else name


def _excluded_kinds(game) -> set:
    if game.state.stage.int() == TAROT_HAND_STAGE:
        return ALWAYS_EXCLUDED_KINDS
    return ALWAYS_EXCLUDED_KINDS | BLIND_ONLY_EXCLUDED_KINDS


def search_actions(game, rng):
    excluded = _excluded_kinds(game)
    kept = []
    play_hands = []
    discard_hands = []
    for action in game.gen_actions():
        kind = _action_kind(action)
        if kind in excluded:
            continue
        elif kind == "PlayHand":
            play_hands.append(action)
        elif kind == "DiscardHand":
            discard_hands.append(action)
        else:
            kept.append(action)
    if len(play_hands) > MAX_PLAY_HAND_CANDIDATES:
        play_hands = rng.sample(play_hands, MAX_PLAY_HAND_CANDIDATES)
    if len(discard_hands) > MAX_DISCARD_HAND_CANDIDATES:
        discard_hands = rng.sample(discard_hands, MAX_DISCARD_HAND_CANDIDATES)
    return kept + play_hands + discard_hands


# Tree-expansion-only PlayHand biasing. Deliberately NOT used by _rollout()
# — that call site fires up to ROLLOUT_HORIZON * n_simulations times per
# real decision with no caching, so ranking there would multiply the extra
# clone+handle_action cost by orders of magnitude. Tree expansion is capped
# at n_simulations new nodes per decision (each ranks once, cached via
# node.untried), so this is where branching-factor quality actually matters
# for a fixed sim budget: it decides which subtrees UCB1 gets to explore at
# all, not just which action one rollout step takes.
#
# Every legal mask is scored (no pre-sampling): an 8-card hand has
# sum(C(8,k) for k in 1..=5) == 218 possible PlayHand masks, cheap enough to
# score exhaustively at tree-expansion call sites. A prior version randomly
# pre-sampled down to 60 before ranking (~27% coverage) — confirmed
# empirically to be the reason the agent consistently landed just short of
# clearing the first blind: the true best-scoring mask (often a
# flush/straight/full-house completion, exactly the higher-variance, higher-
# payoff hand worth playing for) could simply not be in the random 60, so
# the ranker below never saw it to rank it top.
RANKED_PLAY_HAND_TOP_N = 14
RANDOM_PLAY_HAND_TAIL = 6  # top_n + tail == MAX_PLAY_HAND_CANDIDATES (20),
# keeps today's UCB1 branching factor unchanged


def _play_hand_score(game, action) -> int:
    """One-ply lookahead: the real resulting state.score after playing this
    mask, via core's actual scoring (jokers/editions/enhancements included).
    All candidates share the same parent `game`, so ranking by resulting
    absolute score is equivalent to ranking by score delta."""
    preview = game.clone()
    try:
        preview.handle_action(action)
    except Exception:
        return -1
    return preview.state.score


def expansion_actions(game, rng):
    """Like search_actions(), but PlayHand candidates are biased toward
    higher-scoring subsets via exhaustive one-ply lookahead instead of
    uniform sampling. Used only at tree-expansion call sites (root init,
    _select_and_expand) — see module comment above RANKED_PLAY_HAND_TOP_N.
    DiscardHand is left on uniform random sampling, unchanged: discarding
    never moves state.score (the redraw is random per clone), so ranking it
    would need multiple stochastic samples per candidate — out of scope for
    this pass, deferred."""
    excluded = _excluded_kinds(game)
    kept = []
    play_hands = []
    discard_hands = []
    for action in game.gen_actions():
        kind = _action_kind(action)
        if kind in excluded:
            continue
        elif kind == "PlayHand":
            play_hands.append(action)
        elif kind == "DiscardHand":
            discard_hands.append(action)
        else:
            kept.append(action)

    if len(play_hands) > MAX_PLAY_HAND_CANDIDATES:
        ranked = sorted(
            play_hands, key=lambda a: _play_hand_score(game, a), reverse=True
        )
        top = ranked[:RANKED_PLAY_HAND_TOP_N]
        remainder = ranked[RANKED_PLAY_HAND_TOP_N:]
        tail = rng.sample(remainder, min(RANDOM_PLAY_HAND_TAIL, len(remainder)))
        play_hands = top + tail
        # _select_and_expand() pops from the end of `untried` (LIFO), so
        # position within this list determines explore-order priority under
        # a small sim budget, not just membership. Shuffle so the bias is
        # "which candidates made the cut" (top-N-plus-random-tail), not
        # "best candidates get explored last" — the latter is what a static
        # best-first order would silently produce here.
        rng.shuffle(play_hands)

    if len(discard_hands) > MAX_DISCARD_HAND_CANDIDATES:
        discard_hands = rng.sample(discard_hands, MAX_DISCARD_HAND_CANDIDATES)

    return kept + play_hands + discard_hands


TERMINAL_WIN_VALUE = 50.0
TERMINAL_LOSE_FLOOR = -10.0  # never scored a chip
TERMINAL_LOSE_CEILING = -2.0  # ran out of plays one point short of clearing


def heuristic_value(game) -> float:
    """Bounded state-value estimate used as the rollout's leaf evaluation."""
    if game.is_over:
        if game.is_win:
            return TERMINAL_WIN_VALUE
        # Margin-aware loss: a near-miss (plays exhausted with score close to
        # required_score) is far less bad than never having scored at all. A
        # flat value here discards real signal between the two, and is a
        # fragile pattern any future free/stalling action could exploit the
        # same way SkipBlind did (see EXCLUDED_KINDS above). Loss is only
        # reachable with state.plays == 0 (core/src/game.rs's handle_score),
        # so there's no separate reserves term to add beyond the score
        # margin itself. Kept as a plain linear interpolation, no
        # per-mechanic weighting, to avoid overfitting a new formula to the
        # one exploit that motivated it.
        state = game.state
        required = max(state.required_score, 1)
        margin = min(state.score / required, 1.0)
        return (
            TERMINAL_LOSE_FLOOR + (TERMINAL_LOSE_CEILING - TERMINAL_LOSE_FLOOR) * margin
        )
    state = game.state
    progress = state.score_log10 - math.log10(state.required_score + 1)
    return progress + 0.01 * state.money + 0.3 * len(state.jokers) + 0.5 * state.round


class Node:
    def __init__(self, game, parent=None):
        self.game = game
        self.parent = parent
        self.children = []  # list of (action, Node)
        self.untried = None  # lazily populated from search_actions
        self.visits = 0
        self.value_sum = 0.0

    def is_terminal(self) -> bool:
        return self.game.is_over

    def ucb1_child(self, c: float):
        log_n = math.log(self.visits)
        return max(
            self.children,
            key=lambda ac: (
                ac[1].value_sum / ac[1].visits + c * math.sqrt(log_n / ac[1].visits)
            ),
        )


class MctsAgent:
    def __init__(
        self,
        n_simulations: int = 100,
        exploration: float = math.sqrt(2),
        agent_seed: Optional[int] = None,
    ):
        self.n_simulations = n_simulations
        self.exploration = exploration
        # None preserves the old unseeded behavior (random.Random(None) seeds
        # from OS entropy); pass an int for reproducible A/B comparisons
        # across agent versions/configs. Restored here (not part of the
        # original v5 commit) so this file stays compatible with the current
        # eval.py, which always passes agent_seed=.
        self._rng = random.Random(agent_seed)

    def search(self, game) -> "pylatro.Action":
        root = Node(game.clone())
        root.untried = expansion_actions(root.game, self._rng)
        if len(root.untried) == 1:
            return root.untried[0]

        for _ in range(self.n_simulations):
            node = self._select_and_expand(root)
            value = self._rollout(node.game.clone())
            self._backpropagate(node, value)

        if not root.children:
            return self._rng.choice(root.untried)
        best_action, _ = max(root.children, key=lambda ac: ac[1].visits)
        return best_action

    def _select_and_expand(self, root: Node) -> Node:
        node = root
        while not node.is_terminal():
            if node.untried is None:
                node.untried = expansion_actions(node.game, self._rng)
            if node.untried:
                action = node.untried.pop()
                child_game = node.game.clone()
                try:
                    child_game.handle_action(action)
                except Exception:
                    continue
                child = Node(child_game, parent=node)
                node.children.append((action, child))
                return child
            if not node.children:
                return node
            _, node = node.ucb1_child(self.exploration)
        return node

    def _rollout(self, game) -> float:
        for _ in range(ROLLOUT_HORIZON):
            if game.is_over:
                break
            actions = search_actions(game, self._rng)
            if not actions:
                break
            action = self._rng.choice(actions)
            try:
                game.handle_action(action)
            except Exception:
                break
        return heuristic_value(game)

    def _backpropagate(self, node: Node, value: float) -> None:
        while node is not None:
            node.visits += 1
            node.value_sum += value
            node = node.parent

    def run_episode(self, env, seed: int, max_steps: int = 300):
        apply_to_config(env._config)
        env.reset(seed=seed)
        terminated = truncated = False
        steps = 0
        while not (terminated or truncated) and steps < max_steps:
            action = self.search(env._game)
            _, _, terminated, truncated, _ = env.step_action(action)
            steps += 1
        return env
