use std::path::PathBuf;

use test_case::test_case;

use crate::codegen::{emit_bevy_code, emit_flutter, emit_html_css, emit_iced, emit_react, emit_swiftui, emit_tailwind};
use crate::config::NodeConfig;
use crate::fixtures::all_fixtures;

// ─── Snapshot infrastructure ─────────────────────────────────────────────────

fn testdata_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata")
}

fn run_snapshot(name: &str) {
    let fixtures = all_fixtures();
    let f = fixtures.iter().find(|f| f.name == name).unwrap_or_else(|| {
        panic!("unknown fixture: {name}");
    });

    let dir = testdata_dir().join(name);
    let input_path = dir.join("input.json");

    let targets: Vec<(&str, String)> = vec![
        ("expected.html", emit_html_css(&f.node, f.palette).unwrap()),
        ("expected.rs", emit_bevy_code(&f.node, f.palette).unwrap()),
        ("expected.jsx", emit_react(&f.node, f.palette).unwrap()),
        ("expected.tailwind.html", emit_tailwind(&f.node, f.palette).unwrap()),
        ("expected.swift", emit_swiftui(&f.node, f.palette).unwrap()),
        ("expected.dart", emit_flutter(&f.node, f.palette).unwrap()),
        ("expected.iced.rs", emit_iced(&f.node, f.palette).unwrap()),
    ];

    // Read input JSON back and re-generate to verify round-trip
    let json_src = std::fs::read_to_string(&input_path).unwrap_or_else(|e| {
        panic!("missing {}: {e} (run `cargo run -p update-snapshots` to generate)", input_path.display());
    });
    let from_json: NodeConfig = serde_json::from_str(&json_src).unwrap();

    let roundtrip_targets: Vec<(&str, String)> = vec![
        ("expected.html", emit_html_css(&from_json, f.palette).unwrap()),
        ("expected.rs", emit_bevy_code(&from_json, f.palette).unwrap()),
        ("expected.jsx", emit_react(&from_json, f.palette).unwrap()),
        ("expected.tailwind.html", emit_tailwind(&from_json, f.palette).unwrap()),
        ("expected.swift", emit_swiftui(&from_json, f.palette).unwrap()),
        ("expected.dart", emit_flutter(&from_json, f.palette).unwrap()),
        ("expected.iced.rs", emit_iced(&from_json, f.palette).unwrap()),
    ];

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
        let expected = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!("missing {}: {e} (run `cargo run -p update-snapshots` to generate)", path.display());
        });
        if *actual != expected {
            panic!(
                "[{name}] {filename} snapshot mismatch.\n\n--- expected ---\n{expected}\n--- actual ---\n{actual}"
            );
        }
    }
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
fn snapshot(name: &str) {
    run_snapshot(name);
}
