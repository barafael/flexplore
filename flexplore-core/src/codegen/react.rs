use std::fmt::Write;

use crate::config::*;
use anyhow::Result;

use crate::art::palette_color;
use crate::config::{ColorPalette, Corners, NodeConfig, Sides, ValueConfig};

fn format_num(v: f32) -> String {
    if (v - v.round()).abs() < 0.005 {
        format!("{}", v as i32)
    } else {
        format!("{v:.1}")
    }
}

fn css_value(v: &ValueConfig) -> String {
    match v {
        ValueConfig::Auto => "'auto'".into(),
        ValueConfig::Px(n) => format!("'{n:.1}px'"),
        ValueConfig::Percent(n) => format!("'{n:.1}%'"),
        ValueConfig::Vw(n) => format!("'{n:.1}vw'"),
        ValueConfig::Vh(n) => format!("'{n:.1}vh'"),
    }
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

fn camel_direction(d: FlexDirection) -> &'static str {
    match d {
        FlexDirection::Row => "'row'",
        FlexDirection::Column => "'column'",
        FlexDirection::RowReverse => "'row-reverse'",
        FlexDirection::ColumnReverse => "'column-reverse'",
    }
}

fn camel_wrap(w: FlexWrap) -> &'static str {
    match w {
        FlexWrap::NoWrap => "'nowrap'",
        FlexWrap::Wrap => "'wrap'",
        FlexWrap::WrapReverse => "'wrap-reverse'",
    }
}

fn camel_justify(j: JustifyContent) -> &'static str {
    match j {
        JustifyContent::FlexStart => "'flex-start'",
        JustifyContent::FlexEnd => "'flex-end'",
        JustifyContent::Center => "'center'",
        JustifyContent::SpaceBetween => "'space-between'",
        JustifyContent::SpaceAround => "'space-around'",
        JustifyContent::SpaceEvenly => "'space-evenly'",
        JustifyContent::Stretch => "'stretch'",
        JustifyContent::Start => "'start'",
        JustifyContent::End => "'end'",
        _ => "'flex-start'",
    }
}

fn camel_align_items(a: AlignItems) -> &'static str {
    match a {
        AlignItems::FlexStart => "'flex-start'",
        AlignItems::FlexEnd => "'flex-end'",
        AlignItems::Center => "'center'",
        AlignItems::Baseline => "'baseline'",
        AlignItems::Stretch => "'stretch'",
        AlignItems::Start => "'start'",
        AlignItems::End => "'end'",
        _ => "'stretch'",
    }
}

fn camel_align_content(a: AlignContent) -> &'static str {
    match a {
        AlignContent::FlexStart => "'flex-start'",
        AlignContent::FlexEnd => "'flex-end'",
        AlignContent::Center => "'center'",
        AlignContent::SpaceBetween => "'space-between'",
        AlignContent::SpaceAround => "'space-around'",
        AlignContent::SpaceEvenly => "'space-evenly'",
        AlignContent::Stretch => "'stretch'",
        AlignContent::Start => "'start'",
        AlignContent::End => "'end'",
        _ => "'stretch'",
    }
}

fn camel_align_self(a: AlignSelf) -> &'static str {
    match a {
        AlignSelf::Auto => "'auto'",
        AlignSelf::FlexStart => "'flex-start'",
        AlignSelf::FlexEnd => "'flex-end'",
        AlignSelf::Center => "'center'",
        AlignSelf::Baseline => "'baseline'",
        AlignSelf::Stretch => "'stretch'",
        AlignSelf::Start => "'start'",
        AlignSelf::End => "'end'",
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
            let val: Vec<_> = node.grid_template_columns.iter().map(|t| t.display_short()).collect();
            writeln!(buf, "{pad}  gridTemplateColumns: '{}',", val.join(" "))?;
        }
        if !node.grid_template_rows.is_empty() {
            let val: Vec<_> = node.grid_template_rows.iter().map(|t| t.display_short()).collect();
            writeln!(buf, "{pad}  gridTemplateRows: '{}',", val.join(" "))?;
        }
        if !node.grid_auto_columns.is_empty() {
            let val: Vec<_> = node.grid_auto_columns.iter().map(|t| t.display_short()).collect();
            writeln!(buf, "{pad}  gridAutoColumns: '{}',", val.join(" "))?;
        }
        if !node.grid_auto_rows.is_empty() {
            let val: Vec<_> = node.grid_auto_rows.iter().map(|t| t.display_short()).collect();
            writeln!(buf, "{pad}  gridAutoRows: '{}',", val.join(" "))?;
        }
        if node.grid_auto_flow != GridAutoFlow::Row {
            let flow = match node.grid_auto_flow {
                GridAutoFlow::Column => "column",
                GridAutoFlow::RowDense => "row dense",
                GridAutoFlow::ColumnDense => "column dense",
                _ => "row",
            };
            writeln!(buf, "{pad}  gridAutoFlow: '{flow}',")?;
        }
    } else {
        if node.flex_direction != FlexDirection::Row {
            writeln!(
                buf,
                "{pad}  flexDirection: {},",
                camel_direction(node.flex_direction)
            )?;
        }
        if node.flex_wrap != FlexWrap::NoWrap {
            writeln!(buf, "{pad}  flexWrap: {},", camel_wrap(node.flex_wrap))?;
        }
    }
    if !matches!(
        node.justify_content,
        JustifyContent::Default | JustifyContent::FlexStart | JustifyContent::Start
    ) {
        writeln!(
            buf,
            "{pad}  justifyContent: {},",
            camel_justify(node.justify_content)
        )?;
    }
    if !matches!(node.align_items, AlignItems::Default | AlignItems::Stretch) {
        writeln!(
            buf,
            "{pad}  alignItems: {},",
            camel_align_items(node.align_items)
        )?;
    }
    if !matches!(
        node.align_content,
        AlignContent::Default | AlignContent::Stretch
    ) {
        writeln!(
            buf,
            "{pad}  alignContent: {},",
            camel_align_content(node.align_content)
        )?;
    }
    if !matches!(node.row_gap, ValueConfig::Auto)
        && !matches!(node.row_gap, ValueConfig::Px(v) if v == 0.0)
    {
        writeln!(buf, "{pad}  rowGap: {},", css_value(&node.row_gap))?;
    }
    if !matches!(node.column_gap, ValueConfig::Auto)
        && !matches!(node.column_gap, ValueConfig::Px(v) if v == 0.0)
    {
        writeln!(buf, "{pad}  columnGap: {},", css_value(&node.column_gap))?;
    }
    if node.flex_grow != 0.0 {
        writeln!(buf, "{pad}  flexGrow: {},", format_num(node.flex_grow))?;
    }
    if node.flex_shrink != 1.0 {
        writeln!(buf, "{pad}  flexShrink: {},", format_num(node.flex_shrink))?;
    }
    if !matches!(node.flex_basis, ValueConfig::Auto) {
        writeln!(buf, "{pad}  flexBasis: {},", css_value(&node.flex_basis))?;
    }
    if node.align_self != AlignSelf::Auto {
        writeln!(
            buf,
            "{pad}  alignSelf: {},",
            camel_align_self(node.align_self)
        )?;
    }
    if node.grid_column != GridPlacement::Auto {
        writeln!(buf, "{pad}  gridColumn: '{}',", node.grid_column.display_short())?;
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
        writeln!(buf, ">{}</div>", node.label)?;
    } else {
        writeln!(buf, ">")?;
        let mut sorted: Vec<&NodeConfig> = node.children.iter().collect();
        sorted.sort_by_key(|c| c.order);
        for child in sorted {
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
}
