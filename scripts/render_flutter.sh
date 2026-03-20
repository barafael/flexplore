#!/usr/bin/env bash
# Renders each testdata case via Flutter golden tests, then copies the
# resulting PNGs into testdata/*/rendered_flutter.png.
# Usage: bash scripts/render_flutter.sh [case1 case2 ...]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TESTDATA_DIR="$SCRIPT_DIR/../testdata"
FLUTTER_DIR="$SCRIPT_DIR/flutter_golden"
GOLDENS_DIR="$FLUTTER_DIR/test/goldens"

# Add scoop-installed Flutter to PATH if available
if [ -d "$HOME/scoop/apps/flutter/current/bin" ]; then
  export PATH="$PATH:$HOME/scoop/apps/flutter/current/bin"
fi

if ! command -v flutter &>/dev/null; then
  echo "ERROR: flutter not found in PATH" >&2
  exit 1
fi

# Copy Roboto from Flutter SDK cache so golden tests render real text
FLUTTER_ROOT="$(dirname "$(dirname "$(command -v flutter)")")"
FONT_SRC="$FLUTTER_ROOT/bin/cache/artifacts/material_fonts/roboto-regular.ttf"
FONT_DST="$FLUTTER_DIR/fonts/Roboto-Regular.ttf"
if [ -f "$FONT_SRC" ] && [ ! -f "$FONT_DST" ]; then
  mkdir -p "$FLUTTER_DIR/fonts"
  cp "$FONT_SRC" "$FONT_DST"
  echo "  Copied Roboto font from Flutter SDK" >&2
fi

echo "=== Regenerating Flutter widget files ===" >&2
python3 "$FLUTTER_DIR/tool/generate_cases.py" 2>&1

echo "=== Running Flutter golden tests ===" >&2
cd "$FLUTTER_DIR"
flutter test --update-goldens test/golden_test.dart >&2

echo "=== Copying goldens to testdata ===" >&2
if [ $# -gt 0 ]; then
  cases=("$@")
else
  cases=()
  for f in "$GOLDENS_DIR"/*.png; do
    cases+=("$(basename "$f" .png)")
  done
fi

for name in "${cases[@]}"; do
  src="$GOLDENS_DIR/$name.png"
  dst="$TESTDATA_DIR/$name/rendered_flutter.png"
  if [ -f "$src" ] && [ -d "$TESTDATA_DIR/$name" ]; then
    cp "$src" "$dst"
    echo "  Copied: $name/rendered_flutter.png" >&2
  fi
done

echo "All done!" >&2
