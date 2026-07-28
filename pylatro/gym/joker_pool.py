"""Curated joker pool for the training/eval milestone.

Restricts shop generation to a small, well-understood subset of jokers
(`Config.joker_pool`) rather than the full 65 implemented jokers, per the
curriculum design in MACHINE-LEARNING.md. Also doubles as the fixed
id-to-index mapping the observation encoder uses for joker identity.

Picked to include at least one Common, one Uncommon, and one Rare (required —
`JokerGenerator::gen_joker` silently falls back to the unfiltered pool for a
rarity with zero allowed choices), all with simple, already-implemented,
already-tested scoring effects.
"""

import pylatro

CURATED_JOKER_IDS = [
    # Common
    "j_joker",
    "j_greedy_joker",
    "j_lusty_joker",
    "j_wrathful_joker",
    "j_gluttenous_joker",
    "j_jolly",
    "j_zany",
    "j_mad",
    "j_crazy",
    "j_droll",
    "j_sly",
    "j_wily",
    "j_clever",
    "j_devious",
    # Uncommon
    "j_four_fingers",
    "j_mime",
    "j_loyalty_card",
    "j_8_ball",
    "j_hack",
    "j_square",
    "j_photograph",
    # Rare
    "j_dna",
    "j_vagabond",
    "j_baron",
]

JOKER_ID_TO_INDEX = {jid: i for i, jid in enumerate(CURATED_JOKER_IDS)}
NUM_CURATED_JOKERS = len(CURATED_JOKER_IDS)


def curated_jokers():
    """The curated pool as `Jokers` instances, for `Config.joker_pool`."""
    return [pylatro.Jokers.from_id(jid) for jid in CURATED_JOKER_IDS]


def joker_index(joker) -> int:
    """Curated-pool index for a `Jokers` instance, or -1 if outside the pool."""
    return JOKER_ID_TO_INDEX.get(joker.id(), -1)


def apply_to_config(config: "pylatro.Config") -> None:
    config.joker_pool = curated_jokers()
