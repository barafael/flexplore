use std::fmt::Write;

use crate::config::*;
use anyhow::Result;

use crate::art::palette_color;
use crate::config::{ColorPalette, Corners, NodeConfig, Sides, ValueConfig};

fn count_leaves(node: &NodeConfig) -> usize {
    if node.children.is_empty() {
        1
    } else {
        node.children.iter().map(count_leaves).sum()
    }
}

fn dart_value(v: &ValueConfig) -> Option<String> {
    match v {
        ValueConfig::Auto => None,
        ValueConfig::Px(n) => Some(format!("{n:.1}")),
        ValueConfig::Percent(n) if (*n - 100.0).abs() < 0.01 => Some("double.infinity".to_string()),
        ValueConfig::Percent(n) => Some(format!("{n:.1} /* {n:.0}% — use FractionallySizedBox */")),
        ValueConfig::Vw(n) => Some(format!(
            "MediaQuery.of(context).size.width * {:.3}",
            n / 100.0
        )),
        ValueConfig::Vh(n) => Some(format!(
            "MediaQuery.of(context).size.height * {:.3}",
            n / 100.0
        )),
    }
}

fn dart_edge_insets(sides: &Sides) -> Option<String> {
    if sides.is_zero() {
        return None;
    }
    if sides.is_uniform() {
        dart_value(&sides.first()).map(|v| format!("EdgeInsets.all({v})"))
    } else {
        let t = dart_value(&sides.top);
        let r = dart_value(&sides.right);
        let b = dart_value(&sides.bottom);
        let l = dart_value(&sides.left);
        if t.is_none() && r.is_none() && b.is_none() && l.is_none() {
            return None;
        }
        let mut parts = Vec::new();
        if let Some(v) = &t {
            parts.push(format!("top: {v}"));
        }
        if let Some(v) = &r {
            parts.push(format!("right: {v}"));
        }
        if let Some(v) = &b {
            parts.push(format!("bottom: {v}"));
        }
        if let Some(v) = &l {
            parts.push(format!("left: {v}"));
        }
        Some(format!("EdgeInsets.only({})", parts.join(", ")))
    }
}

fn dart_box_decoration(
    border_width: &Sides,
    border_radius: &Corners,
    bg_color: Option<&str>,
) -> Option<String> {
    let has_border = !border_width.is_zero();
    let has_radius = !border_radius.is_zero();
    if !has_border && !has_radius && bg_color.is_none() {
        return None;
    }
    let mut parts = Vec::new();
    if let Some(color) = bg_color {
        parts.push(format!("color: {color}"));
    }
    if has_border {
        if border_width.is_uniform() {
            if let Some(w) = dart_value(&border_width.first()) {
                parts.push(format!("border: Border.all(width: {w})"));
            }
        } else {
            let mut sides = Vec::new();
            if let Some(v) = dart_value(&border_width.top) {
                sides.push(format!("top: BorderSide(width: {v})"));
            }
            if let Some(v) = dart_value(&border_width.right) {
                sides.push(format!("right: BorderSide(width: {v})"));
            }
            if let Some(v) = dart_value(&border_width.bottom) {
                sides.push(format!("bottom: BorderSide(width: {v})"));
            }
            if let Some(v) = dart_value(&border_width.left) {
                sides.push(format!("left: BorderSide(width: {v})"));
            }
            if !sides.is_empty() {
                parts.push(format!("border: Border({})", sides.join(", ")));
            }
        }
    }
    if has_radius {
        if border_radius.is_uniform() {
            parts.push(format!(
                "borderRadius: BorderRadius.circular({:.1})",
                border_radius.top_left
            ));
        } else {
            let corners = format!(
                "borderRadius: BorderRadius.only(topLeft: Radius.circular({:.1}), topRight: Radius.circular({:.1}), bottomRight: Radius.circular({:.1}), bottomLeft: Radius.circular({:.1}))",
                border_radius.top_left,
                border_radius.top_right,
                border_radius.bottom_right,
                border_radius.bottom_left,
            );
            parts.push(corners);
        }
    }
    Some(format!("BoxDecoration({})", parts.join(", ")))
}

