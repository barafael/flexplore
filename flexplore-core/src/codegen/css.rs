use std::fmt::Write;

use crate::config::*;
use anyhow::Result;

use crate::art::palette_color;
use crate::config::{ColorPalette, Corners, NodeConfig, Sides, ValueConfig};

fn is_zero_or_auto(v: &ValueConfig) -> bool {
    matches!(v, ValueConfig::Auto) || matches!(v, ValueConfig::Px(n) if *n == 0.0)
}

fn format_num(v: f32) -> String {
    if (v - v.round()).abs() < 0.005 {
        format!("{}", v as i32)
    } else {
        format!("{v:.1}")
    }
}

fn emit_css_value(v: &ValueConfig) -> String {
    match v {
        ValueConfig::Auto => "auto".into(),
        ValueConfig::Px(n) => format!("{n:.1}px"),
        ValueConfig::Percent(n) => format!("{n:.1}%"),
        ValueConfig::Vw(n) => format!("{n:.1}vw"),
        ValueConfig::Vh(n) => format!("{n:.1}vh"),
    }
}

fn emit_css_sides(css: &mut String, prop: &str, sides: &Sides) -> std::fmt::Result {
    if sides.is_zero() {
        return Ok(());
    }
    if sides.is_uniform() {
        writeln!(css, "  {prop}: {};", emit_css_value(&sides.first()))
    } else {
        writeln!(
            css,
            "  {prop}: {} {} {} {};",
            emit_css_value(&sides.top),
            emit_css_value(&sides.right),
            emit_css_value(&sides.bottom),
            emit_css_value(&sides.left),
        )
    }
}

fn emit_css_corners(css: &mut String, prop: &str, corners: &Corners) -> std::fmt::Result {
    if corners.is_zero() {
        return Ok(());
    }
    if corners.is_uniform() {
        writeln!(css, "  {prop}: {:.1}px;", corners.top_left)
    } else {
        writeln!(
            css,
            "  {prop}: {:.1}px {:.1}px {:.1}px {:.1}px;",
            corners.top_left, corners.top_right, corners.bottom_right, corners.bottom_left,
        )
    }
}

fn css_flex_direction(d: FlexDirection) -> &'static str {
    match d {
        FlexDirection::Row => "row",
        FlexDirection::Column => "column",
        FlexDirection::RowReverse => "row-reverse",
        FlexDirection::ColumnReverse => "column-reverse",
    }
}

fn css_flex_wrap(w: FlexWrap) -> &'static str {
    match w {
        FlexWrap::NoWrap => "nowrap",
        FlexWrap::Wrap => "wrap",
        FlexWrap::WrapReverse => "wrap-reverse",
    }
}

fn css_justify_content(j: JustifyContent) -> &'static str {
    match j {
        JustifyContent::FlexStart => "flex-start",
        JustifyContent::FlexEnd => "flex-end",
        JustifyContent::Center => "center",
        JustifyContent::SpaceBetween => "space-between",
        JustifyContent::SpaceAround => "space-around",
        JustifyContent::SpaceEvenly => "space-evenly",
        JustifyContent::Stretch => "stretch",
        JustifyContent::Start => "start",
        JustifyContent::End => "end",
        _ => "flex-start",
    }
}

fn css_align_items(a: AlignItems) -> &'static str {
    match a {
        AlignItems::FlexStart => "flex-start",
        AlignItems::FlexEnd => "flex-end",
        AlignItems::Center => "center",
        AlignItems::Baseline => "baseline",
        AlignItems::Stretch => "stretch",
        AlignItems::Start => "start",
        AlignItems::End => "end",
        _ => "stretch",
    }
}

fn css_align_content(a: AlignContent) -> &'static str {
    match a {
        AlignContent::FlexStart => "flex-start",
        AlignContent::FlexEnd => "flex-end",
        AlignContent::Center => "center",
        AlignContent::SpaceBetween => "space-between",
        AlignContent::SpaceAround => "space-around",
        AlignContent::SpaceEvenly => "space-evenly",
        AlignContent::Stretch => "stretch",
        AlignContent::Start => "start",
        AlignContent::End => "end",
        _ => "stretch",
    }
}

