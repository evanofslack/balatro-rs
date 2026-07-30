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


# Tarot/Planet identity encoding. Unlike jokers there's no curated-subset
# concept here (all 22 tarots / 12 planets are always in play), so these are
# plain fixed id tables, not a training-pool allow-list like joker_pool.py's.
# Ids must match Tarot::id()/Planets::id() (balatro-types/src/{tarot,planet}.rs)
# exactly — keyed by string id (not enum discriminant order) for stability
# against Rust enum reordering, same reasoning as CURATED_JOKER_IDS.
TAROT_IDS = [
    "c_fool", "c_magician", "c_high_priestess", "c_empress", "c_emperor",
    "c_heirophant", "c_lovers", "c_chariot", "c_justice", "c_hermit",
    "c_wheel_of_fortune", "c_strength", "c_hanged_man", "c_death",
    "c_temperance", "c_devil", "c_tower", "c_star", "c_moon", "c_sun",
    "c_judgement", "c_world",
]  # fmt: skip
PLANET_IDS = [
    "c_pluto", "c_mercury", "c_uranus", "c_venus", "c_saturn", "c_jupiter",
    "c_earth", "c_mars", "c_neptune", "c_planet_x", "c_ceres", "c_eris",
]  # fmt: skip
TAROT_ID_TO_INDEX = {tid: i for i, tid in enumerate(TAROT_IDS)}
PLANET_ID_TO_INDEX = {pid: i for i, pid in enumerate(PLANET_IDS)}
NUM_TAROTS = len(TAROT_IDS)
NUM_PLANETS = len(PLANET_IDS)

# Shop always offers exactly 2 non-pack items (some mix of jokers/consumables)
# and exactly 2 packs — see Shop::refresh_cards/refresh's hardcoded `0..2`
# loops (core/src/shop.rs). NOT the same thing as Config.consumable_slots
# (owned-consumable cap) or Config.store_consumable_slots_max (exists but is
# not actually wired into shop generation — confirmed by reading shop.rs;
# using it here would silently drift from the real shop size).
SHOP_JOKERS_MAX = 2
SHOP_CONSUMABLES_MAX = 2
SHOP_PACKS_MAX = 2

# Max pack contents, from Pack::description()'s count logic
# (core/src/pack.rs): Buffoon-Normal=2, Buffoon-other=4, else-Normal=3,
# else-Jumbo/Mega=5. Intrinsic to Pack's category/size combinatorics, not
# Config-driven, so hardcoded here same as NUM_EDITIONS/NUM_SEALS above.
PACK_CONTENTS_MAX = 5

CONSUMABLE_FIELDS = 3  # [type_code, index, cost]; type_code: 0=Tarot, 1=Planet
PACK_FIELDS = 3  # [category_ordinal, size_ordinal, cost]
# [kind_code, card_value, card_suit, card_enh, card_edition, card_seal, index]
# kind_code: 0=Tarot, 1=Joker, 2=Planet, 3=PlayingCard. One fixed-width row
# covers all PackContent variants (PlayingCard needs the 5 card sub-fields;
# Tarot/Joker/Planet just need `index`) rather than a ragged structure.
PACK_CONTENT_FIELDS = CARD_FIELDS + 2
EMPTY_CONSUMABLE = np.full(CONSUMABLE_FIELDS, -1, dtype=np.int32)
EMPTY_PACK = np.full(PACK_FIELDS, -1, dtype=np.int32)
EMPTY_PACK_CONTENT = np.full(PACK_CONTENT_FIELDS, -1, dtype=np.int32)


def encode_consumable(c) -> np.ndarray:
    tarot = c.as_tarot()
    planet = c.as_planet()
    if tarot is not None:
        type_code, index = 0, TAROT_ID_TO_INDEX.get(tarot.id(), -1)
    elif planet is not None:
        type_code, index = 1, PLANET_ID_TO_INDEX.get(planet.id(), -1)
    else:
        type_code, index = -1, -1
    return np.array([type_code, index, int(c.cost())], dtype=np.int32)


def encode_consumables(consumables, pad_to: int) -> np.ndarray:
    rows = [encode_consumable(c) for c in consumables[:pad_to]]
    rows += [EMPTY_CONSUMABLE.copy() for _ in range(pad_to - len(rows))]
    return np.stack(rows)


def encode_pack(p) -> np.ndarray:
    return np.array(
        [int(p.category), int(p.size), int(p.cost())], dtype=np.int32
    )


def encode_packs(packs, pad_to: int) -> np.ndarray:
    rows = [encode_pack(p) for p in packs[:pad_to]]
    rows += [EMPTY_PACK.copy() for _ in range(pad_to - len(rows))]
    return np.stack(rows)


