#!/usr/bin/env bash
# Open the replay library in balatro-spectator.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
exec cargo run -p balatro-spectator -- --library "$@"
