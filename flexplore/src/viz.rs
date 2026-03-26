use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_egui::EguiContexts;

use flexplore::art::{ArtExpressions, ArtState, palette_bevy_color, render_art};
use flexplore::config::{
    ART_TEXTURE_SIZE, BackgroundMode, DisplayMode, FlexConfig, NodeConfig, PANEL_WIDTH,
    RIGHT_PANEL_WIDTH, RightPanelOpen,
};

// ─── Components ───────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct VizRoot;

#[derive(Component)]
pub struct VizNodePath(pub Vec<usize>);

#[derive(Component)]
pub struct VizNodeInfo(pub String);

#[derive(Component)]
pub struct VizTooltip;

// ─── Rebuild ──────────────────────────────────────────────────────────────────

pub fn rebuild_viz(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut cfg: ResMut<FlexConfig>,
    mut art: ResMut<ArtState>,
    right_panel: Res<RightPanelOpen>,
    roots: Query<Entity, With<VizRoot>>,
) {
    if !cfg.take_rebuild() {
        return;
    }
    for e in &roots {
        commands.entity(e).despawn();
    }
    art.exprs.clear();
    art.seeds.clear();
    art.handles.clear();
    if cfg.bg_mode == BackgroundMode::RandomArt {
        let n = cfg.root.count_leaves();
        let (base, depth, style) = (cfg.art_seed, cfg.art_depth, cfg.art_style);
        for i in 0..n {
            let iseed = base.wrapping_add((i as u64).wrapping_mul(0x9e3779b97f4a7c15));
            let exprs = ArtExpressions::generate(iseed, depth);
            let pixels = render_art(style, &exprs, iseed, 0.0);
            let image = Image::new(
                Extent3d {
                    width: ART_TEXTURE_SIZE,
                    height: ART_TEXTURE_SIZE,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                pixels,
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            );
            art.handles.push(images.add(image));
            art.seeds.push(iseed);
            art.exprs.push(exprs);
        }
    }
    spawn_viz(&mut commands, &cfg, &art, right_panel.0);
}

fn spawn_viz(commands: &mut Commands, cfg: &FlexConfig, art: &ArtState, right_open: bool) {
    let viz_root = commands
        .spawn((
            VizRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Stretch,
                ..default()
            },
        ))
        .id();

    let spacer = commands
        .spawn(Node {
            width: Val::Px(PANEL_WIDTH),
            flex_shrink: 0.0,
            ..default()
        })
        .id();
    let area = commands
        .spawn(Node {
            flex_grow: 1.0,
            height: Val::Percent(100.0),
            display: Display::Block,
            padding: UiRect::all(Val::Px(16.0)),
            ..default()
        })
        .id();

    let mut children = vec![spacer, area];

    if right_open {
        let right_spacer = commands
            .spawn(Node {
                width: Val::Px(RIGHT_PANEL_WIDTH),
                flex_shrink: 0.0,
                ..default()
            })
            .id();
        children.push(right_spacer);
    }

    commands.entity(viz_root).add_children(&children);

    let mut ctx = SpawnCtx {
        cfg,
        art,
        selected_path: cfg.selected(),
        leaf_idx: 0,
    };
    spawn_node(commands, area, &cfg.root, &mut ctx, &[]);
}

struct SpawnCtx<'a> {
    cfg: &'a FlexConfig,
    art: &'a ArtState,
    selected_path: &'a [usize],
    leaf_idx: usize,
}

