use std::fs;
use std::path::{Path, PathBuf};

fn fixture_files(dir: &str, ext: &str) -> Vec<PathBuf> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(dir);
    let mut files: Vec<_> = fs::read_dir(&path)
        .unwrap_or_else(|e| panic!("missing fixture dir {}: {e}", path.display()))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some(ext))
        .collect();
    files.sort();
    files
}

/// Hand-written Lua snippets, one per grammar edge case. Parsing them and
/// re-parsing their printed form must produce the same `LuaValue`.
#[test]
fn synthetic_fixtures_roundtrip() {
    let files = fixture_files("tests/fixtures/synthetic", "lua");
    assert!(!files.is_empty(), "expected at least one synthetic fixture");

    for path in files {
        let text = fs::read_to_string(&path).unwrap();
        let parsed = balatro_jkr::parse(&text)
            .unwrap_or_else(|e| panic!("{}: failed to parse: {e}", path.display()));
        let printed = balatro_jkr::print(&parsed);
        let reparsed = balatro_jkr::parse(&printed)
            .unwrap_or_else(|e| panic!("{}: failed to reparse printed form: {e}", path.display()));
        assert_eq!(
            parsed,
            reparsed,
            "{}: print/parse round trip mismatch",
            path.display()
        );
    }
}

/// Real, private `.jkr` files (see tests/fixtures/real/README.md). A no-op
/// until some are actually present on disk.
#[test]
fn real_fixtures_decode_and_roundtrip() {
    let files = fixture_files("tests/fixtures/real", "jkr");

    for path in files {
        let bytes = fs::read(&path).unwrap();
        let decoded = balatro_jkr::decode(&bytes)
            .unwrap_or_else(|e| panic!("{}: failed to decode: {e}", path.display()));
        let re_decoded = balatro_jkr::decode(&balatro_jkr::encode(&decoded)).unwrap_or_else(|e| {
            panic!(
                "{}: failed to decode after re-encoding: {e}",
                path.display()
            )
        });
        assert_eq!(
            decoded,
            re_decoded,
            "{}: decode/encode round trip mismatch",
            path.display()
        );
    }
}
