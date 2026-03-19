use std::path::PathBuf;

use bevy::prelude::*;
use test_case::test_case;

use crate::codegen::{emit_bevy_code, emit_flutter, emit_html_css, emit_react, emit_swiftui, emit_tailwind};
use crate::config::{ColorPalette, NodeConfig, ValueConfig};
use crate::templates;

// ─── Fixture builders ────────────────────────────────────────────────────────

fn fixture(name: &str, node: NodeConfig, palette: ColorPalette) -> Fixture {
    Fixture { name: name.into(), node, palette }
}

struct Fixture {
    name: String,
    node: NodeConfig,
    palette: ColorPalette,
}

fn all_fixtures() -> Vec<Fixture> {
    vec![
        // Basic shapes
        fixture("single_leaf", {
            let mut r = NodeConfig::new_container("root");
            r.children = vec![NodeConfig::new_leaf("only", 100.0, 60.0)];
            r
        }, ColorPalette::Pastel1),
        fixture("two_children", {
            let mut r = NodeConfig::new_container("root");
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 80.0),
                NodeConfig::new_leaf("B", 120.0, 100.0),
            ];
            r
        }, ColorPalette::Pastel1),

        // Flex-direction variants
        fixture("direction_column", {
            let mut r = NodeConfig::new_container("root");
            r.flex_direction = FlexDirection::Column;
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 80.0),
                NodeConfig::new_leaf("B", 80.0, 80.0),
            ];
            r
        }, ColorPalette::Pastel1),
        fixture("direction_row_reverse", {
            let mut r = NodeConfig::new_container("root");
            r.flex_direction = FlexDirection::RowReverse;
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 80.0),
                NodeConfig::new_leaf("B", 80.0, 80.0),
            ];
            r
        }, ColorPalette::Pastel1),
        fixture("direction_column_reverse", {
            let mut r = NodeConfig::new_container("root");
            r.flex_direction = FlexDirection::ColumnReverse;
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 80.0),
                NodeConfig::new_leaf("B", 80.0, 80.0),
            ];
            r
        }, ColorPalette::Pastel1),

        // Justify-content
        fixture("justify_center", {
            let mut r = NodeConfig::new_container("root");
            r.justify_content = JustifyContent::Center;
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 80.0),
                NodeConfig::new_leaf("B", 80.0, 80.0),
            ];
            r
        }, ColorPalette::Pastel1),
        fixture("justify_space_between", {
            let mut r = NodeConfig::new_container("root");
            r.justify_content = JustifyContent::SpaceBetween;
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 80.0),
                NodeConfig::new_leaf("B", 80.0, 80.0),
            ];
            r
        }, ColorPalette::Pastel1),
        fixture("justify_space_evenly", {
            let mut r = NodeConfig::new_container("root");
            r.justify_content = JustifyContent::SpaceEvenly;
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 80.0),
                NodeConfig::new_leaf("B", 80.0, 80.0),
            ];
            r
        }, ColorPalette::Pastel1),

        // Align-items
        fixture("align_items_center", {
            let mut r = NodeConfig::new_container("root");
            r.align_items = AlignItems::Center;
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 80.0),
                NodeConfig::new_leaf("B", 80.0, 80.0),
            ];
            r
        }, ColorPalette::Pastel1),
        fixture("align_items_stretch", {
            let mut r = NodeConfig::new_container("root");
            r.align_items = AlignItems::Stretch;
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 80.0),
                NodeConfig::new_leaf("B", 80.0, 80.0),
            ];
            r
        }, ColorPalette::Pastel1),

        // Align-content
        fixture("align_content_space_between", {
            let mut r = NodeConfig::new_container("root");
            r.align_content = AlignContent::SpaceBetween;
            r.flex_wrap = FlexWrap::Wrap;
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 80.0),
                NodeConfig::new_leaf("B", 80.0, 80.0),
            ];
            r
        }, ColorPalette::Pastel1),

        // Wrap
        fixture("wrap_nowrap", {
            let mut r = NodeConfig::new_container("root");
            r.flex_wrap = FlexWrap::NoWrap;
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 80.0),
                NodeConfig::new_leaf("B", 80.0, 80.0),
            ];
            r
        }, ColorPalette::Pastel1),
        fixture("wrap_reverse", {
            let mut r = NodeConfig::new_container("root");
            r.flex_wrap = FlexWrap::WrapReverse;
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 80.0),
                NodeConfig::new_leaf("B", 80.0, 80.0),
            ];
            r
        }, ColorPalette::Pastel1),

        // Visibility
        fixture("hidden_child", {
            let mut hidden = NodeConfig::new_leaf("hidden", 80.0, 80.0);
            hidden.visible = false;
            let mut r = NodeConfig::new_container("root");
            r.children = vec![
                NodeConfig::new_leaf("visible", 80.0, 80.0),
                hidden,
            ];
            r
        }, ColorPalette::Pastel1),
        fixture("all_hidden", {
            let mut a = NodeConfig::new_leaf("A", 80.0, 80.0);
            a.visible = false;
            let mut b = NodeConfig::new_leaf("B", 80.0, 80.0);
            b.visible = false;
            let mut r = NodeConfig::new_container("root");
            r.children = vec![a, b];
            r
        }, ColorPalette::Pastel1),

        // Ordering
        fixture("ordered_children", {
            let mut a = NodeConfig::new_leaf("A", 80.0, 80.0);
            a.order = 3;
            let mut b = NodeConfig::new_leaf("B", 80.0, 80.0);
            b.order = -1;
            let c = NodeConfig::new_leaf("C", 80.0, 80.0);
            let mut r = NodeConfig::new_container("root");
            r.children = vec![a, b, c];
            r
        }, ColorPalette::Pastel1),

        // Padding + margin
        fixture("padding_margin", {
            let mut leaf = NodeConfig::new_leaf("spaced", 80.0, 80.0);
            leaf.padding = ValueConfig::Px(20.0);
            leaf.margin = ValueConfig::Px(10.0);
            let mut r = NodeConfig::new_container("root");
            r.children = vec![leaf];
            r
        }, ColorPalette::Pastel1),

        // Min/max sizes
        fixture("min_max_sizes", {
            let mut leaf = NodeConfig::new_leaf("constrained", 80.0, 80.0);
            leaf.min_width = ValueConfig::Px(40.0);
            leaf.max_width = ValueConfig::Px(200.0);
            leaf.min_height = ValueConfig::Px(30.0);
            leaf.max_height = ValueConfig::Px(150.0);
            let mut r = NodeConfig::new_container("root");
            r.children = vec![leaf];
            r
        }, ColorPalette::Pastel1),

        // Nesting
        fixture("nested_mixed", {
            let mut inner = NodeConfig::new_container("inner");
            inner.flex_direction = FlexDirection::Column;
            inner.children = vec![
                NodeConfig::new_leaf("X", 40.0, 40.0),
                NodeConfig::new_leaf("Y", 40.0, 40.0),
            ];
            let mut r = NodeConfig::new_container("outer");
            r.flex_direction = FlexDirection::Row;
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 80.0),
                inner,
                NodeConfig::new_leaf("B", 80.0, 80.0),
            ];
            r
        }, ColorPalette::Pastel1),
        fixture("deep_chain_3", {
            let mut current = NodeConfig::new_leaf("leaf", 50.0, 50.0);
            for i in (0..3_usize).rev() {
                let mut parent = NodeConfig::new_container(format!("level-{i}"));
                parent.children = vec![current];
                current = parent;
            }
            current
        }, ColorPalette::Pastel1),
        fixture("wide_flat_5", {
            let mut r = NodeConfig::new_container("root");
            r.children = (0..5)
                .map(|i| NodeConfig::new_leaf(format!("item-{i}"), 60.0, 60.0))
                .collect();
            r
        }, ColorPalette::Pastel1),

        // Flex item props
        fixture("grow_shrink", {
            let mut leaf = NodeConfig::new_leaf("flex", 80.0, 80.0);
            leaf.flex_grow = 2.5;
            leaf.flex_shrink = 0.0;
            let mut r = NodeConfig::new_container("root");
            r.children = vec![leaf];
            r
        }, ColorPalette::Pastel1),
        fixture("align_self_center", {
            let mut leaf = NodeConfig::new_leaf("centered", 80.0, 80.0);
            leaf.align_self = AlignSelf::Center;
            let mut r = NodeConfig::new_container("root");
            r.children = vec![leaf];
            r
        }, ColorPalette::Pastel1),
        fixture("flex_basis_percent", {
            let mut leaf = NodeConfig::new_leaf("based", 80.0, 80.0);
            leaf.flex_basis = ValueConfig::Percent(50.0);
            let mut r = NodeConfig::new_container("root");
            r.children = vec![leaf];
            r
        }, ColorPalette::Pastel1),
        fixture("gaps_mixed", {
            let mut r = NodeConfig::new_container("root");
            r.row_gap = ValueConfig::Px(16.0);
            r.column_gap = ValueConfig::Percent(5.0);
            r.children = vec![
                NodeConfig::new_leaf("A", 60.0, 60.0),
                NodeConfig::new_leaf("B", 60.0, 60.0),
            ];
            r
        }, ColorPalette::Pastel1),
        fixture("vw_vh_sizes", {
            let mut leaf = NodeConfig::new_leaf("viewport", 100.0, 100.0);
            leaf.width = ValueConfig::Vw(50.0);
            leaf.height = ValueConfig::Vh(75.0);
            let mut r = NodeConfig::new_container("root");
            r.children = vec![leaf];
            r
        }, ColorPalette::Pastel1),

        // Different palette
        fixture("dark2_palette", {
            let mut r = NodeConfig::new_container("root");
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 80.0),
                NodeConfig::new_leaf("B", 80.0, 80.0),
                NodeConfig::new_leaf("C", 80.0, 80.0),
            ];
            r
        }, ColorPalette::Dark2),

        // Templates
        fixture("tpl_holy_grail", templates::holy_grail(), ColorPalette::Pastel1),
        fixture("tpl_sidebar_content", templates::sidebar_content(), ColorPalette::Pastel1),
        fixture("tpl_card_grid", templates::card_grid(), ColorPalette::Pastel1),
        fixture("tpl_nav_bar", templates::nav_bar(), ColorPalette::Pastel1),
    ]
}

