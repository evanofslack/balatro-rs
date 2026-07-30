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
- Once ApplyTarot() becomes legal in Stage::TarotHand (selected_count
  already satisfies min_targets()), SelectCard/DeselectCard are forced out
  in favor of it too (_force_apply_tarot()) — otherwise the same
  free-action-beats-real-risk trap as SkipBlind recurs one layer deeper:
  heuristic_value() can't distinguish "still selecting" from "resolved"
  (score/money/jokers/round untouched by any of the three for most tarots),
  and a rollout that actually resolves risks a real terminal loss (running
  out of plays via uniform-random PlayHand/DiscardHand before reaching
  required_score) that dithering never risks. Confirmed empirically against
  the exact seed (1025) that first surfaced the bug: a leaf-value nudge
  toward ApplyTarot was tried first and rejected — its branch's average
  rollout value came out *worse* than dithering's, because the downside of
  landing in the terminal-loss range ([-10, -2]) outweighs any reasonably
  sized nudge. See docs/mcts.md's "ApplyTarot never chosen" entry.
"""

import math
import random
from dataclasses import dataclass
from typing import Optional

import pylatro
from data_sample import DecisionSample
from features import state_features
from joker_pool import apply_to_config

AGENT_VERSION = "10"  # 1 = original baseline (results/mcts_*.json, pre-versioning)
# 2 = exclude SkipBlind from search tree
# 3 = margin-aware terminal loss value (heuristic_value)
# 4 = rank PlayHand candidates by one-ply lookahead score
# 5 = exclude MoveCard/SortHand everywhere; stage-aware
#     SelectCard/DeselectCard exclusion (Blind-only),
#     fixing the TarotHand dead end
# 6 = score every legal PlayHand mask at tree expansion instead of a random
#     60-of-218 pre-sample, so a flush/straight/full-house completion can't
#     be missed by sampling luck before it's even ranked
# 7 = force ApplyTarot once legal (drop SelectCard/DeselectCard from the
#     tree), fixing the ApplyTarot dead end (see docs/mcts.md) — same
#     dominated-action-exclusion pattern as SkipBlind/MoveCard/SortHand, not
#     a heuristic_value() reweight (tried first, empirically insufficient:
#     see docs/mcts.md)
# 8 = rank DiscardHand candidates at tree expansion by one-ply lookahead
#     (_discard_hand_score(), same clone+apply+real-score technique
#     _play_hand_score() already uses for PlayHand) instead of uniform
#     sampling — expansion-site candidate admission only, same as v4/v6,
#     not a heuristic_value() change. A discard's replacement cards turn
#     out to be fully deterministic given the current state (count-only,
#     from an already-shuffled deck — see _discard_hand_score()'s
#     docstring), not a distribution to Monte-Carlo-sample over, so plain
#     clone() suffices; an earlier version of this change used the new
#     Game::fork() core primitive for MC redraw sampling before that was
#     discovered to be solving a nonexistent problem here. fork() stays in
#     core as a real, tested primitive for genuinely stochastic
#     resampling elsewhere (e.g. shop reroll evaluation), just unused by
#     this version. Scored candidates are pre-sampled to DISCARD_SCORE_POOL
#     (unbiased random draw, not a shape filter) before ranking — the
#     naive "score every legal mask" approach that works for PlayHand is a
#     218x218 nested cost here (_discard_hand_score() itself rescans up to
#     218 masks per candidate), measured at ~222s/episode uncapped, not
#     viable to eval-measure at any real episode count. Deliberately not a
#     hand-authored suit/rank/straight
#     heuristic and not MadeHand.best_hand()/HandRank: jokers can make the
#     classically-best hand not the highest-scoring one, so the discard's
#     resulting hand is scored via real handle_action()-based scoring
#     (_best_play_hand_score()). NOT the same "v8" as the abandoned
#     e75b066 ("v8 not good") commit — that v8 was an unrelated,
#     fully-retired heuristic-rescale/exploration-retune experiment, never
#     reflected in this ladder. See docs/mcts.md.
# 9 = revert v8's DiscardHand ranking. Measured no improvement over v7 (full
#     5-seed x 100-episode sweep: win_rate 0.4%->0.2%, avg_ante_reached
#     0.03->0.02, avg_final_score 288.4->289.7, discard_rate 39.4%->40.5%,
#     all within noise) despite the ranking mechanism working correctly
#     (confirmed via the hand-type histogram this session added: Flush/
#     Straight/FullHouse started appearing, where they structurally
#     couldn't before). Diagnosis: candidate *admission* into the tree
#     isn't sufficient on its own — _rollout() stays uniform-random and
#     heuristic_value() only rewards realized score, so a discard that sets
#     up a strong hand looks identical to a bad one unless the improved
#     hand actually gets played within the rollout horizon; the real
#     bottleneck is downstream of admission, not fixable by better
#     candidate ranking alone. Combined with the nested-rescan cost being
#     too high to keep iterating on (~222s/episode uncapped, ~41-105s/
#     episode even after the v8 DISCARD_SCORE_POOL cost cap), not worth
#     keeping. Game::fork()/FastBackend::reseed() (core) and the hand-type/
#     played_hands logging this attempt also added are kept — both
#     independently useful and unrelated to the cost/benefit problem. Next
#     direction: auto-tune heuristic_value()'s weights instead of hand-
#     guessing them again (see docs/mcts.md, gym/tune.py).
# 10 = adopted gym/tune.py's first tuning run as the new hardcoded defaults
#      (HeuristicParams' terminal/non-terminal weights, MctsAgent's
#      exploration constant) — real, validated improvement, not a hand
#      guess: full held-out EVAL_SEEDS pass, win_rate 0.0%->10.0%,
#      avg_ante_reached 0.01->0.34, avg_final_score 285.4->354.7,
#      discard_rate 39.0%->50.8%. See docs/mcts.md and results/tune_best.json
#      for the full comparison, the exact tuned values, and how to re-tune.

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


def _force_apply_tarot(game, kept: list) -> list:
    """Once ApplyTarot() is legal (selected_count already satisfies
    min_targets()), drop SelectCard/DeselectCard and force it — fixes the
    "ApplyTarot never chosen" dead end (docs/mcts.md). heuristic_value()
    can't distinguish "still selecting" from "resolved": SelectCard/
    DeselectCard/ApplyTarot don't move score/money/jokers/round for most
    tarots (core/src/tarot.rs). Worse, a leaf-value nudge alone doesn't fix
    it either (tried and empirically rejected against seed 1025): a
    _rollout() that actually resolves the tarot risks landing in
    Stage::Blind and exhausting plays/discards via uniform-random PlayHand/
    DiscardHand picks before reaching required_score, hitting the real
    terminal-loss range ([-10, -2]) — a downside SelectCard/DeselectCard
    dithering never risks, since it never consumes a play. That's the same
    "free/safe action beats risking real play" shape as the SkipBlind
    exploit (see EXCLUDED_KINDS above), just one layer deeper, and the same
    fix applies: remove the dithering option from the tree once resolution
    is possible, rather than trying to out-shape the heuristic against a
    risk it can't see coming. Trade-off: this forces resolving with exactly
    min_targets() selected rather than exploring extra targets up to
    max_targets() (e.g. Star can target up to 3) — acceptable since nothing
    in heuristic_value() can currently tell the difference anyway."""
    if game.state.stage.int() != TAROT_HAND_STAGE:
        return kept
    apply_actions = [a for a in kept if _action_kind(a) == "ApplyTarot"]
    return apply_actions or kept


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
    kept = _force_apply_tarot(game, kept)
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
    DiscardHand is left on uniform random sampling: a v8 attempt to rank it
    by real one-ply lookahead (_discard_hand_score()/_best_play_hand_score())
    was tried and reverted in v9 — the ranking mechanism worked (confirmed
    via the hand-type histogram, Flush/Straight/FullHouse started appearing),
    but a full 5-seed x 100-episode sweep showed no measurable improvement
    over v7 despite that, and the nested exhaustive rescan cost was too high
    to keep iterating on regardless. See docs/mcts.md."""
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
    kept = _force_apply_tarot(game, kept)

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


