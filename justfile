# Flexplain task runner

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
snapshots:
    FLEXPLORE_UPDATE=1 cargo test snapshot -- --test-threads=1

# Render Bevy screenshots (pass case names to filter)
render-bevy *cases:
    cargo run --example render_testdata -- {{cases}}

# Render HTML screenshots via Playwright (pass case names to filter)
render-html *cases:
    bash scripts/render_html.sh {{cases}}

# Render Tailwind screenshots via Playwright (pass case names to filter)
render-tailwind *cases:
    bash scripts/render_tailwind.sh {{cases}}

# Build the overview comparison page
build-overview:
    python scripts/build_overview.py

# Render everything: snapshots, Bevy, HTML, Tailwind, then build overview
render-all *cases:
    bash scripts/render_all.sh {{cases}}