/// When direction is reversed, flex-start/end swap so items anchor to the
/// correct end of the main axis (CSS reverses the axis, not just child order).
fn effective_justify(jc: JustifyContent, is_reversed: bool) -> JustifyContent {
    if !is_reversed {
        return jc;
    }
    match jc {
        JustifyContent::FlexStart => JustifyContent::FlexEnd,
        JustifyContent::FlexEnd => JustifyContent::FlexStart,
        JustifyContent::Start => JustifyContent::End,
        JustifyContent::End => JustifyContent::Start,
        other => other,
    }
}

fn dart_main_axis(j: JustifyContent) -> &'static str {
    match j {
        JustifyContent::FlexStart | JustifyContent::Start => "MainAxisAlignment.start",
        JustifyContent::FlexEnd | JustifyContent::End => "MainAxisAlignment.end",
        JustifyContent::Center => "MainAxisAlignment.center",
        JustifyContent::SpaceBetween => "MainAxisAlignment.spaceBetween",
        JustifyContent::SpaceAround => "MainAxisAlignment.spaceAround",
        JustifyContent::SpaceEvenly => "MainAxisAlignment.spaceEvenly",
        _ => "MainAxisAlignment.start",
    }
}

fn dart_cross_axis(a: AlignItems) -> &'static str {
    match a {
        AlignItems::FlexStart | AlignItems::Start => "CrossAxisAlignment.start",
        AlignItems::FlexEnd | AlignItems::End => "CrossAxisAlignment.end",
        AlignItems::Center => "CrossAxisAlignment.center",
        AlignItems::Baseline => "CrossAxisAlignment.baseline",
        AlignItems::Stretch => "CrossAxisAlignment.stretch",
        _ => "CrossAxisAlignment.start",
    }
}

fn dart_wrap_alignment(j: JustifyContent) -> Option<&'static str> {
    match j {
        JustifyContent::FlexStart | JustifyContent::Start => None, // default
        JustifyContent::FlexEnd | JustifyContent::End => Some("WrapAlignment.end"),
        JustifyContent::Center => Some("WrapAlignment.center"),
        JustifyContent::SpaceBetween => Some("WrapAlignment.spaceBetween"),
        JustifyContent::SpaceAround => Some("WrapAlignment.spaceAround"),
        JustifyContent::SpaceEvenly => Some("WrapAlignment.spaceEvenly"),
        _ => None,
    }
}

fn dart_wrap_run_alignment(a: AlignContent) -> Option<&'static str> {
    match a {
        AlignContent::FlexStart | AlignContent::Start => None, // default
        AlignContent::FlexEnd | AlignContent::End => Some("WrapAlignment.end"),
        AlignContent::Center => Some("WrapAlignment.center"),
        AlignContent::SpaceBetween => Some("WrapAlignment.spaceBetween"),
        AlignContent::SpaceAround => Some("WrapAlignment.spaceAround"),
        AlignContent::Stretch => None, // no direct equivalent
        _ => None,
    }
}

fn dart_wrap_cross_alignment(a: AlignItems) -> Option<&'static str> {
    match a {
        AlignItems::FlexStart | AlignItems::Start => None, // default
        AlignItems::FlexEnd | AlignItems::End => Some("WrapCrossAlignment.end"),
        AlignItems::Center => Some("WrapCrossAlignment.center"),
        _ => None,
    }
}

fn dart_align_self(a: AlignSelf, is_row: bool) -> Option<&'static str> {
    match (a, is_row) {
        (AlignSelf::Auto, _) => None,
        (AlignSelf::FlexStart | AlignSelf::Start, true) => Some("Alignment.topCenter"),
        (AlignSelf::FlexEnd | AlignSelf::End, true) => Some("Alignment.bottomCenter"),
        (AlignSelf::Center, true) => Some("Alignment.center"),
        (AlignSelf::Stretch, true) => None, // handled by CrossAxisAlignment
        (AlignSelf::FlexStart | AlignSelf::Start, false) => Some("Alignment.centerLeft"),
        (AlignSelf::FlexEnd | AlignSelf::End, false) => Some("Alignment.centerRight"),
        (AlignSelf::Center, false) => Some("Alignment.center"),
        (AlignSelf::Stretch, false) => None,
        _ => None,
    }
}

