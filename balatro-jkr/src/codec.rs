use crate::error::JkrError;
use crate::parser::parse;
use crate::printer::print;
use crate::value::LuaValue;

/// Default deflate compression level (0-10). Matches the JS reference's
/// `pako.deflateRaw` call, which doesn't affect decompressibility, only size.
const COMPRESSION_LEVEL: u8 = 6;

/// Decodes a `.jkr` file's raw bytes into a [`LuaValue`].
///
/// `.jkr` files are a raw DEFLATE stream (no zlib/gzip header or trailer,
/// matching `pako.inflateRaw`) of UTF-8 Lua source text of the form
/// `return {...}`.
pub fn decode(bytes: &[u8]) -> Result<LuaValue, JkrError> {
    let raw = miniz_oxide::inflate::decompress_to_vec(bytes)
        .map_err(|e| JkrError::Decompress(format!("{e:?}")))?;
    let text = String::from_utf8(raw)?;
    Ok(parse(&text)?)
}

/// Encodes a [`LuaValue`] back into `.jkr` bytes (raw DEFLATE of `return {...}` text).
pub fn encode(value: &LuaValue) -> Vec<u8> {
    let text = format!("return {}", print(value));
    miniz_oxide::deflate::compress_to_vec(text.as_bytes(), COMPRESSION_LEVEL)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::LuaKey;

    #[test]
    fn roundtrips_through_compression() {
        let value = LuaValue::Table(vec![
            (LuaKey::Str("dollars".into()), LuaValue::Num(4.0)),
            (LuaKey::Num(1), LuaValue::Str("j_joker".into())),
        ]);
        let bytes = encode(&value);
        let decoded = decode(&bytes).unwrap();
        assert_eq!(value, decoded);
    }

    #[test]
    fn decode_rejects_garbage_bytes() {
        assert!(decode(b"not a deflate stream").is_err());
    }
}
