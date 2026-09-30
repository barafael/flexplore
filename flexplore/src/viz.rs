use bevy::prelude::*;
use bevy_egui::EguiContexts;

use flexplore::art::{ArtState, palette_bevy_color};
use flexplore::bevy_node;
use flexplore::config::{
    BackgroundMode, FlexConfig, NodeConfig, PANEL_WIDTH, RIGHT_PANEL_WIDTH, RightPanelOpen,
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

#[derive(Component)]
pub struct VizTooltipText;

/// The yellow outline drawn around the selected node.
#[derive(Component)]
pub struct SelectionOutline;

/// Frames per second at which animated art textures are re-rendered.
const ART_ANIM_FPS: f32 = 20.0;

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
    if cfg.take_art_regen() {
        art.clear();
    }
    if cfg.bg_mode == BackgroundMode::RandomArt {
        let n = cfg.root.count_leaves();
        art.rebuild(&mut images, cfg.art_style, cfg.art_seed, cfg.art_depth, n);
    } else {
        art.clear();
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
        leaf_idx: 0,
    };
    spawn_node(commands, area, &cfg.root, &mut ctx, &[]);
}

struct SpawnCtx<'a> {
    cfg: &'a FlexConfig,
    art: &'a ArtState,
    leaf_idx: usize,
}

fn spawn_node(
    commands: &mut Commands,
    parent_entity: Entity,
    node: &NodeConfig,
    ctx: &mut SpawnCtx,
    current_path: &[usize],
) {
    let is_leaf = node.children.is_empty();

    let bg_color = if is_leaf {
        if ctx.cfg.bg_mode == BackgroundMode::Pastel {
            palette_bevy_color(ctx.cfg.palette, ctx.leaf_idx)
        } else {
            Color::WHITE
        }
    } else {
        bevy_node::CONTAINER_BG
    };

    let entity = commands
        .spawn((
            bevy_node::node_to_bevy(node),
            bevy_node::node_visibility(node),
            BackgroundColor(bg_color),
            BorderColor::all(bevy_node::BORDER_COLOR),
            Interaction::None,
            VizNodePath(current_path.to_vec()),
            VizNodeInfo(node.info()),
        ))
        .id();
    commands.entity(parent_entity).add_child(entity);

    // The selection outline is added by `viz_selection` (an absolutely
    // positioned child, so it is not clipped like an `Outline` would be).

    if is_leaf {
        let my_idx = ctx.leaf_idx;
        ctx.leaf_idx += 1;
        if ctx.cfg.bg_mode == BackgroundMode::RandomArt
            && let Some(h) = ctx.art.handle(my_idx)
        {
            commands.entity(entity).insert(ImageNode::new(h.clone()));
        }
        bevy_node::spawn_leaf_text(commands, entity, node);
    } else {
        bevy_node::spawn_container_label(commands, entity, node);
        // Sort children by order for visual display, preserving original indices for paths.
        for i in bevy_node::sorted_child_indices(node) {
            let mut child_path = current_path.to_vec();
            child_path.push(i);
            spawn_node(commands, entity, &node.children[i], ctx, &child_path);
        }
    }
}

// ─── Selection outline ────────────────────────────────────────────────────────