# v10: adopted from a 100-trial Optuna tuning run (best trial #40 of 100,
# gym/tune.py, objective=avg_final_score, TUNING_SEEDS[:30]) — replaces the
# original hand-guessed values below them in comments. Validated against a
# full held-out EVAL_SEEDS pass (100 episodes): win_rate 0.0%->10.0%,
# avg_ante_reached 0.01->0.34, avg_final_score 285.4->354.7, discard_rate
# 39.0%->50.8%. See docs/mcts.md and results/tune_best.json (the saved
# study output) for the full comparison and how to reproduce/re-tune.
TERMINAL_WIN_VALUE = 137.09321642258507  # was 50.0
TERMINAL_LOSE_FLOOR = -28.058568720074668  # was -10.0
TERMINAL_LOSE_CEILING = -26.746741344688278  # was -2.0
DEFAULT_EXPLORATION = 2.0246187402022633  # was math.sqrt(2) =~ 1.41421356


@dataclass(frozen=True)
class HeuristicParams:
    """Injectable weights for heuristic_value(). Three consecutive
    hand-guessed attempts at these weights (this file's original values, an
    abandoned static discard heuristic, v8's real-scored discard ranking)
    all failed to move win/ante-reached rate — see docs/mcts.md. Rather than
    guess a fourth time, these are exposed for gym/tune.py (Optuna) to
    search instead. `frozen=True` makes instances hashable/immutable and,
    since every field is a plain float, trivially picklable — required for
    passing distinct configurations through a ProcessPoolExecutor the way
    tune.py's trials do. Defaults are constructed from the same module
    constants heuristic_value() uses, so the no-override path stays
    bit-identical to whatever this file's current adopted defaults are (now
    the v10 tuned values above, not the original hand-guessed ones)."""

    terminal_win_value: float = TERMINAL_WIN_VALUE
    terminal_lose_floor: float = TERMINAL_LOSE_FLOOR
    terminal_lose_ceiling: float = TERMINAL_LOSE_CEILING
    money_weight: float = 0.0044455600572427196  # was 0.01
    joker_weight: float = 0.12321152103741428  # was 0.3
    round_weight: float = 0.2076322995162519  # was 0.5


