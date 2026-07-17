use std::fmt;

use balatro_types::{
    BossBlind, Consumable, DeckVariant, Edition, Jokers, PackCategory, PackSize, Tag, Voucher,
};

/// A content id from `meta.jkr` or `save.jkr`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ItemId {
    Joker(Jokers),
    Consumable(Consumable),
    Voucher(Voucher),
    Deck(DeckVariant),
    Tag(Tag),
    BossBlind(BossBlind),
    Edition(Edition),
    Pack(PackCategory, PackSize, u8),
}

impl ItemId {
    /// Parses a save-file id into an `ItemId`. `e_base` parses to
    /// `Edition::Base`; `c_base` has no match.
    pub fn from_id(s: &str) -> Option<ItemId> {
        if let Some(pack) = parse_pack_id(s) {
            return Some(pack);
        }
        if let Some(j) = Jokers::from_id(s) {
            return Some(ItemId::Joker(j));
        }
        if let Some(v) = Voucher::from_id(s) {
            return Some(ItemId::Voucher(v));
        }
        if let Some(d) = DeckVariant::from_id(s) {
            return Some(ItemId::Deck(d));
        }
        if let Some(t) = Tag::from_id(s) {
            return Some(ItemId::Tag(t));
        }
        if let Some(b) = BossBlind::from_id(s) {
            return Some(ItemId::BossBlind(b));
        }
        if let Some(e) = Edition::from_id(s) {
            return Some(ItemId::Edition(e));
        }
        if let Some(c) = Consumable::from_id(s) {
            return Some(ItemId::Consumable(c));
        }
        None
    }
}

impl fmt::Display for ItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ItemId::Joker(j) => write!(f, "{}", j.name()),
            ItemId::Consumable(c) => write!(f, "{}", c.name()),
            ItemId::Voucher(v) => write!(f, "{}", v.name()),
            ItemId::Deck(d) => write!(f, "{}", d.name()),
            ItemId::Tag(t) => write!(f, "{}", t.name()),
            ItemId::BossBlind(b) => write!(f, "{}", b.name()),
            ItemId::Edition(e) => write!(f, "{e:?}"),
            ItemId::Pack(cat, size, i) => write!(f, "{cat:?} {size:?} Pack #{i}"),
        }
    }
}

/// Parses pack ids like `"p_arcana_jumbo_1"`.
fn parse_pack_id(s: &str) -> Option<ItemId> {
    let rest = s.strip_prefix("p_")?;
    let mut parts = rest.rsplitn(2, '_');
    let index: u8 = parts.next()?.parse().ok()?;
    let cat_size = parts.next()?;
    let (cat_str, size_str) = cat_size.rsplit_once('_')?;
    let category = PackCategory::from_id(cat_str)?;
    let size = PackSize::from_id(size_str)?;
    Some(ItemId::Pack(category, size, index))
}

#[cfg(test)]
mod tests {
    use super::*;
    use balatro_types::Jokers;
    use balatro_types::joker::ScaryFace;

    #[test]
    fn test_from_id_joker() {
        assert_eq!(
            ItemId::from_id("j_scary_face"),
            Some(ItemId::Joker(Jokers::ScaryFace(ScaryFace::default())))
        );
    }

    #[test]
    fn test_from_id_pack() {
        assert_eq!(
            ItemId::from_id("p_arcana_jumbo_1"),
            Some(ItemId::Pack(PackCategory::Arcana, PackSize::Jumbo, 1))
        );
        assert_eq!(
            ItemId::from_id("p_standard_normal_4"),
            Some(ItemId::Pack(PackCategory::Standard, PackSize::Normal, 4))
        );
    }

    #[test]
    fn test_from_id_skips_structural_defaults() {
        assert_eq!(ItemId::from_id("c_base"), None);
        assert_eq!(
            ItemId::from_id("e_base"),
            Some(ItemId::Edition(Edition::Base))
        );
    }

    #[test]
    fn test_from_id_unknown() {
        assert_eq!(ItemId::from_id("not_a_real_id"), None);
    }
}
