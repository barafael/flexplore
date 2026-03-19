use std::fmt::Write;

use anyhow::Result;
use bevy::prelude::*;

use crate::art::PASTELS;
use crate::config::{NodeConfig, ValueConfig};

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

pub fn emit_flutter(root: &NodeConfig) -> Result<String> {
    let mut buf = String::from("Widget build(BuildContext context) {\n  return ");
    emit_flutter_node(&mut buf, root, 1, &mut 0)?;
    buf.push_str(";\n}\n");
    Ok(buf)
}

fn emit_flutter_node(
    buf: &mut String,
    node: &NodeConfig,
    depth: usize,
    leaf_idx: &mut usize,
) -> Result<()> {
    let pad = "  ".repeat(depth);
    let is_leaf = node.children.is_empty();

    if !node.visible {
        writeln!(buf, "{pad}Offstage(")?;
        writeln!(buf, "{pad}  offstage: true,")?;
        write!(buf, "{pad}  child: ")?;
        emit_flutter_inner(buf, node, depth + 1, leaf_idx, is_leaf)?;
        writeln!(buf, "{pad})")?;
        return Ok(());
    }

    emit_flutter_inner(buf, node, depth, leaf_idx, is_leaf)
}

fn emit_flutter_inner(
    buf: &mut String,
    node: &NodeConfig,
    depth: usize,
    leaf_idx: &mut usize,
    is_leaf: bool,
) -> Result<()> {
    let pad = "  ".repeat(depth);

    if is_leaf {
        let (r, g, b) = PASTELS[*leaf_idx % PASTELS.len()];
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

        if node.flex_wrap != FlexWrap::NoWrap {
            writeln!(buf, "{pad}Wrap(")?;
            writeln!(
                buf,
                "{pad}  direction: {},",
                if is_row { "Axis.horizontal" } else { "Axis.vertical" }
            )?;
            if let Some(s) = dart_value(&node.column_gap) {
                writeln!(buf, "{pad}  spacing: {s},")?;
            }
            if let Some(s) = dart_value(&node.row_gap) {
                writeln!(buf, "{pad}  runSpacing: {s},")?;
            }
        } else {
            let widget = if is_row { "Row" } else { "Column" };
            writeln!(buf, "{pad}{widget}(")?;
            writeln!(buf, "{pad}  mainAxisAlignment: {},", dart_main_axis(node.justify_content))?;
            writeln!(buf, "{pad}  crossAxisAlignment: {},", dart_cross_axis(node.align_items))?;
        }

        writeln!(buf, "{pad}  children: [")?;
        let children: Vec<(usize, &NodeConfig)> = if is_reversed {
            node.children.iter().enumerate().rev().collect()
        } else {
            node.children.iter().enumerate().collect()
        };
        for (_, child) in children {
            if child.flex_grow > 0.0 && node.flex_wrap == FlexWrap::NoWrap {
                writeln!(buf, "{pad}    Expanded(")?;
                writeln!(buf, "{pad}      flex: {},", child.flex_grow as i32)?;
                write!(buf, "{pad}      child: ")?;
                emit_flutter_node(buf, child, depth + 3, leaf_idx)?;
                writeln!(buf, "{pad}    ),")?;
            } else {
                emit_flutter_node(buf, child, depth + 2, leaf_idx)?;
                write!(buf, "{pad}    ,")?;
                buf.push('\n');
            }
        }
        writeln!(buf, "{pad}  ],")?;
        writeln!(buf, "{pad})")?;

        // Container wrapper for sizing/padding/bg
        let has_sizing = dart_value(&node.width).is_some()
            || dart_value(&node.height).is_some()
            || dart_value(&node.padding).is_some();
        if has_sizing {
            writeln!(buf, "{pad}// Wrap in Container for sizing/padding if needed")?;
        }
    }
    Ok(())
}
