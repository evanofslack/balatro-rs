use crate::planet::Planets;
use crate::tarot::Tarot;
#[cfg(feature = "python")]
use pyo3::pyclass;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", pyclass(eq))]
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Consumable {
    Planet(Planets),
    Tarot(Tarot),
    // Spectral(Spectral),   // future
}

impl Consumable {
    pub fn cost(&self) -> usize {
        match self {
            Self::Planet(p) => p.cost(),
            Self::Tarot(t) => t.cost(),
        }
    }

    pub fn name(&self) -> String {
        match self {
            Self::Planet(p) => p.name(),
            Self::Tarot(t) => t.name().to_string(),
        }
    }

    pub fn type_label(&self) -> &str {
        match self {
            Self::Planet(_) => "planet",
            Self::Tarot(_) => "tarot",
        }
    }

    pub fn description(&self) -> String {
        match self {
            Self::Planet(p) => p.desc(),
            Self::Tarot(t) => t.description().to_string(),
        }
    }

    pub fn sell_value(&self) -> usize {
        match self {
            Self::Planet(p) => p.sell_value(),
            Self::Tarot(t) => t.sell_value(),
        }
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
        use crate::tarot::Tarot;
        let c = Consumable::Tarot(Tarot::Fool);
        assert_eq!(c.sell_value(), 1);
    }
}
