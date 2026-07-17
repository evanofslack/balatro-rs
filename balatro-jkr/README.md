# balatro-jkr

Low-level codec for Balatro's `.jkr` save/profile/meta file format, written in Rust.

## Overview

Balatro stores `meta.jkr`, `profile.jkr`, and `save.jkr` as a raw-DEFLATE-compressed
Lua table literal (`return {...}`). This library parses that format into a generic
AST and back, with no Balatro game vocabulary attached (no `Joker`, `Card`,
`Rarity`, etc). It only knows tables, strings, numbers, booleans, and nil, in
whatever shape the Lua literal contains.

The goal is to be the reusable, primative building block for tools that do
understand the game's data (an existing typed engine, a save editor, a stats
dashboard) without each of them re-implementing the container format.

## Example

```rust
use balatro_jkr::{decode, encode, LuaKey, LuaValue};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = std::fs::read("save.jkr")?;
    let value = decode(&bytes)?;

    if let LuaValue::Table(entries) = &value {
        for (key, _) in entries {
            if let LuaKey::Str(name) = key {
                println!("{name}");
            }
        }
    }

    std::fs::write("save.jkr", encode(&value))?;
    Ok(())
}
```

`parse`/`print` do the same conversion one layer down, working directly on Lua
source text instead of compressed bytes, useful for tests or inspecting a
decompressed file by hand.

## CLI

A `jkr` binary is included for parsing and printing `.jkr` files from the terminal:

```
cargo run --bin jkr -- meta.jkr profile.jkr
```

Each file is decoded and printed as indented Lua source, e.g.:

```
{
  ["high_scores"] = {
    ["furthest_ante"] = {
      ["label"] = "Highest Ante",
      ["amt"] = 13,
    },
    ...
  },
  ...
}
```

Errors on one file (bad path, corrupt data) don't stop the rest — the binary exits non-zero if any file failed.

## Features

- [x] raw-DEFLATE (de)compression framing (`pako.deflateRaw`/`inflateRaw`-compatible, no zlib/gzip header)
- [x] parsing and printing the Lua table-literal grammar Balatro uses (nested tables, string/numeric keys, strings, numbers, booleans, nil)
- [x] preserves source field order and the string-vs-numeric key distinction losslessly

The following are intentionally not part of this crate:

- [ ] typed, game-specific schema (mapping `"j_joker"` to an actual joker type, etc.) — left to a higher-level crate built on top of this one
