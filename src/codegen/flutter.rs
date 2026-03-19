use std::fmt::Write;

use anyhow::Result;
use bevy::prelude::*;

use crate::art::palette_color;
use crate::config::{ColorPalette, NodeConfig, ValueConfig};

fn dart_value(v: &ValueConfig) -> Option<String> {
    match v {
        ValueConfig::Auto => None,
        ValueConfig::Px(n) => Some(format!("{n:.1}")),
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

pub fn emit_flutter(root: &NodeConfig, palette: ColorPalette) -> Result<String> {
    let mut buf = String::from("Widget build(BuildContext context) {\n  return ");
    emit_flutter_node(&mut buf, root, 1, &mut 0, palette)?;
    buf.push_str(";\n}\n");
    Ok(buf)
}

fn emit_flutter_node(
    buf: &mut String,
    node: &NodeConfig,
    depth: usize,
    leaf_idx: &mut usize,
    palette: ColorPalette,
) -> Result<()> {
    let pad = "  ".repeat(depth);
    let is_leaf = node.children.is_empty();

    if !node.visible {
        writeln!(buf, "{pad}Offstage(")?;
        writeln!(buf, "{pad}  offstage: true,")?;
        write!(buf, "{pad}  child: ")?;
        emit_flutter_inner(buf, node, depth + 1, leaf_idx, is_leaf, palette)?;
        writeln!(buf, "{pad})")?;
        return Ok(());
    }

    emit_flutter_inner(buf, node, depth, leaf_idx, is_leaf, palette)
}

fn emit_flutter_inner(
    buf: &mut String,
    node: &NodeConfig,
    depth: usize,
    leaf_idx: &mut usize,
    is_leaf: bool,
    palette: ColorPalette,
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
        if let Some(p) = dart_value(&node.padding) {
            writeln!(buf, "{pad}  padding: EdgeInsets.all({p}),")?;
        }
        if let Some(m) = dart_value(&node.margin) {
            writeln!(buf, "{pad}  margin: EdgeInsets.all({m}),")?;
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
        writeln!(
            buf,
            "{pad}  color: Color.fromRGBO({}, {}, {}, 1.0),",
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
        )?;
        writeln!(buf, "{pad}  alignment: Alignment.center,")?;
        writeln!(buf, "{pad}  child: Text('{}'", node.label)?;
        writeln!(buf, "{pad}    style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),")?;
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
        let h = dart_value(&node.height);
        let p = dart_value(&node.padding);
        let m = dart_value(&node.margin);
        let has_container = w.is_some() || h.is_some() || p.is_some() || m.is_some();

        if has_container {
            writeln!(buf, "{pad}Container(")?;
            if let Some(v) = &w {
                writeln!(buf, "{pad}  width: {v},")?;
            }
            if let Some(v) = &h {
                writeln!(buf, "{pad}  height: {v},")?;
            }
            if let Some(v) = &p {
                writeln!(buf, "{pad}  padding: EdgeInsets.all({v}),")?;
            }
            if let Some(v) = &m {
                writeln!(buf, "{pad}  margin: EdgeInsets.all({v}),")?;
            }
            write!(buf, "{pad}  child: ")?;
        }

        let inner_depth = if has_container { depth + 1 } else { depth };
        let ipad = "  ".repeat(inner_depth);

        if node.flex_wrap != FlexWrap::NoWrap {
            writeln!(buf, "{ipad}Wrap(")?;
            writeln!(
                buf,
                "{ipad}  direction: {},",
                if is_row { "Axis.horizontal" } else { "Axis.vertical" }
            )?;
            if let Some(s) = dart_value(&node.column_gap) {
                writeln!(buf, "{ipad}  spacing: {s},")?;
            }
            if let Some(s) = dart_value(&node.row_gap) {
                writeln!(buf, "{ipad}  runSpacing: {s},")?;
            }
        } else {
            let widget = if is_row { "Row" } else { "Column" };
            writeln!(buf, "{ipad}{widget}(")?;
            writeln!(buf, "{ipad}  mainAxisAlignment: {},", dart_main_axis(node.justify_content))?;
            writeln!(buf, "{ipad}  crossAxisAlignment: {},", dart_cross_axis(node.align_items))?;
        }

        writeln!(buf, "{ipad}  children: [")?;
        let mut children: Vec<&NodeConfig> = node.children.iter().collect();
        children.sort_by_key(|c| c.order);
        if is_reversed {
            children.reverse();
        }
        for child in children {
            if child.flex_grow > 0.0 && node.flex_wrap == FlexWrap::NoWrap {
                writeln!(buf, "{ipad}    Expanded(")?;
                writeln!(buf, "{ipad}      flex: {},", child.flex_grow.round().max(1.0) as i32)?;
                write!(buf, "{ipad}      child: ")?;
                emit_flutter_node(buf, child, inner_depth + 3, leaf_idx, palette)?;
                writeln!(buf, "{ipad}    ),")?;
            } else {
                emit_flutter_node(buf, child, inner_depth + 2, leaf_idx, palette)?;
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
    fn emits_offstage_when_hidden() {
        let mut node = NodeConfig::new_leaf("A", 80.0, 80.0);
        node.visible = false;
        let mut root = NodeConfig::new_container("root");
        root.children = vec![node];
        let code = emit_flutter(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains("Offstage("));
    }
}
