use std::fmt::Write;

use anyhow::Result;
use bevy::prelude::*;

use crate::art::palette_color;
use crate::config::{ColorPalette, NodeConfig, ValueConfig};

fn emit_bevy_value(v: &ValueConfig) -> String {
    match v {
        ValueConfig::Auto => "Val::Auto".into(),
        ValueConfig::Px(n) => format!("Val::Px({n:.1})"),
        ValueConfig::Percent(n) => format!("Val::Percent({n:.1})"),
        ValueConfig::Vw(n) => format!("Val::Vw({n:.1})"),
        ValueConfig::Vh(n) => format!("Val::Vh({n:.1})"),
    }
}

pub fn emit_bevy_code(root: &NodeConfig, palette: ColorPalette) -> Result<String> {
    let mut buf = String::from("fn spawn_ui(commands: &mut Commands) {\n");
    emit_node(&mut buf, root, 1, &mut 0, true, palette)?;
    buf.push_str("}\n");
    Ok(buf)
}

fn emit_node(
    buf: &mut String,
    node: &NodeConfig,
    depth: usize,
    leaf_idx: &mut usize,
    is_root: bool,
    palette: ColorPalette,
) -> Result<()> {
    let pad = "    ".repeat(depth);
    let is_leaf = node.children.is_empty();

    let bg = if is_leaf {
        let (r, g, b) = palette_color(palette, *leaf_idx);
        *leaf_idx += 1;
        format!("Color::srgb({r:.2}, {g:.2}, {b:.2})")
    } else {
        "Color::srgba(0.11, 0.11, 0.17, 1.0)".into()
    };

    let spawner = if is_root { "commands" } else { "parent" };
    writeln!(buf, "{pad}// {}", node.label)?;
    writeln!(buf, "{pad}{spawner}.spawn((")?;

    writeln!(buf, "{pad}    Node {{")?;
    // Only emit non-default fields. Bevy defaults: Display::Flex, Row, NoWrap,
    // JustifyContent/AlignItems/AlignContent::Default, gaps Px(0), grow 0, shrink 1,
    // basis/sizes Auto, padding/margin zero.
    if node.flex_direction != FlexDirection::Row {
        emit_field(
            buf,
            &pad,
            "flex_direction",
            &format!("FlexDirection::{:?}", node.flex_direction),
        )?;
    }
    if node.flex_wrap != FlexWrap::NoWrap {
        emit_field(
            buf,
            &pad,
            "flex_wrap",
            &format!("FlexWrap::{:?}", node.flex_wrap),
        )?;
    }
    if !matches!(node.justify_content, JustifyContent::Default) {
        emit_field(
            buf,
            &pad,
            "justify_content",
            &format!("JustifyContent::{:?}", node.justify_content),
        )?;
    }
    if !matches!(node.align_items, AlignItems::Default) {
        emit_field(
            buf,
            &pad,
            "align_items",
            &format!("AlignItems::{:?}", node.align_items),
        )?;
    }
    if !matches!(node.align_content, AlignContent::Default) {
        emit_field(
            buf,
            &pad,
            "align_content",
            &format!("AlignContent::{:?}", node.align_content),
        )?;
    }
    if !matches!(node.row_gap, ValueConfig::Auto)
        && !matches!(node.row_gap, ValueConfig::Px(v) if v == 0.0)
    {
        emit_field(buf, &pad, "row_gap", &emit_bevy_value(&node.row_gap))?;
    }
    if !matches!(node.column_gap, ValueConfig::Auto)
        && !matches!(node.column_gap, ValueConfig::Px(v) if v == 0.0)
    {
        emit_field(buf, &pad, "column_gap", &emit_bevy_value(&node.column_gap))?;
    }
    if node.flex_grow != 0.0 {
        emit_field(buf, &pad, "flex_grow", &format!("{:.1}", node.flex_grow))?;
    }
    if node.flex_shrink != 1.0 {
        emit_field(
            buf,
            &pad,
            "flex_shrink",
            &format!("{:.1}", node.flex_shrink),
        )?;
    }
    if !matches!(node.flex_basis, ValueConfig::Auto) {
        emit_field(buf, &pad, "flex_basis", &emit_bevy_value(&node.flex_basis))?;
    }
    if node.align_self != AlignSelf::Auto {
        emit_field(
            buf,
            &pad,
            "align_self",
            &format!("AlignSelf::{:?}", node.align_self),
        )?;
    }
    if !matches!(node.width, ValueConfig::Auto) {
        emit_field(buf, &pad, "width", &emit_bevy_value(&node.width))?;
    }
    if !matches!(node.height, ValueConfig::Auto) {
        emit_field(buf, &pad, "height", &emit_bevy_value(&node.height))?;
    }
    if !matches!(node.min_width, ValueConfig::Auto) {
        emit_field(buf, &pad, "min_width", &emit_bevy_value(&node.min_width))?;
    }
    if !matches!(node.min_height, ValueConfig::Auto) {
        emit_field(buf, &pad, "min_height", &emit_bevy_value(&node.min_height))?;
    }
    if !matches!(node.max_width, ValueConfig::Auto) {
        emit_field(buf, &pad, "max_width", &emit_bevy_value(&node.max_width))?;
    }
    if !matches!(node.max_height, ValueConfig::Auto) {
        emit_field(buf, &pad, "max_height", &emit_bevy_value(&node.max_height))?;
    }
    if !matches!(node.padding, ValueConfig::Px(v) if v == 0.0) {
        emit_field(
            buf,
            &pad,
            "padding",
            &format!("UiRect::all({})", emit_bevy_value(&node.padding)),
        )?;
    }
    if !matches!(node.margin, ValueConfig::Px(v) if v == 0.0) {
        emit_field(
            buf,
            &pad,
            "margin",
            &format!("UiRect::all({})", emit_bevy_value(&node.margin)),
        )?;
    }
    if node.order != 0 {
        writeln!(
            buf,
            "{pad}        // order: {} (no Bevy equivalent, use entity ordering)",
            node.order
        )?;
    }
    writeln!(buf, "{pad}        ..default()")?;
    writeln!(buf, "{pad}    }},")?;

    writeln!(buf, "{pad}    BackgroundColor({bg}),")?;
    if !node.visible {
        writeln!(buf, "{pad}    Visibility::Hidden,")?;
    }
    write!(buf, "{pad}))")?;

    if is_leaf {
        buf.push_str(".with_children(|parent| {\n");
        writeln!(buf, "{pad}    parent.spawn(Node {{")?;
        writeln!(buf, "{pad}        position_type: PositionType::Absolute,")?;
        writeln!(buf, "{pad}        top: Val::Px(0.0),")?;
        writeln!(buf, "{pad}        left: Val::Px(0.0),")?;
        writeln!(buf, "{pad}        right: Val::Px(0.0),")?;
        writeln!(buf, "{pad}        bottom: Val::Px(0.0),")?;
        writeln!(buf, "{pad}        justify_content: JustifyContent::Center,")?;
        writeln!(buf, "{pad}        align_items: AlignItems::Center,")?;
        writeln!(buf, "{pad}        ..default()")?;
        writeln!(buf, "{pad}    }}).with_child((")?;
        writeln!(buf, "{pad}        Text::new({:?}),", node.label)?;
        writeln!(
            buf,
            "{pad}        TextFont {{ font_size: 26.0, ..default() }},"
        )?;
        writeln!(
            buf,
            "{pad}        TextColor(Color::srgba(0.05, 0.05, 0.1, 0.85)),"
        )?;
        writeln!(buf, "{pad}    ));")?;
        writeln!(buf, "{pad}}});")?;
    } else {
        buf.push_str(".with_children(|parent| {\n");
        let mut sorted: Vec<&NodeConfig> = node.children.iter().collect();
        sorted.sort_by_key(|c| c.order);
        for child in sorted {
            emit_node(buf, child, depth + 1, leaf_idx, false, palette)?;
        }
        writeln!(buf, "{pad}}});")?;
    }
    Ok(())
}

