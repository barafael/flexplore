use std::fmt::Write;

use crate::config::*;
use anyhow::Result;

use super::common::{
    css_align_content, css_align_items, css_align_self, css_flex_direction, css_flex_wrap,
    css_justify_content, dioxus_string_literal, grid_tracks, is_auto_or_zero, rgb8,
    sorted_children, take_leaf_color,
};
use crate::config::{ColorPalette, Corners, NodeConfig, Sides, ValueConfig};

fn css_value(v: &ValueConfig) -> String {
    match v {
        ValueConfig::Auto => "auto".into(),
        ValueConfig::Px(n) => format!("{n:.1}px"),
        ValueConfig::Percent(n) => format!("{n:.1}%"),
        ValueConfig::Vw(n) => format!("{n:.1}vw"),
        ValueConfig::Vh(n) => format!("{n:.1}vh"),
    }
}

fn emit_dioxus_sides(buf: &mut String, pad: &str, prop: &str, sides: &Sides) -> std::fmt::Result {
    if sides.is_zero() {
        return Ok(());
    }
    if sides.is_uniform() {
        writeln!(buf, "{pad}    {prop}: \"{}\",", css_value(&sides.first()))
    } else {
        writeln!(
            buf,
            "{pad}    {prop}: \"{} {} {} {}\",",
            css_value(&sides.top),
            css_value(&sides.right),
            css_value(&sides.bottom),
            css_value(&sides.left),
        )
    }
}

fn emit_dioxus_corners(buf: &mut String, pad: &str, corners: &Corners) -> std::fmt::Result {
    if corners.is_zero() {
        return Ok(());
    }
    if corners.is_uniform() {
        writeln!(
            buf,
            "{pad}    border_radius: \"{:.1}px\",",
            corners.top_left
        )
    } else {
        writeln!(
            buf,
            "{pad}    border_radius: \"{:.1}px {:.1}px {:.1}px {:.1}px\",",
            corners.top_left, corners.top_right, corners.bottom_right, corners.bottom_left,
        )
    }
}

pub fn emit_dioxus(root: &NodeConfig, palette: ColorPalette) -> Result<String> {
    let mut buf =
        String::from("use dioxus::prelude::*;\n\nfn FlexLayout() -> Element {\n    rsx! {\n");
    emit_dioxus_node(&mut buf, root, 2, &mut 0, palette)?;
    buf.push_str("\n    }\n}\n");
    Ok(buf)
}

