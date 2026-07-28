"""Vanilla UCT MCTS agent — no neural net.

Re-plans from scratch at every real decision (clone the current state,
build a fresh tree, act on the most-visited root child). Bump AGENT_VERSION
whenever search/heuristic behavior changes, so saved eval results
(eval.py --out, which records it in meta.agent_version) can be attributed to
the code that produced them.

The rationale and measured evidence behind each version's deliberate
simplifications (rollout heuristic, action exclusions, the TarotHand
dead-end fix, heuristic_value's scale/weights) live in docs/mcts.md, not
here — that file is the changelog, this file is the implementation.
"""

import math
import random
from typing import Optional

import pylatro
from joker_pool import apply_to_config

AGENT_VERSION = "8"  # 1 = original baseline (results/mcts_*.json, pre-versioning)
# 2 = exclude SkipBlind from search tree
# 3 = margin-aware terminal loss value (heuristic_value)
# 4 = rank PlayHand candidates by one-ply lookahead score
# 5 = exclude MoveCard/SortHand everywhere; stage-aware
#     SelectCard/DeselectCard exclusion (Blind-only),
#     fixing the TarotHand dead end
# 6 = heuristic_value() rescaled to [0.0, 1.0]; non-terminal branch
#     reweighted so clearing the next blind (state.round) dominates
# 7 = isolate v6's [0,1] rescale from its round-dominance reweight —
#     v5's original weights, just bounded (docs/mcts.md v7)
# 8 = retune UCB1's exploration constant to match v7's compressed value
#     scale (docs/mcts.md v8) — only change from v7

# Stage::TarotHand's Stage.int() encoding (core/src/stage.rs) — the one
# stage where SelectCard/DeselectCard aren't dominated by anything atomic.
TAROT_HAND_STAGE = 8

# Play()/Discard()/SkipBlind/MoveCard/SortHand: excluded everywhere — see
# docs/mcts.md (v2, v5) for why.
ALWAYS_EXCLUDED_KINDS = {"Play", "Discard", "SkipBlind", "MoveCard", "SortHand"}
# SelectCard/DeselectCard: excluded only in the Blind stage — they're the
# only way to target a tarot in Stage::TarotHand. See docs/mcts.md (v5).
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


# Tree-expansion-only PlayHand biasing — not used by _rollout() (uncached,
# fires far more often). See docs/mcts.md (v4) for why.
RAW_PLAY_HAND_POOL = 60
RANKED_PLAY_HAND_TOP_N = 14
RANDOM_PLAY_HAND_TAIL = 6  # top_n + tail == MAX_PLAY_HAND_CANDIDATES (20)


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
    higher-scoring subsets via one-ply lookahead. DiscardHand stays uniform
    random — see docs/mcts.md (v4, "Next steps") for why."""
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
        pool = play_hands
        if len(pool) > RAW_PLAY_HAND_POOL:
            pool = rng.sample(pool, RAW_PLAY_HAND_POOL)
        ranked = sorted(pool, key=lambda a: _play_hand_score(game, a), reverse=True)
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


TERMINAL_WIN_VALUE = 1.0
TERMINAL_LOSE_FLOOR = 0.0  # never scored a chip
TERMINAL_LOSE_CEILING = 0.2  # ran out of plays one point short of clearing

# v7 isolates the [0,1] rescale from v6's round-dominance reweight (see
# docs/mcts.md v6/v7) — this pass deliberately keeps v5's original relative
# weights, just bounded, to test the rescale alone.
NONTERMINAL_FLOOR = 0.05
NONTERMINAL_CEILING = 0.9
RAW_MIN = -3.0  # ~ boss blind (required=600), score=0: log10(1)-log10(601)
RAW_MAX = 3.5  # ~ round=2, near-max money/jokers, progress near 0


def heuristic_value(game) -> float:
    """Bounded [0.0, 1.0] state-value estimate for the rollout leaf/UCB1
    backprop — see docs/mcts.md (v7) for why it's scaled this way."""
    if game.is_over:
        if game.is_win:
            return TERMINAL_WIN_VALUE
        # Margin-aware loss (near-miss vs. total whiff) — see docs/mcts.md (v3).
        state = game.state
        required = max(state.required_score, 1)
        margin = min(state.score / required, 1.0)
        return (
            TERMINAL_LOSE_FLOOR + (TERMINAL_LOSE_CEILING - TERMINAL_LOSE_FLOOR) * margin
        )
    state = game.state
    progress = state.score_log10 - math.log10(state.required_score + 1)
    raw = progress + 0.01 * state.money + 0.3 * len(state.jokers) + 0.5 * state.round
    frac = max(0.0, min((raw - RAW_MIN) / (RAW_MAX - RAW_MIN), 1.0))
    return NONTERMINAL_FLOOR + frac * (NONTERMINAL_CEILING - NONTERMINAL_FLOOR)


# v8: v7's bounding compressed the non-terminal branch's raw span
# (RAW_MAX - RAW_MIN = 6.5) down to (NONTERMINAL_CEILING - NONTERMINAL_FLOOR)
# = 0.85 — a ~7.6x compression — while UCB1's exploration bonus was left at
# the textbook math.sqrt(2), calibrated for an uncompressed [0,1] reward.
# Scaled proportionally: sqrt(2) * (0.85 / 6.5) =~ 0.185. The terminal
# branch compressed by a different ratio (~60x, [-10,50] -> [0,1]) than the
# non-terminal branch — this constant is tuned toward the non-terminal
# branch specifically, since bounded rollouts reach a true terminal state
# far less often. See docs/mcts.md (v8).
EXPLORATION_CONSTANT = math.sqrt(2) * (NONTERMINAL_CEILING - NONTERMINAL_FLOOR) / (
    RAW_MAX - RAW_MIN
)


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
        exploration: float = EXPLORATION_CONSTANT,
        agent_seed: Optional[int] = None,
    ):
        self.n_simulations = n_simulations
        self.exploration = exploration
        # None preserves the old unseeded behavior (random.Random(None) seeds
        # from OS entropy); pass an int for reproducible A/B comparisons
        # across agent versions/configs.
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