DEFAULT_HEURISTIC_PARAMS = HeuristicParams()


def heuristic_value(game, params: HeuristicParams = DEFAULT_HEURISTIC_PARAMS) -> float:
    """Bounded state-value estimate used as the rollout's leaf evaluation."""
    if game.is_over:
        if game.is_win:
            return params.terminal_win_value
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
            params.terminal_lose_floor
            + (params.terminal_lose_ceiling - params.terminal_lose_floor) * margin
        )
    state = game.state
    progress = state.score_log10 - math.log10(state.required_score + 1)
    return (
        progress
        + params.money_weight * state.money
        + params.joker_weight * len(state.jokers)
        + params.round_weight * state.round
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
        exploration: float = DEFAULT_EXPLORATION,
        agent_seed: Optional[int] = None,
        heuristic_params: Optional[HeuristicParams] = None,
        rollout_horizon: int = ROLLOUT_HORIZON,
        value_fn=None,
    ):
        self.n_simulations = n_simulations
        self.exploration = exploration
        # None preserves the old unseeded behavior (random.Random(None) seeds
        # from OS entropy); pass an int for reproducible A/B comparisons
        # across agent versions/configs. Restored here (not part of the
        # original v5 commit) so this file stays compatible with the current
        # eval.py, which always passes agent_seed=.
        self._rng = random.Random(agent_seed)
        # Two new optional kwargs (gym/tune.py's injection points) — default
        # to today's hardcoded behavior exactly when omitted.
        self.heuristic_params = heuristic_params or DEFAULT_HEURISTIC_PARAMS
        self.rollout_horizon = rollout_horizon
        # Stage 0 (docs/mcts.md): a learned-value-model leaf evaluator,
        # swappable in for heuristic_value() below. `None` (the default)
        # preserves today's exact behavior bit-for-bit — same
        # injectable-with-old-default precedent as heuristic_params/
        # rollout_horizon above. See agents/model_value.py.
        self.value_fn = value_fn

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
        for _ in range(self.rollout_horizon):
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
        if self.value_fn is not None:
            return self.value_fn(game)
        return heuristic_value(game, self.heuristic_params)

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

    def run_episode_with_logging(self, env, seed: int, max_steps: int = 300):
        """Like run_episode, but also returns a list of `DecisionSample`s —
        one per real decision point, feature-encoded via
        features.state_features() and labeled with this episode's eventual
        outcome (final_score/won/ante_reached) once it's known. Standard
        Monte-Carlo-return credit assignment: every state the agent actually
        saw gets the trajectory's final return as its training target.

        A new sibling method, not a modification of run_episode (left
        untouched for every other caller). Deliberately hooks only around
        this loop, not search()/_select_and_expand()/_rollout() — those only
        ever touch cloned/simulated states, never the real `env._game` this
        method snapshots. See docs/mcts.md's "Stage 0" plan.
        """
        apply_to_config(env._config)
        env.reset(seed=seed)
        terminated = truncated = False
        steps = 0
        pending = []  # (feature_vector, step_index) pairs, outcome unknown yet
        while not (terminated or truncated) and steps < max_steps:
            feat = state_features(env._game.state, env._config)
            action = self.search(env._game)
            _, _, terminated, truncated, _ = env.step_action(action)
            pending.append((feat, steps))
            steps += 1

        final_state = env._game.state
        final_score = final_state.score
        won = env._game.is_win
        ante_reached = final_state.round
        samples = [
            DecisionSample(
                features=feat.tolist(),
                final_score=final_score,
                won=won,
                ante_reached=ante_reached,
                seed=seed,
                step_index=step_index,
            )
            for feat, step_index in pending
        ]
        return env, samples

    def run_episode_with_mc_labeling(
        self,
        env,
        seed: int,
        max_steps: int = 300,
        mc_k: int = 6,
        mc_horizon: int = 80,
        rng=None,
    ):
        """Stage 0 v2 (see docs/mcts.md's Stage 0 diagnosis): like
        run_episode_with_logging, same real-episode walk and identical
        starting-state population — but each real decision point is labeled
        with a Monte-Carlo estimate of continued *rollout-policy* play
        (rollout_value.mc_rollout_value()) instead of the real episode's
        eventual outcome. v1's label reflected continued play by this
        (full-strength, `n_simulations`-sim) agent, a mismatch with
        model_value()'s actual call site inside `_rollout()`, which
        evaluates leaves under continued *weak* rollout-policy play —
        diagnosed directly (`gym/diagnose_value_model.py`): v1's model had
        ~zero rank correlation with rollout-policy ground truth even on its
        own training-distribution states. Deliberately changes only the
        label formula, not the states, to keep this a clean, attributable
        A/B against v1.

        Also emits one bonus `DecisionSample` per Monte-Carlo replicate,
        from that replicate's own terminal state (with its own exact,
        already-computed log-score) — free byproducts of computing the
        mean, and the fix for v1's other diagnosed gap: 0% terminal-state
        training coverage despite `_rollout()`'s leaf being terminal 96.2%
        of the time in practice.

        `rollout_value` is imported lazily (here, not at module scope) to
        avoid a circular import — it itself imports `search_actions`/
        `ROLLOUT_HORIZON` from this module.
        """
        from rollout_value import mc_rollout_value

        apply_to_config(env._config)
        env.reset(seed=seed)
        if rng is None:
            rng = self._rng
        terminated = truncated = False
        steps = 0
        samples = []
        while not (terminated or truncated) and steps < max_steps:
            state_feat = state_features(env._game.state, env._config)
            starting_state = env._game.state
            mean_log_score, terminal_clones = mc_rollout_value(
                env._game, rng, k=mc_k, max_extra_steps=mc_horizon
            )
            samples.append(
                DecisionSample(
                    features=state_feat.tolist(),
                    final_score=starting_state.score,
                    won=env._game.is_win,
                    ante_reached=starting_state.round,
                    seed=seed,
                    step_index=steps,
                    mc_log_score=mean_log_score,
                )
            )
            for clone, log_score in terminal_clones:
                clone_state = clone.state
                samples.append(
                    DecisionSample(
                        features=state_features(clone_state, env._config).tolist(),
                        final_score=clone_state.score,
                        won=clone.is_win,
                        ante_reached=clone_state.round,
                        seed=seed,
                        step_index=steps,
                        mc_log_score=log_score,
                    )
                )
            action = self.search(env._game)
            _, _, terminated, truncated, _ = env.step_action(action)
            steps += 1
        return env, samples
