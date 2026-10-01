//! Shared `NodeConfig` → Bevy UI conversion.
//!
//! Used by both the live preview (`viz.rs` in the app) and the golden
//! renderer (`render.rs`, driven by `tools/bevy-golden`), so that the golden
//! screenshots show exactly the layout the app shows.

use bevy::prelude::*;

use crate::config::{DisplayMode, NodeConfig};

/// Background colour of container nodes (and of the viewport).
pub const CONTAINER_BG: Color = Color::srgba(0.11, 0.11, 0.17, 1.0);
/// Colour of user-defined borders.
pub const BORDER_COLOR: Color = Color::srgba(0.4, 0.4, 0.5, 0.8);
/// Colour of the text centred inside leaf nodes.
pub const LEAF_TEXT_COLOR: Color = Color::srgba(0.05, 0.05, 0.1, 0.85);
/// Colour of the small label in the top-left corner of container nodes.
pub const CONTAINER_LABEL_COLOR: Color = Color::srgba(0.7, 0.7, 0.9, 0.55);

/// Convert every layout-relevant field of a `NodeConfig` into a Bevy `Node`.
///
/// Hidden nodes keep their layout space (see [`node_visibility`]), matching
/// `visibility: hidden` in every code generator.
pub fn node_to_bevy(node: &NodeConfig) -> Node {
    let mut style = node_to_bevy_inner(node);
    if node.children.is_empty() {
        // Leaves centre their text, like every generated target does.
        style.justify_content = JustifyContent::Center;
        style.align_items = AlignItems::Center;
    }
    style
}

fn node_to_bevy_inner(node: &NodeConfig) -> Node {
    Node {
        display: match node.display_mode {
            DisplayMode::Grid => Display::Grid,
            DisplayMode::Flex => Display::Flex,
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
        grid_template_columns: node
            .grid_template_columns
            .iter()
            .map(|t| t.to_bevy_repeated_grid_track())
            .collect(),
        grid_template_rows: node
            .grid_template_rows
            .iter()
            .map(|t| t.to_bevy_repeated_grid_track())
            .collect(),
        grid_auto_columns: node
            .grid_auto_columns
            .iter()
            .map(|t| t.to_bevy_grid_track())
            .collect(),
        grid_auto_rows: node
            .grid_auto_rows
            .iter()
            .map(|t| t.to_bevy_grid_track())
            .collect(),
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
        border: node.border_width.to_bevy_ui_rect(),
        border_radius: node.border_radius.to_bevy_border_radius(),
        overflow: Overflow::clip(),
        ..default()
    }
}

/// `Visibility::Hidden` for invisible nodes: they still take up space, like
/// CSS `visibility: hidden` (and unlike `display: none`).
pub fn node_visibility(node: &NodeConfig) -> Visibility {
    if node.visible {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    }
}

/// Font size of the text centred inside a leaf node.
pub fn leaf_font_size(node: &NodeConfig) -> f32 {
    (26.0_f32 * node.text_scale()).clamp(1.0, 52.0)
}

/// Font size of the small label in the top-left corner of a container node.
pub fn container_label_font_size(node: &NodeConfig) -> f32 {
    (10.0_f32 * node.text_scale()).clamp(1.0, 20.0)
}

/// Spawn the centred text overlay of a leaf node as a child of `entity`,
/// sized for the app's live preview (text shrinks to fit small nodes).
pub fn spawn_leaf_text(commands: &mut Commands, entity: Entity, node: &NodeConfig) {
    spawn_leaf_text_sized(commands, entity, node, leaf_font_size(node));
}

/// Spawn the centred text overlay of a leaf node with an explicit font size.
/// The golden renderer uses the fixed 26 px the generated code uses, so its
/// screenshots are comparable with the other backends.
pub fn spawn_leaf_text_sized(
    commands: &mut Commands,
    entity: Entity,
    node: &NodeConfig,
    font_size: f32,
) {
    // In-flow (not an absolute overlay) so an auto-sized leaf grows to fit
    // its text, exactly like the `display: flex; align-items: center;
    // justify-content: center` leaf the code generators emit.
    let text = commands
        .spawn((
            Text::new(node.display_text()),
            TextFont {
                font_size: FontSize::Px(font_size),
                ..default()
            },
            TextColor(LEAF_TEXT_COLOR),
            Pickable::IGNORE,
        ))
        .id();
    commands.entity(entity).add_child(text);
}

/// Spawn the small corner label of a container node as a child of `entity`.
pub fn spawn_container_label(commands: &mut Commands, entity: Entity, node: &NodeConfig) {
    let label = commands
        .spawn((
            Text::new(node.display_text()),
            TextFont {
                font_size: FontSize::Px(container_label_font_size(node)),
                ..default()
            },
            TextColor(CONTAINER_LABEL_COLOR),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(2.0),
                left: Val::Px(4.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    commands.entity(entity).add_child(label);
}

/// Children indices sorted by `order` (stable, so ties keep tree order).
pub fn sorted_child_indices(node: &NodeConfig) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..node.children.len()).collect();
    idx.sort_by_key(|&i| node.children[i].order);
    idx
}
