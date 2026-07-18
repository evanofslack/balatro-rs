//! Per-node RNG cache and lock table, plus the generic draw primitives.
//! Balatro-specific draw methods (next_joker, next_tarot, ...) live in
//! `draws.rs`.

use crate::node_id::NodeId;
use crate::pool::Pool;
use crate::rng::{LuaRandom, pseudohash, round13};
use balatro_types::Named;
use std::collections::{HashMap, HashSet};

/// `version` gates which pool variant a draw uses (see `pools.rs`);
/// `showman` disables lock-triggered resample entirely.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct InstParams {
    pub showman: bool,
    pub version: i64,
    pub deck: String,
    pub vouchers: Vec<String>,
}

impl Default for InstParams {
    fn default() -> Self {
        InstParams {
            showman: false,
            version: 10106,
            deck: "Red Deck".to_string(),
            vouchers: Vec::new(),
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Instance {
    seed: String,
    hashed_seed: f64,
    nodes: HashMap<String, f64>,
    locked: HashSet<String>,
    /// The run's first pack is always Buffoon; tracked so `next_pack`
    /// only special-cases it once.
    pub(crate) generated_first_pack: bool,
    pub params: InstParams,
}

impl Instance {
    pub fn new(seed: &str) -> Self {
        Instance {
            seed: seed.to_string(),
            hashed_seed: pseudohash(seed),
            nodes: HashMap::new(),
            locked: HashSet::new(),
            generated_first_pack: false,
            params: InstParams::default(),
        }
    }

    pub fn lock(&mut self, item: &(impl Named + ?Sized)) {
        self.locked.insert(item.name().to_string());
    }

    pub fn unlock(&mut self, item: &(impl Named + ?Sized)) {
        self.locked.remove(item.name());
    }

    pub fn is_locked(&self, item: &(impl Named + ?Sized)) -> bool {
        self.locked.contains(item.name())
    }

    pub fn is_voucher_active(&self, voucher: &(impl Named + ?Sized)) -> bool {
        self.params.vouchers.iter().any(|v| v == voucher.name())
    }

    pub fn activate_voucher(&mut self, voucher: &(impl Named + ?Sized)) {
        self.params.vouchers.push(voucher.name().to_string());
    }

    /// Mutates its stored value on every access — this is the reroll
    /// mechanism, not an explicit counter.
    fn get_node(&mut self, id: &str) -> f64 {
        if !self.nodes.contains_key(id) {
            let initial = pseudohash(&format!("{id}{}", self.seed));
            self.nodes.insert(id.to_string(), initial);
        }
        let entry = self.nodes.get_mut(id).expect("just inserted if absent");
        *entry = round13((*entry * 1.72431234 + 2.134453429141) % 1.0);
        (*entry + self.hashed_seed) / 2.0
    }

    pub(crate) fn random(&mut self, id: NodeId) -> f64 {
        let node = self.get_node(&id.to_string());
        LuaRandom::new(node).random()
    }

    #[allow(dead_code)]
    pub(crate) fn randint(&mut self, id: NodeId, min: i32, max: i32) -> i32 {
        let node = self.get_node(&id.to_string());
        LuaRandom::new(node).randint(min, max)
    }

    /// Uniform pick with lock-triggered resample. Preserves one source
    /// asymmetry: the resample loop's exit check ignores `showman`.
    pub(crate) fn randchoice<'a>(&mut self, id: NodeId, items: &[&'a str]) -> &'a str {
        let id = id.to_string();
        let node = self.get_node(&id);
        let mut rng = LuaRandom::new(node);
        let item = items[rng.randint(0, items.len() as i32 - 1) as usize];

        if (!self.params.showman && self.is_locked(item)) || item == "RETRY" {
            let mut resample = 2u32;
            loop {
                let resample_id = format!("{id}_resample{resample}");
                let node = self.get_node(&resample_id);
                let mut rng = LuaRandom::new(node);
                let item = items[rng.randint(0, items.len() as i32 - 1) as usize];
                resample += 1;
                if (item != "RETRY" && !self.is_locked(item)) || resample > 1000 {
                    return item;
                }
            }
        }
        item
    }

    /// Weighted pick. `items[0]` is a sentinel weight (sum of the rest),
    /// never itself returned.
    pub(crate) fn randweightedchoice<'a>(
        &mut self,
        id: NodeId,
        items: &[(&'a str, f64)],
    ) -> &'a str {
        let node = self.get_node(&id.to_string());
        let mut rng = LuaRandom::new(node);
        let poll = rng.random() * items[0].1;
        let mut idx = 1usize;
        let mut weight = 0.0;
        while weight < poll {
            weight += items[idx].1;
            idx += 1;
        }
        items[idx - 1].0
    }

    /// [`randchoice`](Self::randchoice) plus resolution against `pool`.
    pub(crate) fn randchoice_typed<T>(&mut self, id: NodeId, pool: &Pool<T>) -> T {
        let name = self.randchoice(id, pool.names);
        pool.resolve(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_node_advances_on_repeated_access() {
        let mut inst = Instance::new("TESTSEED");
        let a = inst.random(NodeId::Custom("cdt1"));
        let b = inst.random(NodeId::Custom("cdt1"));
        assert_ne!(a, b);
    }

    #[test]
    fn get_node_is_deterministic_across_instances() {
        let mut a = Instance::new("TESTSEED");
        let mut b = Instance::new("TESTSEED");
        assert_eq!(
            a.random(NodeId::Custom("cdt1")),
            b.random(NodeId::Custom("cdt1"))
        );
    }

    #[test]
    fn randchoice_avoids_locked_items_by_default() {
        let mut inst = Instance::new("TESTSEED");
        let items = ["A", "B"];
        inst.lock("A");
        for ante in 0..50 {
            let id = format!("choice{ante}");
            let picked = inst.randchoice(NodeId::Custom(&id), &items);
            assert_eq!(picked, "B");
        }
    }

    #[test]
    fn randchoice_allows_locked_items_with_showman() {
        let mut inst = Instance::new("TESTSEED");
        inst.params.showman = true;
        let items = ["A", "B"];
        inst.lock("A");
        let any_a = (0..50).any(|ante| {
            let id = format!("showman_choice{ante}");
            inst.randchoice(NodeId::Custom(&id), &items) == "A"
        });
        assert!(any_a, "Showman should allow a locked item to be drawn");
    }

    #[test]
    fn randchoice_never_returns_retry_sentinel() {
        let mut inst = Instance::new("TESTSEED");
        let items = ["RETRY", "RETRY", "Real"];
        for ante in 0..50 {
            let id = format!("retry_choice{ante}");
            assert_eq!(inst.randchoice(NodeId::Custom(&id), &items), "Real");
        }
    }

    #[test]
    fn randweightedchoice_never_returns_sentinel() {
        let mut inst = Instance::new("TESTSEED");
        // Sentinel weight (index 0) equals the sum of the real entries.
        let items = [("SENTINEL", 3.0), ("A", 1.0), ("B", 1.0), ("C", 1.0)];
        for ante in 0..200 {
            let id = format!("weighted{ante}");
            let picked = inst.randweightedchoice(NodeId::Custom(&id), &items);
            assert_ne!(picked, "SENTINEL");
        }
    }

    #[test]
    fn lock_unlock_roundtrip() {
        let mut inst = Instance::new("TESTSEED");
        assert!(!inst.is_locked("X"));
        inst.lock("X");
        assert!(inst.is_locked("X"));
        inst.unlock("X");
        assert!(!inst.is_locked("X"));
    }
}
