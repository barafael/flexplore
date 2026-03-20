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
        // ── Basic shapes ─────────────────────────────────────────────────
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

        // ── Flex-direction variants ──────────────────────────────────────
        // 3 differently-sized children so direction change is unambiguous
        fixture("direction_column", {
            let mut r = NodeConfig::new_container("root");
            r.flex_direction = FlexDirection::Column;
            r.children = vec![
                NodeConfig::new_leaf("A", 200.0, 60.0),
                NodeConfig::new_leaf("B", 120.0, 80.0),
                NodeConfig::new_leaf("C", 60.0, 40.0),
            ];
            r
        }, ColorPalette::Pastel1),
        fixture("direction_row_reverse", {
            let mut r = NodeConfig::new_container("root");
            r.flex_direction = FlexDirection::RowReverse;
            r.children = vec![
                NodeConfig::new_leaf("A", 200.0, 60.0),
                NodeConfig::new_leaf("B", 120.0, 80.0),
                NodeConfig::new_leaf("C", 60.0, 40.0),
            ];
            r
        }, ColorPalette::Pastel1),
        fixture("direction_column_reverse", {
            let mut r = NodeConfig::new_container("root");
            r.flex_direction = FlexDirection::ColumnReverse;
            r.children = vec![
                NodeConfig::new_leaf("A", 200.0, 60.0),
                NodeConfig::new_leaf("B", 120.0, 80.0),
                NodeConfig::new_leaf("C", 60.0, 40.0),
            ];
            r
        }, ColorPalette::Pastel1),

        // ── Justify-content ──────────────────────────────────────────────
        // 4 differently-sized items so the spacing pattern is obvious
        fixture("justify_center", {
            let mut r = NodeConfig::new_container("root");
            r.justify_content = JustifyContent::Center;
            r.flex_wrap = FlexWrap::NoWrap;
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 60.0),
                NodeConfig::new_leaf("B", 40.0, 60.0),
                NodeConfig::new_leaf("C", 100.0, 60.0),
                NodeConfig::new_leaf("D", 60.0, 60.0),
            ];
            r
        }, ColorPalette::Pastel1),
        fixture("justify_space_between", {
            let mut r = NodeConfig::new_container("root");
            r.justify_content = JustifyContent::SpaceBetween;
            r.flex_wrap = FlexWrap::NoWrap;
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 60.0),
                NodeConfig::new_leaf("B", 40.0, 60.0),
                NodeConfig::new_leaf("C", 100.0, 60.0),
                NodeConfig::new_leaf("D", 60.0, 60.0),
            ];
            r
        }, ColorPalette::Pastel1),
        fixture("justify_space_evenly", {
            let mut r = NodeConfig::new_container("root");
            r.justify_content = JustifyContent::SpaceEvenly;
            r.flex_wrap = FlexWrap::NoWrap;
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 60.0),
                NodeConfig::new_leaf("B", 40.0, 60.0),
                NodeConfig::new_leaf("C", 100.0, 60.0),
                NodeConfig::new_leaf("D", 60.0, 60.0),
            ];
            r
        }, ColorPalette::Pastel1),

        // ── Align-items ──────────────────────────────────────────────────
        // Column + different widths + NoWrap: centering is visible as horizontal indent
        fixture("align_items_center", {
            let mut r = NodeConfig::new_container("root");
            r.flex_direction = FlexDirection::Column;
            r.flex_wrap = FlexWrap::NoWrap;
            r.align_items = AlignItems::Center;
            r.children = vec![
                NodeConfig::new_leaf("A", 200.0, 50.0),
                NodeConfig::new_leaf("B", 120.0, 50.0),
                NodeConfig::new_leaf("C", 80.0, 50.0),
            ];
            r
        }, ColorPalette::Pastel1),
        // Row + explicit container height, children have NO height → they stretch tall
        fixture("align_items_stretch", {
            let mut r = NodeConfig::new_container("root");
            r.align_items = AlignItems::Stretch;
            r.height = ValueConfig::Px(300.0);
            r.flex_wrap = FlexWrap::NoWrap;
            let mut a = NodeConfig::new_leaf("A", 100.0, 80.0);
            a.height = ValueConfig::Auto;
            let mut b = NodeConfig::new_leaf("B", 80.0, 80.0);
            b.height = ValueConfig::Auto;
            let mut c = NodeConfig::new_leaf("C", 60.0, 80.0);
            c.height = ValueConfig::Auto;
            r.children = vec![a, b, c];
            r
        }, ColorPalette::Pastel1),

        // ── Align-content ────────────────────────────────────────────────
        // 6 items that wrap into 2 rows, tall container → rows pushed to top/bottom
        fixture("align_content_space_between", {
            let mut r = NodeConfig::new_container("root");
            r.align_content = AlignContent::SpaceBetween;
            r.flex_wrap = FlexWrap::Wrap;
            r.height = ValueConfig::Px(280.0);
            r.children = vec![
                NodeConfig::new_leaf("A", 200.0, 60.0),
                NodeConfig::new_leaf("B", 200.0, 60.0),
                NodeConfig::new_leaf("C", 200.0, 60.0),
                NodeConfig::new_leaf("D", 200.0, 60.0),
                NodeConfig::new_leaf("E", 200.0, 60.0),
                NodeConfig::new_leaf("F", 200.0, 60.0),
            ];
            r
        }, ColorPalette::Pastel1),

        // ── Wrap ─────────────────────────────────────────────────────────
        // 6 wide items that overflow when nowrap → visibly crammed/shrunk
        fixture("wrap_nowrap", {
            let mut r = NodeConfig::new_container("root");
            r.flex_wrap = FlexWrap::NoWrap;
            r.children = vec![
                NodeConfig::new_leaf("A", 200.0, 80.0),
                NodeConfig::new_leaf("B", 200.0, 80.0),
                NodeConfig::new_leaf("C", 200.0, 80.0),
                NodeConfig::new_leaf("D", 200.0, 80.0),
                NodeConfig::new_leaf("E", 200.0, 80.0),
                NodeConfig::new_leaf("F", 200.0, 80.0),
            ];
            r
        }, ColorPalette::Pastel1),
        // Same 6 items with wrap-reverse → bottom row comes first
        fixture("wrap_reverse", {
            let mut r = NodeConfig::new_container("root");
            r.flex_wrap = FlexWrap::WrapReverse;
            r.children = vec![
                NodeConfig::new_leaf("A", 200.0, 80.0),
                NodeConfig::new_leaf("B", 200.0, 80.0),
                NodeConfig::new_leaf("C", 200.0, 80.0),
                NodeConfig::new_leaf("D", 200.0, 80.0),
                NodeConfig::new_leaf("E", 200.0, 80.0),
                NodeConfig::new_leaf("F", 200.0, 80.0),
            ];
            r
        }, ColorPalette::Pastel1),

        // ── Visibility ───────────────────────────────────────────────────
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

        // ── Ordering ─────────────────────────────────────────────────────
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

        // ── Padding + margin ─────────────────────────────────────────────
        // 3 items with margin inside a padded container → visible inset + gaps
        fixture("padding_margin", {
            let mut r = NodeConfig::new_container("root");
            r.padding = ValueConfig::Px(20.0);
            r.flex_wrap = FlexWrap::NoWrap;
            let items: Vec<_> = ["A", "B", "C"].iter().map(|label| {
                let mut leaf = NodeConfig::new_leaf(*label, 100.0, 60.0);
                leaf.margin = ValueConfig::Px(16.0);
                leaf
            }).collect();
            r.children = items;
            r
        }, ColorPalette::Pastel1),

        // ── Min/max sizes ────────────────────────────────────────────────
        // 3 items all grow=1 but with different constraints → visibly different widths
        fixture("min_max_sizes", {
            let mut r = NodeConfig::new_container("root");
            r.flex_wrap = FlexWrap::NoWrap;
            // A: capped small
            let mut a = NodeConfig::new_leaf("capped", 80.0, 80.0);
            a.flex_grow = 1.0;
            a.width = ValueConfig::Auto;
            a.max_width = ValueConfig::Px(100.0);
            // B: unconstrained
            let mut b = NodeConfig::new_leaf("free", 80.0, 80.0);
            b.flex_grow = 1.0;
            b.width = ValueConfig::Auto;
            // C: forced large
            let mut c = NodeConfig::new_leaf("wide", 80.0, 80.0);
            c.flex_grow = 1.0;
            c.width = ValueConfig::Auto;
            c.min_width = ValueConfig::Px(200.0);
            r.children = vec![a, b, c];
            r
        }, ColorPalette::Pastel1),

        // ── Nesting ──────────────────────────────────────────────────────
        fixture("nested_mixed", {
            let mut inner = NodeConfig::new_container("inner");
            inner.flex_direction = FlexDirection::Column;
            inner.width = ValueConfig::Px(200.0);
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

        // ── Flex item props ──────────────────────────────────────────────
        // 3 items with different grow factors → proportional widths
        fixture("grow_shrink", {
            let mut r = NodeConfig::new_container("root");
            r.flex_wrap = FlexWrap::NoWrap;
            // A: grow 1 (gets 1/3 of remaining space)
            let mut a = NodeConfig::new_leaf("grow-1", 80.0, 80.0);
            a.flex_grow = 1.0;
            a.width = ValueConfig::Auto;
            // B: grow 2 (gets 2/3 of remaining space)
            let mut b = NodeConfig::new_leaf("grow-2", 80.0, 80.0);
            b.flex_grow = 2.0;
            b.width = ValueConfig::Auto;
            // C: grow 0, fixed width (stays at 100px)
            let mut c = NodeConfig::new_leaf("fixed", 100.0, 80.0);
            c.flex_grow = 0.0;
            c.flex_shrink = 0.0;
            r.children = vec![a, b, c];
            r
        }, ColorPalette::Pastel1),
        // 3 items in a tall container; middle one centered, others at top
        fixture("align_self_center", {
            let mut r = NodeConfig::new_container("root");
            r.height = ValueConfig::Px(300.0);
            r.align_items = AlignItems::FlexStart;
            r.flex_wrap = FlexWrap::NoWrap;
            let mut centered = NodeConfig::new_leaf("centered", 120.0, 60.0);
            centered.align_self = AlignSelf::Center;
            r.children = vec![
                NodeConfig::new_leaf("top", 100.0, 60.0),
                centered,
                NodeConfig::new_leaf("top", 100.0, 60.0),
            ];
            r
        }, ColorPalette::Pastel1),
        // 3 items with different basis percentages → 50% / 25% / 25%
        fixture("flex_basis_percent", {
            let mut r = NodeConfig::new_container("root");
            r.flex_wrap = FlexWrap::NoWrap;
            let mut a = NodeConfig::new_leaf("50%", 80.0, 80.0);
            a.flex_basis = ValueConfig::Percent(50.0);
            a.width = ValueConfig::Auto;
            let mut b = NodeConfig::new_leaf("25%", 80.0, 80.0);
            b.flex_basis = ValueConfig::Percent(25.0);
            b.width = ValueConfig::Auto;
            let mut c = NodeConfig::new_leaf("25%", 80.0, 80.0);
            c.flex_basis = ValueConfig::Percent(25.0);
            c.width = ValueConfig::Auto;
            r.children = vec![a, b, c];
            r
        }, ColorPalette::Pastel1),
        // 6 items that wrap → visible column gap AND row gap
        fixture("gaps_mixed", {
            let mut r = NodeConfig::new_container("root");
            r.row_gap = ValueConfig::Px(24.0);
            r.column_gap = ValueConfig::Px(40.0);
            r.flex_wrap = FlexWrap::Wrap;
            r.children = vec![
                NodeConfig::new_leaf("A", 150.0, 60.0),
                NodeConfig::new_leaf("B", 150.0, 60.0),
                NodeConfig::new_leaf("C", 150.0, 60.0),
                NodeConfig::new_leaf("D", 150.0, 60.0),
                NodeConfig::new_leaf("E", 150.0, 60.0),
                NodeConfig::new_leaf("F", 150.0, 60.0),
            ];
            r
        }, ColorPalette::Pastel1),
        // 2 bars at different viewport-relative sizes
        fixture("vw_vh_sizes", {
            let mut r = NodeConfig::new_container("root");
            r.flex_direction = FlexDirection::Column;
            let mut a = NodeConfig::new_leaf("50vw x 20vh", 100.0, 100.0);
            a.width = ValueConfig::Vw(50.0);
            a.height = ValueConfig::Vh(20.0);
            let mut b = NodeConfig::new_leaf("75vw x 30vh", 100.0, 100.0);
            b.width = ValueConfig::Vw(75.0);
            b.height = ValueConfig::Vh(30.0);
            r.children = vec![a, b];
            r
        }, ColorPalette::Pastel1),

        // ── Different palette ────────────────────────────────────────────
        fixture("dark2_palette", {
            let mut r = NodeConfig::new_container("root");
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 80.0),
                NodeConfig::new_leaf("B", 80.0, 80.0),
                NodeConfig::new_leaf("C", 80.0, 80.0),
            ];
            r
        }, ColorPalette::Dark2),

        // ── Templates ────────────────────────────────────────────────────
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
