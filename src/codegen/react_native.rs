use std::fmt::Write;

use anyhow::Result;
use bevy::prelude::*;

use crate::art::palette_color;
use crate::config::{ColorPalette, NodeConfig, ValueConfig};

fn format_num(v: f32) -> String {
    if (v - v.round()).abs() < 0.005 {
        format!("{}", v as i32)
    } else {
        format!("{v:.1}")
    }
}

/// React Native uses plain numbers for dp, strings for percentages.
/// vw/vh have no native equivalent — we emit a `Dimensions` expression.
fn rn_value(v: &ValueConfig) -> String {
    match v {
        ValueConfig::Auto => "'auto'".into(),
        ValueConfig::Px(n) => format_num(*n),
        ValueConfig::Percent(n) => format!("'{n:.1}%'"),
        ValueConfig::Vw(n) => format!("Dimensions.get('window').width * {:.2}", n / 100.0),
        ValueConfig::Vh(n) => format!("Dimensions.get('window').height * {:.2}", n / 100.0),
    }
}

fn rn_direction(d: FlexDirection) -> &'static str {
    match d {
        FlexDirection::Row => "'row'",
        FlexDirection::Column => "'column'",
        FlexDirection::RowReverse => "'row-reverse'",
        FlexDirection::ColumnReverse => "'column-reverse'",
    }
}

fn rn_wrap(w: FlexWrap) -> &'static str {
    match w {
        FlexWrap::NoWrap => "'nowrap'",
        FlexWrap::Wrap => "'wrap'",
        FlexWrap::WrapReverse => "'wrap-reverse'",
    }
}

fn rn_justify(j: JustifyContent) -> &'static str {
    match j {
        JustifyContent::FlexStart => "'flex-start'",
        JustifyContent::FlexEnd => "'flex-end'",
        JustifyContent::Center => "'center'",
        JustifyContent::SpaceBetween => "'space-between'",
        JustifyContent::SpaceAround => "'space-around'",
        JustifyContent::SpaceEvenly => "'space-evenly'",
        JustifyContent::Start => "'flex-start'",
        JustifyContent::End => "'flex-end'",
        _ => "'flex-start'",
    }
}

fn rn_align_items(a: AlignItems) -> &'static str {
    match a {
        AlignItems::FlexStart => "'flex-start'",
        AlignItems::FlexEnd => "'flex-end'",
        AlignItems::Center => "'center'",
        AlignItems::Baseline => "'baseline'",
        AlignItems::Stretch => "'stretch'",
        AlignItems::Start => "'flex-start'",
        AlignItems::End => "'flex-end'",
        _ => "'stretch'",
    }
}

fn rn_align_content(a: AlignContent) -> &'static str {
    match a {
        AlignContent::FlexStart => "'flex-start'",
        AlignContent::FlexEnd => "'flex-end'",
        AlignContent::Center => "'center'",
        AlignContent::SpaceBetween => "'space-between'",
        AlignContent::SpaceAround => "'space-around'",
        AlignContent::SpaceEvenly => "'space-evenly'",
        AlignContent::Stretch => "'stretch'",
        AlignContent::Start => "'flex-start'",
        AlignContent::End => "'flex-end'",
        _ => "'flex-start'",
    }
}

fn rn_align_self(a: AlignSelf) -> &'static str {
    match a {
        AlignSelf::Auto => "'auto'",
        AlignSelf::FlexStart => "'flex-start'",
        AlignSelf::FlexEnd => "'flex-end'",
        AlignSelf::Center => "'center'",
        AlignSelf::Baseline => "'baseline'",
        AlignSelf::Stretch => "'stretch'",
        AlignSelf::Start => "'flex-start'",
        AlignSelf::End => "'flex-end'",
    }
}

fn needs_dimensions(node: &NodeConfig) -> bool {
    let vals = [
        &node.width,
        &node.height,
        &node.min_width,
        &node.min_height,
        &node.max_width,
        &node.max_height,
        &node.padding,
        &node.margin,
        &node.row_gap,
        &node.column_gap,
        &node.flex_basis,
    ];
    vals.iter().any(|v| matches!(v, ValueConfig::Vw(_) | ValueConfig::Vh(_)))
        || node.children.iter().any(|c| needs_dimensions(c))
}

