//! Vouchers — run-long shop upgrades.
//!
//! One voucher is offered per ante. It sits in its own shop slot, survives
//! rerolls, and stays on offer for every shop of that ante until bought.
//! Buying it is permanent for the run: there is no sell action.
//!
//! The enum itself lives in `balatro-types` (16 base/upgrade pairs, with
//! `Voucher::requires` naming the base an upgrade is gated behind). This
//! module adds what `core` needs on top: which vouchers are currently
//! offerable, and the modifiers an owned set implies.
//!
//! Most effects are *derived*, not applied: `Vouchers` answers questions
//! ("how many consumable slots does the player get?") and callers add the
//! bonus to their `Config` baseline. That keeps effects idempotent across
//! save/load and makes double-application impossible. The two exceptions
//! are the one-shot effects — Hieroglyph/Petroglyph's "-1 Ante" and the
//! reroll-cost drop — which mutate `Game` at purchase time.
//!
//! See `vouchers.md` for per-voucher implementation status.

pub use balatro_types::Voucher;
use strum::IntoEnumIterator;

/// The vouchers redeemed so far this run, plus every modifier they imply.
///
/// Ordered by purchase, which is also the order the TUI's Run Info panel
/// lists them in.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Vouchers {
    owned: Vec<Voucher>,
}

impl Vouchers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has(&self, voucher: Voucher) -> bool {
        self.owned.contains(&voucher)
    }

    pub fn owned(&self) -> &[Voucher] {
        &self.owned
    }

    pub fn len(&self) -> usize {
        self.owned.len()
    }

    pub fn is_empty(&self) -> bool {
        self.owned.is_empty()
    }

    pub(crate) fn redeem(&mut self, voucher: Voucher) {
        if !self.has(voucher) {
            self.owned.push(voucher);
        }
    }

    /// Vouchers that may be drawn into a shop right now: not already
    /// redeemed, and — for the 16 tier-2 upgrades — only once their base
    /// has been redeemed.
    pub fn offerable(&self) -> Vec<Voucher> {
        Voucher::iter().filter(|v| self.is_offerable(*v)).collect()
    }

    pub fn is_offerable(&self, voucher: Voucher) -> bool {
        if self.has(voucher) {
            return false;
        }
        match voucher.requires() {
            Some(base) => self.has(base),
            None => true,
        }
    }

    // --- shop layout & pricing ---

    /// Extra shop card slots from Overstock/Overstock Plus (+1 each).
    pub fn shop_card_slots_bonus(&self) -> usize {
        usize::from(self.has(Voucher::Overstock)) + usize::from(self.has(Voucher::OverstockPlus))
    }

    /// Percent off cards and packs, from Clearance Sale / Liquidation.
    /// Vouchers themselves are never discounted (matches the real game).
    pub fn discount_pct(&self) -> usize {
        if self.has(Voucher::Liquidation) {
            50
        } else if self.has(Voucher::ClearanceSale) {
            25
        } else {
            0
        }
    }

    /// `cost` after the shop discount, floored, never below $1 for a
    /// nonzero base price.
    pub fn discounted(&self, cost: usize) -> usize {
        discounted(cost, self.discount_pct())
    }

    /// Dollars off each reroll, from Reroll Surplus / Reroll Glut.
    pub fn reroll_discount(&self) -> usize {
        2 * (usize::from(self.has(Voucher::RerollSurplus))
            + usize::from(self.has(Voucher::RerollGlut)))
    }

    /// How much more often Foil/Holographic/Polychrome roll, from
    /// Hone (2x) / Glow Up (4x). Multiplies the edition probabilities in
    /// `shop::gen_edition`.
    pub fn edition_rate_mult(&self) -> u32 {
        if self.has(Voucher::GlowUp) {
            4
        } else if self.has(Voucher::Hone) {
            2
        } else {
            1
        }
    }

    /// Weight of Tarot cards in the shop item roll (base 4.0), raised by
    /// Tarot Merchant / Tarot Tycoon. Values match the real game's.
    pub fn tarot_rate(&self) -> f64 {
        if self.has(Voucher::TarotTycoon) {
            32.0
        } else if self.has(Voucher::TarotMerchant) {
            9.6
        } else {
            4.0
        }
    }

    /// Planet counterpart of [`Vouchers::tarot_rate`].
    pub fn planet_rate(&self) -> f64 {
        if self.has(Voucher::PlanetTycoon) {
            32.0
        } else if self.has(Voucher::PlanetMerchant) {
            9.6
        } else {
            4.0
        }
    }

    /// Weight of playing cards in the shop item roll — zero until Magic
    /// Trick makes them purchasable at all.
    pub fn playing_card_rate(&self) -> f64 {
        if self.has(Voucher::MagicTrick) {
            4.0
        } else {
            0.0
        }
    }

    /// Illusion: shop playing cards may roll an enhancement, edition,
    /// and/or seal. Without it they are always plain.
    pub fn shop_cards_are_modified(&self) -> bool {
        self.has(Voucher::Illusion)
    }

    // --- slots & per-round resources ---

    /// Crystal Ball: +1 consumable slot.
    pub fn consumable_slot_bonus(&self) -> usize {
        usize::from(self.has(Voucher::CrystalBall))
    }

    /// Antimatter: +1 joker slot.
    pub fn joker_slot_bonus(&self) -> usize {
        usize::from(self.has(Voucher::Antimatter))
    }

    /// Paint Brush / Palette: +1 hand size each.
    pub fn hand_size_bonus(&self) -> usize {
        usize::from(self.has(Voucher::PaintBrush)) + usize::from(self.has(Voucher::Palette))
    }

    /// Grabber/Nacho Tong give +1 hand each; Hieroglyph costs one.
    pub fn hands_delta(&self) -> isize {
        isize::from(self.has(Voucher::Grabber)) + isize::from(self.has(Voucher::NachoTong))
            - isize::from(self.has(Voucher::Hieroglyph))
    }

    /// Wasteful/Recyclomancy give +1 discard each; Petroglyph costs one.
    pub fn discards_delta(&self) -> isize {
        isize::from(self.has(Voucher::Wasteful)) + isize::from(self.has(Voucher::Recyclomancy))
            - isize::from(self.has(Voucher::Petroglyph))
    }

    /// Seed Money raises the interest cap by $5, Money Tree by $10 more.
    pub fn interest_cap_bonus(&self) -> usize {
        5 * usize::from(self.has(Voucher::SeedMoney))
            + 10 * usize::from(self.has(Voucher::MoneyTree))
    }

    // --- packs & scoring ---

    /// Telescope: every Celestial Pack contains the Planet card for the
    /// most played hand.
    pub fn telescope(&self) -> bool {
        self.has(Voucher::Telescope)
    }

    /// Omen Globe: Spectral cards may appear in Arcana Packs.
    pub fn omen_globe(&self) -> bool {
        self.has(Voucher::OmenGlobe)
    }

    /// Observatory: each held Planet card gives x1.5 Mult when its own
    /// hand is scored.
    pub fn observatory(&self) -> bool {
        self.has(Voucher::Observatory)
    }

    /// Hieroglyph/Petroglyph lower the ante by one when redeemed. Only
    /// these two do anything at purchase time beyond joining the set.
    pub(crate) fn lowers_ante(voucher: Voucher) -> bool {
        matches!(voucher, Voucher::Hieroglyph | Voucher::Petroglyph)
    }
}