fn emit_field(buf: &mut String, pad: &str, name: &str, value: &str) -> Result<()> {
    writeln!(buf, "{pad}        {name}: {value},")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_container() -> NodeConfig {
        let mut root = NodeConfig::new_container("root");
        root.children = vec![
            NodeConfig::new_leaf("A", 80.0, 80.0),
            NodeConfig::new_leaf("B", 120.0, 100.0),
        ];
        root
    }

    #[test]
    fn emits_spawn_function() {
        let code = emit_bevy_code(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains("fn spawn_ui(commands: &mut Commands)"));
    }

    #[test]
    fn emits_flex_direction() {
        let mut node = NodeConfig::new_container("root");
        node.flex_direction = FlexDirection::Column;
        node.children = vec![NodeConfig::new_leaf("A", 80.0, 80.0)];
        let code = emit_bevy_code(&node, ColorPalette::Pastel1).unwrap();
        assert!(code.contains("FlexDirection::Column"));
    }

    #[test]
    fn emits_leaf_text() {
        let code = emit_bevy_code(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains("Text::new(\"A\")"));
        assert!(code.contains("Text::new(\"B\")"));
    }

    #[test]
    fn emits_visibility_hidden_when_not_visible() {
        let mut node = NodeConfig::new_leaf("hidden", 80.0, 80.0);
        node.visible = false;
        let mut root = NodeConfig::new_container("root");
        root.children = vec![node];
        let code = emit_bevy_code(&root, ColorPalette::Pastel1).unwrap();
        assert!(
            code.contains("Visibility::Hidden"),
            "should use Visibility::Hidden, not Display::None"
        );
        assert!(
            !code.contains("Display::None"),
            "should not use Display::None"
        );
    }

    #[test]
    fn emits_value_types() {
        let mut leaf = NodeConfig::new_leaf("A", 80.0, 80.0);
        leaf.width = ValueConfig::Percent(50.0);
        let mut root = NodeConfig::new_container("root");
        root.children = vec![leaf];
        let code = emit_bevy_code(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains("Val::Percent(50.0)"));
    }
}
