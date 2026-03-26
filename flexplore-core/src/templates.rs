use crate::config::*;

pub fn holy_grail() -> NodeConfig {
    let mut root = NodeConfig::new_container("root");
    root.flex_direction = FlexDirection::Column;
    root.flex_wrap = FlexWrap::NoWrap;
    root.align_items = AlignItems::Stretch;
    root.width = ValueConfig::Percent(100.0);
    root.height = ValueConfig::Percent(100.0);
    root.padding = Sides::zero();
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
    middle.padding = Sides::zero();
    middle.row_gap = ValueConfig::Px(0.0);
    middle.column_gap = ValueConfig::Px(0.0);

    let mut sidebar_left = NodeConfig::new_leaf("sidebar-left", 120.0, 100.0);
    sidebar_left.width = ValueConfig::Px(120.0);
    sidebar_left.height = ValueConfig::Auto;
    sidebar_left.flex_shrink = 0.0;
    sidebar_left.flex_grow = 0.0;

    let mut content = NodeConfig::new_leaf("content", 100.0, 100.0);
    content.flex_grow = 1.0;
    content.width = ValueConfig::Auto;
    content.height = ValueConfig::Auto;

    let mut sidebar_right = NodeConfig::new_leaf("sidebar-right", 120.0, 100.0);
    sidebar_right.width = ValueConfig::Px(120.0);
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
    root.padding = Sides::zero();
    root.column_gap = ValueConfig::Px(0.0);

    let mut sidebar = NodeConfig::new_container("sidebar");
    sidebar.flex_direction = FlexDirection::Column;
    sidebar.flex_wrap = FlexWrap::NoWrap;
    sidebar.align_items = AlignItems::Stretch;
    sidebar.width = ValueConfig::Px(120.0);
    sidebar.height = ValueConfig::Auto;
    sidebar.flex_shrink = 0.0;
    sidebar.flex_grow = 0.0;
    sidebar.padding = Sides::uniform(ValueConfig::Px(8.0));
    sidebar.row_gap = ValueConfig::Px(4.0);
    let nav_items: Vec<_> = (1..=3)
        .map(|i| {
            let mut n = NodeConfig::new_leaf(format!("nav-{i}"), 100.0, 44.0);
            n.width = ValueConfig::Auto;
            n
        })
        .collect();
    sidebar.children = nav_items;

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
    root.padding = Sides::uniform(ValueConfig::Px(16.0));

    root.children = (1..=6)
        .map(|i| NodeConfig::new_leaf(format!("card-{i}"), 170.0, 80.0))
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
    root.padding = Sides::uniform(ValueConfig::Px(12.0));
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
    links.padding = Sides::zero();
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
    actions.padding = Sides::zero();
    actions.children = vec![
        NodeConfig::new_leaf("btn-1", 36.0, 36.0),
        NodeConfig::new_leaf("btn-2", 36.0, 36.0),
    ];

    root.children = vec![logo, links, actions];
    root
}

/// A 3-column responsive-style grid dashboard.
pub fn grid_dashboard() -> NodeConfig {
    let mut root = NodeConfig::new_grid(
        "dashboard",
        vec![GridTrackSize::Fr(1.0), GridTrackSize::Fr(1.0), GridTrackSize::Fr(1.0)],
    );
    root.grid_template_rows = vec![GridTrackSize::Px(60.0), GridTrackSize::Fr(1.0), GridTrackSize::Px(40.0)];
    root.width = ValueConfig::Percent(100.0);
    root.height = ValueConfig::Percent(100.0);
    root.padding = Sides::zero();
    root.row_gap = ValueConfig::Px(0.0);
    root.column_gap = ValueConfig::Px(0.0);

    let mut header = NodeConfig::new_leaf("header", 100.0, 60.0);
    header.grid_column = GridPlacement::StartSpan(1, 3);
    header.width = ValueConfig::Auto;
    header.height = ValueConfig::Auto;

    let mut sidebar = NodeConfig::new_leaf("sidebar", 100.0, 100.0);
    sidebar.width = ValueConfig::Auto;
    sidebar.height = ValueConfig::Auto;

    let mut main = NodeConfig::new_leaf("main", 100.0, 100.0);
    main.grid_column = GridPlacement::Span(2);
    main.width = ValueConfig::Auto;
    main.height = ValueConfig::Auto;

    let mut footer = NodeConfig::new_leaf("footer", 100.0, 40.0);
    footer.grid_column = GridPlacement::StartSpan(1, 3);
    footer.width = ValueConfig::Auto;
    footer.height = ValueConfig::Auto;

    root.children = vec![header, sidebar, main, footer];
    root
}

/// A photo-gallery-style grid with items of varying spans.
pub fn grid_gallery() -> NodeConfig {
    let mut root = NodeConfig::new_grid(
        "gallery",
        vec![
            GridTrackSize::Fr(1.0),
            GridTrackSize::Fr(1.0),
            GridTrackSize::Fr(1.0),
            GridTrackSize::Fr(1.0),
        ],
    );
    root.grid_auto_rows = vec![GridTrackSize::Px(120.0)];
    root.width = ValueConfig::Percent(100.0);
    root.height = ValueConfig::Auto;
    root.padding = Sides::uniform(ValueConfig::Px(8.0));
    root.row_gap = ValueConfig::Px(8.0);
    root.column_gap = ValueConfig::Px(8.0);

    let mut wide = NodeConfig::new_leaf("wide", 100.0, 120.0);
    wide.grid_column = GridPlacement::Span(2);
    wide.width = ValueConfig::Auto;
    wide.height = ValueConfig::Auto;

    let mut tall = NodeConfig::new_leaf("tall", 100.0, 120.0);
    tall.grid_row = GridPlacement::Span(2);
    tall.width = ValueConfig::Auto;
    tall.height = ValueConfig::Auto;

    let items: Vec<NodeConfig> = (1..=4)
        .map(|i| {
            let mut n = NodeConfig::new_leaf(format!("img-{i}"), 100.0, 120.0);
            n.width = ValueConfig::Auto;
            n.height = ValueConfig::Auto;
            n
        })
        .collect();

    root.children = vec![wide, items[0].clone(), items[1].clone(), tall, items[2].clone(), items[3].clone()];
    root
}
