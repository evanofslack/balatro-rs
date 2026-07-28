import logging
from typing import Optional

import gymnasium as gym
import numpy as np
import pylatro
from gymnasium import spaces

from joker_pool import NUM_CURATED_JOKERS, apply_to_config, joker_index

logger = logging.getLogger(__name__)

# Card identity encoding: [value, suit, enhancement, edition, seal], each
# -1 for "absent" (empty slot, or no enhancement/seal on a real card).
NUM_VALUES = 13
NUM_SUITS = 4
NUM_ENHANCEMENTS = 8
NUM_EDITIONS = 5
NUM_SEALS = 4
CARD_FIELDS = 5
EMPTY_CARD = np.array([-1, -1, -1, -1, -1], dtype=np.int32)


def encode_card(card) -> np.ndarray:
    return np.array(
        [
            int(card.value),
            int(card.suit),
            int(card.enhancement) if card.enhancement is not None else -1,
            int(card.edition),
            int(card.seal) if card.seal is not None else -1,
        ],
        dtype=np.int32,
    )


def encode_cards(cards, pad_to: int) -> np.ndarray:
    rows = [encode_card(c) for c in cards[:pad_to]]
    rows += [EMPTY_CARD.copy() for _ in range(pad_to - len(rows))]
    return np.stack(rows)


def encode_jokers(jokers, pad_to: int):
    ids = np.full(pad_to, -1, dtype=np.int32)
    editions = np.full(pad_to, -1, dtype=np.int32)
    counters = np.zeros(pad_to, dtype=np.float32)
    for i, j in enumerate(jokers[:pad_to]):
        ids[i] = joker_index(j)
        editions[i] = int(j.edition())
        counters[i] = j.state().counter
    return ids, editions, counters


