use strum::EnumIter;

/// Vouchers come in 16 base/upgrade pairs
/// The upgrade only becomes available in the shop pool
/// once its base voucher has been purchased.
/// See `requires`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, EnumIter)]
pub enum Voucher {
    Overstock,
    OverstockPlus,
    ClearanceSale,
    Liquidation,
    Hone,
    GlowUp,
    RerollSurplus,
    RerollGlut,
    CrystalBall,
    OmenGlobe,
    Telescope,
    Observatory,
    Grabber,
    NachoTong,
    Wasteful,
    Recyclomancy,
    TarotMerchant,
    TarotTycoon,
    PlanetMerchant,
    PlanetTycoon,
    SeedMoney,
    MoneyTree,
    Blank,
    Antimatter,
    MagicTrick,
    Illusion,
    Hieroglyph,
    Petroglyph,
    DirectorsCut,
    Retcon,
    PaintBrush,
    Palette,
}

impl Voucher {
    pub fn name(&self) -> &str {
        match self {
            Self::Overstock => "Overstock",
            Self::OverstockPlus => "Overstock Plus",
            Self::ClearanceSale => "Clearance Sale",
            Self::Liquidation => "Liquidation",
            Self::Hone => "Hone",
            Self::GlowUp => "Glow Up",
            Self::RerollSurplus => "Reroll Surplus",
            Self::RerollGlut => "Reroll Glut",
            Self::CrystalBall => "Crystal Ball",
            Self::OmenGlobe => "Omen Globe",
            Self::Telescope => "Telescope",
            Self::Observatory => "Observatory",
            Self::Grabber => "Grabber",
            Self::NachoTong => "Nacho Tong",
            Self::Wasteful => "Wasteful",
            Self::Recyclomancy => "Recyclomancy",
            Self::TarotMerchant => "Tarot Merchant",
            Self::TarotTycoon => "Tarot Tycoon",
            Self::PlanetMerchant => "Planet Merchant",
            Self::PlanetTycoon => "Planet Tycoon",
            Self::SeedMoney => "Seed Money",
            Self::MoneyTree => "Money Tree",
            Self::Blank => "Blank",
            Self::Antimatter => "Antimatter",
            Self::MagicTrick => "Magic Trick",
            Self::Illusion => "Illusion",
            Self::Hieroglyph => "Hieroglyph",
            Self::Petroglyph => "Petroglyph",
            Self::DirectorsCut => "Director's Cut",
            Self::Retcon => "Retcon",
            Self::PaintBrush => "Paint Brush",
            Self::Palette => "Palette",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::Overstock => "+1 card slot available in shop",
            Self::OverstockPlus => "+1 additional card slot available in shop",
            Self::ClearanceSale => "All cards and packs in shop are 25% off",
            Self::Liquidation => "All cards and packs in shop are 50% off",
            Self::Hone => "Foil, Holographic, and Polychrome cards appear 2x more often",
            Self::GlowUp => "Foil, Holographic, and Polychrome cards appear 4x more often",
            Self::RerollSurplus => "Rerolls cost $2 less",
            Self::RerollGlut => "Rerolls cost $2 less again",
            Self::CrystalBall => "+1 consumable slot",
            Self::OmenGlobe => "Spectral cards may appear in Arcana Packs",
            Self::Telescope => {
                "Celestial Packs always contain the Planet card for your most played hand"
            }
            Self::Observatory => {
                "Planet cards in your consumable area give x1.5 Mult for their hand"
            }
            Self::Grabber => "+1 hand per round",
            Self::NachoTong => "+1 additional hand per round",
            Self::Wasteful => "+1 discard per round",
            Self::Recyclomancy => "+1 additional discard per round",
            Self::TarotMerchant => "Tarot cards appear 2x more often in the shop",
            Self::TarotTycoon => "Tarot cards appear 4x more often in the shop",
            Self::PlanetMerchant => "Planet cards appear 2x more often in the shop",
            Self::PlanetTycoon => "Planet cards appear 4x more often in the shop",
            Self::SeedMoney => "Raises interest cap by $5",
            Self::MoneyTree => "Raises interest cap by $10 more",
            Self::Blank => "Does nothing?",
            Self::Antimatter => "+1 Joker slot",
            Self::MagicTrick => "Playing cards can be purchased from the shop",
            Self::Illusion => "Shop playing cards may have an Enhancement, Edition, and/or Seal",
            Self::Hieroglyph => "-1 Ante, -1 hand per round",
            Self::Petroglyph => "-1 Ante, -1 discard per round",
            Self::DirectorsCut => "Reroll the Boss Blind once per Ante, $10 per reroll",
            Self::Retcon => "Reroll the Boss Blind unlimited times per Ante, $10 per reroll",
            Self::PaintBrush => "+1 hand size",
            Self::Palette => "+1 additional hand size",
        }
    }

    pub fn cost(&self) -> usize {
        10
    }

    /// `Some(base)` if this is a tier-2 upgrade voucher (only appears in the
    /// shop pool once `base` has been purchased in the current run),
    /// `None` if this is a tier-1 base voucher.
    pub fn requires(&self) -> Option<Voucher> {
        match self {
            Self::OverstockPlus => Some(Self::Overstock),
            Self::Liquidation => Some(Self::ClearanceSale),
            Self::GlowUp => Some(Self::Hone),
            Self::RerollGlut => Some(Self::RerollSurplus),
            Self::OmenGlobe => Some(Self::CrystalBall),
            Self::Observatory => Some(Self::Telescope),
            Self::NachoTong => Some(Self::Grabber),
            Self::Recyclomancy => Some(Self::Wasteful),
            Self::TarotTycoon => Some(Self::TarotMerchant),
            Self::PlanetTycoon => Some(Self::PlanetMerchant),
            Self::MoneyTree => Some(Self::SeedMoney),
            Self::Antimatter => Some(Self::Blank),
            Self::Illusion => Some(Self::MagicTrick),
            Self::Petroglyph => Some(Self::Hieroglyph),
            Self::Retcon => Some(Self::DirectorsCut),
            Self::Palette => Some(Self::PaintBrush),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn test_voucher_count() {
        assert_eq!(Voucher::iter().count(), 32);
    }

    #[test]
    fn test_voucher_pairing() {
        assert_eq!(Voucher::Overstock.requires(), None);
        assert_eq!(Voucher::OverstockPlus.requires(), Some(Voucher::Overstock));
    }

    #[test]
    fn test_every_voucher_pairs_exactly_once() {
        let upgrades: Vec<Voucher> = Voucher::iter().filter(|v| v.requires().is_some()).collect();
        assert_eq!(upgrades.len(), 16);
    }
}
