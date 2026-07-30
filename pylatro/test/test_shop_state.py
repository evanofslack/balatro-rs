"""Smoke tests for the shop/consumable Python bindings added alongside the
Stage 0 learned-value-function work (see docs/mcts.md). A full real-engine
integration test that reliably drives into Stage.Shop()/TarotHand/PackOpen
via actual play was already tried and abandoned as impractical elsewhere in
this codebase (docs/mcts.md's "Testing caveat, worth recording" — even 150
MCTS simulations don't reliably clear ante 1's small blind across several
seeds), so these tests instead verify: (1) the new getters are readable and
correctly empty on a fresh game (a real, deterministic state every game
starts in), and (2) the new pyclass accessor methods (`as_tarot`/`as_planet`/
`as_joker`/`as_playing_card`) work correctly against directly-constructed
values, which doesn't require ever reaching those stages via real play.
"""

import pylatro


def test_fresh_game_shop_and_consumable_state_is_empty():
    game = pylatro.GameEngine(pylatro.Config())
    state = game.state

    assert state.shop.jokers == []
    assert state.shop.consumables == []
    assert state.shop.packs == []
    assert state.open_pack is None
    assert state.consumables == []
    assert state.last_consumable_used is None
    assert state.active_tarot is None
    # No fixed expected value asserted (Game::new's default isn't 0) — just
    # confirm the getter is wired and returns a plain number.
    assert isinstance(state.reroll_cost, int)


def test_tarot_targets_known_values():
    # Fool requires no targets; Death requires exactly 2 — both hand-picked
    # from balatro-types/src/tarot.rs's min_targets()/max_targets() match arms.
    fool = pylatro.Tarot.Fool
    assert fool.min_targets() == 0
    assert fool.max_targets() == 0
    assert not fool.requires_targets()

    death = pylatro.Tarot.Death
    assert death.min_targets() == 2
    assert death.max_targets() == 2
    assert death.requires_targets()

    assert fool.id() == "c_fool"
    assert pylatro.Tarot.from_id("c_fool") == fool


def test_consumable_as_tarot_and_as_planet_are_mutually_exclusive():
    tarot_consumable = pylatro.Consumable.from_id("c_fool")
    assert tarot_consumable.as_tarot() == pylatro.Tarot.Fool
    assert tarot_consumable.as_planet() is None

    planet_consumable = pylatro.Consumable.from_id("c_pluto")
    assert planet_consumable.as_planet() == pylatro.Planets.Pluto
    assert planet_consumable.as_tarot() is None


def test_pack_content_accessors():
    tarot_content = pylatro.PackContent.Tarot(pylatro.Tarot.Star)
    assert tarot_content.as_tarot() == pylatro.Tarot.Star
    assert tarot_content.as_joker() is None
    assert tarot_content.as_planet() is None
    assert tarot_content.as_playing_card() is None

    planet_content = pylatro.PackContent.Planet(pylatro.Planets.Mars)
    assert planet_content.as_planet() == pylatro.Planets.Mars
    assert planet_content.as_tarot() is None

    # A real Card, pulled from a fresh deck rather than constructed directly
    # (Value/Suit aren't individually registered pyclasses). `Card` isn't
    # `pyclass(eq)` (only derives Rust-side PartialEq), so `==` falls back to
    # Python object identity — compare fields instead.
    card = pylatro.GameEngine(pylatro.Config()).state.deck[0]
    card_content = pylatro.PackContent.PlayingCard(card)
    round_tripped = card_content.as_playing_card()
    assert round_tripped.value == card.value
    assert round_tripped.suit == card.suit
    assert card_content.as_tarot() is None


def test_shop_state_reachable_via_random_play_does_not_crash():
    """Opportunistic, not required to trigger: if random legal play happens
    to reach Stage.Shop()/PackOpen/TarotHand within the step budget, confirm
    the new getters return internally-consistent, non-crashing data. Doesn't
    assert anything if those stages are never reached (see module docstring)."""
    import random

    rng = random.Random(0)
    for seed in range(5):
        game = pylatro.GameEngine(pylatro.Config())
        for _ in range(300):
            if game.is_over:
                break
            actions = game.gen_actions()
            if not actions:
                break
            game.handle_action(rng.choice(actions))
            state = game.state
            stage_int = state.stage.int()
            if stage_int == 5:  # Stage::Shop()
                assert len(state.shop.packs) == 2
                assert len(state.shop.jokers) + len(state.shop.consumables) == 2
            elif stage_int == 9:  # Stage::PackOpen()
                assert state.open_pack is not None
                assert state.open_pack.picks_remaining >= 1
            elif stage_int == 8:  # Stage::TarotHand
                assert state.active_tarot is not None
                assert state.active_tarot.min_targets() <= state.active_tarot.max_targets()
