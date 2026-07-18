//! Port of Immolate's `Instance`/`Cache` (`TheSoul/include/instance.hpp`):
//! the per-node cache and lock table that make node-hash generation work,
//! plus the generic `randchoice`/`randweightedchoice` draw primitives.
//! Balatro-specific draw methods (next_joker, next_tarot, ...) live in
//! `draws.rs`; this file only knows about strings and locks.

use crate::rng::{LuaRandom, pseudohash, round13};
use std::collections::{HashMap, HashSet};

/// Mirrors Immolate's `InstParams`. `version` gates which pool variant a
/// draw uses (see `pools.rs`); `showman` disables the lock-triggered
/// resample entirely (Jokers::Showman's real effect).
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
    /// The real game's first pack of the run is always a Buffoon pack
    /// (`functions.hpp::nextPack`) — tracked here so `next_pack` only
    /// special-cases it once.
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

    pub fn lock(&mut self, item: &str) {
        self.locked.insert(item.to_string());
    }

    pub fn unlock(&mut self, item: &str) {
        self.locked.remove(item);
    }

    pub fn is_locked(&self, item: &str) -> bool {
        self.locked.contains(item)
    }

    pub fn is_voucher_active(&self, voucher: &str) -> bool {
        self.params.vouchers.iter().any(|v| v == voucher)
    }

    pub fn activate_voucher(&mut self, voucher: &str) {
        self.params.vouchers.push(voucher.to_string());
    }

    /// Per-ID cache whose stored value mutates on every access — this is
    /// what makes rerolling work without any explicit reroll counter:
    /// calling the same node ID again advances its cached value one step.
    fn get_node(&mut self, id: &str) -> f64 {
        if !self.nodes.contains_key(id) {
            let initial = pseudohash(&format!("{id}{}", self.seed));
            self.nodes.insert(id.to_string(), initial);
        }
        let entry = self.nodes.get_mut(id).expect("just inserted if absent");
        *entry = round13((*entry * 1.72431234 + 2.134453429141) % 1.0);
        (*entry + self.hashed_seed) / 2.0
    }

    pub fn random(&mut self, id: &str) -> f64 {
        let node = self.get_node(id);
        LuaRandom::new(node).random()
    }

    pub fn randint(&mut self, id: &str, min: i32, max: i32) -> i32 {
        let node = self.get_node(id);
        LuaRandom::new(node).randint(min, max)
    }

    /// Uniform pick with lock-triggered resample. Mirrors
    /// `Instance::randchoice` (`instance.hpp:63-76`) exactly, including an
    /// asymmetry preserved from the source: the resample loop's exit check
    /// ignores `showman` even though the initial pick's trigger doesn't.
    pub fn randchoice<'a>(&mut self, id: &str, items: &[&'a str]) -> &'a str {
        let node = self.get_node(id);
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

    /// Weighted pick. `items[0]` is a sentinel whose weight is the sum of
    /// every other entry's weight — never returned itself, only used to
    /// scale the poll (mirrors `PACKS[0]` in Immolate's `items.hpp`).
    pub fn randweightedchoice<'a>(&mut self, id: &str, items: &[(&'a str, f64)]) -> &'a str {
        let node = self.get_node(id);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_node_advances_on_repeated_access() {
        let mut inst = Instance::new("TESTSEED");
        let a = inst.random("cdt1");
        let b = inst.random("cdt1");
        // Same node ID, called twice: values differ because the cache entry
        // mutates on each access — this is the reroll mechanism.
        assert_ne!(a, b);
    }

    #[test]
    fn get_node_is_deterministic_across_instances() {
        let mut a = Instance::new("TESTSEED");
        let mut b = Instance::new("TESTSEED");
        assert_eq!(a.random("cdt1"), b.random("cdt1"));
    }

    #[test]
    fn randchoice_avoids_locked_items_by_default() {
        let mut inst = Instance::new("TESTSEED");
        let items = ["A", "B"];
        inst.lock("A");
        // With only one unlocked item in a 2-item pool, any resample must
        // eventually land on "B" (capped at 1000 tries) if "A" is ever hit.
        for ante in 0..50 {
            let id = format!("choice{ante}");
            let picked = inst.randchoice(&id, &items);
            assert_eq!(picked, "B");
        }
    }

    #[test]
    fn randchoice_allows_locked_items_with_showman() {
        let mut inst = Instance::new("TESTSEED");
        inst.params.showman = true;
        let items = ["A", "B"];
        inst.lock("A");
        // Showman bypasses the lock gate on the initial draw; "A" should be
        // reachable now (not asserting it's picked every time, just that the
        // lock no longer forces every draw to "B").
        let any_a = (0..50).any(|ante| {
            let id = format!("showman_choice{ante}");
            inst.randchoice(&id, &items) == "A"
        });
        assert!(any_a, "Showman should allow a locked item to be drawn");
    }

    #[test]
    fn randchoice_never_returns_retry_sentinel() {
        let mut inst = Instance::new("TESTSEED");
        let items = ["RETRY", "RETRY", "Real"];
        for ante in 0..50 {
            let id = format!("retry_choice{ante}");
            assert_eq!(inst.randchoice(&id, &items), "Real");
        }
    }

    #[test]
    fn randweightedchoice_never_returns_sentinel() {
        let mut inst = Instance::new("TESTSEED");
        // Sentinel weight (index 0) equals the sum of the real entries.
        let items = [("SENTINEL", 3.0), ("A", 1.0), ("B", 1.0), ("C", 1.0)];
        for ante in 0..200 {
            let id = format!("weighted{ante}");
            let picked = inst.randweightedchoice(&id, &items);
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