fn emit_dioxus_node(
    buf: &mut String,
    node: &NodeConfig,
    depth: usize,
    leaf_idx: &mut usize,
    palette: ColorPalette,
) -> Result<()> {
    let pad = "    ".repeat(depth);
    let is_leaf = node.children.is_empty();

    let bg = if is_leaf {
        let (r, g, b) = take_leaf_color(palette, leaf_idx);
        let (r, g, b) = rgb8(r, g, b);
        format!("rgb({r}, {g}, {b})")
    } else {
        "rgba(28, 28, 43, 1)".into()
    };

    writeln!(buf, "{pad}div {{")?;
    let is_grid = node.display_mode == DisplayMode::Grid;
    if is_grid {
        writeln!(buf, "{pad}    display: \"grid\",")?;
    } else {
        writeln!(buf, "{pad}    display: \"flex\",")?;
    }
    if !node.visible {
        writeln!(buf, "{pad}    visibility: \"hidden\",")?;
    }
    if is_grid {
        if !node.grid_template_columns.is_empty() {
            writeln!(
                buf,
                "{pad}    grid_template_columns: \"{}\",",
                grid_tracks(&node.grid_template_columns)
            )?;
        }
        if !node.grid_template_rows.is_empty() {
            writeln!(
                buf,
                "{pad}    grid_template_rows: \"{}\",",
                grid_tracks(&node.grid_template_rows)
            )?;
        }
        if !node.grid_auto_columns.is_empty() {
            writeln!(
                buf,
                "{pad}    grid_auto_columns: \"{}\",",
                grid_tracks(&node.grid_auto_columns)
            )?;
        }
        if !node.grid_auto_rows.is_empty() {
            writeln!(
                buf,
                "{pad}    grid_auto_rows: \"{}\",",
                grid_tracks(&node.grid_auto_rows)
            )?;
        }
        if node.grid_auto_flow != GridAutoFlow::Row {
            writeln!(
                buf,
                "{pad}    grid_auto_flow: \"{}\",",
                node.grid_auto_flow.to_css_str()
            )?;
        }
    } else {
        if node.flex_direction != FlexDirection::Row {
            writeln!(
                buf,
                "{pad}    flex_direction: \"{}\",",
                css_flex_direction(node.flex_direction)
            )?;
        }
        if node.flex_wrap != FlexWrap::NoWrap {
            writeln!(
                buf,
                "{pad}    flex_wrap: \"{}\",",
                css_flex_wrap(node.flex_wrap)
            )?;
        }
    }
    if !matches!(
        node.justify_content,
        JustifyContent::Default | JustifyContent::FlexStart | JustifyContent::Start
    ) {
        writeln!(
            buf,
            "{pad}    justify_content: \"{}\",",
            css_justify_content(node.justify_content)
        )?;
    }
    if !matches!(node.align_items, AlignItems::Default | AlignItems::Stretch) {
        writeln!(
            buf,
            "{pad}    align_items: \"{}\",",
            css_align_items(node.align_items)
        )?;
    }
    if !matches!(
        node.align_content,
        AlignContent::Default | AlignContent::Stretch
    ) {
        writeln!(
            buf,
            "{pad}    align_content: \"{}\",",
            css_align_content(node.align_content)
        )?;
    }
    if !is_auto_or_zero(&node.row_gap) {
        writeln!(buf, "{pad}    row_gap: \"{}\",", css_value(&node.row_gap))?;
    }
    if !is_auto_or_zero(&node.column_gap) {
        writeln!(
            buf,
            "{pad}    column_gap: \"{}\",",
            css_value(&node.column_gap)
        )?;
    }
    if node.flex_grow != 0.0 {
        writeln!(
            buf,
            "{pad}    flex_grow: \"{}\",",
            format_float(node.flex_grow)
        )?;
    }
    if node.flex_shrink != 1.0 {
        writeln!(
            buf,
            "{pad}    flex_shrink: \"{}\",",
            format_float(node.flex_shrink)
        )?;
    }
    if !matches!(node.flex_basis, ValueConfig::Auto) {
        writeln!(
            buf,
            "{pad}    flex_basis: \"{}\",",
            css_value(&node.flex_basis)
        )?;
    }
    if node.align_self != AlignSelf::Auto {
        writeln!(
            buf,
            "{pad}    align_self: \"{}\",",
            css_align_self(node.align_self)
        )?;
    }
    if node.grid_column != GridPlacement::Auto {
        writeln!(
            buf,
            "{pad}    grid_column: \"{}\",",
            node.grid_column.display_short()
        )?;
    }
    if node.grid_row != GridPlacement::Auto {
        writeln!(
            buf,
            "{pad}    grid_row: \"{}\",",
            node.grid_row.display_short()
        )?;
    }
    if !matches!(node.width, ValueConfig::Auto) {
        writeln!(buf, "{pad}    width: \"{}\",", css_value(&node.width))?;
    }
    if !matches!(node.height, ValueConfig::Auto) {
        writeln!(buf, "{pad}    height: \"{}\",", css_value(&node.height))?;
    }
    if !matches!(node.min_width, ValueConfig::Auto) {
        writeln!(
            buf,
            "{pad}    min_width: \"{}\",",
            css_value(&node.min_width)
        )?;
    }
    if !matches!(node.min_height, ValueConfig::Auto) {
        writeln!(
            buf,
            "{pad}    min_height: \"{}\",",
            css_value(&node.min_height)
        )?;
    }
    if !matches!(node.max_width, ValueConfig::Auto) {
        writeln!(
            buf,
            "{pad}    max_width: \"{}\",",
            css_value(&node.max_width)
        )?;
    }
    if !matches!(node.max_height, ValueConfig::Auto) {
        writeln!(
            buf,
            "{pad}    max_height: \"{}\",",
            css_value(&node.max_height)
        )?;
    }
    emit_dioxus_sides(buf, &pad, "padding", &node.padding)?;
    emit_dioxus_sides(buf, &pad, "margin", &node.margin)?;
    emit_dioxus_sides(buf, &pad, "border_width", &node.border_width)?;
    emit_dioxus_corners(buf, &pad, &node.border_radius)?;
    if !node.border_width.is_zero() {
        writeln!(buf, "{pad}    border_style: \"solid\",")?;
    }
    if node.order != 0 {
        writeln!(buf, "{pad}    order: \"{}\",", node.order)?;
    }
    writeln!(buf, "{pad}    background: \"{bg}\",")?;
    writeln!(buf, "{pad}    box_sizing: \"border-box\",")?;
    if is_leaf {
        writeln!(buf, "{pad}    color: \"rgba(13, 13, 26, 0.85)\",")?;
        writeln!(buf, "{pad}    font_size: \"26px\",")?;
    }

    if is_leaf {
        write!(
            buf,
            "{pad}    {}\n{pad}}}",
            dioxus_string_literal(node.display_text())
        )?;
    } else {
        for child in sorted_children(node) {
            emit_dioxus_node(buf, child, depth + 1, leaf_idx, palette)?;
            writeln!(buf)?;
        }
        write!(buf, "{pad}}}")?;
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
    fn emits_rsx_function() {
        let code = emit_dioxus(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains("fn FlexLayout() -> Element"));
        assert!(code.contains("rsx!"));
    }

    #[test]
    fn emits_div_with_display_flex() {
        let code = emit_dioxus(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains("display: \"flex\""));
    }

    #[test]
    fn emits_use_dioxus() {
        let code = emit_dioxus(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains("use dioxus::prelude::*;"));
    }

    #[test]
    fn emits_visibility_hidden_when_not_visible() {
        let mut node = NodeConfig::new_leaf("A", 80.0, 80.0);
        node.visible = false;
        let mut root = NodeConfig::new_container("root");
        root.children = vec![node];
        let code = emit_dioxus(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains("visibility: \"hidden\""));
        assert!(code.contains("display: \"flex\""));
    }

    #[test]
    fn emits_order_property() {
        let mut node = NodeConfig::new_leaf("A", 80.0, 80.0);
        node.order = 5;
        let mut root = NodeConfig::new_container("root");
        root.children = vec![node];
        let code = emit_dioxus(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains("order: \"5\""));
    }

    #[test]
    fn emits_leaf_label() {
        let code = emit_dioxus(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains("\"A\""));
        assert!(code.contains("\"B\""));
    }

    #[test]
    fn escapes_rsx_format_braces_and_quotes() {
        let mut root = NodeConfig::new_container("root");
        root.children = vec![NodeConfig::new_leaf("{x} \"q\" \\", 80.0, 80.0)];
        let code = emit_dioxus(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains(r#""{{x}} \"q\" \\""#), "{code}");
    }
}