pub fn emit_react_native(root: &NodeConfig, palette: ColorPalette) -> Result<String> {
    let use_dimensions = needs_dimensions(root);
    let mut buf = String::from("import React from 'react';\n");
    if use_dimensions {
        writeln!(buf, "import {{ View, Text, Dimensions }} from 'react-native';")?;
    } else {
        writeln!(buf, "import {{ View, Text }} from 'react-native';")?;
    }
    buf.push_str("\nexport default function FlexLayout() {\n  return (\n");
    emit_rn_node(&mut buf, root, 2, &mut 0, palette)?;
    buf.push_str("  );\n}\n");
    Ok(buf)
}

fn emit_rn_node(
    buf: &mut String,
    node: &NodeConfig,
    depth: usize,
    leaf_idx: &mut usize,
    palette: ColorPalette,
) -> Result<()> {
    let pad = "  ".repeat(depth);
    let is_leaf = node.children.is_empty();

    let bg = if is_leaf {
        let (r, g, b) = palette_color(palette, *leaf_idx);
        *leaf_idx += 1;
        format!(
            "'rgb({}, {}, {})'",
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
        )
    } else {
        "'rgba(28, 28, 43, 1)'".into()
    };

    writeln!(buf, "{pad}<View style={{{{")?;

    // React Native defaults: flexDirection 'column', unlike CSS 'row'.
    // Always emit flexDirection so the layout intent is explicit.
    if node.flex_direction != FlexDirection::Column {
        writeln!(
            buf,
            "{pad}  flexDirection: {},",
            rn_direction(node.flex_direction)
        )?;
    }
    if !node.visible {
        writeln!(buf, "{pad}  opacity: 0,")?;
    }
    if node.flex_wrap != FlexWrap::NoWrap {
        writeln!(buf, "{pad}  flexWrap: {},", rn_wrap(node.flex_wrap))?;
    }
    if !matches!(
        node.justify_content,
        JustifyContent::Default | JustifyContent::FlexStart | JustifyContent::Start
    ) {
        writeln!(
            buf,
            "{pad}  justifyContent: {},",
            rn_justify(node.justify_content)
        )?;
    }
    if !matches!(node.align_items, AlignItems::Default | AlignItems::Stretch) {
        writeln!(
            buf,
            "{pad}  alignItems: {},",
            rn_align_items(node.align_items)
        )?;
    }
    // RN default alignContent is 'flex-start', not 'stretch' like CSS.
    if !matches!(
        node.align_content,
        AlignContent::Default | AlignContent::FlexStart | AlignContent::Start
    ) {
        writeln!(
            buf,
            "{pad}  alignContent: {},",
            rn_align_content(node.align_content)
        )?;
    }
    if !matches!(node.row_gap, ValueConfig::Auto)
        && !matches!(node.row_gap, ValueConfig::Px(v) if v == 0.0)
    {
        writeln!(buf, "{pad}  rowGap: {},", rn_value(&node.row_gap))?;
    }
    if !matches!(node.column_gap, ValueConfig::Auto)
        && !matches!(node.column_gap, ValueConfig::Px(v) if v == 0.0)
    {
        writeln!(buf, "{pad}  columnGap: {},", rn_value(&node.column_gap))?;
    }
    if node.flex_grow != 0.0 {
        writeln!(buf, "{pad}  flexGrow: {},", format_num(node.flex_grow))?;
    }
    // RN default flexShrink is 0, not 1 like CSS.
    if node.flex_shrink != 0.0 {
        writeln!(buf, "{pad}  flexShrink: {},", format_num(node.flex_shrink))?;
    }
    if !matches!(node.flex_basis, ValueConfig::Auto) {
        writeln!(buf, "{pad}  flexBasis: {},", rn_value(&node.flex_basis))?;
    }
    if node.align_self != AlignSelf::Auto {
        writeln!(
            buf,
            "{pad}  alignSelf: {},",
            rn_align_self(node.align_self)
        )?;
    }
    if !matches!(node.width, ValueConfig::Auto) {
        writeln!(buf, "{pad}  width: {},", rn_value(&node.width))?;
    }
    if !matches!(node.height, ValueConfig::Auto) {
        writeln!(buf, "{pad}  height: {},", rn_value(&node.height))?;
    }
    if !matches!(node.min_width, ValueConfig::Auto) {
        writeln!(buf, "{pad}  minWidth: {},", rn_value(&node.min_width))?;
    }
    if !matches!(node.min_height, ValueConfig::Auto) {
        writeln!(buf, "{pad}  minHeight: {},", rn_value(&node.min_height))?;
    }
    if !matches!(node.max_width, ValueConfig::Auto) {
        writeln!(buf, "{pad}  maxWidth: {},", rn_value(&node.max_width))?;
    }
    if !matches!(node.max_height, ValueConfig::Auto) {
        writeln!(buf, "{pad}  maxHeight: {},", rn_value(&node.max_height))?;
    }
    if !matches!(node.padding, ValueConfig::Px(v) if v == 0.0) {
        writeln!(buf, "{pad}  padding: {},", rn_value(&node.padding))?;
    }
    if !matches!(node.margin, ValueConfig::Px(v) if v == 0.0) {
        writeln!(buf, "{pad}  margin: {},", rn_value(&node.margin))?;
    }
    writeln!(buf, "{pad}  backgroundColor: {bg},")?;

    if is_leaf {
        writeln!(buf, "{pad}}}}}>")? ;
        writeln!(
            buf,
            "{pad}  <Text style={{{{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}}}>{}</Text>",
            node.label
        )?;
        writeln!(buf, "{pad}</View>")?;
    } else {
        writeln!(buf, "{pad}}}}}>")? ;
        let mut sorted: Vec<&NodeConfig> = node.children.iter().collect();
        sorted.sort_by_key(|c| c.order);
        for child in sorted {
            emit_rn_node(buf, child, depth + 1, leaf_idx, palette)?;
        }
        writeln!(buf, "{pad}</View>")?;
    }
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
    fn emits_function_component() {
        let code = emit_react_native(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains("export default function FlexLayout()"));
    }

    #[test]
    fn uses_view_and_text_components() {
        let code = emit_react_native(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains("<View style={{"));
        assert!(code.contains("<Text"));
        assert!(code.contains("</View>"));
        assert!(!code.contains("<div"), "should use View, not div");
    }

    #[test]
    fn imports_react_native() {
        let code = emit_react_native(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains("import { View, Text }"));
        assert!(code.contains("from 'react-native'"));
    }

    #[test]
    fn no_display_flex() {
        let code = emit_react_native(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(
            !code.contains("display:"),
            "RN Views are flex by default — no display property needed"
        );
    }

    #[test]
    fn emits_opacity_zero_when_not_visible() {
        let mut node = NodeConfig::new_leaf("A", 80.0, 80.0);
        node.visible = false;
        let mut root = NodeConfig::new_container("root");
        root.children = vec![node];
        let code = emit_react_native(&root, ColorPalette::Pastel1).unwrap();
        assert!(
            code.contains("opacity: 0"),
            "should use opacity:0 to hide while preserving layout"
        );
    }

    #[test]
    fn emits_leaf_label() {
        let code = emit_react_native(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains(">A</Text>"));
        assert!(code.contains(">B</Text>"));
    }

    #[test]
    fn no_box_sizing() {
        let code = emit_react_native(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(
            !code.contains("boxSizing"),
            "RN always uses border-box — no boxSizing needed"
        );
    }

    #[test]
    fn imports_dimensions_for_vw() {
        let mut leaf = NodeConfig::new_leaf("A", 80.0, 80.0);
        leaf.width = ValueConfig::Vw(50.0);
        let mut root = NodeConfig::new_container("root");
        root.children = vec![leaf];
        let code = emit_react_native(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains("Dimensions"));
        assert!(code.contains("Dimensions.get('window').width"));
    }
}