fn css_align_self(a: AlignSelf) -> &'static str {
    match a {
        AlignSelf::Auto => "auto",
        AlignSelf::FlexStart => "flex-start",
        AlignSelf::FlexEnd => "flex-end",
        AlignSelf::Center => "center",
        AlignSelf::Baseline => "baseline",
        AlignSelf::Stretch => "stretch",
        AlignSelf::Start => "start",
        AlignSelf::End => "end",
    }
}

pub fn emit_html_css(root: &NodeConfig, palette: ColorPalette) -> Result<String> {
    let mut css = String::new();
    let mut html = String::new();
    emit_html_node(&mut css, &mut html, root, 0, &mut 0, &mut 0, palette)?;
    // Wrap in a complete HTML document. Body acts as Bevy's viewport container:
    // a column flex layout so flex-grow fills height, with align-items: start
    // so the root's explicit width is respected (not stretched).
    Ok(format!(
        "<!DOCTYPE html>\n<html>\n<head>\n<style>\n\
         html, body {{\n  margin: 0;\n  height: 100%;\n}}\n\
         body {{\n  display: flex;\n  flex-direction: column;\n  align-items: flex-start;\n}}\n\n\
         {css}\
         </style>\n</head>\n<body>\n{html}</body>\n</html>"
    ))
}

