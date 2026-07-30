# Vouchers

All 32 vouchers (16 base/upgrade pairs) are generated, offered, and
purchasable. This file records what each one actually *does* in `core`.

## How they work

One voucher is drawn per ante and sits in its own shop slot. Unlike the
rest of the shop it survives rerolls and stays on offer across every shop
visit of that ante until bought; once bought, the slot stays empty until
the next ante. An upgrade (tier-2) voucher is only ever offered once its
base has been redeemed — see `Voucher::requires` in `balatro-types`.
Vouchers are permanent: there is no sell action, and Clearance Sale /
Liquidation never discount them.

Effects are *derived* rather than applied. `Game.vouchers` is the source
of truth and the limits are recomputed on demand (`Game::hand_size`,
`Game::consumable_slots`, `Game::plays_per_round`, `Game::price`, …), so
nothing can double-apply across a save/load round trip. The only
purchase-time mutations are Hieroglyph/Petroglyph's "-1 Ante", the reroll
price drop, and Overstock topping up the shop the player is standing in.

Both RNG backends offer vouchers. `RngMode::Fast` draws uniformly from
the offerable pool. `RngMode::Real` draws through `balatro-seed`'s
`next_voucher`, which consumes the same node the real game would, and
upgrades in place when the drawn base is already owned.

## Status

| Voucher         | Effect                                       | Status                                                         |
| --------------- | -------------------------------------------- | -------------------------------------------------------------- |
| Overstock       | +1 shop card slot                            | ✅                                                              |
| Overstock Plus  | +1 more shop card slot                       | ✅                                                              |
| Clearance Sale  | Cards and packs 25% off                      | ✅                                                              |
| Liquidation     | Cards and packs 50% off                      | ✅                                                              |
| Hone            | Foil/Holo/Poly 2x more often                 | ✅ (`Fast` only — `Real` reads its own active-voucher list)     |
| Glow Up         | Foil/Holo/Poly 4x more often                 | ✅ (same)                                                       |
| Reroll Surplus  | Rerolls cost $2 less                         | ✅                                                              |
| Reroll Glut     | Rerolls cost $2 less again                   | ✅                                                              |
| Crystal Ball    | +1 consumable slot                           | ✅                                                              |
| Omen Globe      | Spectrals may appear in Arcana Packs         | ⚠️ `Real` mode only — `core` has no spectral use-effects        |
| Telescope       | Celestial Packs contain most played hand     | ✅                                                              |
| Observatory     | Held Planet gives x1.5 Mult for its hand     | ✅                                                              |
| Grabber         | +1 hand per round                            | ✅                                                              |
| Nacho Tong      | +1 more hand per round                       | ✅                                                              |
| Wasteful        | +1 discard per round                         | ✅                                                              |
| Recyclomancy    | +1 more discard per round                    | ✅                                                              |
| Tarot Merchant  | Tarots 2x more often in shop                 | ✅                                                              |
| Tarot Tycoon    | Tarots 4x more often in shop                 | ✅                                                              |
| Planet Merchant | Planets 2x more often in shop                | ✅                                                              |
| Planet Tycoon   | Planets 4x more often in shop                | ✅                                                              |
| Seed Money      | Interest cap +$5                             | ✅                                                              |
| Money Tree      | Interest cap +$10 more                       | ✅                                                              |
| Blank           | Nothing                                      | ✅ (faithfully does nothing)                                    |
| Antimatter      | +1 Joker slot                                | ✅                                                              |
| Magic Trick     | Playing cards buyable from the shop          | ✅ (`Action::BuyPlayingCard`)                                   |
| Illusion        | Shop playing cards may have modifiers        | ✅                                                              |
| Hieroglyph      | -1 Ante, -1 hand per round                   | ✅                                                              |
| Petroglyph      | -1 Ante, -1 discard per round                | ✅                                                              |
| Director's Cut  | Reroll Boss Blind once per ante, $10         | ❌ inert — `core` has no boss blind modifiers to reroll         |
| Retcon          | Reroll Boss Blind unlimited times, $10       | ❌ inert — same                                                 |
| Paint Brush     | +1 hand size                                 | ✅                                                              |
| Palette         | +1 more hand size                            | ✅                                                              |

The three non-✅ entries are blocked on features `core` doesn't have yet
(spectral use-effects, boss blind modifiers), not on voucher plumbing.
They can still be drawn, bought, and shown; they just have nothing to act
on. See the "missing" list in [`README.md`](README.md).

## Where the code lives

| Concern                                         | File                    |
| ----------------------------------------------- | ----------------------- |
| `Voucher` enum, names, costs, upgrade pairs     | `balatro-types/src/voucher.rs` |
| `Vouchers` set and every derived modifier       | `core/src/voucher.rs`   |
| Shop slot, offer/purchase, playing-card slots   | `core/src/shop.rs`      |
| Per-ante draw, purchase, one-shot effects       | `core/src/game.rs`      |
| Backend draw hooks (`gen_voucher`, activation)  | `core/src/rng.rs`       |
| `BuyVoucher` / `BuyPlayingCard` move generation | `core/src/generator.rs`, `core/src/space.rs` |
