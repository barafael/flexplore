use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_egui::EguiContexts;

use crate::art::{ArtExpressions, ArtState, pastel, render_art};
use crate::config::*;

// ─── Components ───────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct VizRoot;

#[derive(Component)]
#[allow(dead_code)]
pub struct ArtItemNode(usize);

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
    roots: Query<Entity, With<VizRoot>>,
) {
    if !cfg.needs_rebuild {
        return;
    }
    cfg.needs_rebuild = false;
    for e in &roots {
        commands.entity(e).despawn();
    }
    art.exprs.clear();
    art.handles.clear();
    if cfg.bg_mode == BackgroundMode::RandomArt {
        let n = count_leaves(&cfg.root);
        let (base, depth, style) = (cfg.art_seed, cfg.art_depth, cfg.art_style.clone());
        for i in 0..n {
            let iseed = base.wrapping_add((i as u64).wrapping_mul(0x9e3779b97f4a7c15));
            let exprs = ArtExpressions::generate(iseed, depth);
            let pixels = render_art(&style, &exprs, iseed, 0.0);
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
            art.exprs.push(exprs);
        }
    }
    spawn_viz(&mut commands, &cfg, &art);
}

fn spawn_viz(commands: &mut Commands, cfg: &FlexConfig, art: &ArtState) {
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
    commands.entity(viz_root).add_children(&[spacer, area]);

    let mut leaf_idx = 0usize;
    spawn_node(
        commands,
        area,
        &cfg.root,
        cfg,
        art,
        &cfg.selected,
        &[],
        &mut leaf_idx,
    );
}

fn spawn_node(
    commands: &mut Commands,
    parent_entity: Entity,
    node: &NodeConfig,
    cfg: &FlexConfig,
    art: &ArtState,
    selected_path: &[usize],
    current_path: &[usize],
    leaf_idx: &mut usize,
) {
    let is_selected = current_path == selected_path;
    let is_leaf = node.children.is_empty();

    let (border_width, border_color) = if is_selected {
        (3.0, Color::srgba(1.0, 0.85, 0.1, 1.0))
    } else {
        (1.5, Color::srgba(0.0, 0.0, 0.0, 0.35))
    };

    let bg_color = if is_leaf {
        if cfg.bg_mode == BackgroundMode::Pastel {
            pastel(*leaf_idx)
        } else {
            Color::WHITE
        }
    } else {
        Color::srgba(0.11, 0.11, 0.17, 1.0)
    };

    let node_bevy = Node {
        display: Display::Flex,
        flex_direction: node.flex_direction,
        flex_wrap: node.flex_wrap,
        justify_content: node.justify_content,
        align_items: node.align_items,
        align_content: node.align_content,
        row_gap: node.row_gap.to_val(),
        column_gap: node.column_gap.to_val(),
        flex_grow: node.flex_grow,
        flex_shrink: node.flex_shrink,
        flex_basis: node.flex_basis.to_val(),
        align_self: node.align_self,
        width: node.width.to_val(),
        height: node.height.to_val(),
        min_width: node.min_width.to_val(),
        min_height: node.min_height.to_val(),
        max_width: node.max_width.to_val(),
        max_height: node.max_height.to_val(),
        padding: UiRect::all(node.padding.to_val()),
        margin: UiRect::all(node.margin.to_val()),
        border: UiRect::all(Val::Px(border_width)),
        overflow: Overflow::clip(),
        ..default()
    };

    if is_leaf {
        let my_idx = *leaf_idx;
        *leaf_idx += 1;
        let entity = commands
            .spawn((
                ArtItemNode(my_idx),
                node_bevy,
                BackgroundColor(bg_color),
                BorderColor::all(border_color),
                Interaction::None,
                VizNodePath(current_path.to_vec()),
                VizNodeInfo(node_info(node)),
            ))
            .id();
        if cfg.bg_mode == BackgroundMode::RandomArt
            && let Some(h) = art.handles.get(my_idx)
        {
            commands.entity(entity).insert(ImageNode::new(h.clone()));
        }
        let scale = text_scale(node);
        let overlay = commands
            .spawn(Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_child((
                Text::new(node.label.clone()),
                TextFont {
                    font_size: (26.0_f32 * scale).clamp(1.0, 52.0),
                    ..default()
                },
                TextColor(Color::srgba(0.05, 0.05, 0.1, 0.85)),
            ))
            .id();
        commands.entity(entity).add_child(overlay);
        commands.entity(parent_entity).add_child(entity);
    } else {
        let entity = commands
            .spawn((
                node_bevy,
                BackgroundColor(bg_color),
                BorderColor::all(border_color),
                Interaction::None,
                VizNodePath(current_path.to_vec()),
                VizNodeInfo(node_info(node)),
            ))
            .id();
        let cscale = text_scale(node);
        let lbl = commands
            .spawn((
                Text::new(node.label.clone()),
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
            ))
            .id();
        commands.entity(entity).add_child(lbl);
        commands.entity(parent_entity).add_child(entity);
        for (i, child) in node.children.iter().enumerate() {
            let mut child_path = current_path.to_vec();
            child_path.push(i);
            spawn_node(
                commands,
                entity,
                child,
                cfg,
                art,
                selected_path,
                &child_path,
                leaf_idx,
            );
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

// ─── Click-to-select ──────────────────────────────────────────────────────────

pub fn viz_click(
    nodes: Query<(&Interaction, &VizNodePath), Changed<Interaction>>,
    mut cfg: ResMut<FlexConfig>,
) {
    for (interaction, path) in &nodes {
        if *interaction == Interaction::Pressed && cfg.selected != path.0 {
            cfg.selected = path.0.clone();
            cfg.needs_rebuild = true;
        }
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
    for (exprs, handle) in art.exprs.iter().zip(art.handles.iter()) {
        if let Some(image) = images.get_mut(handle) {
            image.data = Some(exprs.render(ART_TEXTURE_SIZE, ART_TEXTURE_SIZE, t));
        }
    }
}