def encode_pack_content(content) -> np.ndarray:
    tarot = content.as_tarot()
    joker = content.as_joker()
    planet = content.as_planet()
    playing_card = content.as_playing_card()
    row = EMPTY_PACK_CONTENT.copy()
    if tarot is not None:
        row[0] = 0
        row[6] = TAROT_ID_TO_INDEX.get(tarot.id(), -1)
    elif joker is not None:
        row[0] = 1
        row[6] = joker_index(joker)
    elif planet is not None:
        row[0] = 2
        row[6] = PLANET_ID_TO_INDEX.get(planet.id(), -1)
    elif playing_card is not None:
        row[0] = 3
        row[1:6] = encode_card(playing_card)
    return row


def encode_pack_contents(contents, pad_to: int) -> np.ndarray:
    rows = [encode_pack_content(c) for c in contents[:pad_to]]
    rows += [EMPTY_PACK_CONTENT.copy() for _ in range(pad_to - len(rows))]
    return np.stack(rows)


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
                "shop_jokers_id": spaces.Box(
                    low=-1,
                    high=max(NUM_CURATED_JOKERS - 1, 0),
                    shape=(SHOP_JOKERS_MAX,),
                    dtype=np.int32,
                ),
                "shop_jokers_edition": spaces.Box(
                    low=-1,
                    high=NUM_EDITIONS - 1,
                    shape=(SHOP_JOKERS_MAX,),
                    dtype=np.int32,
                ),
                "shop_consumables": spaces.Box(
                    low=-1,
                    high=max(NUM_TAROTS, NUM_PLANETS),
                    shape=(SHOP_CONSUMABLES_MAX, CONSUMABLE_FIELDS),
                    dtype=np.int32,
                ),
                "shop_packs": spaces.Box(
                    low=-1,
                    high=8,
                    shape=(SHOP_PACKS_MAX, PACK_FIELDS),
                    dtype=np.int32,
                ),
                "reroll_cost": spaces.Box(
                    low=0, high=1000, shape=(1,), dtype=np.float32
                ),
                "open_pack_contents": spaces.Box(
                    low=-1,
                    high=max(NUM_CURATED_JOKERS - 1, NUM_TAROTS, NUM_PLANETS),
                    shape=(PACK_CONTENTS_MAX, PACK_CONTENT_FIELDS),
                    dtype=np.int32,
                ),
                "open_pack_picks_remaining": spaces.Box(
                    low=0, high=10, shape=(1,), dtype=np.float32
                ),
                "consumables": spaces.Box(
                    low=-1,
                    high=max(NUM_TAROTS, NUM_PLANETS),
                    shape=(config.consumable_slots, CONSUMABLE_FIELDS),
                    dtype=np.int32,
                ),
                "active_tarot_index": spaces.Box(
                    low=-1, high=max(NUM_TAROTS - 1, 0), shape=(1,), dtype=np.int32
                ),
                "active_tarot_min_targets": spaces.Box(
                    low=0, high=3, shape=(1,), dtype=np.int32
                ),
                "active_tarot_max_targets": spaces.Box(
                    low=0, high=3, shape=(1,), dtype=np.int32
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
        shop_jokers_id, shop_jokers_edition, _ = encode_jokers(
            state.shop.jokers, SHOP_JOKERS_MAX
        )
        active_tarot = state.active_tarot
        open_pack = state.open_pack
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
            "shop_jokers_id": shop_jokers_id,
            "shop_jokers_edition": shop_jokers_edition,
            "shop_consumables": encode_consumables(
                state.shop.consumables, SHOP_CONSUMABLES_MAX
            ),
            "shop_packs": encode_packs(state.shop.packs, SHOP_PACKS_MAX),
            "reroll_cost": np.array([state.reroll_cost], dtype=np.float32),
            "open_pack_contents": encode_pack_contents(
                open_pack.contents if open_pack is not None else [],
                PACK_CONTENTS_MAX,
            ),
            "open_pack_picks_remaining": np.array(
                [open_pack.picks_remaining if open_pack is not None else 0],
                dtype=np.float32,
            ),
            "consumables": encode_consumables(
                state.consumables, self._config.consumable_slots
            ),
            "active_tarot_index": np.array(
                [TAROT_ID_TO_INDEX.get(active_tarot.id(), -1) if active_tarot else -1],
                dtype=np.int32,
            ),
            "active_tarot_min_targets": np.array(
                [active_tarot.min_targets() if active_tarot else 0], dtype=np.int32
            ),
            "active_tarot_max_targets": np.array(
                [active_tarot.max_targets() if active_tarot else 0], dtype=np.int32
            ),
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
