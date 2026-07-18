//! Pairs a raw draw-order pool with the resolver that turns a drawn name
//! into its typed `balatro_types` value.

pub(crate) struct Pool<T: 'static> {
    pub(crate) names: &'static [&'static str],
    resolver: fn(&str) -> Option<T>,
}

impl<T> Pool<T> {
    pub(crate) const fn new(names: &'static [&'static str], resolver: fn(&str) -> Option<T>) -> Self {
        Pool { names, resolver }
    }

    pub(crate) fn resolve(&self, name: &str) -> T {
        (self.resolver)(name)
            .unwrap_or_else(|| panic!("pool name {name:?} has no matching balatro_types value"))
    }

    #[cfg(test)]
    pub(crate) fn unresolved(&self) -> Vec<&'static str> {
        self.names
            .iter()
            .copied()
            .filter(|&n| n != "RETRY" && (self.resolver)(n).is_none())
            .collect()
    }
}
