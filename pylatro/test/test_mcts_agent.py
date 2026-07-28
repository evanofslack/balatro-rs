import math
import random
import types

from agents.mcts_agent import (
    MAX_PLAY_HAND_CANDIDATES,
    TERMINAL_LOSE_CEILING,
    TERMINAL_LOSE_FLOOR,
    TERMINAL_WIN_VALUE,
    MctsAgent,
    _action_kind,
    _play_hand_score,
    expansion_actions,
    heuristic_value,
    search_actions,
)
from env import BalatroEnv
from joker_pool import apply_to_config


def _fake_game(
    is_over, is_win=False, score=0, required_score=100, round=0, money=0, jokers=None
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
        ),
    )


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


def test_heuristic_value_stays_bounded():
    # v7 deliberately does not reweight (see docs/mcts.md v7) — it keeps v5's
    # original relative weights and only bounds the output, so a round=1
    # state does not necessarily beat every round=0 state here (unlike v6).
    # That property is deferred to whenever v8 reintroduces the reweight on
    # top of this bounding.
    states = [
        _fake_game(
            is_over=False,
            score=s,
            required_score=300,
            round=r,
            money=m,
            jokers=[None] * j,
        )
        for s in (0, 150, 299)
        for m in (0, 40)
        for j in (0, 5)
        for r in (0, 1, 2)
    ]
    values = [heuristic_value(g) for g in states]
    assert all(0.0 <= v <= 1.0 for v in values)


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
