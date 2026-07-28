import math
import random
import types

from agents.mcts_agent import (
    MAX_PLAY_HAND_CANDIDATES,
    TAROT_HAND_STAGE,
    TERMINAL_LOSE_CEILING,
    TERMINAL_LOSE_FLOOR,
    TERMINAL_WIN_VALUE,
    MctsAgent,
    _action_kind,
    _force_apply_tarot,
    _play_hand_score,
    expansion_actions,
    heuristic_value,
    search_actions,
)
from env import BalatroEnv
from joker_pool import apply_to_config

# Any stage other than TarotHand.
_NON_TAROT_HAND_STAGE = TAROT_HAND_STAGE + 1


def _fake_game(
    is_over,
    is_win=False,
    score=0,
    required_score=100,
    round=0,
    money=0,
    jokers=None,
    stage_int=_NON_TAROT_HAND_STAGE,
):
    return types.SimpleNamespace(
        is_over=is_over,
        is_win=is_win,
        state=types.SimpleNamespace(
            score=score,
            required_score=required_score,
            score_log10=math.log10(score + 1),
            money=money,
            jokers=jokers if jokers is not None else [],
            round=round,
            stage=types.SimpleNamespace(int=lambda: stage_int),
        ),
    )


class Action_SelectCard:
    pass


class Action_DeselectCard:
    pass


class Action_ApplyTarot:
    pass


def test_search_actions_excludes_skip_blind():
    env = BalatroEnv()
    env.reset(seed=1)
    rng = random.Random(0)
    actions = search_actions(env._game, rng)
    kinds = {_action_kind(a) for a in actions}
    assert "SkipBlind" not in kinds
    assert kinds == {"SelectBlind"}


def test_heuristic_value_win_and_bounds():
    assert heuristic_value(_fake_game(is_over=True, is_win=True)) == TERMINAL_WIN_VALUE

    never_scored = heuristic_value(_fake_game(is_over=True, score=0, required_score=100))
    near_miss = heuristic_value(_fake_game(is_over=True, score=95, required_score=100))
    mid = heuristic_value(_fake_game(is_over=True, score=50, required_score=100))

    assert never_scored == TERMINAL_LOSE_FLOOR
    assert TERMINAL_LOSE_FLOOR <= never_scored < mid < near_miss <= TERMINAL_LOSE_CEILING


def test_force_apply_tarot_selects_only_apply_when_legal():
    # Regression test for the ApplyTarot dead end (docs/mcts.md): once
    # ApplyTarot is legal (selected_count already satisfies min_targets()),
    # SelectCard/DeselectCard must be dropped so search()'s len(untried)==1
    # shortcut forces immediate resolution instead of leaving room to keep
    # dithering. A heuristic_value() leaf-value nudge was tried first and
    # rejected: empirically, against the real seed (1025) that surfaced the
    # bug, a rollout that actually resolves the tarot risks a real terminal
    # loss (exhausting plays before reaching required_score), which
    # outweighs any reasonably sized nudge — see docs/mcts.md.
    game = _fake_game(is_over=False, stage_int=TAROT_HAND_STAGE)
    kept = [Action_SelectCard(), Action_DeselectCard(), Action_ApplyTarot()]
    result = _force_apply_tarot(game, kept)
    assert len(result) == 1
    assert _action_kind(result[0]) == "ApplyTarot"


def test_force_apply_tarot_allows_selecting_below_min_targets():
    # Before ApplyTarot is legal (not enough targets selected yet), it's
    # simply absent from `kept` — selection must be allowed to proceed
    # unfiltered.
    game = _fake_game(is_over=False, stage_int=TAROT_HAND_STAGE)
    kept = [Action_SelectCard(), Action_SelectCard()]
    result = _force_apply_tarot(game, kept)
    assert result == kept


def test_force_apply_tarot_is_noop_outside_tarot_hand():
    game = _fake_game(is_over=False, stage_int=_NON_TAROT_HAND_STAGE)
    kept = [Action_SelectCard(), Action_ApplyTarot()]
    result = _force_apply_tarot(game, kept)
    assert result == kept


def test_no_skip_blind_in_episode():
    env = BalatroEnv(max_steps=20)
    apply_to_config(env._config)
    agent = MctsAgent(n_simulations=5)
    agent.run_episode(env, seed=1, max_steps=20)
    kinds = {_action_kind(a) for a in env._game.state.action_history}
    assert "SkipBlind" not in kinds


def test_agent_seed_makes_episode_deterministic():
    def play(agent_seed):
        env = BalatroEnv(max_steps=30)
        apply_to_config(env._config)
        agent = MctsAgent(n_simulations=5, agent_seed=agent_seed)
        agent.run_episode(env, seed=1, max_steps=30)
        state = env._game.state
        return [_action_kind(a) for a in state.action_history], state.score

    first = play(agent_seed=42)
    second = play(agent_seed=42)
    assert first == second


def test_expansion_actions_retains_true_best_play_hand():
    env = BalatroEnv()
    env.reset(seed=1)
    rng = random.Random(0)
    select_blind = next(
        a for a in env._game.gen_actions() if _action_kind(a) == "SelectBlind"
    )
    env._game.handle_action(select_blind)

    raw_play_hands = [
        a for a in env._game.gen_actions() if _action_kind(a) == "PlayHand"
    ]
    assert len(raw_play_hands) > MAX_PLAY_HAND_CANDIDATES
    best = max(raw_play_hands, key=lambda a: _play_hand_score(env._game, a))
    best_score = _play_hand_score(env._game, best)

    offered = expansion_actions(env._game, rng)
    offered_play_hands = [a for a in offered if _action_kind(a) == "PlayHand"]
    assert any(
        _play_hand_score(env._game, a) == best_score for a in offered_play_hands
    )
