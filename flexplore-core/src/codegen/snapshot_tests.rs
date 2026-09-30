use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use test_case::test_case;

use crate::codegen::{TARGETS, emit_all};
use crate::config::LayoutInput;
use crate::fixtures::all_fixtures;

// ─── Snapshot infrastructure ─────────────────────────────────────────────────

fn testdata_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("testdata")
}

fn run_snapshot(name: &str) -> Result<()> {
    let fixtures = all_fixtures();
    let f = fixtures
        .iter()
        .find(|f| f.name == name)
        .context(format!("unknown fixture: {name}"))?;

    let dir = testdata_dir().join(name);
    let input_path = dir.join("input.json");

    let targets = emit_all(&f.node, f.palette)?;

    // Read input JSON back and re-generate to verify round-trip
    let json_src = std::fs::read_to_string(&input_path).with_context(|| {
        format!(
            "missing {} (run `cargo run -p update-snapshots` to generate)",
            input_path.display()
        )
    })?;
    let from_json: LayoutInput = serde_json::from_str(&json_src)?;
    let roundtrip_targets = emit_all(&from_json.node, from_json.palette)?;

    // Verify JSON round-trip produces identical codegen
    for ((filename, actual), (_, from_json_out)) in targets.iter().zip(roundtrip_targets.iter()) {
        assert_eq!(
            actual, from_json_out,
            "[{name}] {filename} differs between in-memory node and JSON-deserialized node"
        );
    }

    // Compare against stored snapshots
    for (filename, actual) in &targets {
        let path = dir.join(filename);
        let expected = std::fs::read_to_string(&path).with_context(|| {
            format!(
                "missing {} (run `cargo run -p update-snapshots` to generate)",
                path.display()
            )
        })?;
        if *actual != expected {
            bail!(
                "[{name}] {filename} snapshot mismatch.\n\n--- expected ---\n{expected}\n--- actual ---\n{actual}"
            );
        }
    }

    Ok(())
}

// ─── One test per fixture ────────────────────────────────────────────────────

#[test_case("single_leaf" ; "single_leaf")]
#[test_case("two_children" ; "two_children")]
#[test_case("direction_column" ; "direction_column")]
#[test_case("direction_row_reverse" ; "direction_row_reverse")]
#[test_case("direction_column_reverse" ; "direction_column_reverse")]
#[test_case("justify_center" ; "justify_center")]
#[test_case("justify_space_between" ; "justify_space_between")]
#[test_case("justify_space_evenly" ; "justify_space_evenly")]
#[test_case("align_items_center" ; "align_items_center")]
#[test_case("align_items_stretch" ; "align_items_stretch")]
#[test_case("align_content_space_between" ; "align_content_space_between")]
#[test_case("wrap_nowrap" ; "wrap_nowrap")]
#[test_case("wrap_reverse" ; "wrap_reverse")]
#[test_case("hidden_child" ; "hidden_child")]
#[test_case("all_hidden" ; "all_hidden")]
#[test_case("ordered_children" ; "ordered_children")]
#[test_case("padding_margin" ; "padding_margin")]
#[test_case("min_max_sizes" ; "min_max_sizes")]
#[test_case("nested_mixed" ; "nested_mixed")]
#[test_case("deep_chain_3" ; "deep_chain_3")]
#[test_case("wide_flat_5" ; "wide_flat_5")]
#[test_case("grow_shrink" ; "grow_shrink")]
#[test_case("align_self_center" ; "align_self_center")]
#[test_case("flex_basis_percent" ; "flex_basis_percent")]
#[test_case("gaps_mixed" ; "gaps_mixed")]
#[test_case("vw_vh_sizes" ; "vw_vh_sizes")]
#[test_case("dark2_palette" ; "dark2_palette")]
#[test_case("tpl_holy_grail" ; "tpl_holy_grail")]
#[test_case("tpl_sidebar_content" ; "tpl_sidebar_content")]
#[test_case("tpl_card_grid" ; "tpl_card_grid")]
#[test_case("tpl_nav_bar" ; "tpl_nav_bar")]
#[test_case("tpl_grid_dashboard" ; "tpl_grid_dashboard")]
#[test_case("tpl_grid_gallery" ; "tpl_grid_gallery")]
#[test_case("grid_3col" ; "grid_3col")]
#[test_case("grid_span" ; "grid_span")]
#[test_case("grid_auto_flow_column" ; "grid_auto_flow_column")]
fn snapshot(name: &str) -> Result<()> {
    run_snapshot(name)
}

// ─── Every fixture, every target ─────────────────────────────────────────────

/// Safety net for the hand-written list above: a fixture added to
/// `all_fixtures()` without a `test_case` is still checked here, and every
/// target in `TARGETS` must have a stored snapshot for it.
#[test]
fn every_fixture_has_a_snapshot_for_every_target() {
    let dir = testdata_dir();
    let mut failures = Vec::new();
    for f in all_fixtures() {
        for (filename, _) in TARGETS {
            let path = dir.join(&f.name).join(filename);
            if !path.is_file() {
                failures.push(format!("missing snapshot {}", path.display()));
            }
        }
        if let Err(e) = run_snapshot(&f.name) {
            failures.push(format!("{}: {e:#}", f.name));
        }
    }
    assert!(
        failures.is_empty(),
        "{} fixture(s) failed (run `cargo run -p update-snapshots` after intentional changes):\n{}",
        failures.len(),
        failures.join("\n")
    );
}
