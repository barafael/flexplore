#!/usr/bin/env bash
# Renders each testdata case via SwiftUI snapshot tests on macOS, then copies
# the resulting PNGs into testdata/*/rendered_swift.png.
# Usage: bash scripts/render_swift.sh [case1 case2 ...]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TESTDATA_DIR="$SCRIPT_DIR/../testdata"
SWIFT_DIR="$SCRIPT_DIR/swift_golden"
SNAPSHOTS_DIR="$SWIFT_DIR/Tests/SwiftGoldenTests/__Snapshots__/GoldenTests"

if ! command -v swift &>/dev/null; then
  echo "ERROR: swift not found in PATH (requires macOS with Xcode CLI tools)" >&2
  exit 1
fi

echo "=== Regenerating Swift view files ===" >&2
python3 "$SWIFT_DIR/tool/generate_cases.py" 2>&1

echo "=== Running Swift snapshot tests ===" >&2
cd "$SWIFT_DIR"
# Record mode always "fails" tests (by design) — ignore the exit code.
SWIFT_SNAPSHOT_RECORD=1 swift test 2>&1 || true

echo "=== Copying snapshots to testdata ===" >&2
if [ $# -gt 0 ]; then
  cases=("$@")
else
  cases=()
  for f in "$SNAPSHOTS_DIR"/test_*.1.png; do
    [ -f "$f" ] || continue
    base="$(basename "$f" .1.png)"
    cases+=("${base#test_}")
  done
fi

for name in "${cases[@]}"; do
  src="$SNAPSHOTS_DIR/test_${name}.1.png"
  dst="$TESTDATA_DIR/$name/rendered_swift.png"
  if [ -f "$src" ] && [ -d "$TESTDATA_DIR/$name" ]; then
    cp "$src" "$dst"
    echo "  Copied: $name/rendered_swift.png" >&2
  fi
done

echo "All done!" >&2
