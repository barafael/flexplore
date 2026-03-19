use std::path::PathBuf;

use flexplore::render::{RenderJob, render_to_images};

/// Reuse the same fixture list as the snapshot tests.
/// This avoids duplicating the test case definitions.
fn all_jobs() -> Vec<RenderJob> {
    use bevy::prelude::*;
    use flexplore::config::*;
    use flexplore::templates;

    vec![
        job("single_leaf", {
            let mut r = NodeConfig::new_container("root");
            r.children = vec![NodeConfig::new_leaf("only", 100.0, 60.0)];
            r
        }, ColorPalette::Pastel1),
        job("two_children", {
            let mut r = NodeConfig::new_container("root");
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 80.0),
                NodeConfig::new_leaf("B", 120.0, 100.0),
            ];
            r
        }, ColorPalette::Pastel1),
        job("direction_column", {
            let mut r = NodeConfig::new_container("root");
            r.flex_direction = FlexDirection::Column;
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 80.0),
                NodeConfig::new_leaf("B", 80.0, 80.0),
            ];
            r
        }, ColorPalette::Pastel1),
        job("direction_row_reverse", {
            let mut r = NodeConfig::new_container("root");
            r.flex_direction = FlexDirection::RowReverse;
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 80.0),
                NodeConfig::new_leaf("B", 80.0, 80.0),
            ];
            r
        }, ColorPalette::Pastel1),
        job("direction_column_reverse", {
            let mut r = NodeConfig::new_container("root");
            r.flex_direction = FlexDirection::ColumnReverse;
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 80.0),
                NodeConfig::new_leaf("B", 80.0, 80.0),
            ];
            r
        }, ColorPalette::Pastel1),
        job("justify_center", {
            let mut r = NodeConfig::new_container("root");
            r.justify_content = JustifyContent::Center;
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 80.0),
                NodeConfig::new_leaf("B", 80.0, 80.0),
            ];
            r
        }, ColorPalette::Pastel1),
        job("justify_space_between", {
            let mut r = NodeConfig::new_container("root");
            r.justify_content = JustifyContent::SpaceBetween;
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 80.0),
                NodeConfig::new_leaf("B", 80.0, 80.0),
            ];
            r
        }, ColorPalette::Pastel1),
        job("justify_space_evenly", {
            let mut r = NodeConfig::new_container("root");
            r.justify_content = JustifyContent::SpaceEvenly;
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 80.0),
                NodeConfig::new_leaf("B", 80.0, 80.0),
            ];
            r
        }, ColorPalette::Pastel1),
        job("align_items_center", {
            let mut r = NodeConfig::new_container("root");
            r.align_items = AlignItems::Center;
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 80.0),
                NodeConfig::new_leaf("B", 80.0, 80.0),
            ];
            r
        }, ColorPalette::Pastel1),
        job("align_items_stretch", {
            let mut r = NodeConfig::new_container("root");
            r.align_items = AlignItems::Stretch;
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 80.0),
                NodeConfig::new_leaf("B", 80.0, 80.0),
            ];
            r
        }, ColorPalette::Pastel1),
        job("align_content_space_between", {
            let mut r = NodeConfig::new_container("root");
            r.align_content = AlignContent::SpaceBetween;
            r.flex_wrap = FlexWrap::Wrap;
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 80.0),
                NodeConfig::new_leaf("B", 80.0, 80.0),
            ];
            r
        }, ColorPalette::Pastel1),
        job("wrap_nowrap", {
            let mut r = NodeConfig::new_container("root");
            r.flex_wrap = FlexWrap::NoWrap;
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 80.0),
                NodeConfig::new_leaf("B", 80.0, 80.0),
            ];
            r
        }, ColorPalette::Pastel1),
        job("wrap_reverse", {
            let mut r = NodeConfig::new_container("root");
            r.flex_wrap = FlexWrap::WrapReverse;
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 80.0),
                NodeConfig::new_leaf("B", 80.0, 80.0),
            ];
            r
        }, ColorPalette::Pastel1),
        job("hidden_child", {
            let mut hidden = NodeConfig::new_leaf("hidden", 80.0, 80.0);
            hidden.visible = false;
            let mut r = NodeConfig::new_container("root");
            r.children = vec![
                NodeConfig::new_leaf("visible", 80.0, 80.0),
                hidden,
            ];
            r
        }, ColorPalette::Pastel1),
        job("all_hidden", {
            let mut a = NodeConfig::new_leaf("A", 80.0, 80.0);
            a.visible = false;
            let mut b = NodeConfig::new_leaf("B", 80.0, 80.0);
            b.visible = false;
            let mut r = NodeConfig::new_container("root");
            r.children = vec![a, b];
            r
        }, ColorPalette::Pastel1),
        job("ordered_children", {
            let mut a = NodeConfig::new_leaf("A", 80.0, 80.0);
            a.order = 3;
            let mut b = NodeConfig::new_leaf("B", 80.0, 80.0);
            b.order = -1;
            let c = NodeConfig::new_leaf("C", 80.0, 80.0);
            let mut r = NodeConfig::new_container("root");
            r.children = vec![a, b, c];
            r
        }, ColorPalette::Pastel1),
        job("padding_margin", {
            let mut leaf = NodeConfig::new_leaf("spaced", 80.0, 80.0);
            leaf.padding = ValueConfig::Px(20.0);
            leaf.margin = ValueConfig::Px(10.0);
            let mut r = NodeConfig::new_container("root");
            r.children = vec![leaf];
            r
        }, ColorPalette::Pastel1),
        job("min_max_sizes", {
            let mut leaf = NodeConfig::new_leaf("constrained", 80.0, 80.0);
            leaf.min_width = ValueConfig::Px(40.0);
            leaf.max_width = ValueConfig::Px(200.0);
            leaf.min_height = ValueConfig::Px(30.0);
            leaf.max_height = ValueConfig::Px(150.0);
            let mut r = NodeConfig::new_container("root");
            r.children = vec![leaf];
            r
        }, ColorPalette::Pastel1),
        job("nested_mixed", {
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
        job("deep_chain_3", {
            let mut current = NodeConfig::new_leaf("leaf", 50.0, 50.0);
            for i in (0..3_usize).rev() {
                let mut parent = NodeConfig::new_container(format!("level-{i}"));
                parent.children = vec![current];
                current = parent;
            }
            current
        }, ColorPalette::Pastel1),
        job("wide_flat_5", {
            let mut r = NodeConfig::new_container("root");
            r.children = (0..5)
                .map(|i| NodeConfig::new_leaf(format!("item-{i}"), 60.0, 60.0))
                .collect();
            r
        }, ColorPalette::Pastel1),
        job("grow_shrink", {
            let mut leaf = NodeConfig::new_leaf("flex", 80.0, 80.0);
            leaf.flex_grow = 2.5;
            leaf.flex_shrink = 0.0;
            let mut r = NodeConfig::new_container("root");
            r.children = vec![leaf];
            r
        }, ColorPalette::Pastel1),
        job("align_self_center", {
            let mut leaf = NodeConfig::new_leaf("centered", 80.0, 80.0);
            leaf.align_self = AlignSelf::Center;
            let mut r = NodeConfig::new_container("root");
            r.children = vec![leaf];
            r
        }, ColorPalette::Pastel1),
        job("flex_basis_percent", {
            let mut leaf = NodeConfig::new_leaf("based", 80.0, 80.0);
            leaf.flex_basis = ValueConfig::Percent(50.0);
            let mut r = NodeConfig::new_container("root");
            r.children = vec![leaf];
            r
        }, ColorPalette::Pastel1),
        job("gaps_mixed", {
            let mut r = NodeConfig::new_container("root");
            r.row_gap = ValueConfig::Px(16.0);
            r.column_gap = ValueConfig::Percent(5.0);
            r.children = vec![
                NodeConfig::new_leaf("A", 60.0, 60.0),
                NodeConfig::new_leaf("B", 60.0, 60.0),
            ];
            r
        }, ColorPalette::Pastel1),
        job("vw_vh_sizes", {
            let mut leaf = NodeConfig::new_leaf("viewport", 100.0, 100.0);
            leaf.width = ValueConfig::Vw(50.0);
            leaf.height = ValueConfig::Vh(75.0);
            let mut r = NodeConfig::new_container("root");
            r.children = vec![leaf];
            r
        }, ColorPalette::Pastel1),
        job("dark2_palette", {
            let mut r = NodeConfig::new_container("root");
            r.children = vec![
                NodeConfig::new_leaf("A", 80.0, 80.0),
                NodeConfig::new_leaf("B", 80.0, 80.0),
                NodeConfig::new_leaf("C", 80.0, 80.0),
            ];
            r
        }, ColorPalette::Dark2),
        job("tpl_holy_grail", templates::holy_grail(), ColorPalette::Pastel1),
        job("tpl_sidebar_content", templates::sidebar_content(), ColorPalette::Pastel1),
        job("tpl_card_grid", templates::card_grid(), ColorPalette::Pastel1),
        job("tpl_nav_bar", templates::nav_bar(), ColorPalette::Pastel1),
    ]
}

fn job(name: &str, node: flexplore::config::NodeConfig, palette: flexplore::config::ColorPalette) -> RenderJob {
    RenderJob { name: name.into(), node, palette }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut jobs = all_jobs();

    // Filter to specific cases if names given on CLI
    if args.len() > 1 {
        let requested: Vec<&str> = args[1..].iter().map(|s| s.as_str()).collect();
        jobs.retain(|j| requested.contains(&j.name.as_str()));
    }

    render_to_images(jobs, PathBuf::from("testdata"));
}
