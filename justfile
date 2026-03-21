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

# Regenerate golden files (input.json + expected.* for all fixtures)
update-snapshots:
    cargo run -p update-snapshots

# Run snapshot tests (verify golden files match codegen output)
[env('FLEXPLORE_UPDATE' , '1')]
snapshots: update-snapshots
    cargo test snapshot -- --test-threads=1
