use crate::blind::BossBlind;
use crate::consumable::Consumable;
use crate::joker::Jokers;
use crate::planet::Planets;
use crate::spectral::Spectral;
use crate::tag::Tag;
use crate::tarot::Tarot;
use crate::voucher::Voucher;

/// Shared display-name accessor for enums looked up or locked by name.
pub trait Named {
    fn name(&self) -> &str;
}

impl Named for str {
    fn name(&self) -> &str {
        self
    }
}

impl Named for Jokers {
    fn name(&self) -> &str {
        Jokers::name(self)
    }
}

impl Named for Tarot {
    fn name(&self) -> &str {
        Tarot::name(self)
    }
}

impl Named for Planets {
    fn name(&self) -> &str {
        Planets::name(self)
    }
}

impl Named for Spectral {
    fn name(&self) -> &str {
        Spectral::name(self)
    }
}

impl Named for Voucher {
    fn name(&self) -> &str {
        Voucher::name(self)
    }
}

impl Named for Tag {
    fn name(&self) -> &str {
        Tag::name(self)
    }
}

impl Named for BossBlind {
    fn name(&self) -> &str {
        BossBlind::name(self)
    }
}

impl Named for Consumable {
    fn name(&self) -> &str {
        Consumable::name(self)
    }
}
