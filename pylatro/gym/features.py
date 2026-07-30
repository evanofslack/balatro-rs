"""Pure state -> flat feature vector for a tabular (gradient-boosted-tree)
value model — the "Stage 0" learned value function (see docs/mcts.md).

Deliberately takes a `pylatro.GameState`/`pylatro.Config` directly, not a
live `BalatroEnv`, so it's usable both from the data collector (mid-
`MctsAgent.search()`, operating on `env._game.state`/`env._config`) and later
from the trained model's leaf-evaluator at inference time. Reuses env.py's
existing per-card/per-joker/per-shop encoders verbatim rather than
duplicating them — a `GradientBoostingRegressor` has no structural notion of
"slot 3 vs slot 4" the way an attention-based net would, so every encoder's
padded array is simply flattened and concatenated into one 1D vector; there's
no need to preserve the 2D per-slot shape env.py's gym `Box` spaces use for
RL purposes.
"""

import math

import numpy as np

from env import (
    PACK_CONTENTS_MAX,
    SHOP_CONSUMABLES_MAX,
    SHOP_JOKERS_MAX,
    SHOP_PACKS_MAX,
    encode_cards,
    encode_consumables,
    encode_jokers,
    encode_pack_contents,
    encode_packs,
)


def state_features(state, config) -> np.ndarray:
    """Flat float feature vector for one game state. Fixed length for a
    given `config` (padding sizes come from `config`/the module constants
    above, never from data-dependent lengths), so this is safe to stack into
    a training matrix across many states."""
    active_tarot = state.active_tarot
    open_pack = state.open_pack

    jokers_id, jokers_edition, jokers_counter = encode_jokers(
        state.jokers, config.joker_slots_max
    )
    shop_jokers_id, shop_jokers_edition, _ = encode_jokers(
        state.shop.jokers, SHOP_JOKERS_MAX
    )

    scalars = np.array(
        [
            state.score_log10,
            math.log10(state.required_score + 1),
            state.stage.int(),
            state.round,
            state.plays,
            state.discards,
            state.money,
            len(state.deck),
            len(state.discarded),
            state.reroll_cost,
            open_pack.picks_remaining if open_pack is not None else 0,
            active_tarot.min_targets() if active_tarot is not None else 0,
            active_tarot.max_targets() if active_tarot is not None else 0,
        ],
        dtype=np.float64,
    )

    arrays = [
        scalars,
        encode_cards(state.available, config.available_max).ravel(),
        encode_cards(state.selected, config.selected_max).ravel(),
        jokers_id,
        jokers_edition,
        jokers_counter,
        shop_jokers_id,
        shop_jokers_edition,
        encode_consumables(state.shop.consumables, SHOP_CONSUMABLES_MAX).ravel(),
        encode_packs(state.shop.packs, SHOP_PACKS_MAX).ravel(),
        encode_pack_contents(
            open_pack.contents if open_pack is not None else [],
            PACK_CONTENTS_MAX,
        ).ravel(),
        encode_consumables(state.consumables, config.consumable_slots).ravel(),
    ]
    return np.concatenate([a.astype(np.float64) for a in arrays])