class BalatroEnv(gym.Env):
    def __init__(
        self,
        max_steps: int = 300,
        use_curated_joker_pool: bool = True,
        max_reorder_actions: Optional[int] = 8,
    ):
        super().__init__()

        config = pylatro.Config()
        config.ante_end = 1
        # Unbounded by default at the engine level (so human/CLI/TUI play is
        # unaffected) — training envs should always set a cap. Without one,
        # MoveCard/SortHand are free, reversible, zero-cost actions that a
        # search/RL agent can exploit to stall a rollout budget without ever
        # progressing (see MACHINE-LEARNING.md's "unbounded free-action
        # loop" gap) — confirmed empirically: an early MCTS eval run against
        # an uncapped env spent ~75% of its total actions on MoveCard.
        config.max_reorder_actions = max_reorder_actions
        if use_curated_joker_pool:
            apply_to_config(config)

        self._config = config
        self._max_steps = max_steps
        self._game = pylatro.GameEngine(self._config)
        self._steps = 0
        self._prev_score_log10 = 0.0

        # Legacy index-based action space, kept for gym.Env/SB3 compatibility
        # and the random baseline. It cannot represent the atomic
        # PlayHand/DiscardHand actions (unbounded combinatorial size) — use
        # `legal_actions()` / `step_action()` for those.
        self.action_space = spaces.Discrete(len(self._game.gen_action_space()))

        card_box = spaces.Box(
            low=-1,
            high=max(NUM_VALUES, NUM_SUITS, NUM_ENHANCEMENTS, NUM_EDITIONS, NUM_SEALS),
            shape=(CARD_FIELDS,),
            dtype=np.int32,
        )
        self.observation_space = spaces.Dict(
            {
                "score_log10": spaces.Box(low=0, high=20, shape=(1,), dtype=np.float32),
                "required_score_log10": spaces.Box(
                    low=0, high=20, shape=(1,), dtype=np.float32
                ),
                "stage": spaces.Discrete(config.stage_max + 1),
                "round": spaces.Discrete(config.ante_end + 1),
                "plays": spaces.Discrete(config.plays + 1),
                "discards": spaces.Discrete(config.discards + 1),
                "money": spaces.Discrete(config.money_max + 1),
                "deck_len": spaces.Discrete(config.deck_max + 1),
                "discarded_len": spaces.Discrete(config.discarded_max + 1),
                "available": spaces.Box(
                    low=-1,
                    high=card_box.high[0],
                    shape=(config.available_max, CARD_FIELDS),
                    dtype=np.int32,
                ),
                "selected": spaces.Box(
                    low=-1,
                    high=card_box.high[0],
                    shape=(config.selected_max, CARD_FIELDS),
                    dtype=np.int32,
                ),
                "jokers_id": spaces.Box(
                    low=-1,
                    high=max(NUM_CURATED_JOKERS - 1, 0),
                    shape=(config.joker_slots_max,),
                    dtype=np.int32,
                ),
                "jokers_edition": spaces.Box(
                    low=-1,
                    high=NUM_EDITIONS - 1,
                    shape=(config.joker_slots_max,),
                    dtype=np.int32,
                ),
                "jokers_counter": spaces.Box(
                    low=-1e6,
                    high=1e6,
                    shape=(config.joker_slots_max,),
                    dtype=np.float32,
                ),
            }
        )
        self.score_queue = []
        self.actions_queue = []

    def _get_obs(self):
        state = self._game.state
        jokers_id, jokers_edition, jokers_counter = encode_jokers(
            state.jokers, self._config.joker_slots_max
        )
        return {
            "score_log10": np.array([state.score_log10], dtype=np.float32),
            "required_score_log10": np.array(
                [np.log10(state.required_score + 1)], dtype=np.float32
            ),
            "stage": state.stage.int(),
            "round": state.round,
            "plays": state.plays,
            "discards": state.discards,
            "money": state.money,
            "deck_len": len(state.deck),
            "discarded_len": len(state.discarded),
            "available": encode_cards(state.available, self._config.available_max),
            "selected": encode_cards(state.selected, self._config.selected_max),
            "jokers_id": jokers_id,
            "jokers_edition": jokers_edition,
            "jokers_counter": jokers_counter,
        }

    def _get_info(self):
        state = self._game.state
        return {"score": state.score, "required_score": state.required_score}

    def _reward_and_bookkeeping(self, legal: bool, was_win: bool):
        state = self._game.state
        score_log10 = state.score_log10
        reward = score_log10 - self._prev_score_log10
        self._prev_score_log10 = score_log10

        terminated = self._game.is_over
        if terminated:
            reward += (10.0 if was_win else -5.0) * state.round
            self.actions_queue.append(len(state.action_history))
            self.score_queue.append(state.score)
        if not legal:
            reward = -1.0
            logger.debug("illegal action index rejected; mask=%s", self.action_mask())
        return reward, terminated

    def step(self, index):
        legal = bool(self._game.gen_action_space()[index] == 1)
        if legal:
            self._game.handle_action_index(index)

        reward, terminated = self._reward_and_bookkeeping(legal, self._game.is_win)
        self._steps += 1
        truncated = (not terminated) and self._steps >= self._max_steps

        return self._get_obs(), reward, terminated, truncated, self._get_info()

    def legal_actions(self):
        """Full legal-move list (including atomic PlayHand/DiscardHand),
        the primary interface for anything beyond the fixed index mask —
        e.g. MCTS."""
        return self._game.gen_actions()

    def step_action(self, action):
        """Step with a real `Action` object (from `legal_actions()`),
        bypassing the fixed index encoding."""
        try:
            self._game.handle_action(action)
            legal = True
        except Exception:
            legal = False
        reward, terminated = self._reward_and_bookkeeping(legal, self._game.is_win)
        self._steps += 1
        truncated = (not terminated) and self._steps >= self._max_steps
        return self._get_obs(), reward, terminated, truncated, self._get_info()

    def reset(self, seed: Optional[int] = None, options: Optional[dict] = None):
        super().reset(seed=seed)
        if seed is not None:
            self._config.seed = seed
        self._game = pylatro.GameEngine(self._config)
        self._steps = 0
        self._prev_score_log10 = 0.0
        return self._get_obs(), self._get_info()

    def render(self, mode="human"):
        return

    def action_mask(self):
        return np.asarray(self._game.gen_action_space(), dtype=np.int8)


def register():
    gym.register(
        id="gymnasium_env/Balatro-v0",
        entry_point=BalatroEnv,
    )
