//! Byte-accurate port of Balatro's real seed/RNG algorithm, reverse
//! engineered and reference-implemented by `TheSoul`/Immolate
//! (<https://github.com/SpectralPack/TheSoul>). Given the same seed string,
//! this crate and the real Balatro client should produce identical
//! shops/packs/tags/vouchers/bosses.
//!
//! See `Instance` for the entry point.

mod draws;
mod instance;
mod pools;
mod resolve;
mod rng;

pub use draws::{ShopItem, pack_info, voucher_upgrade};
pub use instance::{InstParams, Instance};
pub use rng::{LuaRandom, pseudohash, round13};
