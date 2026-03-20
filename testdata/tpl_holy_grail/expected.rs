fn spawn_ui(commands: &mut Commands) {
    // root
    commands.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Stretch,
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
                flex_shrink: 0.0,
                height: Val::Px(60.0),
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
        // middle
        parent.spawn((
            Node {
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Stretch,
                align_content: AlignContent::FlexStart,
                flex_grow: 1.0,
                min_height: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.11, 0.11, 0.17, 1.0)),
        )).with_children(|parent| {
            // sidebar-left
            parent.spawn((
                Node {
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    align_content: AlignContent::FlexStart,
                    row_gap: Val::Px(4.0),
                    column_gap: Val::Px(4.0),
                    flex_shrink: 0.0,
                    width: Val::Px(120.0),
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
                    Text::new("sidebar-left"),
                    TextFont { font_size: 26.0, ..default() },
                    TextColor(Color::srgba(0.05, 0.05, 0.1, 0.85)),
                ));
            });
            // content
            parent.spawn((
                Node {
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    align_content: AlignContent::FlexStart,
                    row_gap: Val::Px(4.0),
                    column_gap: Val::Px(4.0),
                    flex_grow: 1.0,
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
                    Text::new("content"),
                    TextFont { font_size: 26.0, ..default() },
                    TextColor(Color::srgba(0.05, 0.05, 0.1, 0.85)),
                ));
            });
            // sidebar-right
            parent.spawn((
                Node {
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    align_content: AlignContent::FlexStart,
                    row_gap: Val::Px(4.0),
                    column_gap: Val::Px(4.0),
                    flex_shrink: 0.0,
                    width: Val::Px(120.0),
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
                    Text::new("sidebar-right"),
                    TextFont { font_size: 26.0, ..default() },
                    TextColor(Color::srgba(0.05, 0.05, 0.1, 0.85)),
                ));
            });
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
                flex_shrink: 0.0,
                height: Val::Px(60.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(1.00, 0.85, 0.65)),
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
