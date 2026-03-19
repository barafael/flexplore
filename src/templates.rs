use bevy::prelude::*;

use crate::config::{NodeConfig, ValueConfig};

pub fn holy_grail() -> NodeConfig {
    let mut root = NodeConfig::new_container("root");
    root.flex_direction = FlexDirection::Column;
    root.flex_wrap = FlexWrap::NoWrap;
    root.align_items = AlignItems::Stretch;
    root.width = ValueConfig::Percent(100.0);
    root.height = ValueConfig::Percent(100.0);
    root.padding = ValueConfig::Px(0.0);
    root.row_gap = ValueConfig::Px(0.0);
    root.column_gap = ValueConfig::Px(0.0);

    let mut header = NodeConfig::new_leaf("header", 100.0, 60.0);
    header.width = ValueConfig::Auto;
    header.height = ValueConfig::Px(60.0);
    header.flex_grow = 0.0;
    header.flex_shrink = 0.0;

    let mut middle = NodeConfig::new_container("middle");
    middle.flex_direction = FlexDirection::Row;
    middle.flex_wrap = FlexWrap::NoWrap;
    middle.align_items = AlignItems::Stretch;
    middle.flex_grow = 1.0;
    middle.width = ValueConfig::Auto;
    middle.height = ValueConfig::Auto;
    middle.padding = ValueConfig::Px(0.0);
    middle.row_gap = ValueConfig::Px(0.0);
    middle.column_gap = ValueConfig::Px(0.0);

    let mut sidebar_left = NodeConfig::new_leaf("sidebar-left", 200.0, 100.0);
    sidebar_left.width = ValueConfig::Px(200.0);
    sidebar_left.height = ValueConfig::Auto;
    sidebar_left.flex_shrink = 0.0;
    sidebar_left.flex_grow = 0.0;

    let mut content = NodeConfig::new_leaf("content", 100.0, 100.0);
    content.flex_grow = 1.0;
    content.width = ValueConfig::Auto;
    content.height = ValueConfig::Auto;

    let mut sidebar_right = NodeConfig::new_leaf("sidebar-right", 200.0, 100.0);
    sidebar_right.width = ValueConfig::Px(200.0);
    sidebar_right.height = ValueConfig::Auto;
    sidebar_right.flex_shrink = 0.0;
    sidebar_right.flex_grow = 0.0;

    middle.children = vec![sidebar_left, content, sidebar_right];

    let mut footer = NodeConfig::new_leaf("footer", 100.0, 60.0);
    footer.width = ValueConfig::Auto;
    footer.height = ValueConfig::Px(60.0);
    footer.flex_grow = 0.0;
    footer.flex_shrink = 0.0;

    root.children = vec![header, middle, footer];
    root
}

pub fn sidebar_content() -> NodeConfig {
    let mut root = NodeConfig::new_container("root");
    root.flex_direction = FlexDirection::Row;
    root.flex_wrap = FlexWrap::NoWrap;
    root.align_items = AlignItems::Stretch;
    root.width = ValueConfig::Percent(100.0);
    root.height = ValueConfig::Percent(100.0);
    root.padding = ValueConfig::Px(0.0);
    root.column_gap = ValueConfig::Px(0.0);

    let mut sidebar = NodeConfig::new_container("sidebar");
    sidebar.flex_direction = FlexDirection::Column;
    sidebar.align_items = AlignItems::Stretch;
    sidebar.width = ValueConfig::Px(250.0);
    sidebar.height = ValueConfig::Auto;
    sidebar.flex_shrink = 0.0;
    sidebar.flex_grow = 0.0;
    sidebar.padding = ValueConfig::Px(8.0);
    sidebar.row_gap = ValueConfig::Px(4.0);
    sidebar.children = vec![
        NodeConfig::new_leaf("nav-1", 200.0, 44.0),
        NodeConfig::new_leaf("nav-2", 200.0, 44.0),
        NodeConfig::new_leaf("nav-3", 200.0, 44.0),
    ];
    for child in &mut sidebar.children {
        child.width = ValueConfig::Auto;
        child.height = ValueConfig::Px(44.0);
    }

    let mut content = NodeConfig::new_leaf("content", 100.0, 100.0);
    content.flex_grow = 1.0;
    content.width = ValueConfig::Auto;
    content.height = ValueConfig::Auto;

    root.children = vec![sidebar, content];
    root
}

pub fn card_grid() -> NodeConfig {
    let mut root = NodeConfig::new_container("grid");
    root.flex_direction = FlexDirection::Row;
    root.flex_wrap = FlexWrap::Wrap;
    root.align_items = AlignItems::FlexStart;
    root.align_content = AlignContent::FlexStart;
    root.row_gap = ValueConfig::Px(16.0);
    root.column_gap = ValueConfig::Px(16.0);
    root.width = ValueConfig::Percent(100.0);
    root.height = ValueConfig::Auto;
    root.justify_content = JustifyContent::FlexStart;
    root.padding = ValueConfig::Px(16.0);

    root.children = (1..=6)
        .map(|i| NodeConfig::new_leaf(format!("card-{i}"), 200.0, 250.0))
        .collect();

    root
}

pub fn nav_bar() -> NodeConfig {
    let mut root = NodeConfig::new_container("nav");
    root.flex_direction = FlexDirection::Row;
    root.flex_wrap = FlexWrap::NoWrap;
    root.width = ValueConfig::Percent(100.0);
    root.height = ValueConfig::Px(56.0);
    root.justify_content = JustifyContent::SpaceBetween;
    root.align_items = AlignItems::Center;
    root.padding = ValueConfig::Px(12.0);
    root.column_gap = ValueConfig::Px(0.0);

    let logo = NodeConfig::new_leaf("logo", 48.0, 48.0);

    let mut links = NodeConfig::new_container("links");
    links.flex_direction = FlexDirection::Row;
    links.flex_wrap = FlexWrap::NoWrap;
    links.align_items = AlignItems::Center;
    links.column_gap = ValueConfig::Px(8.0);
    links.row_gap = ValueConfig::Px(0.0);
    links.width = ValueConfig::Auto;
    links.height = ValueConfig::Auto;
    links.flex_grow = 0.0;
    links.padding = ValueConfig::Px(0.0);
    links.children = vec![
        NodeConfig::new_leaf("link-1", 80.0, 36.0),
        NodeConfig::new_leaf("link-2", 80.0, 36.0),
        NodeConfig::new_leaf("link-3", 80.0, 36.0),
    ];

    let mut actions = NodeConfig::new_container("actions");
    actions.flex_direction = FlexDirection::Row;
    actions.flex_wrap = FlexWrap::NoWrap;
    actions.align_items = AlignItems::Center;
    actions.column_gap = ValueConfig::Px(8.0);
    actions.row_gap = ValueConfig::Px(0.0);
    actions.width = ValueConfig::Auto;
    actions.height = ValueConfig::Auto;
    actions.flex_grow = 0.0;
    actions.padding = ValueConfig::Px(0.0);
    actions.children = vec![
        NodeConfig::new_leaf("btn-1", 36.0, 36.0),
        NodeConfig::new_leaf("btn-2", 36.0, 36.0),
    ];

    root.children = vec![logo, links, actions];
    root
}
