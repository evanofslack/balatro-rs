Drop real, private `.jkr` files here (exported `meta.jkr`, `profile.jkr`, `save.jkr`
from an actual Balatro install) to exercise `tests/roundtrip.rs` against real data.

This directory is empty on a fresh checkout — `real_fixtures_decode_and_roundtrip`
in `tests/roundtrip.rs` is a no-op until files are added.
