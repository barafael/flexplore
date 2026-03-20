#!/usr/bin/env bash
# Renders each testdata/*/expected.tailwind.html to testdata/*/rendered_tailwind.png
# Wraps the fragment in a full HTML page with the Tailwind CDN, writes a temp file,
# then screenshots it with Playwright.
# Usage: bash scripts/render_tailwind.sh [case1 case2 ...]
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

echo "Will render ${#cases[@]} Tailwind test case(s)" >&2

# Clean up temp files on error/interrupt
_cleanup_tmp=""
trap '[ -n "$_cleanup_tmp" ] && rm -f "$_cleanup_tmp"' EXIT

for name in "${cases[@]}"; do
  tw_file="$TESTDATA_DIR/$name/expected.tailwind.html"
  out_file="$TESTDATA_DIR/$name/rendered_tailwind.png"
  tmp_file="$TESTDATA_DIR/$name/_tmp_tailwind.html"
  _cleanup_tmp="$tmp_file"

  if [ ! -f "$tw_file" ]; then
    echo "  SKIP: $tw_file not found" >&2
    continue
  fi

  # Wrap the fragment in a full page with Tailwind CDN
  {
    cat <<'HEADER'
<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<script src="https://cdn.tailwindcss.com"></script>
<style>html,body{margin:0;height:100%}body{display:flex;flex-direction:column;align-items:flex-start}</style>
</head>
<body>
HEADER
    cat "$tw_file"
    cat <<'FOOTER'
</body>
</html>
FOOTER
  } > "$tmp_file"

  win_path="$(cygpath -w "$tmp_file" 2>/dev/null || echo "$tmp_file")"
  file_url="file:///$(echo "$win_path" | sed 's|\\|/|g')"

  # Tailwind CDN needs network + time to process classes
  npx playwright screenshot \
    --browser chromium \
    --viewport-size="$VIEWPORT" \
    --wait-for-timeout=2000 \
    "$file_url" "$out_file" > /dev/null 2>&1

  rm -f "$tmp_file"
  _cleanup_tmp=""
  echo "  Saved: $name/rendered_tailwind.png" >&2
done

echo "All done!" >&2
