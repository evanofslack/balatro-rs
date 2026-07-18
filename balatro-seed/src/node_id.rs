//! Typed draw node IDs. Each variant's `Display` impl must match Balatro's
//! node-ID string exactly, it's hashed to seed a per-decision RNG.

use std::fmt;

pub(crate) enum NodeId<'a> {
    SoulTarot(i32),
    Tarot {
        source: &'a str,
        ante: i32,
    },
    SoulPlanet(i32),
    Planet {
        source: &'a str,
        ante: i32,
    },
    SoulSpectral(i32),
    Spectral {
        source: &'a str,
        ante: i32,
    },
    Rarity {
        source: &'a str,
        ante: i32,
    },
    Edition {
        source: &'a str,
        ante: i32,
    },
    Joker4,
    Joker3 {
        source: &'a str,
        ante: i32,
    },
    Joker2 {
        source: &'a str,
        ante: i32,
    },
    Joker1 {
        source: &'a str,
        ante: i32,
    },
    Voucher(i32),
    Tag(i32),
    Boss,
    ShopPack(i32),
    OmenGlobe,
    StdSet(i32),
    EnhancedStandard(i32),
    FrontStandard(i32),
    StandardEdition(i32),
    StdSeal(i32),
    StdSealType(i32),
    Cdt(i32),
    /// Escape hatch for tests exercising `Instance`'s primitives directly.
    #[allow(dead_code)]
    Custom(&'a str),
}

impl fmt::Display for NodeId<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeId::SoulTarot(ante) => write!(f, "soul_Tarot{ante}"),
            NodeId::Tarot { source, ante } => write!(f, "Tarot{source}{ante}"),
            NodeId::SoulPlanet(ante) => write!(f, "soul_Planet{ante}"),
            NodeId::Planet { source, ante } => write!(f, "Planet{source}{ante}"),
            NodeId::SoulSpectral(ante) => write!(f, "soul_Spectral{ante}"),
            NodeId::Spectral { source, ante } => write!(f, "Spectral{source}{ante}"),
            // Field order is ante-then-source here, unlike every other variant.
            NodeId::Rarity { source, ante } => write!(f, "rarity{ante}{source}"),
            NodeId::Edition { source, ante } => write!(f, "edi{source}{ante}"),
            NodeId::Joker4 => write!(f, "Joker4"),
            NodeId::Joker3 { source, ante } => write!(f, "Joker3{source}{ante}"),
            NodeId::Joker2 { source, ante } => write!(f, "Joker2{source}{ante}"),
            NodeId::Joker1 { source, ante } => write!(f, "Joker1{source}{ante}"),
            NodeId::Voucher(ante) => write!(f, "Voucher{ante}"),
            NodeId::Tag(ante) => write!(f, "Tag{ante}"),
            NodeId::Boss => write!(f, "boss"),
            NodeId::ShopPack(ante) => write!(f, "shop_pack{ante}"),
            NodeId::OmenGlobe => write!(f, "omen_globe"),
            NodeId::StdSet(ante) => write!(f, "stdset{ante}"),
            NodeId::EnhancedStandard(ante) => write!(f, "Enhancedsta{ante}"),
            NodeId::FrontStandard(ante) => write!(f, "frontsta{ante}"),
            NodeId::StandardEdition(ante) => write!(f, "standard_edition{ante}"),
            NodeId::StdSeal(ante) => write!(f, "stdseal{ante}"),
            NodeId::StdSealType(ante) => write!(f, "stdsealtype{ante}"),
            NodeId::Cdt(ante) => write!(f, "cdt{ante}"),
            NodeId::Custom(s) => write!(f, "{s}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Pins every variant's string form against the crate's prior literals.
    #[test]
    fn node_id_strings_match_immolate() {
        let source = "src";
        let ante = 3;
        let cases: &[(NodeId, &str)] = &[
            (NodeId::SoulTarot(ante), "soul_Tarot3"),
            (NodeId::Tarot { source, ante }, "Tarotsrc3"),
            (NodeId::SoulPlanet(ante), "soul_Planet3"),
            (NodeId::Planet { source, ante }, "Planetsrc3"),
            (NodeId::SoulSpectral(ante), "soul_Spectral3"),
            (NodeId::Spectral { source, ante }, "Spectralsrc3"),
            (NodeId::Rarity { source, ante }, "rarity3src"),
            (NodeId::Edition { source, ante }, "edisrc3"),
            (NodeId::Joker4, "Joker4"),
            (NodeId::Joker3 { source, ante }, "Joker3src3"),
            (NodeId::Joker2 { source, ante }, "Joker2src3"),
            (NodeId::Joker1 { source, ante }, "Joker1src3"),
            (NodeId::Voucher(ante), "Voucher3"),
            (NodeId::Tag(ante), "Tag3"),
            (NodeId::Boss, "boss"),
            (NodeId::ShopPack(ante), "shop_pack3"),
            (NodeId::OmenGlobe, "omen_globe"),
            (NodeId::StdSet(ante), "stdset3"),
            (NodeId::EnhancedStandard(ante), "Enhancedsta3"),
            (NodeId::FrontStandard(ante), "frontsta3"),
            (NodeId::StandardEdition(ante), "standard_edition3"),
            (NodeId::StdSeal(ante), "stdseal3"),
            (NodeId::StdSealType(ante), "stdsealtype3"),
            (NodeId::Cdt(ante), "cdt3"),
            (NodeId::Custom("whatever"), "whatever"),
        ];
        for (id, expected) in cases {
            assert_eq!(id.to_string(), *expected);
        }
    }
}
