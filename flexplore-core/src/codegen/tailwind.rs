use std::fmt::Write;

use crate::config::*;
use anyhow::Result;

use crate::art::palette_color;
use crate::config::{ColorPalette, Corners, NodeConfig, Sides, ValueConfig};

fn tailwind_flex_direction(d: FlexDirection) -> &'static str {
    match d {
        FlexDirection::Row => "flex-row",
        FlexDirection::Column => "flex-col",
        FlexDirection::RowReverse => "flex-row-reverse",
        FlexDirection::ColumnReverse => "flex-col-reverse",
    }
}

fn tailwind_flex_wrap(w: FlexWrap) -> &'static str {
    match w {
        FlexWrap::NoWrap => "flex-nowrap",
        FlexWrap::Wrap => "flex-wrap",
        FlexWrap::WrapReverse => "flex-wrap-reverse",
    }
}

fn tailwind_justify_content(j: JustifyContent) -> &'static str {
    match j {
        JustifyContent::FlexStart => "justify-start",
        JustifyContent::FlexEnd => "justify-end",
        JustifyContent::Center => "justify-center",
        JustifyContent::SpaceBetween => "justify-between",
        JustifyContent::SpaceAround => "justify-around",
        JustifyContent::SpaceEvenly => "justify-evenly",
        JustifyContent::Stretch => "justify-stretch",
        JustifyContent::Start => "justify-start",
        JustifyContent::End => "justify-end",
        _ => "justify-start",
    }
}

fn tailwind_align_items(a: AlignItems) -> &'static str {
    match a {
        AlignItems::FlexStart => "items-start",
        AlignItems::FlexEnd => "items-end",
        AlignItems::Center => "items-center",
        AlignItems::Baseline => "items-baseline",
        AlignItems::Stretch => "items-stretch",
        AlignItems::Start => "items-start",
        AlignItems::End => "items-end",
        _ => "items-stretch",
    }
}

fn tailwind_align_content(a: AlignContent) -> &'static str {
    match a {
        AlignContent::FlexStart => "content-start",
        AlignContent::FlexEnd => "content-end",
        AlignContent::Center => "content-center",
        AlignContent::SpaceBetween => "content-between",
        AlignContent::SpaceAround => "content-around",
        AlignContent::SpaceEvenly => "content-evenly",
        AlignContent::Stretch => "content-stretch",
        AlignContent::Start => "content-start",
        AlignContent::End => "content-end",
        _ => "content-stretch",
    }
}

fn tailwind_align_self(a: AlignSelf) -> &'static str {
    match a {
        AlignSelf::Auto => "self-auto",
        AlignSelf::FlexStart => "self-start",
        AlignSelf::FlexEnd => "self-end",
        AlignSelf::Center => "self-center",
        AlignSelf::Baseline => "self-baseline",
        AlignSelf::Stretch => "self-stretch",
        AlignSelf::Start => "self-start",
        AlignSelf::End => "self-end",
    }
}

fn tailwind_value(property: &str, v: &ValueConfig) -> String {
    match v {
        ValueConfig::Auto => match property {
            "w" | "h" | "basis" | "m" => format!("{property}-auto"),
            "max-w" | "max-h" => format!("{property}-none"),
            "p" | "gap-x" | "gap-y" | "min-w" | "min-h" => format!("{property}-0"),
            _ => format!("{property}-auto"),
        },
        ValueConfig::Px(n) => format!("{property}-[{n:.1}px]"),
        ValueConfig::Percent(n) => format!("{property}-[{n:.1}%]"),
        ValueConfig::Vw(n) => format!("{property}-[{n:.1}vw]"),
        ValueConfig::Vh(n) => format!("{property}-[{n:.1}vh]"),
    }
}

fn push_tailwind_sides(
    classes: &mut Vec<String>,
    uniform_prefix: &str,
    top: &str,
    right: &str,
    bottom: &str,
    left: &str,
    sides: &Sides,
) {
    if sides.is_zero() {
        return;
    }
    if sides.is_uniform() {
        classes.push(tailwind_value(uniform_prefix, &sides.first()));
    } else {
        if !sides.top.is_zero_px() {
            classes.push(tailwind_value(top, &sides.top));
        }
        if !sides.right.is_zero_px() {
            classes.push(tailwind_value(right, &sides.right));
        }
        if !sides.bottom.is_zero_px() {
            classes.push(tailwind_value(bottom, &sides.bottom));
        }
        if !sides.left.is_zero_px() {
            classes.push(tailwind_value(left, &sides.left));
        }
    }
}

