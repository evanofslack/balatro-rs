//! Codec for Balatro's `.jkr` save/profile/meta file format: a raw-DEFLATE
//! compressed Lua table literal. This crate only knows the generic container
//! format ([`LuaValue`]/[`LuaKey`]) — it has no Balatro game vocabulary
//! (no jokers, cards, editions, etc.). Semantic interpretation of the parsed
//! data belongs in a separate, higher-level crate.

mod codec;
mod error;
mod lexer;
mod parser;
mod printer;
mod value;

pub use codec::{decode, encode};
pub use error::{JkrError, LuaError};
pub use parser::parse;
pub use printer::{print, print_pretty};
pub use value::{LuaKey, LuaValue};
