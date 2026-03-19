use std::fmt::Write;

use anyhow::Result;
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

fn swift_spacing_value(v: &ValueConfig) -> Option<String> {
    match v {
        ValueConfig::Px(n) => Some(format!("{n:.1}")),
        ValueConfig::Vw(n) => Some(format!(
            "UIScreen.main.bounds.width * {:.3}",
            n / 100.0
        )),
        ValueConfig::Vh(n) => Some(format!(
            "UIScreen.main.bounds.height * {:.3}",
            n / 100.0
        )),
        ValueConfig::Percent(n) => Some(format!(
            "{n:.1} /* {n:.1}% — no direct SwiftUI equivalent for percentage spacing */"
        )),
        ValueConfig::Auto => None,
    }
}

fn swift_alignment(a: AlignItems) -> &'static str {
    match a {
        AlignItems::FlexStart | AlignItems::Start => ".top",
        AlignItems::FlexEnd | AlignItems::End => ".bottom",
        AlignItems::Center => ".center",
        AlignItems::Baseline => ".firstTextBaseline",
        AlignItems::Stretch => ".center",
        _ => ".center",
    }
}

fn swift_h_alignment(a: AlignItems) -> &'static str {
    match a {
        AlignItems::FlexStart | AlignItems::Start => ".leading",
        AlignItems::FlexEnd | AlignItems::End => ".trailing",
        AlignItems::Center => ".center",
        _ => ".center",
    }
}

pub fn emit_swiftui(root: &NodeConfig) -> Result<String> {
    let mut buf = String::from("struct ContentView: View {\n    var body: some View {\n");
    emit_swiftui_node(&mut buf, root, 2, &mut 0)?;
    buf.push_str("    }\n}\n");
    Ok(buf)
}