/// `cost` reduced by `pct` percent, floored, never dropping a priced item
/// to free. Split out of [`Vouchers::discounted`] so move-closures can
/// capture the percentage alone.
pub(crate) fn discounted(cost: usize, pct: usize) -> usize {
    if pct == 0 || cost == 0 {
        return cost;
    }
    (cost * (100 - pct) / 100).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owning(vouchers: &[Voucher]) -> Vouchers {
        let mut v = Vouchers::new();
        for voucher in vouchers {
            v.redeem(*voucher);
        }
        v
    }

    #[test]
    fn test_upgrades_gated_behind_base() {
        let none = Vouchers::new();
        assert!(none.is_offerable(Voucher::Overstock));
        assert!(!none.is_offerable(Voucher::OverstockPlus));

        let base = owning(&[Voucher::Overstock]);
        assert!(!base.is_offerable(Voucher::Overstock), "already owned");
        assert!(base.is_offerable(Voucher::OverstockPlus));
    }

    #[test]
    fn test_offerable_starts_at_the_16_base_vouchers() {
        assert_eq!(Vouchers::new().offerable().len(), 16);
    }

    #[test]
    fn test_redeem_is_idempotent() {
        let mut v = Vouchers::new();
        v.redeem(Voucher::Blank);
        v.redeem(Voucher::Blank);
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn test_discount_tiers() {
        assert_eq!(Vouchers::new().discounted(8), 8);
        assert_eq!(owning(&[Voucher::ClearanceSale]).discounted(8), 6);
        assert_eq!(
            owning(&[Voucher::ClearanceSale, Voucher::Liquidation]).discounted(8),
            4
        );
        // never free, even at 50% off a $1 item
        assert_eq!(owning(&[Voucher::Liquidation]).discounted(1), 1);
        assert_eq!(owning(&[Voucher::Liquidation]).discounted(0), 0);
    }

    #[test]
    fn test_stacking_bonuses() {
        let both = owning(&[Voucher::Overstock, Voucher::OverstockPlus]);
        assert_eq!(both.shop_card_slots_bonus(), 2);

        let rerolls = owning(&[Voucher::RerollSurplus, Voucher::RerollGlut]);
        assert_eq!(rerolls.reroll_discount(), 4);

        let money = owning(&[Voucher::SeedMoney, Voucher::MoneyTree]);
        assert_eq!(money.interest_cap_bonus(), 15);

        let editions = owning(&[Voucher::Hone, Voucher::GlowUp]);
        assert_eq!(editions.edition_rate_mult(), 4);
    }

    #[test]
    fn test_ante_lowering_vouchers_trade_a_resource() {
        let hiero = owning(&[Voucher::Hieroglyph]);
        assert_eq!(hiero.hands_delta(), -1);
        assert_eq!(hiero.discards_delta(), 0);
        assert!(Vouchers::lowers_ante(Voucher::Hieroglyph));

        let petro = owning(&[Voucher::Petroglyph]);
        assert_eq!(petro.discards_delta(), -1);
        assert_eq!(petro.hands_delta(), 0);
        assert!(Vouchers::lowers_ante(Voucher::Petroglyph));

        assert!(!Vouchers::lowers_ante(Voucher::Blank));
    }

    #[test]
    fn test_grabber_and_hieroglyph_cancel_out() {
        let both = owning(&[Voucher::Grabber, Voucher::Hieroglyph]);
        assert_eq!(both.hands_delta(), 0);
    }
}
