#!/usr/bin/env bash
# Regenerate all golden files, render every backend, build the HTML overview.
# Usage: bash scripts/view_overview.sh [case1 case2 ...]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."
cd "$ROOT_DIR"

# Ensure cargo is on PATH
for d in "${CARGO_HOME:-$HOME/.cargo}/bin" "$HOME/.cargo/bin"; do
    [ -d "$d" ] && export PATH="$d:$PATH"
done

echo ">>> Regenerating golden files"
cargo run -p update-snapshots

echo ">>> Rendering Bevy screenshots"
cargo run --example render_testdata -- "$@"

echo ">>> Rendering HTML screenshots"
bash "$SCRIPT_DIR/render_html.sh" "$@"

echo ">>> Rendering Tailwind screenshots"
bash "$SCRIPT_DIR/render_tailwind.sh" "$@"

echo ">>> Rendering Flutter screenshots"
if command -v flutter &>/dev/null; then
    bash "$SCRIPT_DIR/render_flutter.sh" "$@"
else
    echo "  SKIP: flutter not found"
fi

echo ">>> Rendering Swift screenshots"
if command -v swift &>/dev/null; then
    bash "$SCRIPT_DIR/render_swift.sh" "$@"
else
    echo "  SKIP: swift not found (requires macOS)"
fi

echo ">>> Rendering Iced screenshots"
bash "$SCRIPT_DIR/render_iced.sh" "$@"

echo ">>> Building overview page"
python "$SCRIPT_DIR/build_overview.py"

echo ""
echo "Done! Open testdata/overview.html to view the comparison."