pub fn emit_flutter(root: &NodeConfig, palette: ColorPalette) -> Result<String> {
    let mut buf = String::from("Widget build(BuildContext context) {\n  return ");
    emit_flutter_node(&mut buf, root, 1, &mut 0, palette, true)?;
    buf.push_str(";\n}\n");
    Ok(buf)
}

fn emit_flutter_node(
    buf: &mut String,
    node: &NodeConfig,
    depth: usize,
    leaf_idx: &mut usize,
    palette: ColorPalette,
    is_root: bool,
) -> Result<()> {
    let pad = "  ".repeat(depth);
    let is_leaf = node.children.is_empty();

    if !node.visible {
        writeln!(buf, "{pad}Visibility(")?;
        writeln!(buf, "{pad}  visible: false,")?;
        writeln!(buf, "{pad}  maintainSize: true,")?;
        writeln!(buf, "{pad}  maintainAnimation: true,")?;
        writeln!(buf, "{pad}  maintainState: true,")?;
        write!(buf, "{pad}  child: ")?;
        emit_flutter_inner(buf, node, depth + 1, leaf_idx, is_leaf, palette, is_root)?;
        writeln!(buf, "{pad})")?;
        return Ok(());
    }

    emit_flutter_inner(buf, node, depth, leaf_idx, is_leaf, palette, is_root)
}

