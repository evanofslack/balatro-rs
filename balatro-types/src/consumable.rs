use crate::planet::Planets;
use crate::spectral::Spectral;
use crate::tarot::Tarot;
#[cfg(feature = "python")]
use pyo3::pyclass;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Consumable {
    Planet(Planets),
    Tarot(Tarot),
    Spectral(Spectral),
}

impl Consumable {
    pub fn cost(&self) -> usize {
        match self {
            Self::Planet(p) => p.cost(),
            Self::Tarot(t) => t.cost(),
            Self::Spectral(s) => s.cost(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Planet(p) => p.name(),
            Self::Tarot(t) => t.name(),
            Self::Spectral(s) => s.name(),
        }
    }

    pub fn type_label(&self) -> &str {
        match self {
            Self::Planet(_) => "planet",
            Self::Tarot(_) => "tarot",
            Self::Spectral(_) => "spectral",
        }
    }

    pub fn description(&self) -> String {
        match self {
            Self::Planet(p) => p.desc(),
            Self::Tarot(t) => t.description().to_string(),
            Self::Spectral(s) => s.description().to_string(),
        }
    }

    pub fn sell_value(&self) -> usize {
        match self {
            Self::Planet(p) => p.sell_value(),
            Self::Tarot(t) => t.sell_value(),
            Self::Spectral(s) => s.sell_value(),
        }
    }

    /// Save-file id for this consumable.
    pub fn id(&self) -> &'static str {
        match self {
            Self::Planet(p) => p.id(),
            Self::Tarot(t) => t.id(),
            Self::Spectral(s) => s.id(),
        }
    }

    /// Parses a save-file id into a `Consumable`.
    pub fn from_id(s: &str) -> Option<Self> {
        Planets::from_id(s)
            .map(Self::Planet)
            .or_else(|| Tarot::from_id(s).map(Self::Tarot))
            .or_else(|| Spectral::from_id(s).map(Self::Spectral))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consumable_planet_cost() {
        let c = Consumable::Planet(Planets::Mercury);
        assert_eq!(c.cost(), 3);
    }

    #[test]
    fn test_consumable_id_round_trip() {
        let cases = [
            Consumable::Planet(Planets::Saturn),
            Consumable::Tarot(Tarot::Hierophant),
            Consumable::Spectral(Spectral::Familiar),
        ];
        for c in cases {
            assert_eq!(Consumable::from_id(c.id()), Some(c));
        }
        assert_eq!(Consumable::Planet(Planets::Saturn).id(), "c_saturn");
        assert_eq!(Consumable::from_id("not_a_real_id"), None);
    }

    #[test]
    fn test_consumable_planet_name() {
        let c = Consumable::Planet(Planets::Mercury);
        assert_eq!(c.name(), "Mercury");
    }

    #[test]
    fn test_consumable_planet_sell_value() {
        let c = Consumable::Planet(Planets::Mercury);
        assert_eq!(c.sell_value(), 1);
    }

    #[test]
    fn test_consumable_tarot_sell_value() {
        let c = Consumable::Tarot(Tarot::Fool);
        assert_eq!(c.sell_value(), 1);
    }

    #[test]
    fn test_consumable_spectral() {
        let c = Consumable::Spectral(Spectral::Familiar);
        assert_eq!(c.type_label(), "spectral");
        assert_eq!(c.name(), "Familiar");
        assert_eq!(c.cost(), 4);
    }
}
