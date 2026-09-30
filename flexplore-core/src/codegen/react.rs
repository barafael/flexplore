use std::fmt::Write;

use crate::config::*;
use anyhow::Result;

use super::common::{
    css_align_content, css_align_items, css_align_self, css_flex_direction, css_flex_wrap,
    css_justify_content, grid_tracks, is_auto_or_zero, jsx_text_escape, rgb8, sorted_children,
    take_leaf_color,
};
use crate::config::{ColorPalette, Corners, NodeConfig, Sides, ValueConfig};

fn css_value(v: &ValueConfig) -> String {
    match v {
        ValueConfig::Auto => "'auto'".into(),
        ValueConfig::Px(n) => format!("'{n:.1}px'"),
        ValueConfig::Percent(n) => format!("'{n:.1}%'"),
        ValueConfig::Vw(n) => format!("'{n:.1}vw'"),
        ValueConfig::Vh(n) => format!("'{n:.1}vh'"),
    }
}

/// Single-quoted JS string for a CSS keyword.
fn quoted(keyword: &str) -> String {
    format!("'{keyword}'")
}

fn emit_react_sides(buf: &mut String, pad: &str, prop: &str, sides: &Sides) -> std::fmt::Result {
    if sides.is_zero() {
        return Ok(());
    }
    if sides.is_uniform() {
        writeln!(buf, "{pad}  {prop}: {},", css_value(&sides.first()))
    } else {
        writeln!(buf, "{pad}  {prop}Top: {},", css_value(&sides.top))?;
        writeln!(buf, "{pad}  {prop}Right: {},", css_value(&sides.right))?;
        writeln!(buf, "{pad}  {prop}Bottom: {},", css_value(&sides.bottom))?;
        writeln!(buf, "{pad}  {prop}Left: {},", css_value(&sides.left))
    }
}

fn emit_react_corners(buf: &mut String, pad: &str, corners: &Corners) -> std::fmt::Result {
    if corners.is_zero() {
        return Ok(());
    }
    if corners.is_uniform() {
        writeln!(buf, "{pad}  borderRadius: '{:.1}px',", corners.top_left)
    } else {
        writeln!(
            buf,
            "{pad}  borderTopLeftRadius: '{:.1}px',",
            corners.top_left
        )?;
        writeln!(
            buf,
            "{pad}  borderTopRightRadius: '{:.1}px',",
            corners.top_right
        )?;
        writeln!(
            buf,
            "{pad}  borderBottomRightRadius: '{:.1}px',",
            corners.bottom_right
        )?;
        writeln!(
            buf,
            "{pad}  borderBottomLeftRadius: '{:.1}px',",
            corners.bottom_left
        )
    }
}

pub fn emit_react(root: &NodeConfig, palette: ColorPalette) -> Result<String> {
    let mut buf = String::from("export default function FlexLayout() {\n  return (\n");
    emit_react_node(&mut buf, root, 2, &mut 0, palette)?;
    buf.push_str("  );\n}\n");
    Ok(buf)
}

