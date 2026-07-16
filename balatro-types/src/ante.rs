use std::convert::TryFrom;

/// All ante levels.
// Goes above 8 for endless mode.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Copy)]
pub enum Ante {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
}

impl Ante {
    // Base chip requirement.
    // Eventually this depends on deck and stake.
    pub fn base(&self) -> usize {
        match self {
            Self::Zero => 100,
            Self::One => 300,
            Self::Two => 800,
            Self::Three => 2000,
            Self::Four => 5000,
            Self::Five => 11000,
            Self::Six => 20000,
            Self::Seven => 35000,
            Self::Eight => 50000,
        }
    }
    pub fn next(&self, max: Ante) -> Option<Self> {
        if *self == max {
            return None;
        }
        match self {
            Self::Zero => Some(Self::One),
            Self::One => Some(Self::Two),
            Self::Two => Some(Self::Three),
            Self::Three => Some(Self::Four),
            Self::Four => Some(Self::Five),
            Self::Five => Some(Self::Six),
            Self::Six => Some(Self::Seven),
            Self::Seven => Some(Self::Eight),
            Self::Eight => None,
        }
    }
}

impl TryFrom<usize> for Ante {
    type Error = ();

    fn try_from(i: usize) -> Result<Self, Self::Error> {
        match i {
            0 => Ok(Self::Zero),
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            3 => Ok(Self::Three),
            4 => Ok(Self::Four),
            5 => Ok(Self::Five),
            6 => Ok(Self::Six),
            7 => Ok(Self::Seven),
            8 => Ok(Self::Eight),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ante_base() {
        assert_eq!(Ante::Zero.base(), 100);
        assert_eq!(Ante::Eight.base(), 50000);
    }

    #[test]
    fn test_ante_next_stops_at_max() {
        assert_eq!(Ante::Zero.next(Ante::Eight), Some(Ante::One));
        assert_eq!(Ante::Eight.next(Ante::Eight), None);
    }
}
