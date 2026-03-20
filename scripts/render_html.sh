#!/usr/bin/env bash
# Renders each testdata/*/expected.html to testdata/*/rendered_html.png
# Usage: bash scripts/render_html.sh [case1 case2 ...]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TESTDATA_DIR="$SCRIPT_DIR/../testdata"
VIEWPORT="400,300"

if [ $# -gt 0 ]; then
  cases=("$@")
else
  cases=()
  for dir in "$TESTDATA_DIR"/*/; do
    cases+=("$(basename "$dir")")
  done
fi

echo "Will render ${#cases[@]} HTML test case(s)" >&2

for name in "${cases[@]}"; do
  html_file="$TESTDATA_DIR/$name/expected.html"
  out_file="$TESTDATA_DIR/$name/rendered_html.png"

  if [ ! -f "$html_file" ]; then
    echo "  SKIP: $html_file not found" >&2
    continue
  fi

  # Convert to Windows-style file:// URL for Playwright on Windows
  win_path="$(cygpath -w "$html_file" 2>/dev/null || echo "$html_file")"
  # Normalize to forward slashes for URL
  file_url="file:///$(echo "$win_path" | sed 's|\\|/|g')"

  npx playwright screenshot \
    --browser chromium \
    --viewport-size="$VIEWPORT" \
    "$file_url" "$out_file" > /dev/null 2>&1

  echo "  Saved: $name/rendered_html.png" >&2
done

echo "All done!" >&2