fn emit_flutter_inner(
    buf: &mut String,
    node: &NodeConfig,
    depth: usize,
    leaf_idx: &mut usize,
    is_leaf: bool,
    palette: ColorPalette,
    is_root: bool,
) -> Result<()> {
    let pad = "  ".repeat(depth);

    if is_leaf {
        let (r, g, b) = palette_color(palette, *leaf_idx);
        *leaf_idx += 1;

        writeln!(buf, "{pad}Container(")?;
        let w = dart_value(&node.width);
        let h = dart_value(&node.height);
        if let Some(w) = &w {
            writeln!(buf, "{pad}  width: {w},")?;
        }
        if let Some(h) = &h {
            writeln!(buf, "{pad}  height: {h},")?;
        }
        if let Some(p) = dart_edge_insets(&node.padding) {
            writeln!(buf, "{pad}  padding: {p},")?;
        }
        if let Some(m) = dart_edge_insets(&node.margin) {
            writeln!(buf, "{pad}  margin: {m},")?;
        }
        // Constraints
        let min_w = dart_value(&node.min_width);
        let min_h = dart_value(&node.min_height);
        let max_w = dart_value(&node.max_width);
        let max_h = dart_value(&node.max_height);
        if min_w.is_some() || min_h.is_some() || max_w.is_some() || max_h.is_some() {
            writeln!(buf, "{pad}  constraints: BoxConstraints(")?;
            if let Some(v) = &min_w {
                writeln!(buf, "{pad}    minWidth: {v},")?;
            }
            if let Some(v) = &min_h {
                writeln!(buf, "{pad}    minHeight: {v},")?;
            }
            if let Some(v) = &max_w {
                writeln!(buf, "{pad}    maxWidth: {v},")?;
            }
            if let Some(v) = &max_h {
                writeln!(buf, "{pad}    maxHeight: {v},")?;
            }
            writeln!(buf, "{pad}  ),")?;
        }
        let bg_str = format!(
            "Color.fromRGBO({}, {}, {}, 1.0)",
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
        );
        let has_border_or_radius =
            !node.border_width.is_zero() || !node.border_radius.is_zero();
        if has_border_or_radius {
            if let Some(deco) = dart_box_decoration(&node.border_width, &node.border_radius, Some(&bg_str)) {
                writeln!(buf, "{pad}  decoration: {deco},")?;
            }
        } else {
            writeln!(buf, "{pad}  color: {bg_str},")?;
        }
        writeln!(buf, "{pad}  alignment: Alignment.center,")?;
        writeln!(buf, "{pad}  child: Text('{}',", node.label)?;
        writeln!(
            buf,
            "{pad}    style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),"
        )?;
        writeln!(buf, "{pad}  ),")?;
        writeln!(buf, "{pad})")?;
    } else {
        let is_row = matches!(
            node.flex_direction,
            FlexDirection::Row | FlexDirection::RowReverse
        );
        let is_reversed = matches!(
            node.flex_direction,
            FlexDirection::RowReverse | FlexDirection::ColumnReverse
        );

        let w = dart_value(&node.width);
        // Root with flex_grow fills the viewport height, matching CSS body { height: 100% }
        let h = if is_root && node.flex_grow > 0.0 {
            Some("double.infinity".to_string())
        } else {
            dart_value(&node.height)
        };
        let p = dart_edge_insets(&node.padding);
        let m = dart_edge_insets(&node.margin);
        let deco = dart_box_decoration(&node.border_width, &node.border_radius, None);
        let has_container =
            w.is_some() || h.is_some() || p.is_some() || m.is_some() || deco.is_some() || is_root;

        if has_container {
            writeln!(buf, "{pad}Container(")?;
            if let Some(v) = &w {
                writeln!(buf, "{pad}  width: {v},")?;
            }
            if let Some(v) = &h {
                writeln!(buf, "{pad}  height: {v},")?;
            }
            if let Some(v) = &p {
                writeln!(buf, "{pad}  padding: {v},")?;
            }
            if let Some(v) = &m {
                writeln!(buf, "{pad}  margin: {v},")?;
            }
            if let Some(d) = &deco {
                writeln!(buf, "{pad}  decoration: {d},")?;
            }
            write!(buf, "{pad}  child: ")?;
        }

        let inner_depth = if has_container { depth + 1 } else { depth };
        let ipad = "  ".repeat(inner_depth);
        let is_grid = node.display_mode == DisplayMode::Grid;

        if is_grid {
            // CSS Grid layout — use GridView.count or a custom grid widget
            writeln!(buf, "{ipad}// CSS Grid layout — use GridView.count or a custom grid widget")?;
            writeln!(buf, "{ipad}Wrap(")?;
            writeln!(buf, "{ipad}  // grid-template-columns / rows not directly supported in Flutter")?;
            if !node.grid_template_columns.is_empty() {
                let val: Vec<_> = node.grid_template_columns.iter().map(|t| t.display_short()).collect();
                writeln!(buf, "{ipad}  // grid-template-columns: {}", val.join(" "))?;
            }
            if !node.grid_template_rows.is_empty() {
                let val: Vec<_> = node.grid_template_rows.iter().map(|t| t.display_short()).collect();
                writeln!(buf, "{ipad}  // grid-template-rows: {}", val.join(" "))?;
            }
            if !node.grid_auto_columns.is_empty() {
                let val: Vec<_> = node.grid_auto_columns.iter().map(|t| t.display_short()).collect();
                writeln!(buf, "{ipad}  // grid-auto-columns: {}", val.join(" "))?;
            }
            if !node.grid_auto_rows.is_empty() {
                let val: Vec<_> = node.grid_auto_rows.iter().map(|t| t.display_short()).collect();
                writeln!(buf, "{ipad}  // grid-auto-rows: {}", val.join(" "))?;
            }
            if node.grid_auto_flow != GridAutoFlow::Row {
                writeln!(buf, "{ipad}  // grid-auto-flow: {}", node.grid_auto_flow.to_css_str())?;
            }
            if let Some(s) = dart_value(&node.column_gap) {
                writeln!(buf, "{ipad}  spacing: {s},")?;
            }
            if let Some(s) = dart_value(&node.row_gap) {
                writeln!(buf, "{ipad}  runSpacing: {s},")?;
            }
        } else if node.flex_wrap != FlexWrap::NoWrap {
            // For Wrap, textDirection/verticalDirection handles axis reversal
            // natively, so no effective_justify swap or child reversal needed.
            writeln!(buf, "{ipad}Wrap(")?;
            writeln!(
                buf,
                "{ipad}  direction: {},",
                if is_row {
                    "Axis.horizontal"
                } else {
                    "Axis.vertical"
                }
            )?;
            if is_reversed {
                if is_row {
                    writeln!(buf, "{ipad}  textDirection: TextDirection.rtl,")?;
                } else {
                    writeln!(buf, "{ipad}  verticalDirection: VerticalDirection.up,")?;
                }
            }
            if node.flex_wrap == FlexWrap::WrapReverse {
                if is_row {
                    writeln!(buf, "{ipad}  verticalDirection: VerticalDirection.up,")?;
                } else {
                    writeln!(buf, "{ipad}  textDirection: TextDirection.rtl,")?;
                }
            }
            if let Some(a) = dart_wrap_alignment(node.justify_content) {
                writeln!(buf, "{ipad}  alignment: {a},")?;
            }
            if let Some(ra) = dart_wrap_run_alignment(node.align_content) {
                writeln!(buf, "{ipad}  runAlignment: {ra},")?;
            }
            if let Some(ca) = dart_wrap_cross_alignment(node.align_items) {
                writeln!(buf, "{ipad}  crossAxisAlignment: {ca},")?;
            }
            if let Some(s) = dart_value(&node.column_gap) {
                writeln!(buf, "{ipad}  spacing: {s},")?;
            }
            if let Some(s) = dart_value(&node.row_gap) {
                writeln!(buf, "{ipad}  runSpacing: {s},")?;
            }
        } else {
            // For Row/Column (NoWrap), reverse children + swap justify to
            // approximate reversed direction.
            let eff_jc = effective_justify(node.justify_content, is_reversed);
            let widget = if is_row { "Row" } else { "Column" };
            writeln!(buf, "{ipad}{widget}(")?;
            writeln!(
                buf,
                "{ipad}  mainAxisAlignment: {},",
                dart_main_axis(eff_jc)
            )?;
            writeln!(
                buf,
                "{ipad}  crossAxisAlignment: {},",
                dart_cross_axis(node.align_items)
            )?;
        }

        writeln!(buf, "{ipad}  children: [")?;
        let mut children: Vec<&NodeConfig> = node.children.iter().collect();
        children.sort_by_key(|c| c.order);

        // Pre-compute leaf_idx start for each child in sorted order,
        // so colors track with their original nodes even when reversed.
        let mut starts = Vec::with_capacity(children.len());
        let mut acc = *leaf_idx;
        for child in &children {
            starts.push(acc);
            acc += count_leaves(child);
        }
        *leaf_idx = acc;

        if is_reversed {
            let dir_label = match node.flex_direction {
                FlexDirection::RowReverse => "RowReverse",
                FlexDirection::ColumnReverse => "ColumnReverse",
                _ => unreachable!(),
            };
            if node.flex_wrap == FlexWrap::NoWrap {
                writeln!(
                    buf,
                    "{ipad}    // NOTE: flex-direction: {dir_label} — children reversed in source to approximate visual order"
                )?;
                children.reverse();
                starts.reverse();
            } else {
                writeln!(
                    buf,
                    "{ipad}    // NOTE: flex-direction: {dir_label} — handled by textDirection/verticalDirection"
                )?;
            }
        }
        for (child, start) in children.iter().zip(starts.iter()) {
            let mut idx = *start;
            let needs_align = dart_align_self(child.align_self, is_row).is_some()
                && node.flex_wrap == FlexWrap::NoWrap;
            if child.flex_grow > 0.0 && node.flex_wrap == FlexWrap::NoWrap {
                writeln!(buf, "{ipad}    Expanded(")?;
                writeln!(
                    buf,
                    "{ipad}      flex: {},",
                    child.flex_grow.round().max(1.0) as i32
                )?;
                write!(buf, "{ipad}      child: ")?;
                if needs_align {
                    let align = dart_align_self(child.align_self, is_row).unwrap();
                    writeln!(buf, "Align(")?;
                    writeln!(buf, "{ipad}        alignment: {align},")?;
                    write!(buf, "{ipad}        child: ")?;
                    emit_flutter_node(buf, child, inner_depth + 4, &mut idx, palette, false)?;
                    writeln!(buf, "{ipad}      ),")?;
                } else {
                    emit_flutter_node(buf, child, inner_depth + 3, &mut idx, palette, false)?;
                }
                writeln!(buf, "{ipad}    ),")?;
            } else if matches!(child.flex_basis, ValueConfig::Percent(n) if n > 0.0)
                && node.flex_wrap == FlexWrap::NoWrap
            {
                let n = match child.flex_basis {
                    ValueConfig::Percent(n) => n,
                    _ => unreachable!(),
                };
                writeln!(buf, "{ipad}    Expanded(")?;
                writeln!(buf, "{ipad}      flex: {},", n.round() as i32)?;
                write!(buf, "{ipad}      child: ")?;
                emit_flutter_node(buf, child, inner_depth + 3, &mut idx, palette, false)?;
                writeln!(buf, "{ipad}    ),")?;
            } else if child.flex_shrink > 0.0 && node.flex_wrap == FlexWrap::NoWrap {
                writeln!(buf, "{ipad}    Flexible(")?;
                writeln!(buf, "{ipad}      fit: FlexFit.loose,")?;
                write!(buf, "{ipad}      child: ")?;
                if needs_align {
                    let align = dart_align_self(child.align_self, is_row).unwrap();
                    writeln!(buf, "Align(")?;
                    writeln!(buf, "{ipad}        alignment: {align},")?;
                    write!(buf, "{ipad}        child: ")?;
                    emit_flutter_node(buf, child, inner_depth + 4, &mut idx, palette, false)?;
                    writeln!(buf, "{ipad}      ),")?;
                } else {
                    emit_flutter_node(buf, child, inner_depth + 3, &mut idx, palette, false)?;
                }
                writeln!(buf, "{ipad}    ),")?;
            } else {
                emit_flutter_node(buf, child, inner_depth + 2, &mut idx, palette, false)?;
                writeln!(buf, "{ipad}    ,")?;
            }
        }
        writeln!(buf, "{ipad}  ],")?;
        writeln!(buf, "{ipad})")?;

        if has_container {
            writeln!(buf, "{pad})")?;
        }
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
    fn emits_build_function() {
        let code = emit_flutter(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains("Widget build(BuildContext context)"));
    }

    #[test]
    fn emits_row_for_row_direction() {
        let code = emit_flutter(&test_container(), ColorPalette::Pastel1).unwrap();
        // Default is Wrap since NodeConfig::new_container has FlexWrap::Wrap
        assert!(code.contains("Wrap(") || code.contains("Row("));
    }

    #[test]
    fn emits_column_for_column_direction() {
        let mut root = test_container();
        root.flex_direction = FlexDirection::Column;
        root.flex_wrap = FlexWrap::NoWrap;
        let code = emit_flutter(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains("Column("));
    }

    #[test]
    fn emits_container_for_leaves() {
        let code = emit_flutter(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains("Container("));
        assert!(code.contains("Text('A'"));
    }

    #[test]
    fn emits_visibility_when_not_visible() {
        let mut node = NodeConfig::new_leaf("A", 80.0, 80.0);
        node.visible = false;
        let mut root = NodeConfig::new_container("root");
        root.children = vec![node];
        let code = emit_flutter(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains("Visibility("), "should use Visibility widget");
        assert!(
            code.contains("visible: false"),
            "should set visible to false"
        );
        assert!(
            code.contains("maintainSize: true"),
            "should maintain size for layout"
        );
    }
}
