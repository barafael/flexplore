#!/usr/bin/env bash
# Re-generates all rendered images and the HTML overview.
# Usage: bash scripts/render_all.sh [case1 case2 ...]
#   No args = all cases. Pass case names to render a subset.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."

echo "=== Regenerating snapshot test data ===" >&2
cd "$ROOT_DIR"
FLEXPLORE_UPDATE=1 cargo test snapshot -- --test-threads=1 >&2

echo ""
echo "=== Rendering Bevy screenshots ===" >&2
cargo run --example render_testdata -- "$@" >&2

echo ""
echo "=== Rendering HTML screenshots ===" >&2
bash "$SCRIPT_DIR/render_html.sh" "$@" 2>&1

echo ""
echo "=== Rendering Tailwind screenshots ===" >&2
bash "$SCRIPT_DIR/render_tailwind.sh" "$@" 2>&1

echo ""
echo "=== Rendering Flutter screenshots ===" >&2
bash "$SCRIPT_DIR/render_flutter.sh" "$@" 2>&1

echo ""
echo "=== Rendering Swift screenshots ===" >&2
bash "$SCRIPT_DIR/render_swift.sh" "$@" 2>&1

echo ""
echo "=== Building overview page ===" >&2
python3 "$SCRIPT_DIR/build_overview.py" 2>&1

echo ""
echo "Done! Open testdata/overview.html to compare." >&2
