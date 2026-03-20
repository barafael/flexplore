# Flexplain task runner

set windows-powershell := true

# List available recipes
default:
    @just --list

# Run all tests
test *args:
    cargo test {{args}}

# Run clippy lints
clippy:
    cargo clippy --all-targets

# Build the project
build:
    cargo build

# Update snapshot test data
[env('FLEXPLORE_UPDATE' , '1')]
snapshots:
    cargo test snapshot -- --test-threads=1

# Render Bevy screenshots (pass case names to filter)
render-bevy *cases:
    cargo run --example render_testdata -- {{cases}}

# Render HTML screenshots via Playwright (pass case names to filter)
render-html *cases:
    bash scripts/render_html.sh {{cases}}

# Render Tailwind screenshots via Playwright (pass case names to filter)
render-tailwind *cases:
    bash scripts/render_tailwind.sh {{cases}}

# Render Flutter screenshots via golden tests (pass case names to filter)
render-flutter *cases:
    bash scripts/render_flutter.sh {{cases}}

# Render SwiftUI screenshots via snapshot tests (requires macOS)
render-swift *cases:
    bash scripts/render_swift.sh {{cases}}

# Build the overview comparison page
build-overview:
    python scripts/build_overview.py

# Open the overview comparison page in the default browser
view-overview:
    {{ if os() == "windows" { "Start-Process testdata/overview.html" } else if os() == "macos" { "open testdata/overview.html" } else { "xdg-open testdata/overview.html" } }}

# Render everything: snapshots, Bevy, HTML, Tailwind, Flutter, then build overview
render-all *cases:
    bash scripts/render_all.sh {{cases}}
