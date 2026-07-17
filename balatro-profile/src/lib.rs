//! Typed Balatro save data, bridging `balatro-jkr`'s generic `LuaValue`
//! into `balatro-types` vocabulary. Two independent outputs:
//! [`Profile`] (`meta.jkr` + `profile.jkr` — unlocks, stats, career
//! progress) and `SaveSnapshot` (`save.jkr` — an in-progress run).

mod error;
mod fmt_num;
mod item_id;
mod lua_ext;
mod profile;
mod save_snapshot;

pub use error::ProfileError;
pub use item_id::ItemId;
pub use profile::{Profile, ProfileSummary};
pub use save_snapshot::{SaveSnapshot, SaveSnapshotSummary};
