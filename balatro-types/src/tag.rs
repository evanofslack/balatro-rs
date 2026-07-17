use strum::EnumIter;

/// Tags are awarded for skipping a Blind and trigger an effect on entering
/// the next shop.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, EnumIter)]
pub enum Tag {
    Uncommon,
    Rare,
    Negative,
    Foil,
    Holographic,
    Polychrome,
    Investment,
    Voucher,
    Boss,
    Standard,
    Charm,
    Meteor,
    Buffoon,
    Handy,
    Garbage,
    Ethereal,
    Coupon,
    Double,
    Juggle,
    D6,
    TopUp,
    Speed,
    Orbital,
    Economy,
}

impl Tag {
    pub fn name(&self) -> &str {
        match self {
            Self::Uncommon => "Uncommon Tag",
            Self::Rare => "Rare Tag",
            Self::Negative => "Negative Tag",
            Self::Foil => "Foil Tag",
            Self::Holographic => "Holographic Tag",
            Self::Polychrome => "Polychrome Tag",
            Self::Investment => "Investment Tag",
            Self::Voucher => "Voucher Tag",
            Self::Boss => "Boss Tag",
            Self::Standard => "Standard Tag",
            Self::Charm => "Charm Tag",
            Self::Meteor => "Meteor Tag",
            Self::Buffoon => "Buffoon Tag",
            Self::Handy => "Handy Tag",
            Self::Garbage => "Garbage Tag",
            Self::Ethereal => "Ethereal Tag",
            Self::Coupon => "Coupon Tag",
            Self::Double => "Double Tag",
            Self::Juggle => "Juggle Tag",
            Self::D6 => "D6 Tag",
            Self::TopUp => "Top-up Tag",
            Self::Speed => "Speed Tag",
            Self::Orbital => "Orbital Tag",
            Self::Economy => "Economy Tag",
        }
    }

    /// Save-file id for this tag.
    pub fn id(&self) -> &'static str {
        match self {
            Self::Uncommon => "tag_uncommon",
            Self::Rare => "tag_rare",
            Self::Negative => "tag_negative",
            Self::Foil => "tag_foil",
            Self::Holographic => "tag_holo",
            Self::Polychrome => "tag_polychrome",
            Self::Investment => "tag_investment",
            Self::Voucher => "tag_voucher",
            Self::Boss => "tag_boss",
            Self::Standard => "tag_standard",
            Self::Charm => "tag_charm",
            Self::Meteor => "tag_meteor",
            Self::Buffoon => "tag_buffoon",
            Self::Handy => "tag_handy",
            Self::Garbage => "tag_garbage",
            Self::Ethereal => "tag_ethereal",
            Self::Coupon => "tag_coupon",
            Self::Double => "tag_double",
            Self::Juggle => "tag_juggle",
            Self::D6 => "tag_d_six",
            Self::TopUp => "tag_top_up",
            Self::Speed => "tag_skip",
            Self::Orbital => "tag_orbital",
            Self::Economy => "tag_economy",
        }
    }

    /// Parses a save-file id back into a `Tag`.
    pub fn from_id(s: &str) -> Option<Self> {
        match s {
            "tag_uncommon" => Some(Self::Uncommon),
            "tag_rare" => Some(Self::Rare),
            "tag_negative" => Some(Self::Negative),
            "tag_foil" => Some(Self::Foil),
            "tag_holo" => Some(Self::Holographic),
            "tag_polychrome" => Some(Self::Polychrome),
            "tag_investment" => Some(Self::Investment),
            "tag_voucher" => Some(Self::Voucher),
            "tag_boss" => Some(Self::Boss),
            "tag_standard" => Some(Self::Standard),
            "tag_charm" => Some(Self::Charm),
            "tag_meteor" => Some(Self::Meteor),
            "tag_buffoon" => Some(Self::Buffoon),
            "tag_handy" => Some(Self::Handy),
            "tag_garbage" => Some(Self::Garbage),
            "tag_ethereal" => Some(Self::Ethereal),
            "tag_coupon" => Some(Self::Coupon),
            "tag_double" => Some(Self::Double),
            "tag_juggle" => Some(Self::Juggle),
            "tag_d_six" => Some(Self::D6),
            "tag_top_up" => Some(Self::TopUp),
            "tag_skip" => Some(Self::Speed),
            "tag_orbital" => Some(Self::Orbital),
            "tag_economy" => Some(Self::Economy),
            _ => None,
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::Uncommon => "Next shop guarantees an Uncommon Joker",
            Self::Rare => "Next shop guarantees a Rare Joker",
            Self::Negative => "Next base edition Joker in shop is free and becomes Negative",
            Self::Foil => "Next base edition Joker in shop is free and becomes Foil",
            Self::Holographic => "Next base edition Joker in shop is free and becomes Holographic",
            Self::Polychrome => "Next base edition Joker in shop is free and becomes Polychrome",
            Self::Investment => {
                "Jokers, Consumables, and Booster Packs in next shop's initial stock are free"
            }
            Self::Voucher => "Next shop has an additional Voucher available",
            Self::Boss => "Rerolls the upcoming Boss Blind",
            Self::Standard => "Gives a free Mega Standard Pack",
            Self::Charm => "Gives a free Mega Arcana Pack",
            Self::Meteor => "Gives a free Mega Celestial Pack",
            Self::Buffoon => "Gives a free Mega Buffoon Pack",
            Self::Handy => "Gives $1 for each hand played so far this run",
            Self::Garbage => "Gives $1 for each unused discard so far this run",
            Self::Ethereal => "Gives a free Spectral Pack",
            Self::Coupon => "Jokers and Consumables in next shop's initial stock are free",
            Self::Double => "Doubles the next Tag applied (after skipping another Blind)",
            Self::Juggle => "+3 hand size for the next round only",
            Self::D6 => "Next shop's rerolls start at $0",
            Self::TopUp => "Spawns up to 2 Common Jokers with no stickers, filling available slots",
            Self::Speed => "Gives $5 for each Blind skipped so far, including this one",
            Self::Orbital => "Upgrades the poker hand type shown on the tag by 3 levels",
            Self::Economy => "Doubles your money, capped at +$40",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn test_tag_id_round_trip() {
        for t in Tag::iter() {
            assert_eq!(Tag::from_id(t.id()), Some(t));
        }
    }

    #[test]
    fn test_tag_count() {
        assert_eq!(Tag::iter().count(), 24);
    }

    #[test]
    fn test_tag_name() {
        assert_eq!(Tag::D6.name(), "D6 Tag");
    }
}
