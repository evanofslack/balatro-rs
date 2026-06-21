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
}
