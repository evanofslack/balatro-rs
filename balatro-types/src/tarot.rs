#[cfg(feature = "python")]
use pyo3::pyclass;
use strum::{EnumIter, EnumString};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, EnumIter, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Tarot {
    Fool,
    Magician,
    HighPriestess,
    Empress,
    Emperor,
    Hierophant,
    Lovers,
    Chariot,
    Justice,
    Hermit,
    WheelOfFortune,
    Strength,
    HangedMan,
    Death,
    Temperance,
    Devil,
    Tower,
    Star,
    Moon,
    Sun,
    Judgement,
    World,
}

impl Tarot {
    pub fn name(&self) -> &str {
        match self {
            Self::Fool => "The Fool",
            Self::Magician => "The Magician",
            Self::HighPriestess => "The High Priestess",
            Self::Empress => "The Empress",
            Self::Emperor => "The Emperor",
            Self::Hierophant => "The Hierophant",
            Self::Lovers => "The Lovers",
            Self::Chariot => "The Chariot",
            Self::Justice => "Justice",
            Self::Hermit => "The Hermit",
            Self::WheelOfFortune => "The Wheel of Fortune",
            Self::Strength => "Strength",
            Self::HangedMan => "The Hanged Man",
            Self::Death => "Death",
            Self::Temperance => "Temperance",
            Self::Devil => "The Devil",
            Self::Tower => "The Tower",
            Self::Star => "The Star",
            Self::Moon => "The Moon",
            Self::Sun => "The Sun",
            Self::Judgement => "Judgement",
            Self::World => "The World",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::Fool => "Creates a copy of the last Tarot used",
            Self::Magician => "Enhances up to 2 selected cards into Lucky Cards",
            Self::HighPriestess => "Gives 2 random Planet cards",
            Self::Empress => "Enhances up to 2 selected cards into Mult Cards",
            Self::Emperor => "Gives 2 random Tarot cards",
            Self::Hierophant => "Enhances up to 2 selected cards into Bonus Cards",
            Self::Lovers => "Enhances 1 selected card into a Wild Card",
            Self::Chariot => "Enhances 1 selected card into a Steel Card",
            Self::Justice => "Enhances 1 selected card into a Glass Card",
            Self::Hermit => "Doubles your money (up to $20)",
            Self::WheelOfFortune => "Adds a Foil, Holo or Polychrome to 1 random Joker",
            Self::Strength => "Increases the rank of up to 2 selected cards",
            Self::HangedMan => "Destroys up to 2 selected cards",
            Self::Death => "Converts the left selected card into the right",
            Self::Temperance => "Gives $1 per Joker sell value (max $50)",
            Self::Devil => "Enhances 1 selected card into a Gold Card",
            Self::Tower => "Enhances 1 selected card into a Stone Card",
            Self::Star => "Converts up to 3 selected cards to Diamonds",
            Self::Moon => "Converts up to 3 selected cards to Clubs",
            Self::Sun => "Converts up to 3 selected cards to Hearts",
            Self::Judgement => "Creates a random Joker card",
            Self::World => "Converts up to 3 selected cards to Spades",
        }
    }

    pub fn cost(&self) -> usize {
        3
    }

    pub fn sell_value(&self) -> usize {
        1
    }

    pub fn min_targets(&self) -> usize {
        match self {
            Self::Fool
            | Self::HighPriestess
            | Self::Emperor
            | Self::Hermit
            | Self::WheelOfFortune
            | Self::Temperance
            | Self::Judgement => 0,
            Self::Lovers
            | Self::Chariot
            | Self::Justice
            | Self::Devil
            | Self::Tower
            | Self::Magician
            | Self::Empress
            | Self::Hierophant
            | Self::Strength
            | Self::HangedMan
            | Self::Star
            | Self::Moon
            | Self::Sun
            | Self::World => 1,
            Self::Death => 2,
        }
    }

    pub fn max_targets(&self) -> usize {
        match self {
            Self::Fool
            | Self::HighPriestess
            | Self::Emperor
            | Self::Hermit
            | Self::WheelOfFortune
            | Self::Temperance
            | Self::Judgement => 0,
            Self::Lovers | Self::Chariot | Self::Justice | Self::Devil | Self::Tower => 1,
            Self::Magician
            | Self::Empress
            | Self::Hierophant
            | Self::Strength
            | Self::HangedMan
            | Self::Death => 2,
            Self::Star | Self::Moon | Self::Sun | Self::World => 3,
        }
    }

    pub fn requires_targets(&self) -> bool {
        self.min_targets() > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn test_tarot_count() {
        assert_eq!(Tarot::iter().count(), 22);
    }

    #[test]
    fn test_tarot_cost_and_sell_value() {
        assert_eq!(Tarot::Fool.cost(), 3);
        assert_eq!(Tarot::Fool.sell_value(), 1);
    }

    #[test]
    fn test_tarot_targets() {
        assert_eq!(Tarot::Fool.min_targets(), 0);
        assert!(!Tarot::Fool.requires_targets());
        assert_eq!(Tarot::Death.min_targets(), 2);
        assert_eq!(Tarot::Death.max_targets(), 2);
        assert!(Tarot::Death.requires_targets());
    }

    #[test]
    fn test_tarot_from_str_round_trip() {
        for tarot in Tarot::iter() {
            assert_eq!(format!("{tarot:?}").parse::<Tarot>(), Ok(tarot));
        }
    }

    #[test]
    fn test_tarot_from_str_case_insensitive() {
        assert_eq!("highpriestess".parse::<Tarot>(), Ok(Tarot::HighPriestess));
        assert_eq!("HIGHPRIESTESS".parse::<Tarot>(), Ok(Tarot::HighPriestess));
    }

    #[test]
    fn test_tarot_from_str_invalid() {
        assert!("NotATarot".parse::<Tarot>().is_err());
    }
}