fn emit_html_node(
    css: &mut String,
    html: &mut String,
    node: &NodeConfig,
    depth: usize,
    leaf_idx: &mut usize,
    id_counter: &mut usize,
    palette: ColorPalette,
) -> Result<()> {
    let id = *id_counter;
    *id_counter += 1;
    let is_leaf = node.children.is_empty();
    let pad_html = "  ".repeat(depth);
    let class = format!("node-{id}");

    let bg = if is_leaf {
        let (r, g, b) = palette_color(palette, *leaf_idx);
        *leaf_idx += 1;
        format!(
            "rgb({}, {}, {})",
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
        )
    } else {
        "rgba(28, 28, 43, 1)".into()
    };

    writeln!(css, ".{class} {{")?;

    // Only emit properties that differ from CSS defaults.
    css.push_str("  display: flex;\n");
    if !node.visible {
        css.push_str("  visibility: hidden;\n");
    }
    if node.flex_direction != FlexDirection::Row {
        writeln!(
            css,
            "  flex-direction: {};",
            css_flex_direction(node.flex_direction)
        )?;
    }
    if node.flex_wrap != FlexWrap::NoWrap {
        writeln!(css, "  flex-wrap: {};", css_flex_wrap(node.flex_wrap))?;
    }
    if !matches!(
        node.justify_content,
        JustifyContent::Default | JustifyContent::FlexStart | JustifyContent::Start
    ) {
        writeln!(
            css,
            "  justify-content: {};",
            css_justify_content(node.justify_content)
        )?;
    }
    if !matches!(node.align_items, AlignItems::Default | AlignItems::Stretch) {
        writeln!(css, "  align-items: {};", css_align_items(node.align_items))?;
    }
    if !matches!(
        node.align_content,
        AlignContent::Default | AlignContent::Stretch
    ) {
        writeln!(
            css,
            "  align-content: {};",
            css_align_content(node.align_content)
        )?;
    }
    if !is_zero_or_auto(&node.row_gap) {
        writeln!(css, "  row-gap: {};", emit_css_value(&node.row_gap))?;
    }
    if !is_zero_or_auto(&node.column_gap) {
        writeln!(css, "  column-gap: {};", emit_css_value(&node.column_gap))?;
    }
    if node.flex_grow != 0.0 {
        writeln!(css, "  flex-grow: {};", format_num(node.flex_grow))?;
    }
    if node.flex_shrink != 1.0 {
        writeln!(css, "  flex-shrink: {};", format_num(node.flex_shrink))?;
    }
    if !matches!(node.flex_basis, ValueConfig::Auto) {
        writeln!(css, "  flex-basis: {};", emit_css_value(&node.flex_basis))?;
    }
    if node.align_self != AlignSelf::Auto {
        writeln!(css, "  align-self: {};", css_align_self(node.align_self))?;
    }
    if !matches!(node.width, ValueConfig::Auto) {
        writeln!(css, "  width: {};", emit_css_value(&node.width))?;
    }
    if !matches!(node.height, ValueConfig::Auto) {
        writeln!(css, "  height: {};", emit_css_value(&node.height))?;
    }
    if !matches!(node.min_width, ValueConfig::Auto) {
        writeln!(css, "  min-width: {};", emit_css_value(&node.min_width))?;
    }
    if !matches!(node.min_height, ValueConfig::Auto) {
        writeln!(css, "  min-height: {};", emit_css_value(&node.min_height))?;
    }
    if !matches!(node.max_width, ValueConfig::Auto) {
        writeln!(css, "  max-width: {};", emit_css_value(&node.max_width))?;
    }
    if !matches!(node.max_height, ValueConfig::Auto) {
        writeln!(css, "  max-height: {};", emit_css_value(&node.max_height))?;
    }
    emit_css_sides(css, "padding", &node.padding)?;
    emit_css_sides(css, "margin", &node.margin)?;
    emit_css_sides(css, "border-width", &node.border_width)?;
    emit_css_corners(css, "border-radius", &node.border_radius)?;
    if !node.border_width.is_zero() {
        css.push_str("  border-style: solid;\n");
    }
    if node.order != 0 {
        writeln!(css, "  order: {};", node.order)?;
    }
    writeln!(css, "  background: {bg};")?;
    css.push_str("  box-sizing: border-box;\n");
    if is_leaf {
        css.push_str("  display: flex;\n");
        css.push_str("  align-items: center;\n");
        css.push_str("  justify-content: center;\n");
        css.push_str("  color: rgba(13, 13, 26, 0.85);\n");
        css.push_str("  font-size: 26px;\n");
    }
    css.push_str("}\n\n");

    if is_leaf {
        writeln!(
            html,
            "{pad_html}<div class=\"{class}\">{}</div>",
            node.label
        )?;
    } else {
        writeln!(html, "{pad_html}<div class=\"{class}\">")?;
        let mut sorted: Vec<&NodeConfig> = node.children.iter().collect();
        sorted.sort_by_key(|c| c.order);
        for child in sorted {
            emit_html_node(css, html, child, depth + 1, leaf_idx, id_counter, palette)?;
        }
        writeln!(html, "{pad_html}</div>")?;
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
    fn emits_style_and_html() {
        let code = emit_html_css(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains("<style>"));
        assert!(code.contains("<div class=\"node-"));
    }

    #[test]
    fn emits_flex_direction() {
        let mut root = test_container();
        root.flex_direction = FlexDirection::Column;
        let code = emit_html_css(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains("flex-direction: column"));
    }

    #[test]
    fn emits_visibility_hidden_when_not_visible() {
        let mut node = NodeConfig::new_leaf("hidden", 80.0, 80.0);
        node.visible = false;
        let mut root = NodeConfig::new_container("root");
        root.children = vec![node];
        let code = emit_html_css(&root, ColorPalette::Pastel1).unwrap();
        assert!(
            code.contains("visibility: hidden"),
            "should use visibility:hidden, not display:none"
        );
        assert!(
            code.contains("display: flex"),
            "should keep display:flex alongside visibility:hidden"
        );
    }

    #[test]
    fn emits_order() {
        let mut node = NodeConfig::new_leaf("A", 80.0, 80.0);
        node.order = 3;
        let mut root = NodeConfig::new_container("root");
        root.children = vec![node];
        let code = emit_html_css(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains("order: 3"));
    }

    #[test]
    fn emits_css_values() {
        let mut leaf = NodeConfig::new_leaf("A", 80.0, 80.0);
        leaf.width = ValueConfig::Vw(50.0);
        let mut root = NodeConfig::new_container("root");
        root.children = vec![leaf];
        let code = emit_html_css(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains("50.0vw"));
    }
}
