//! Snapshot test fixtures — shared between tests and the `update-snapshots` tool.

use bevy::prelude::*;

use crate::config::{ColorPalette, NodeConfig, ValueConfig};
use crate::templates;

pub struct Fixture {
    pub name: String,
    pub node: NodeConfig,
    pub palette: ColorPalette,
}

fn fixture(name: &str, node: NodeConfig, palette: ColorPalette) -> Fixture {
    Fixture {
        name: name.into(),
        node,
        palette,
    }
}

pub fn all_fixtures() -> Vec<Fixture> {
    vec![
        // ── Basic shapes ─────────────────────────────────────────────────
        fixture(
            "single_leaf",
            {
                let mut r = NodeConfig::new_container("root");
                r.children = vec![NodeConfig::new_leaf("only", 100.0, 60.0)];
                r
            },
            ColorPalette::Pastel1,
        ),
        fixture(
            "two_children",
            {
                let mut r = NodeConfig::new_container("root");
                r.children = vec![
                    NodeConfig::new_leaf("A", 80.0, 80.0),
                    NodeConfig::new_leaf("B", 120.0, 100.0),
                ];
                r
            },
            ColorPalette::Pastel1,
        ),
        // ── Flex-direction variants ──────────────────────────────────────
        // 3 differently-sized children so direction change is unambiguous
        fixture(
            "direction_column",
            {
                let mut r = NodeConfig::new_container("root");
                r.flex_direction = FlexDirection::Column;
                r.children = vec![
                    NodeConfig::new_leaf("A", 200.0, 60.0),
                    NodeConfig::new_leaf("B", 120.0, 80.0),
                    NodeConfig::new_leaf("C", 60.0, 40.0),
                ];
                r
            },
            ColorPalette::Pastel1,
        ),
        fixture(
            "direction_row_reverse",
            {
                let mut r = NodeConfig::new_container("root");
                r.flex_direction = FlexDirection::RowReverse;
                r.children = vec![
                    NodeConfig::new_leaf("A", 200.0, 60.0),
                    NodeConfig::new_leaf("B", 120.0, 80.0),
                    NodeConfig::new_leaf("C", 60.0, 40.0),
                ];
                r
            },
            ColorPalette::Pastel1,
        ),
        fixture(
            "direction_column_reverse",
            {
                let mut r = NodeConfig::new_container("root");
                r.flex_direction = FlexDirection::ColumnReverse;
                r.children = vec![
                    NodeConfig::new_leaf("A", 200.0, 60.0),
                    NodeConfig::new_leaf("B", 120.0, 80.0),
                    NodeConfig::new_leaf("C", 60.0, 40.0),
                ];
                r
            },
            ColorPalette::Pastel1,
        ),
        // ── Justify-content ──────────────────────────────────────────────
        // 4 differently-sized items so the spacing pattern is obvious
        fixture(
            "justify_center",
            {
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
            },
            ColorPalette::Pastel1,
        ),
        fixture(
            "justify_space_between",
            {
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
            },
            ColorPalette::Pastel1,
        ),
        fixture(
            "justify_space_evenly",
            {
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
            },
            ColorPalette::Pastel1,
        ),
        // ── Align-items ──────────────────────────────────────────────────
        // Column + different widths + NoWrap: centering is visible as horizontal indent
        fixture(
            "align_items_center",
            {
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
            },
            ColorPalette::Pastel1,
        ),
        // Row + explicit container height, children have NO height → they stretch tall
        fixture(
            "align_items_stretch",
            {
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
            },
            ColorPalette::Pastel1,
        ),
        // ── Align-content ────────────────────────────────────────────────
        // 6 items that wrap into 3 rows of 2, tall container → rows pushed to top/bottom
        fixture(
            "align_content_space_between",
            {
                let mut r = NodeConfig::new_container("root");
                r.align_content = AlignContent::SpaceBetween;
                r.flex_wrap = FlexWrap::Wrap;
                r.height = ValueConfig::Px(280.0);
                r.children = vec![
                    NodeConfig::new_leaf("A", 170.0, 60.0),
                    NodeConfig::new_leaf("B", 170.0, 60.0),
                    NodeConfig::new_leaf("C", 170.0, 60.0),
                    NodeConfig::new_leaf("D", 170.0, 60.0),
                    NodeConfig::new_leaf("E", 170.0, 60.0),
                    NodeConfig::new_leaf("F", 170.0, 60.0),
                ];
                r
            },
            ColorPalette::Pastel1,
        ),
        // ── Wrap ─────────────────────────────────────────────────────────
        // 6 wide items that overflow when nowrap → visibly crammed/shrunk
        fixture(
            "wrap_nowrap",
            {
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
            },
            ColorPalette::Pastel1,
        ),
        // Same 6 items with wrap-reverse → bottom row comes first
        fixture(
            "wrap_reverse",
            {
                let mut r = NodeConfig::new_container("root");
                r.flex_wrap = FlexWrap::WrapReverse;
                r.children = vec![
                    NodeConfig::new_leaf("A", 170.0, 80.0),
                    NodeConfig::new_leaf("B", 170.0, 80.0),
                    NodeConfig::new_leaf("C", 170.0, 80.0),
                    NodeConfig::new_leaf("D", 170.0, 80.0),
                    NodeConfig::new_leaf("E", 170.0, 80.0),
                    NodeConfig::new_leaf("F", 170.0, 80.0),
                ];
                r
            },
            ColorPalette::Pastel1,
        ),
        // ── Visibility ───────────────────────────────────────────────────
        fixture(
            "hidden_child",
            {
                let mut hidden = NodeConfig::new_leaf("hidden", 80.0, 80.0);
                hidden.visible = false;
                let mut r = NodeConfig::new_container("root");
                r.children = vec![NodeConfig::new_leaf("visible", 80.0, 80.0), hidden];
                r
            },
            ColorPalette::Pastel1,
        ),
        fixture(
            "all_hidden",
            {
                let mut a = NodeConfig::new_leaf("A", 80.0, 80.0);
                a.visible = false;
                let mut b = NodeConfig::new_leaf("B", 80.0, 80.0);
                b.visible = false;
                let mut r = NodeConfig::new_container("root");
                r.children = vec![a, b];
                r
            },
            ColorPalette::Pastel1,
        ),
        // ── Ordering ─────────────────────────────────────────────────────
        fixture(
            "ordered_children",
            {
                let mut a = NodeConfig::new_leaf("A", 80.0, 80.0);
                a.order = 3;
                let mut b = NodeConfig::new_leaf("B", 80.0, 80.0);
                b.order = -1;
                let c = NodeConfig::new_leaf("C", 80.0, 80.0);
                let mut r = NodeConfig::new_container("root");
                r.children = vec![a, b, c];
                r
            },
            ColorPalette::Pastel1,
        ),
        // ── Padding + margin ─────────────────────────────────────────────
        // 3 items with margin inside a padded container → visible inset + gaps
        fixture(
            "padding_margin",
            {
                let mut r = NodeConfig::new_container("root");
                r.padding = ValueConfig::Px(20.0);
                r.flex_wrap = FlexWrap::NoWrap;
                let items: Vec<_> = ["A", "B", "C"]
                    .iter()
                    .map(|label| {
                        let mut leaf = NodeConfig::new_leaf(*label, 100.0, 60.0);
                        leaf.margin = ValueConfig::Px(16.0);
                        leaf
                    })
                    .collect();
                r.children = items;
                r
            },
            ColorPalette::Pastel1,
        ),
        // ── Min/max sizes ────────────────────────────────────────────────
        // 3 items all grow=1 but with different constraints → visibly different widths
        fixture(
            "min_max_sizes",
            {
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
            },
            ColorPalette::Pastel1,
        ),
        // ── Nesting ──────────────────────────────────────────────────────
        fixture(
            "nested_mixed",
            {
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
            },
            ColorPalette::Pastel1,
        ),
        fixture(
            "deep_chain_3",
            {
                let mut current = NodeConfig::new_leaf("leaf", 50.0, 50.0);
                for i in (0..3_usize).rev() {
                    let mut parent = NodeConfig::new_container(format!("level-{i}"));
                    parent.children = vec![current];
                    current = parent;
                }
                current
            },
            ColorPalette::Pastel1,
        ),
        fixture(
            "wide_flat_5",
            {
                let mut r = NodeConfig::new_container("root");
                r.children = (0..5)
                    .map(|i| NodeConfig::new_leaf(format!("item-{i}"), 60.0, 60.0))
                    .collect();
                r
            },
            ColorPalette::Pastel1,
        ),
        // ── Flex item props ──────────────────────────────────────────────
        // 3 items with different grow factors → proportional widths
        fixture(
            "grow_shrink",
            {
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
            },
            ColorPalette::Pastel1,
        ),
        // 3 items in a tall container; middle one centered, others at top
        fixture(
            "align_self_center",
            {
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
            },
            ColorPalette::Pastel1,
        ),
        // 3 items with different basis percentages → 50% / 25% / 25%
        fixture(
            "flex_basis_percent",
            {
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
            },
            ColorPalette::Pastel1,
        ),
        // 6 items that wrap → visible column gap AND row gap
        fixture(
            "gaps_mixed",
            {
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
            },
            ColorPalette::Pastel1,
        ),
        // 2 bars at different viewport-relative sizes
        fixture(
            "vw_vh_sizes",
            {
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
            },
            ColorPalette::Pastel1,
        ),
        // ── Different palette ────────────────────────────────────────────
        fixture(
            "dark2_palette",
            {
                let mut r = NodeConfig::new_container("root");
                r.children = vec![
                    NodeConfig::new_leaf("A", 80.0, 80.0),
                    NodeConfig::new_leaf("B", 80.0, 80.0),
                    NodeConfig::new_leaf("C", 80.0, 80.0),
                ];
                r
            },
            ColorPalette::Dark2,
        ),
        // ── Templates ────────────────────────────────────────────────────
        fixture(
            "tpl_holy_grail",
            templates::holy_grail(),
            ColorPalette::Pastel1,
        ),
        fixture(
            "tpl_sidebar_content",
            templates::sidebar_content(),
            ColorPalette::Pastel1,
        ),
        fixture(
            "tpl_card_grid",
            templates::card_grid(),
            ColorPalette::Pastel1,
        ),
        fixture("tpl_nav_bar", templates::nav_bar(), ColorPalette::Pastel1),
    ]
}