/// Keep exactly one outline, parented to the selected node. Runs after
/// `rebuild_viz` so a freshly spawned tree gets its outline in the same frame;
/// moving the selection never rebuilds the tree.
pub fn viz_selection(
    mut commands: Commands,
    cfg: Res<FlexConfig>,
    nodes: Query<(Entity, &VizNodePath)>,
    outlines: Query<(Entity, &ChildOf), With<SelectionOutline>>,
) {
    let target = nodes
        .iter()
        .find(|(_, path)| path.0.as_slice() == cfg.selected())
        .map(|(e, _)| e);

    let mut have_target = false;
    for (outline, parent) in &outlines {
        if Some(parent.parent()) == target && !have_target {
            have_target = true;
        } else {
            commands.entity(outline).despawn();
        }
    }
    if have_target {
        return;
    }
    let Some(target) = target else { return };

    let sel = commands
        .spawn((
            SelectionOutline,
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
    commands.entity(target).add_child(sel);
}

// ─── Tooltip ──────────────────────────────────────────────────────────────────

/// What the tooltip currently shows, so the UI tree is only touched on change.
#[derive(Default)]
pub struct TooltipState {
    entity: Option<Entity>,
    text_entity: Option<Entity>,
    /// `Some((cursor, info))` while shown, `None` while hidden.
    shown: Option<(Vec2, String)>,
}

pub fn viz_tooltip(
    mut commands: Commands,
    windows: Query<&Window>,
    mut contexts: EguiContexts,
    nodes: Query<(&Interaction, &VizNodeInfo, &VizNodePath)>,
    mut tooltip_nodes: Query<&mut Node, With<VizTooltip>>,
    mut tooltip_texts: Query<&mut Text, With<VizTooltipText>>,
    mut state: Local<TooltipState>,
) {
    let egui_owns_pointer = contexts
        .ctx_mut()
        .is_ok_and(|ctx| ctx.is_pointer_over_egui());
    // Hover bubbles to every ancestor, so pick the deepest hovered node.
    let mut hovered: Option<(&str, usize)> = None;
    if !egui_owns_pointer {
        for (interaction, info, path) in &nodes {
            if *interaction == Interaction::Hovered
                && !path.0.is_empty()
                && hovered.is_none_or(|(_, depth)| path.0.len() > depth)
            {
                hovered = Some((&info.0, path.0.len()));
            }
        }
    }
    let hovered_info = hovered.map(|(info, _)| info);

    let Ok(window) = windows.single() else { return };
    let cursor = window.cursor_position();

    let want = match (hovered_info, cursor) {
        (Some(info), Some(cursor)) => Some((cursor, info)),
        _ => None,
    };

    // Unchanged since last frame: nothing to write.
    if state.shown.as_ref().map(|(c, s)| (*c, s.as_str())) == want {
        return;
    }

    match want {
        Some((cursor, info)) => {
            let (Some(entity), Some(text_entity)) = (state.entity, state.text_entity) else {
                // First hover ever: spawn the tooltip.
                let text_id = commands
                    .spawn((
                        VizTooltipText,
                        Text::new(info.to_owned()),
                        TextFont {
                            font_size: FontSize::Px(11.0),
                            ..default()
                        },
                        TextColor(Color::srgba(0.9, 0.9, 0.9, 1.0)),
                    ))
                    .id();
                let entity = commands
                    .spawn((
                        VizTooltip,
                        tooltip_node(cursor, Display::Flex),
                        GlobalZIndex(100),
                        BackgroundColor(Color::srgba(0.12, 0.12, 0.18, 0.95)),
                        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.2)),
                    ))
                    .id();
                commands.entity(entity).add_child(text_id);
                state.entity = Some(entity);
                state.text_entity = Some(text_id);
                state.shown = Some((cursor, info.to_owned()));
                return;
            };
            if let Ok(mut node) = tooltip_nodes.get_mut(entity) {
                let moved = state.shown.as_ref().is_none_or(|(c, _)| *c != cursor);
                if moved || node.display != Display::Flex {
                    *node = tooltip_node(cursor, Display::Flex);
                }
            }
            if state.shown.as_ref().is_none_or(|(_, s)| s != info)
                && let Ok(mut text) = tooltip_texts.get_mut(text_entity)
            {
                text.0 = info.to_owned();
            }
            state.shown = Some((cursor, info.to_owned()));
        }
        None => {
            if let Some(entity) = state.entity
                && let Ok(mut node) = tooltip_nodes.get_mut(entity)
            {
                node.display = Display::None;
            }
            state.shown = None;
        }
    }
}

fn tooltip_node(cursor: Vec2, display: Display) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(cursor.x + 12.0),
        top: Val::Px(cursor.y + 12.0),
        padding: UiRect::all(Val::Px(6.0)),
        border: UiRect::all(Val::Px(1.0)),
        display,
        ..default()
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
    mut contexts: EguiContexts,
    mut cfg: ResMut<FlexConfig>,
) {
    // Bevy UI's `Interaction` does not know about egui windows, popups or the
    // side panels drawn on top of the viz; ignore clicks that land on egui.
    if contexts
        .ctx_mut()
        .is_ok_and(|ctx| ctx.is_pointer_over_egui())
    {
        return;
    }
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

/// Re-render animated art textures on the CPU, at most [`ART_ANIM_FPS`] times
/// per second. `art_anim` is the animation speed; 0 means static.
pub fn animate_art(
    mut images: ResMut<Assets<Image>>,
    art: Res<ArtState>,
    cfg: Res<FlexConfig>,
    time: Res<Time>,
    mut last_t: Local<f32>,
    mut last_render: Local<f32>,
) {
    if cfg.art_anim < 1e-4 || cfg.bg_mode != BackgroundMode::RandomArt {
        return;
    }
    let now = time.elapsed_secs();
    if now - *last_render < 1.0 / ART_ANIM_FPS {
        return;
    }
    let t = (now * cfg.art_anim).sin();
    if (t - *last_t).abs() < 1e-4 {
        return;
    }
    *last_t = t;
    *last_render = now;
    for tex in art.textures() {
        if let Some(mut image) = images.get_mut(&tex.handle) {
            image.data = Some(tex.render(t));
        }
    }
}