fn emit_react_node(
    buf: &mut String,
    node: &NodeConfig,
    depth: usize,
    leaf_idx: &mut usize,
    palette: ColorPalette,
) -> Result<()> {
    let pad = "  ".repeat(depth);
    let is_leaf = node.children.is_empty();

    let bg = if is_leaf {
        let (r, g, b) = take_leaf_color(palette, leaf_idx);
        let (r, g, b) = rgb8(r, g, b);
        format!("'rgb({r}, {g}, {b})'")
    } else {
        "'rgba(28, 28, 43, 1)'".into()
    };

    writeln!(buf, "{pad}<div style={{{{")?;
    let is_grid = node.display_mode == DisplayMode::Grid;
    if is_grid {
        writeln!(buf, "{pad}  display: 'grid',")?;
    } else {
        writeln!(buf, "{pad}  display: 'flex',")?;
    }
    if !node.visible {
        writeln!(buf, "{pad}  visibility: 'hidden',")?;
    }
    if is_grid {
        if !node.grid_template_columns.is_empty() {
            writeln!(
                buf,
                "{pad}  gridTemplateColumns: '{}',",
                grid_tracks(&node.grid_template_columns)
            )?;
        }
        if !node.grid_template_rows.is_empty() {
            writeln!(
                buf,
                "{pad}  gridTemplateRows: '{}',",
                grid_tracks(&node.grid_template_rows)
            )?;
        }
        if !node.grid_auto_columns.is_empty() {
            writeln!(
                buf,
                "{pad}  gridAutoColumns: '{}',",
                grid_tracks(&node.grid_auto_columns)
            )?;
        }
        if !node.grid_auto_rows.is_empty() {
            writeln!(
                buf,
                "{pad}  gridAutoRows: '{}',",
                grid_tracks(&node.grid_auto_rows)
            )?;
        }
        if node.grid_auto_flow != GridAutoFlow::Row {
            writeln!(
                buf,
                "{pad}  gridAutoFlow: '{}',",
                node.grid_auto_flow.to_css_str()
            )?;
        }
    } else {
        if node.flex_direction != FlexDirection::Row {
            writeln!(
                buf,
                "{pad}  flexDirection: {},",
                quoted(css_flex_direction(node.flex_direction))
            )?;
        }
        if node.flex_wrap != FlexWrap::NoWrap {
            writeln!(
                buf,
                "{pad}  flexWrap: {},",
                quoted(css_flex_wrap(node.flex_wrap))
            )?;
        }
    }
    if !matches!(
        node.justify_content,
        JustifyContent::Default | JustifyContent::FlexStart | JustifyContent::Start
    ) {
        writeln!(
            buf,
            "{pad}  justifyContent: {},",
            quoted(css_justify_content(node.justify_content))
        )?;
    }
    if !matches!(node.align_items, AlignItems::Default | AlignItems::Stretch) {
        writeln!(
            buf,
            "{pad}  alignItems: {},",
            quoted(css_align_items(node.align_items))
        )?;
    }
    if !matches!(
        node.align_content,
        AlignContent::Default | AlignContent::Stretch
    ) {
        writeln!(
            buf,
            "{pad}  alignContent: {},",
            quoted(css_align_content(node.align_content))
        )?;
    }
    if !is_auto_or_zero(&node.row_gap) {
        writeln!(buf, "{pad}  rowGap: {},", css_value(&node.row_gap))?;
    }
    if !is_auto_or_zero(&node.column_gap) {
        writeln!(buf, "{pad}  columnGap: {},", css_value(&node.column_gap))?;
    }
    if node.flex_grow != 0.0 {
        writeln!(buf, "{pad}  flexGrow: {},", format_float(node.flex_grow))?;
    }
    if node.flex_shrink != 1.0 {
        writeln!(
            buf,
            "{pad}  flexShrink: {},",
            format_float(node.flex_shrink)
        )?;
    }
    if !matches!(node.flex_basis, ValueConfig::Auto) {
        writeln!(buf, "{pad}  flexBasis: {},", css_value(&node.flex_basis))?;
    }
    if node.align_self != AlignSelf::Auto {
        writeln!(
            buf,
            "{pad}  alignSelf: {},",
            quoted(css_align_self(node.align_self))
        )?;
    }
    if node.grid_column != GridPlacement::Auto {
        writeln!(
            buf,
            "{pad}  gridColumn: '{}',",
            node.grid_column.display_short()
        )?;
    }
    if node.grid_row != GridPlacement::Auto {
        writeln!(buf, "{pad}  gridRow: '{}',", node.grid_row.display_short())?;
    }
    if !matches!(node.width, ValueConfig::Auto) {
        writeln!(buf, "{pad}  width: {},", css_value(&node.width))?;
    }
    if !matches!(node.height, ValueConfig::Auto) {
        writeln!(buf, "{pad}  height: {},", css_value(&node.height))?;
    }
    if !matches!(node.min_width, ValueConfig::Auto) {
        writeln!(buf, "{pad}  minWidth: {},", css_value(&node.min_width))?;
    }
    if !matches!(node.min_height, ValueConfig::Auto) {
        writeln!(buf, "{pad}  minHeight: {},", css_value(&node.min_height))?;
    }
    if !matches!(node.max_width, ValueConfig::Auto) {
        writeln!(buf, "{pad}  maxWidth: {},", css_value(&node.max_width))?;
    }
    if !matches!(node.max_height, ValueConfig::Auto) {
        writeln!(buf, "{pad}  maxHeight: {},", css_value(&node.max_height))?;
    }
    emit_react_sides(buf, &pad, "padding", &node.padding)?;
    emit_react_sides(buf, &pad, "margin", &node.margin)?;
    emit_react_sides(buf, &pad, "borderWidth", &node.border_width)?;
    emit_react_corners(buf, &pad, &node.border_radius)?;
    if !node.border_width.is_zero() {
        writeln!(buf, "{pad}  borderStyle: 'solid',")?;
    }
    if node.order != 0 {
        writeln!(buf, "{pad}  order: {},", node.order)?;
    }
    writeln!(buf, "{pad}  background: {bg},")?;
    writeln!(buf, "{pad}  boxSizing: 'border-box',")?;
    if is_leaf {
        writeln!(buf, "{pad}  color: 'rgba(13, 13, 26, 0.85)',")?;
        writeln!(buf, "{pad}  fontSize: 26,")?;
    }
    write!(buf, "{pad}}}}}")?;

    if is_leaf {
        writeln!(buf, ">{}</div>", jsx_text_escape(node.display_text()))?;
    } else {
        writeln!(buf, ">")?;
        for child in sorted_children(node) {
            emit_react_node(buf, child, depth + 1, leaf_idx, palette)?;
        }
        writeln!(buf, "{pad}</div>")?;
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
        let code = emit_react(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains("export default function FlexLayout()"));
    }

    #[test]
    fn emits_inline_styles() {
        let code = emit_react(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains("display: 'flex'"));
        assert!(code.contains("style={{"));
    }

    #[test]
    fn emits_visibility_hidden_when_not_visible() {
        let mut node = NodeConfig::new_leaf("A", 80.0, 80.0);
        node.visible = false;
        let mut root = NodeConfig::new_container("root");
        root.children = vec![node];
        let code = emit_react(&root, ColorPalette::Pastel1).unwrap();
        assert!(
            code.contains("visibility: 'hidden'"),
            "should use visibility:hidden, not display:none"
        );
        assert!(
            code.contains("display: 'flex'"),
            "should keep display:flex alongside visibility:hidden"
        );
    }

    #[test]
    fn emits_order_property() {
        let mut node = NodeConfig::new_leaf("A", 80.0, 80.0);
        node.order = 5;
        let mut root = NodeConfig::new_container("root");
        root.children = vec![node];
        let code = emit_react(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains("order: 5"));
    }

    #[test]
    fn emits_leaf_label() {
        let code = emit_react(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains(">A</div>"));
    }

    #[test]
    fn escapes_jsx_text() {
        let mut root = NodeConfig::new_container("root");
        root.children = vec![NodeConfig::new_leaf("{x} <b>", 80.0, 80.0)];
        let code = emit_react(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains(">&#123;x&#125; &lt;b&gt;</div>"), "{code}");
    }
}