fn emit_swiftui_node(
    buf: &mut String,
    node: &NodeConfig,
    depth: usize,
    leaf_idx: &mut usize,
) -> Result<()> {
    let pad = "    ".repeat(depth);
    let is_leaf = node.children.is_empty();

    if is_leaf {
        let (r, g, b) = PASTELS[*leaf_idx % PASTELS.len()];
        *leaf_idx += 1;

        writeln!(buf, "{pad}Text({:?})", node.label)?;
        writeln!(buf, "{pad}    .font(.system(size: 26))")?;
        writeln!(
            buf,
            "{pad}    .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))"
        )?;

        let w = swift_optional_value(&node.width);
        let h = swift_optional_value(&node.height);
        if w.is_some() || h.is_some() {
            let w_str = w.as_deref().unwrap_or("nil");
            let h_str = h.as_deref().unwrap_or("nil");
            writeln!(buf, "{pad}    .frame(width: {w_str}, height: {h_str})")?;
        }
        let min_w = swift_optional_value(&node.min_width);
        let min_h = swift_optional_value(&node.min_height);
        let max_w = swift_optional_value(&node.max_width);
        let max_h = swift_optional_value(&node.max_height);
        if min_w.is_some() || min_h.is_some() || max_w.is_some() || max_h.is_some() {
            writeln!(
                buf,
                "{pad}    .frame(minWidth: {}, minHeight: {}, maxWidth: {}, maxHeight: {})",
                min_w.as_deref().unwrap_or("nil"),
                min_h.as_deref().unwrap_or("nil"),
                max_w.as_deref().unwrap_or("nil"),
                max_h.as_deref().unwrap_or("nil"),
            )?;
        }
        if let Some(p) = swift_optional_value(&node.padding) {
            writeln!(buf, "{pad}    .padding({p})")?;
        }
        if let Some(m) = swift_optional_value(&node.margin) {
            writeln!(buf, "{pad}    .padding({m}) /* margin */",)?;
        }
        if node.align_self != AlignSelf::Auto {
            writeln!(
                buf,
                "{pad}    /* align-self: {:?} — override manually with .alignmentGuide() */",
                node.align_self
            )?;
        }
        if node.flex_grow > 0.0 {
            writeln!(
                buf,
                "{pad}    .layoutPriority({:.1}) /* flex-grow */",
                node.flex_grow
            )?;
        }
        writeln!(
            buf,
            "{pad}    .background(Color(red: {r:.2}, green: {g:.2}, blue: {b:.2}))"
        )?;
        if !node.visible {
            writeln!(buf, "{pad}    .hidden()")?;
        }
        if node.order != 0 {
            writeln!(buf, "{pad}    // order: {} (no SwiftUI equivalent)", node.order)?;
        }
    } else {
        let is_row = matches!(
            node.flex_direction,
            FlexDirection::Row | FlexDirection::RowReverse
        );
        let is_reversed = matches!(
            node.flex_direction,
            FlexDirection::RowReverse | FlexDirection::ColumnReverse
        );

        let gap = if is_row {
            &node.column_gap
        } else {
            &node.row_gap
        };
        let spacing = swift_spacing_value(gap)
            .map(|s| format!(", spacing: {s}"))
            .unwrap_or_default();

        let alignment = if is_row {
            swift_alignment(node.align_items)
        } else {
            swift_h_alignment(node.align_items)
        };

        let stack = if is_row { "HStack" } else { "VStack" };
        writeln!(buf, "{pad}{stack}(alignment: {alignment}{spacing}) {{")?;

        if node.flex_wrap != FlexWrap::NoWrap {
            writeln!(
                buf,
                "{pad}    // NOTE: flex-wrap: {:?} — SwiftUI stacks don't wrap; consider a custom Layout",
                node.flex_wrap
            )?;
        }
        if !matches!(
            node.justify_content,
            JustifyContent::Default | JustifyContent::FlexStart | JustifyContent::Start
        ) {
            writeln!(
                buf,
                "{pad}    // NOTE: justify-content: {:?} — use Spacer() or custom Layout to replicate",
                node.justify_content
            )?;
        }

        let children: Vec<&NodeConfig> = if is_reversed {
            node.children.iter().rev().collect()
        } else {
            node.children.iter().collect()
        };
        for child in children {
            emit_swiftui_node(buf, child, depth + 1, leaf_idx)?;
        }

        writeln!(buf, "{pad}}}")?;

        let w = swift_optional_value(&node.width);
        let h = swift_optional_value(&node.height);
        if w.is_some() || h.is_some() {
            let w_str = w.as_deref().unwrap_or("nil");
            let h_str = h.as_deref().unwrap_or("nil");
            writeln!(buf, "{pad}.frame(width: {w_str}, height: {h_str})")?;
        }
        let min_w = swift_optional_value(&node.min_width);
        let min_h = swift_optional_value(&node.min_height);
        let max_w = swift_optional_value(&node.max_width);
        let max_h = swift_optional_value(&node.max_height);
        if min_w.is_some() || min_h.is_some() || max_w.is_some() || max_h.is_some() {
            writeln!(
                buf,
                "{pad}.frame(minWidth: {}, minHeight: {}, maxWidth: {}, maxHeight: {})",
                min_w.as_deref().unwrap_or("nil"),
                min_h.as_deref().unwrap_or("nil"),
                max_w.as_deref().unwrap_or("nil"),
                max_h.as_deref().unwrap_or("nil"),
            )?;
        }
        if let Some(p) = swift_optional_value(&node.padding) {
            writeln!(buf, "{pad}.padding({p})")?;
        }
        if let Some(m) = swift_optional_value(&node.margin) {
            writeln!(buf, "{pad}.padding({m}) /* margin */")?;
        }
        writeln!(
            buf,
            "{pad}.background(Color(red: 0.11, green: 0.11, blue: 0.17))"
        )?;
        if !node.visible {
            writeln!(buf, "{pad}.hidden()")?;
        }
        if node.order != 0 {
            writeln!(buf, "{pad}// order: {} (no SwiftUI equivalent)", node.order)?;
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
    fn emits_struct_wrapper() {
        let code = emit_swiftui(&test_container()).unwrap();
        assert!(code.contains("struct ContentView: View"));
        assert!(code.contains("var body: some View"));
    }

    #[test]
    fn emits_hstack_for_row() {
        let code = emit_swiftui(&test_container()).unwrap();
        assert!(code.contains("HStack"));
    }

    #[test]
    fn emits_vstack_for_column() {
        let mut root = test_container();
        root.flex_direction = FlexDirection::Column;
        let code = emit_swiftui(&root).unwrap();
        assert!(code.contains("VStack"));
    }

    #[test]
    fn emits_text_for_leaves() {
        let code = emit_swiftui(&test_container()).unwrap();
        assert!(code.contains("Text(\"A\")"));
        assert!(code.contains("Text(\"B\")"));
    }

    #[test]
    fn emits_hidden_modifier() {
        let mut node = NodeConfig::new_leaf("A", 80.0, 80.0);
        node.visible = false;
        let mut root = NodeConfig::new_container("root");
        root.children = vec![node];
        let code = emit_swiftui(&root).unwrap();
        assert!(code.contains(".hidden()"));
    }
}