fn spawn_node(
    commands: &mut Commands,
    parent_entity: Entity,
    node: &NodeConfig,
    ctx: &mut SpawnCtx,
    current_path: &[usize],
) {
    let is_selected = current_path == ctx.selected_path;
    let is_leaf = node.children.is_empty();

    let bg_color = if is_leaf {
        if ctx.cfg.bg_mode == BackgroundMode::Pastel {
            palette_bevy_color(ctx.cfg.palette, ctx.leaf_idx)
        } else {
            Color::WHITE
        }
    } else {
        Color::srgba(0.11, 0.11, 0.17, 1.0)
    };

    // User-defined border + selection highlight via outline
    let user_border = node.border_width.to_bevy_ui_rect();
    let user_border_color = Color::srgba(0.4, 0.4, 0.5, 0.8);

    let node_bevy = {
        let mut n = Node {
            display: if !node.visible {
                Display::None
            } else {
                match node.display_mode {
                    DisplayMode::Grid => Display::Grid,
                    DisplayMode::Flex => Display::Flex,
                }
            },
            // Flex container
            flex_direction: node.flex_direction.into(),
            flex_wrap: node.flex_wrap.into(),
            justify_content: node.justify_content.into(),
            align_items: node.align_items.into(),
            align_content: node.align_content.into(),
            row_gap: node.row_gap.to_bevy_val(),
            column_gap: node.column_gap.to_bevy_val(),
            // Flex item
            flex_grow: node.flex_grow,
            flex_shrink: node.flex_shrink,
            flex_basis: node.flex_basis.to_bevy_val(),
            align_self: node.align_self.into(),
            // Grid
            grid_auto_flow: node.grid_auto_flow.to_bevy(),
            grid_column: node.grid_column.to_bevy(),
            grid_row: node.grid_row.to_bevy(),
            // Sizing
            width: node.width.to_bevy_val(),
            height: node.height.to_bevy_val(),
            min_width: node.min_width.to_bevy_val(),
            min_height: node.min_height.to_bevy_val(),
            max_width: node.max_width.to_bevy_val(),
            max_height: node.max_height.to_bevy_val(),
            // Spacing
            padding: node.padding.to_bevy_ui_rect(),
            margin: node.margin.to_bevy_ui_rect(),
            border: user_border,
            border_radius: node.border_radius.to_bevy_border_radius(),
            overflow: Overflow::clip(),
            ..default()
        };
        // Grid template tracks
        n.grid_template_columns = node
            .grid_template_columns
            .iter()
            .map(|t| t.to_bevy_repeated_grid_track())
            .collect();
        n.grid_template_rows = node
            .grid_template_rows
            .iter()
            .map(|t| t.to_bevy_repeated_grid_track())
            .collect();
        n.grid_auto_columns = node
            .grid_auto_columns
            .iter()
            .map(|t| t.to_bevy_grid_track())
            .collect();
        n.grid_auto_rows = node
            .grid_auto_rows
            .iter()
            .map(|t| t.to_bevy_grid_track())
            .collect();
        n
    };

    // Selection highlight: an absolutely-positioned child with a colored border
    // and GlobalZIndex so it always renders on top. We use this instead of
    // Outline because Outline extends outward and gets clipped by the parent's
    // `overflow: clip()`.

    let display_text = node.display_text().to_owned();

    if is_leaf {
        let my_idx = ctx.leaf_idx;
        ctx.leaf_idx += 1;
        let entity = commands
            .spawn((
                node_bevy,
                BackgroundColor(bg_color),
                BorderColor::all(user_border_color),
                Interaction::None,
                VizNodePath(current_path.to_vec()),
                VizNodeInfo(node.info()),
            ))
            .id();
        if ctx.cfg.bg_mode == BackgroundMode::RandomArt
            && let Some(h) = ctx.art.handles.get(my_idx)
        {
            commands.entity(entity).insert(ImageNode::new(h.clone()));
        }
        let scale = node.text_scale();
        let overlay = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_child((
                Text::new(display_text),
                TextFont {
                    font_size: (26.0_f32 * scale).clamp(1.0, 52.0),
                    ..default()
                },
                TextColor(Color::srgba(0.05, 0.05, 0.1, 0.85)),
                Pickable::IGNORE,
            ))
            .id();
        commands.entity(entity).add_child(overlay);
        if is_selected {
            let sel = commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(0.0),
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                        bottom: Val::Px(0.0),
                        border: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    GlobalZIndex(99),
                    BorderColor::all(Color::srgba(1.0, 0.85, 0.1, 1.0)),
                    Pickable::IGNORE,
                ))
                .id();
            commands.entity(entity).add_child(sel);
        }
        commands.entity(parent_entity).add_child(entity);
    } else {
        let entity = commands
            .spawn((
                node_bevy,
                BackgroundColor(bg_color),
                BorderColor::all(user_border_color),
                Interaction::None,
                VizNodePath(current_path.to_vec()),
                VizNodeInfo(node.info()),
            ))
            .id();
        let cscale = node.text_scale();
        let lbl = commands
            .spawn((
                Text::new(display_text),
                TextFont {
                    font_size: (10.0_f32 * cscale).clamp(1.0, 20.0),
                    ..default()
                },
                TextColor(Color::srgba(0.7, 0.7, 0.9, 0.55)),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(2.0),
                    left: Val::Px(4.0),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .id();
        commands.entity(entity).add_child(lbl);
        if is_selected {
            let sel = commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(0.0),
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                        bottom: Val::Px(0.0),
                        border: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    GlobalZIndex(99),
                    BorderColor::all(Color::srgba(1.0, 0.85, 0.1, 1.0)),
                    Pickable::IGNORE,
                ))
                .id();
            commands.entity(entity).add_child(sel);
        }
        commands.entity(parent_entity).add_child(entity);
        // Sort children by order for visual display, preserving original indices for paths.
        let mut sorted_indices: Vec<usize> = (0..node.children.len()).collect();
        sorted_indices.sort_by_key(|&i| node.children[i].order);
        for i in sorted_indices {
            let child = &node.children[i];
            let mut child_path = current_path.to_vec();
            child_path.push(i);
            spawn_node(commands, entity, child, ctx, &child_path);
        }
    }
}

// ─── Tooltip ──────────────────────────────────────────────────────────────────

