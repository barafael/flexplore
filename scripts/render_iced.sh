#!/usr/bin/env bash
# Renders each testdata case via the Iced golden renderer.
# The binary reads input.json files and saves rendered_iced.png directly.
# Usage: bash scripts/render_iced.sh [case1 case2 ...]
set -euo pipefail

# Ensure cargo is on PATH
for d in "${CARGO_HOME:-$HOME/.cargo}/bin" "$HOME/.cargo/bin"; do
    [ -d "$d" ] && export PATH="$d:$PATH"
done

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TESTDATA_DIR="$SCRIPT_DIR/../testdata"
ICED_DIR="$SCRIPT_DIR/iced_golden"

echo "=== Building Iced golden renderer ===" >&2
cargo build --release --manifest-path "$ICED_DIR/Cargo.toml" >&2

echo "=== Rendering Iced screenshots ===" >&2
"$ICED_DIR/target/release/iced-golden" "$TESTDATA_DIR" "$@" 2>&1

echo "All done!" >&2
