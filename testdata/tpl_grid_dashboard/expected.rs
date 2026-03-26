fn spawn_ui(commands: &mut Commands) {
    // dashboard
    commands.spawn((
        Node {
            display: Display::Grid,
            grid_template_columns: vec![RepeatedGridTrack::fr(1, 1.0), RepeatedGridTrack::fr(1, 1.0), RepeatedGridTrack::fr(1, 1.0)],
            grid_template_rows: vec![RepeatedGridTrack::px(1, 60.0), RepeatedGridTrack::fr(1, 1.0), RepeatedGridTrack::px(1, 40.0)],
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            align_content: AlignContent::FlexStart,
            flex_grow: 1.0,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            min_height: Val::Px(0.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.11, 0.11, 0.17, 1.0)),
    )).with_children(|parent| {
        // header
        parent.spawn((
            Node {
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                align_content: AlignContent::FlexStart,
                row_gap: Val::Px(4.0),
                column_gap: Val::Px(4.0),
                grid_column: GridPlacement::start_span(1, 3),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.98, 0.71, 0.68)),
        )).with_children(|parent| {
            parent.spawn(Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            }).with_child((
                Text::new("header"),
                TextFont { font_size: 26.0, ..default() },
                TextColor(Color::srgba(0.05, 0.05, 0.1, 0.85)),
            ));
        });
        // sidebar
        parent.spawn((
            Node {
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                align_content: AlignContent::FlexStart,
                row_gap: Val::Px(4.0),
                column_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.70, 0.80, 0.89)),
        )).with_children(|parent| {
            parent.spawn(Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            }).with_child((
                Text::new("sidebar"),
                TextFont { font_size: 26.0, ..default() },
                TextColor(Color::srgba(0.05, 0.05, 0.1, 0.85)),
            ));
        });
        // main
        parent.spawn((
            Node {
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                align_content: AlignContent::FlexStart,
                row_gap: Val::Px(4.0),
                column_gap: Val::Px(4.0),
                grid_column: GridPlacement::span(2),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.80, 0.92, 0.77)),
        )).with_children(|parent| {
            parent.spawn(Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            }).with_child((
                Text::new("main"),
                TextFont { font_size: 26.0, ..default() },
                TextColor(Color::srgba(0.05, 0.05, 0.1, 0.85)),
            ));
        });
        // footer
        parent.spawn((
            Node {
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                align_content: AlignContent::FlexStart,
                row_gap: Val::Px(4.0),
                column_gap: Val::Px(4.0),
                grid_column: GridPlacement::start_span(1, 3),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.87, 0.80, 0.89)),
        )).with_children(|parent| {
            parent.spawn(Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            }).with_child((
                Text::new("footer"),
                TextFont { font_size: 26.0, ..default() },
                TextColor(Color::srgba(0.05, 0.05, 0.1, 0.85)),
            ));
        });
    });
}