// ─── Snapshot infrastructure ─────────────────────────────────────────────────

fn testdata_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata")
}

fn updating() -> bool {
    std::env::var("FLEXPLORE_UPDATE").is_ok()
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
    ];

    let input_json = serde_json::to_string_pretty(&f.node).unwrap();

    if updating() {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&input_path, &input_json).unwrap();
        for (filename, content) in &targets {
            std::fs::write(dir.join(filename), content).unwrap();
        }
        eprintln!("  updated snapshot: {name}");
        return;
    }

    // Read input JSON back and re-generate to verify round-trip
    let json_src = std::fs::read_to_string(&input_path).unwrap_or_else(|e| {
        panic!("missing {}: {e} (run with FLEXPLORE_UPDATE=1 to generate)", input_path.display());
    });
    let from_json: NodeConfig = serde_json::from_str(&json_src).unwrap();

    let roundtrip_targets: Vec<(&str, String)> = vec![
        ("expected.html", emit_html_css(&from_json, f.palette).unwrap()),
        ("expected.rs", emit_bevy_code(&from_json, f.palette).unwrap()),
        ("expected.jsx", emit_react(&from_json, f.palette).unwrap()),
        ("expected.tailwind.html", emit_tailwind(&from_json, f.palette).unwrap()),
        ("expected.swift", emit_swiftui(&from_json, f.palette).unwrap()),
        ("expected.dart", emit_flutter(&from_json, f.palette).unwrap()),
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
            panic!("missing {}: {e} (run with FLEXPLORE_UPDATE=1 to generate)", path.display());
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