fn push_tailwind_corners(classes: &mut Vec<String>, corners: &Corners) {
    if corners.is_zero() {
        return;
    }
    if corners.is_uniform() {
        classes.push(format!("rounded-[{:.1}px]", corners.top_left));
    } else {
        if corners.top_left != 0.0 {
            classes.push(format!("rounded-tl-[{:.1}px]", corners.top_left));
        }
        if corners.top_right != 0.0 {
            classes.push(format!("rounded-tr-[{:.1}px]", corners.top_right));
        }
        if corners.bottom_right != 0.0 {
            classes.push(format!("rounded-br-[{:.1}px]", corners.bottom_right));
        }
        if corners.bottom_left != 0.0 {
            classes.push(format!("rounded-bl-[{:.1}px]", corners.bottom_left));
        }
    }
}

pub fn emit_tailwind(root: &NodeConfig, palette: ColorPalette) -> Result<String> {
    let mut buf = String::new();
    emit_tailwind_node(&mut buf, root, 0, &mut 0, palette)?;
    Ok(buf)
}

fn emit_tailwind_node(
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
            "bg-[rgb({},{},{})]",
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
        )
    } else {
        "bg-[rgba(28,28,43,1)]".into()
    };

    let is_grid = node.display_mode == DisplayMode::Grid;
    let mut classes: Vec<String> = vec![if is_grid { "grid" } else { "flex" }.into()];
    if !node.visible {
        classes.push("invisible".into());
    }
    if is_grid {
        // Grid template — use arbitrary Tailwind values
        if !node.grid_template_columns.is_empty() {
            let val: Vec<_> = node.grid_template_columns.iter().map(|t| t.display_short()).collect();
            classes.push(format!("grid-cols-[{}]", val.join("_")));
        }
        if !node.grid_template_rows.is_empty() {
            let val: Vec<_> = node.grid_template_rows.iter().map(|t| t.display_short()).collect();
            classes.push(format!("grid-rows-[{}]", val.join("_")));
        }
        if !node.grid_auto_columns.is_empty() {
            let val: Vec<_> = node.grid_auto_columns.iter().map(|t| t.display_short()).collect();
            classes.push(format!("auto-cols-[{}]", val.join("_")));
        }
        if !node.grid_auto_rows.is_empty() {
            let val: Vec<_> = node.grid_auto_rows.iter().map(|t| t.display_short()).collect();
            classes.push(format!("auto-rows-[{}]", val.join("_")));
        }
        if node.grid_auto_flow != GridAutoFlow::Row {
            classes.push(match node.grid_auto_flow {
                GridAutoFlow::Column => "grid-flow-col",
                GridAutoFlow::RowDense => "grid-flow-row-dense",
                GridAutoFlow::ColumnDense => "grid-flow-col-dense",
                _ => "grid-flow-row",
            }.into());
        }
    } else {
        if node.flex_direction != FlexDirection::Row {
            classes.push(tailwind_flex_direction(node.flex_direction).into());
        }
        if node.flex_wrap != FlexWrap::NoWrap {
            classes.push(tailwind_flex_wrap(node.flex_wrap).into());
        }
    }
    if !matches!(
        node.justify_content,
        JustifyContent::Default | JustifyContent::FlexStart | JustifyContent::Start
    ) {
        classes.push(tailwind_justify_content(node.justify_content).into());
    }
    if !matches!(node.align_items, AlignItems::Default | AlignItems::Stretch) {
        classes.push(tailwind_align_items(node.align_items).into());
    }
    if !matches!(
        node.align_content,
        AlignContent::Default | AlignContent::Stretch
    ) {
        classes.push(tailwind_align_content(node.align_content).into());
    }
    if !matches!(node.column_gap, ValueConfig::Auto)
        && !matches!(node.column_gap, ValueConfig::Px(v) if v == 0.0)
    {
        classes.push(tailwind_value("gap-x", &node.column_gap));
    }
    if !matches!(node.row_gap, ValueConfig::Auto)
        && !matches!(node.row_gap, ValueConfig::Px(v) if v == 0.0)
    {
        classes.push(tailwind_value("gap-y", &node.row_gap));
    }
    if node.flex_grow != 0.0 {
        classes.push(format!("grow-[{}]", node.flex_grow));
    }
    if node.flex_shrink != 1.0 {
        classes.push(format!("shrink-[{}]", node.flex_shrink));
    }
    if !matches!(node.flex_basis, ValueConfig::Auto) {
        classes.push(tailwind_value("basis", &node.flex_basis));
    }
    if node.align_self != AlignSelf::Auto {
        classes.push(tailwind_align_self(node.align_self).into());
    }
    // Grid item placement
    if node.grid_column != GridPlacement::Auto {
        classes.push(format!("col-[{}]", node.grid_column.display_short()));
    }
    if node.grid_row != GridPlacement::Auto {
        classes.push(format!("row-[{}]", node.grid_row.display_short()));
    }
    if !matches!(node.width, ValueConfig::Auto) {
        classes.push(tailwind_value("w", &node.width));
    }
    if !matches!(node.height, ValueConfig::Auto) {
        classes.push(tailwind_value("h", &node.height));
    }
    if !matches!(node.min_width, ValueConfig::Auto) {
        classes.push(tailwind_value("min-w", &node.min_width));
    }
    if !matches!(node.min_height, ValueConfig::Auto) {
        classes.push(tailwind_value("min-h", &node.min_height));
    }
    if !matches!(node.max_width, ValueConfig::Auto) {
        classes.push(tailwind_value("max-w", &node.max_width));
    }
    if !matches!(node.max_height, ValueConfig::Auto) {
        classes.push(tailwind_value("max-h", &node.max_height));
    }
    push_tailwind_sides(&mut classes, "p", "pt", "pr", "pb", "pl", &node.padding);
    push_tailwind_sides(&mut classes, "m", "mt", "mr", "mb", "ml", &node.margin);
    push_tailwind_sides(
        &mut classes,
        "border",
        "border-t",
        "border-r",
        "border-b",
        "border-l",
        &node.border_width,
    );
    push_tailwind_corners(&mut classes, &node.border_radius);
    if !node.border_width.is_zero() {
        classes.push("border-solid".into());
    }
    classes.push(bg);
    classes.push("box-border".into());
    if node.order != 0 {
        classes.push(format!("order-[{}]", node.order));
    }

    if is_leaf {
        if !is_grid {
            classes.push("flex".into());
        }
        classes.push("items-center".into());
        classes.push("justify-center".into());
        classes.push("text-[26px]".into());
        classes.push("text-[rgba(13,13,26,0.85)]".into());
    }

    let cls = classes.join(" ");

    if is_leaf {
        writeln!(buf, "{pad}<div class=\"{cls}\">{}</div>", node.label)?;
    } else {
        writeln!(buf, "{pad}<div class=\"{cls}\">")?;
        let mut sorted: Vec<&NodeConfig> = node.children.iter().collect();
        sorted.sort_by_key(|c| c.order);
        for child in sorted {
            emit_tailwind_node(buf, child, depth + 1, leaf_idx, palette)?;
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
    fn emits_flex_classes() {
        let code = emit_tailwind(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains("flex"));
        // flex-row is the default and should be omitted
        assert!(!code.contains("flex-row"));
    }

    #[test]
    fn emits_column_direction() {
        let mut root = test_container();
        root.flex_direction = FlexDirection::Column;
        let code = emit_tailwind(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains("flex-col"));
    }

    #[test]
    fn emits_invisible_when_not_visible() {
        let mut node = NodeConfig::new_leaf("A", 80.0, 80.0);
        node.visible = false;
        let mut root = NodeConfig::new_container("root");
        root.children = vec![node];
        let code = emit_tailwind(&root, ColorPalette::Pastel1).unwrap();
        assert!(
            code.contains("invisible"),
            "should use invisible, not hidden"
        );
        assert!(
            code.contains("flex"),
            "should keep flex alongside invisible"
        );
    }

    #[test]
    fn emits_order_class() {
        let mut node = NodeConfig::new_leaf("A", 80.0, 80.0);
        node.order = -2;
        let mut root = NodeConfig::new_container("root");
        root.children = vec![node];
        let code = emit_tailwind(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains("order-[-2]"));
    }

    #[test]
    fn emits_leaf_label() {
        let code = emit_tailwind(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains(">A</div>"));
        assert!(code.contains(">B</div>"));
    }
}