pub fn viz_tooltip(
    mut commands: Commands,
    windows: Query<&Window>,
    mut contexts: EguiContexts,
    nodes: Query<(&Interaction, &VizNodeInfo, &VizNodePath)>,
    mut tooltip_entity: Local<Option<Entity>>,
    mut tooltip_text: Local<Option<Entity>>,
) {
    let egui_owns_pointer = contexts
        .ctx_mut()
        .is_ok_and(|ctx| ctx.is_pointer_over_area());
    let mut hovered_info: Option<&str> = None;
    if !egui_owns_pointer {
        for (interaction, info, path) in &nodes {
            if *interaction == Interaction::Hovered && !path.0.is_empty() {
                hovered_info = Some(&info.0);
            }
        }
    }

    let Ok(window) = windows.single() else { return };
    let cursor = window.cursor_position();

    if let (Some(info), Some(cursor)) = (hovered_info, cursor) {
        if let Some(entity) = *tooltip_entity {
            commands.entity(entity).insert(Node {
                position_type: PositionType::Absolute,
                left: Val::Px(cursor.x + 12.0),
                top: Val::Px(cursor.y + 12.0),
                padding: UiRect::all(Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.0)),
                display: Display::Flex,
                ..default()
            });
            if let Some(text_entity) = *tooltip_text {
                commands
                    .entity(text_entity)
                    .insert(Text::new(info.to_owned()));
            }
        } else {
            let text_id = commands
                .spawn((
                    Text::new(info.to_owned()),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.9, 0.9, 0.9, 1.0)),
                ))
                .id();
            let entity = commands
                .spawn((
                    VizTooltip,
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(cursor.x + 12.0),
                        top: Val::Px(cursor.y + 12.0),
                        padding: UiRect::all(Val::Px(6.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    GlobalZIndex(100),
                    BackgroundColor(Color::srgba(0.12, 0.12, 0.18, 0.95)),
                    BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.2)),
                ))
                .id();
            commands.entity(entity).add_child(text_id);
            *tooltip_entity = Some(entity);
            *tooltip_text = Some(text_id);
        }
    } else if let Some(entity) = *tooltip_entity {
        commands.entity(entity).insert(Node {
            display: Display::None,
            ..default()
        });
    }
}

// ─── Arrow-key spatial navigation ─────────────────────────────────────────

/// Direction request produced by the egui panel (which owns keyboard input)
/// and consumed here (which owns the GlobalTransform positions).
#[derive(Resource, Default)]
pub struct ArrowNav(pub Option<Vec2>);

pub fn viz_arrow_nav(
    mut nav: ResMut<ArrowNav>,
    nodes: Query<(&UiGlobalTransform, &VizNodePath)>,
    mut cfg: ResMut<FlexConfig>,
) {
    let Some(dir) = nav.0.take() else { return };

    let selected = cfg.selected();
    if selected.is_empty() {
        return;
    }

    let Some(sel_pos) = nodes
        .iter()
        .find(|(_, path)| path.0.as_slice() == selected)
        .map(|(gt, _)| gt.translation)
    else {
        return;
    };

    let mut best: Option<(f32, &Vec<usize>)> = None;
    for (gt, path) in &nodes {
        if path.0.as_slice() == selected || path.0.is_empty() {
            continue;
        }
        let pos = gt.translation;
        let offset = pos - sel_pos;
        let forward = offset.dot(dir);
        if forward < 1.0 {
            continue;
        }
        let lateral = (offset - forward * dir).length();
        // Prefer close nodes in the arrow direction; penalise off-axis drift.
        let cost = forward + lateral * 2.0;
        if best.is_none_or(|(b, _)| cost < b) {
            best = Some((cost, &path.0));
        }
    }

    if let Some((_, path)) = best {
        cfg.select(path.clone());
    }
}

// ─── Click-to-select ──────────────────────────────────────────────────────────

pub fn viz_click(
    nodes: Query<(&Interaction, &VizNodePath), Changed<Interaction>>,
    mut cfg: ResMut<FlexConfig>,
) {
    // Pick the deepest pressed node — clicks bubble up to ancestors,
    // so multiple nodes report Pressed simultaneously.
    let mut best: Option<&Vec<usize>> = None;
    for (interaction, path) in &nodes {
        if *interaction == Interaction::Pressed && best.is_none_or(|b| path.0.len() > b.len()) {
            best = Some(&path.0);
        }
    }
    if let Some(path) = best
        && cfg.selected() != *path
    {
        cfg.select(path.clone());
    }
}

// ─── Animation ────────────────────────────────────────────────────────────────

pub fn animate_art(
    mut images: ResMut<Assets<Image>>,
    art: Res<ArtState>,
    cfg: Res<FlexConfig>,
    time: Res<Time>,
    mut last_t: Local<f32>,
) {
    if cfg.art_anim < 1e-4 || cfg.bg_mode != BackgroundMode::RandomArt {
        return;
    }
    let t = (time.elapsed_secs() * cfg.art_anim).sin();
    if (t - *last_t).abs() < 1e-4 {
        return;
    }
    *last_t = t;
    for ((exprs, handle), seed) in art
        .exprs
        .iter()
        .zip(art.handles.iter())
        .zip(art.seeds.iter())
    {
        if let Some(image) = images.get_mut(handle) {
            image.data = Some(render_art(cfg.art_style, exprs, *seed, t));
        }
    }
}
