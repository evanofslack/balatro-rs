//! Port of Balatro's real RNG primitives (`TheSoul`/Immolate `include/util.hpp`).
//! Pure math, no Balatro domain knowledge.
//!
//! Uses `std::f64::consts::{PI,E}` instead of the source's hand-typed
//! 18-20 digit literals — both round to the same nearest f64.

use std::f64::consts::{E, PI};

/// Balatro's own string hash, used to seed a fresh [`LuaRandom`] per decision.
pub fn pseudohash(s: &str) -> f64 {
    let bytes = s.as_bytes();
    let mut num: f64 = 1.0;
    for i in (1..=bytes.len()).rev() {
        let c = bytes[i - 1] as f64;
        num = (1.1239285023 / num * c * PI + PI * i as f64).fract();
    }
    num
}

const INV_PREC: f64 = 1e13;
const TWO_INV_PREC: f64 = 8192.0; // 2^13
const FIVE_INV_PREC: f64 = 1_220_703_125.0; // 5^13

/// `nextafter(x, 1.0)` — moves `x` one representable step toward `1.0`.
fn next_toward_one(x: f64) -> f64 {
    match x.partial_cmp(&1.0) {
        Some(std::cmp::Ordering::Less) => x.next_up(),
        Some(std::cmp::Ordering::Greater) => x.next_down(),
        _ => x,
    }
}

/// Rounds `x` to 13 significant decimal digits, stabilizing the per-node
/// cache against floating point drift across repeated updates.
pub fn round13(x: f64) -> f64 {
    let tentative = (x * INV_PREC).floor() / INV_PREC;
    let truncated = ((x * TWO_INV_PREC) % 1.0) * FIVE_INV_PREC;
    if tentative != x && tentative != next_toward_one(x) && (truncated % 1.0) >= 0.5 {
        return ((x * INV_PREC).floor() + 1.0) / INV_PREC;
    }
    tentative
}

/// Balatro's reimplementation of Lua 5.4's `math.random`: a seeded
/// xoshiro256-family generator, reseeded from scratch (via [`LuaRandom::new`])
/// for every single decision the real game makes.
pub struct LuaRandom {
    state: [u64; 4],
}

impl LuaRandom {
    pub fn new(seed: f64) -> Self {
        let mut d = seed;
        let mut r: u64 = 0x1109_0601;
        let mut state = [0u64; 4];
        for entry in state.iter_mut() {
            let m: u64 = 1u64 << (r & 255);
            r >>= 8;
            d = d * PI + E;
            let mut bits = d.to_bits();
            if bits < m {
                bits = bits.wrapping_add(m);
            }
            *entry = bits;
        }
        let mut rng = LuaRandom { state };
        for _ in 0..10 {
            rng.next_u64();
        }
        rng
    }

    fn next_u64(&mut self) -> u64 {
        let mut r: u64 = 0;

        let mut z = self.state[0];
        z = (((z << 31) ^ z) >> 45) ^ ((z & (u64::MAX << 1)) << 18);
        r ^= z;
        self.state[0] = z;

        let mut z = self.state[1];
        z = (((z << 19) ^ z) >> 30) ^ ((z & (u64::MAX << 6)) << 28);
        r ^= z;
        self.state[1] = z;

        let mut z = self.state[2];
        z = (((z << 24) ^ z) >> 48) ^ ((z & (u64::MAX << 9)) << 7);
        r ^= z;
        self.state[2] = z;

        let mut z = self.state[3];
        z = (((z << 21) ^ z) >> 39) ^ ((z & (u64::MAX << 17)) << 8);
        r ^= z;
        self.state[3] = z;

        r
    }

    fn rand_dbl_mem(&mut self) -> u64 {
        (self.next_u64() & 4_503_599_627_370_495) | 4_607_182_418_800_017_408
    }

    /// A pseudorandom double in `[0, 1)`.
    pub fn random(&mut self) -> f64 {
        f64::from_bits(self.rand_dbl_mem()) - 1.0
    }

    /// A pseudorandom integer in `[min, max]` inclusive.
    pub fn randint(&mut self, min: i32, max: i32) -> i32 {
        (self.random() * (max - min + 1) as f64) as i32 + min
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Hand-computed by running pseudohash("") through the definition:
    // the loop body never executes (empty string), so num stays 1.0.
    #[test]
    fn pseudohash_empty_string_is_identity() {
        assert_eq!(pseudohash(""), 1.0);
    }

    #[test]
    fn pseudohash_is_deterministic() {
        assert_eq!(pseudohash("ABCDEFGH"), pseudohash("ABCDEFGH"));
        assert_ne!(pseudohash("ABCDEFGH"), pseudohash("ABCDEFGI"));
    }

    #[test]
    fn pseudohash_stays_in_unit_interval() {
        // fract() always returns a value in (-1, 1); pseudohash's inputs here
        // never go negative in practice (verified against TheSoul separately),
        // but the definition itself only guarantees the fract() bound.
        for s in ["A", "SEED1234", "Joker1sho3", "boss"] {
            let h = pseudohash(s);
            assert!(h.is_finite());
            assert!((-1.0..1.0).contains(&h), "{s} -> {h}");
        }
    }

    #[test]
    fn round13_fixed_point_on_already_rounded_values() {
        // A value with far fewer than 13 significant digits should be
        // returned unchanged (tentative == x, short-circuits the branch).
        assert_eq!(round13(0.5), 0.5);
        assert_eq!(round13(0.0), 0.0);
    }

    #[test]
    fn lua_random_is_deterministic_given_same_seed() {
        let mut a = LuaRandom::new(0.5);
        let mut b = LuaRandom::new(0.5);
        for _ in 0..20 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn lua_random_random_is_in_unit_interval() {
        let mut rng = LuaRandom::new(pseudohash("SEED1234"));
        for _ in 0..1000 {
            let v = rng.random();
            assert!((0.0..1.0).contains(&v), "{v} out of range");
        }
    }

    #[test]
    fn lua_random_randint_respects_bounds() {
        let mut rng = LuaRandom::new(pseudohash("SEED1234"));
        for _ in 0..1000 {
            let v = rng.randint(0, 9);
            assert!((0..=9).contains(&v), "{v} out of range");
        }
    }

    #[test]
    fn lua_random_different_seeds_diverge() {
        let mut a = LuaRandom::new(pseudohash("seedA"));
        let mut b = LuaRandom::new(pseudohash("seedB"));
        // Overwhelmingly unlikely to collide on the very first draw.
        assert_ne!(a.random(), b.random());
    }
}
