use std::fmt::Write;

use bevy::prelude::*;

use crate::art::PASTELS;
use crate::config::{NodeConfig, ValueConfig};

fn swift_value(v: &ValueConfig) -> String {
    match v {
        ValueConfig::Auto => ".infinity".into(),
        ValueConfig::Px(n) => format!("{n:.1}"),
        ValueConfig::Percent(n) => {
            format!("{n:.1} /* {n:.1}% — use GeometryReader for relative sizing */")
        }
        ValueConfig::Vw(n) => format!("UIScreen.main.bounds.width * {:.3}", n / 100.0),
        ValueConfig::Vh(n) => format!("UIScreen.main.bounds.height * {:.3}", n / 100.0),
    }
}

fn swift_optional_value(v: &ValueConfig) -> Option<String> {
    match v {
        ValueConfig::Auto => None,
        _ => Some(swift_value(v)),
    }
}

fn swift_alignment(a: AlignItems) -> &'static str {
    match a {
        AlignItems::FlexStart => ".top",
        AlignItems::FlexEnd => ".bottom",
        AlignItems::Center => ".center",
        AlignItems::Baseline => ".firstTextBaseline",
        AlignItems::Stretch => ".center",
        _ => ".center",
    }
}

fn swift_h_alignment(a: AlignItems) -> &'static str {
    match a {
        AlignItems::FlexStart => ".leading",
        AlignItems::FlexEnd => ".trailing",
        AlignItems::Center => ".center",
        _ => ".center",
    }
}

pub fn emit_swiftui(root: &NodeConfig) -> String {
    let mut buf = String::from("struct ContentView: View {\n    var body: some View {\n");
    emit_swiftui_node(&mut buf, root, 2, &mut 0);
    buf.push_str("    }\n}\n");
    buf
}

fn emit_swiftui_node(buf: &mut String, node: &NodeConfig, depth: usize, leaf_idx: &mut usize) {
    let pad = "    ".repeat(depth);
    let is_leaf = node.children.is_empty();

    if is_leaf {
        let (r, g, b) = PASTELS[*leaf_idx % PASTELS.len()];
        *leaf_idx += 1;

        let _ = writeln!(buf, "{pad}Text({:?})", node.label);
        let _ = writeln!(buf, "{pad}    .font(.system(size: 26))");
        let _ = writeln!(buf, "{pad}    .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))");

        let w = swift_optional_value(&node.width);
        let h = swift_optional_value(&node.height);
        if w.is_some() || h.is_some() {
            let w_str = w.as_deref().unwrap_or("nil");
            let h_str = h.as_deref().unwrap_or("nil");
            let _ = writeln!(buf, "{pad}    .frame(width: {w_str}, height: {h_str})");
        }
        let min_w = swift_optional_value(&node.min_width);
        let min_h = swift_optional_value(&node.min_height);
        let max_w = swift_optional_value(&node.max_width);
        let max_h = swift_optional_value(&node.max_height);
        if min_w.is_some() || min_h.is_some() || max_w.is_some() || max_h.is_some() {
            let _ = writeln!(buf,
                "{pad}    .frame(minWidth: {}, minHeight: {}, maxWidth: {}, maxHeight: {})",
                min_w.as_deref().unwrap_or("nil"),
                min_h.as_deref().unwrap_or("nil"),
                max_w.as_deref().unwrap_or("nil"),
                max_h.as_deref().unwrap_or("nil"),
            );
        }
        if let Some(p) = swift_optional_value(&node.padding) {
            let _ = writeln!(buf, "{pad}    .padding({p})");
        }
        let _ = writeln!(buf, "{pad}    .background(Color(red: {r:.2}, green: {g:.2}, blue: {b:.2}))");
    } else {
        let is_row = matches!(
            node.flex_direction,
            FlexDirection::Row | FlexDirection::RowReverse
        );

        let spacing = match &node.column_gap {
            ValueConfig::Px(n) if is_row => format!(", spacing: {n:.1}"),
            _ => match &node.row_gap {
                ValueConfig::Px(n) if !is_row => format!(", spacing: {n:.1}"),
                _ => String::new(),
            },
        };

        let alignment = if is_row {
            swift_alignment(node.align_items)
        } else {
            swift_h_alignment(node.align_items)
        };

        let stack = if is_row { "HStack" } else { "VStack" };
        let _ = writeln!(buf, "{pad}{stack}(alignment: {alignment}{spacing}) {{");

        for child in &node.children {
            emit_swiftui_node(buf, child, depth + 1, leaf_idx);
        }

        let _ = writeln!(buf, "{pad}}}");

        let w = swift_optional_value(&node.width);
        let h = swift_optional_value(&node.height);
        if w.is_some() || h.is_some() {
            let w_str = w.as_deref().unwrap_or("nil");
            let h_str = h.as_deref().unwrap_or("nil");
            let _ = writeln!(buf, "{pad}.frame(width: {w_str}, height: {h_str})");
        }
        let min_w = swift_optional_value(&node.min_width);
        let min_h = swift_optional_value(&node.min_height);
        let max_w = swift_optional_value(&node.max_width);
        let max_h = swift_optional_value(&node.max_height);
        if min_w.is_some() || min_h.is_some() || max_w.is_some() || max_h.is_some() {
            let _ = writeln!(buf,
                "{pad}.frame(minWidth: {}, minHeight: {}, maxWidth: {}, maxHeight: {})",
                min_w.as_deref().unwrap_or("nil"),
                min_h.as_deref().unwrap_or("nil"),
                max_w.as_deref().unwrap_or("nil"),
                max_h.as_deref().unwrap_or("nil"),
            );
        }
        if let Some(p) = swift_optional_value(&node.padding) {
            let _ = writeln!(buf, "{pad}.padding({p})");
        }
        let _ = writeln!(buf, "{pad}.background(Color(red: 0.11, green: 0.11, blue: 0.17))");
    }
}
