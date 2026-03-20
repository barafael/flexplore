use std::fmt::Write;

use anyhow::Result;
use bevy::prelude::*;

use crate::art::palette_color;
use crate::config::{ColorPalette, NodeConfig, ValueConfig};

fn count_leaves(node: &NodeConfig) -> usize {
    if node.children.is_empty() {
        1
    } else {
        node.children.iter().map(count_leaves).sum()
    }
}

fn is_zero_px(v: &ValueConfig) -> bool {
    matches!(v, ValueConfig::Px(n) if *n == 0.0)
}

fn is_full_percent(v: &ValueConfig) -> bool {
    matches!(v, ValueConfig::Percent(n) if *n >= 100.0)
}

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
        AlignItems::Stretch => ".top",
        _ => ".center",
    }
}

fn swift_h_alignment(a: AlignItems) -> &'static str {
    match a {
        AlignItems::FlexStart | AlignItems::Start => ".leading",
        AlignItems::FlexEnd | AlignItems::End => ".trailing",
        AlignItems::Center => ".center",
        AlignItems::Stretch => ".leading",
        _ => ".center",
    }
}

pub fn emit_swiftui(root: &NodeConfig, palette: ColorPalette) -> Result<String> {
    let mut buf = String::from("struct ContentView: View {\n    public var body: some View {\n");
    emit_swiftui_node(&mut buf, root, 2, &mut 0, palette, true)?;
    buf.push_str("    }\n}\n");
    Ok(buf)
}

fn emit_swiftui_node(
    buf: &mut String,
    node: &NodeConfig,
    depth: usize,
    leaf_idx: &mut usize,
    palette: ColorPalette,
    parent_is_row: bool,
) -> Result<()> {
    let pad = "    ".repeat(depth);
    let is_leaf = node.children.is_empty();

    if is_leaf {
        let (r, g, b) = palette_color(palette, *leaf_idx);
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
                "{pad}    .frame(minWidth: {}, maxWidth: {}, minHeight: {}, maxHeight: {})",
                min_w.as_deref().unwrap_or("nil"),
                max_w.as_deref().unwrap_or("nil"),
                min_h.as_deref().unwrap_or("nil"),
                max_h.as_deref().unwrap_or("nil"),
            )?;
        }
        if node.flex_grow > 0.0 {
            if parent_is_row && max_w.is_none() {
                writeln!(buf, "{pad}    .frame(maxWidth: .infinity)")?;
            } else if !parent_is_row && max_h.is_none() {
                writeln!(buf, "{pad}    .frame(maxHeight: .infinity)")?;
            }
        }
        if !is_zero_px(&node.padding) {
            if let Some(p) = swift_optional_value(&node.padding) {
                writeln!(buf, "{pad}    .padding({p})")?;
            }
        }
        writeln!(
            buf,
            "{pad}    .background(Color(red: {r:.2}, green: {g:.2}, blue: {b:.2}))"
        )?;
        if !is_zero_px(&node.margin) {
            if let Some(m) = swift_optional_value(&node.margin) {
                writeln!(buf, "{pad}    .padding({m}) /* margin */",)?;
            }
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

        let jc = node.justify_content;
        let uses_zero_spacing = matches!(
            jc,
            JustifyContent::SpaceBetween | JustifyContent::SpaceEvenly | JustifyContent::SpaceAround
        );
        let spacing = if uses_zero_spacing {
            ", spacing: 0".to_string()
        } else {
            swift_spacing_value(gap)
                .map(|s| format!(", spacing: {s}"))
                .unwrap_or_default()
        };

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
        if node.align_items == AlignItems::Stretch {
            let axis = if is_row { "maxHeight" } else { "maxWidth" };
            writeln!(
                buf,
                "{pad}    // NOTE: align-items: Stretch — add .frame({axis}: .infinity) to children",
            )?;
        }

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
            writeln!(buf, "{pad}    // NOTE: flex-direction: {dir_label} — children reversed in source to approximate visual order")?;
            children.reverse();
            starts.reverse();
        }

        match jc {
            JustifyContent::SpaceBetween => {
                for (i, (child, start)) in children.iter().zip(starts.iter()).enumerate() {
                    if i > 0 {
                        writeln!(buf, "{pad}    Spacer(minLength: 0)")?;
                    }
                    let mut idx = *start;
                    emit_swiftui_node(buf, child, depth + 1, &mut idx, palette, is_row)?;
                }
            }
            JustifyContent::Center => {
                writeln!(buf, "{pad}    Spacer(minLength: 0)")?;
                for (child, start) in children.iter().zip(starts.iter()) {
                    let mut idx = *start;
                    emit_swiftui_node(buf, child, depth + 1, &mut idx, palette, is_row)?;
                }
                writeln!(buf, "{pad}    Spacer(minLength: 0)")?;
            }
            JustifyContent::SpaceEvenly | JustifyContent::SpaceAround => {
                for (child, start) in children.iter().zip(starts.iter()) {
                    writeln!(buf, "{pad}    Spacer(minLength: 0)")?;
                    let mut idx = *start;
                    emit_swiftui_node(buf, child, depth + 1, &mut idx, palette, is_row)?;
                }
                writeln!(buf, "{pad}    Spacer(minLength: 0)")?;
            }
            JustifyContent::FlexEnd | JustifyContent::End => {
                writeln!(buf, "{pad}    Spacer(minLength: 0)")?;
                for (child, start) in children.iter().zip(starts.iter()) {
                    let mut idx = *start;
                    emit_swiftui_node(buf, child, depth + 1, &mut idx, palette, is_row)?;
                }
            }
            _ => {
                for (child, start) in children.iter().zip(starts.iter()) {
                    let mut idx = *start;
                    emit_swiftui_node(buf, child, depth + 1, &mut idx, palette, is_row)?;
                }
            }
        }

        writeln!(buf, "{pad}}}")?;

        // Container frame: map Percent(100%) to maxWidth/maxHeight: .infinity
        let full_w = is_full_percent(&node.width);
        let full_h = is_full_percent(&node.height);
        let w = if full_w { None } else { swift_optional_value(&node.width) };
        let h = if full_h { None } else { swift_optional_value(&node.height) };

        if w.is_some() || h.is_some() {
            let w_str = w.as_deref().unwrap_or("nil");
            let h_str = h.as_deref().unwrap_or("nil");
            writeln!(buf, "{pad}.frame(width: {w_str}, height: {h_str}, alignment: .topLeading)")?;
        }

        // Min/max constraints — merge 100% dimensions as .infinity, skip zero mins
        let min_w = if is_zero_px(&node.min_width) { None } else { swift_optional_value(&node.min_width) };
        let min_h = if is_zero_px(&node.min_height) { None } else { swift_optional_value(&node.min_height) };
        let max_w = if full_w { Some(".infinity".to_string()) } else { swift_optional_value(&node.max_width) };
        let max_h = if full_h { Some(".infinity".to_string()) } else { swift_optional_value(&node.max_height) };

        if min_w.is_some() || min_h.is_some() || max_w.is_some() || max_h.is_some() {
            writeln!(
                buf,
                "{pad}.frame(minWidth: {}, maxWidth: {}, minHeight: {}, maxHeight: {}, alignment: .topLeading)",
                min_w.as_deref().unwrap_or("nil"),
                max_w.as_deref().unwrap_or("nil"),
                min_h.as_deref().unwrap_or("nil"),
                max_h.as_deref().unwrap_or("nil"),
            )?;
        }

        // Flex-grow expansion (when not already handled by percent → infinity)
        if node.flex_grow > 0.0 && !full_w && !full_h {
            if parent_is_row {
                writeln!(buf, "{pad}.frame(maxWidth: .infinity, alignment: .topLeading)")?;
            } else {
                writeln!(buf, "{pad}.frame(maxHeight: .infinity, alignment: .topLeading)")?;
            }
        }

        if !is_zero_px(&node.padding) {
            if let Some(p) = swift_optional_value(&node.padding) {
                writeln!(buf, "{pad}.padding({p})")?;
            }
        }
        writeln!(
            buf,
            "{pad}.background(Color(red: 0.11, green: 0.11, blue: 0.17))"
        )?;
        if !is_zero_px(&node.margin) {
            if let Some(m) = swift_optional_value(&node.margin) {
                writeln!(buf, "{pad}.padding({m}) /* margin */")?;
            }
        }
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
        let code = emit_swiftui(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains("struct ContentView: View"));
        assert!(code.contains("public var body: some View"));
    }

    #[test]
    fn emits_hstack_for_row() {
        let code = emit_swiftui(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains("HStack"));
    }

    #[test]
    fn emits_vstack_for_column() {
        let mut root = test_container();
        root.flex_direction = FlexDirection::Column;
        let code = emit_swiftui(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains("VStack"));
    }

    #[test]
    fn emits_text_for_leaves() {
        let code = emit_swiftui(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains("Text(\"A\")"));
        assert!(code.contains("Text(\"B\")"));
    }

    #[test]
    fn emits_hidden_modifier() {
        let mut node = NodeConfig::new_leaf("A", 80.0, 80.0);
        node.visible = false;
        let mut root = NodeConfig::new_container("root");
        root.children = vec![node];
        let code = emit_swiftui(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains(".hidden()"));
    }

    #[test]
    fn percent_100_becomes_infinity() {
        let code = emit_swiftui(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains("maxWidth: .infinity"), "Percent(100) should map to maxWidth: .infinity");
        assert!(!code.contains("width: 100.0"), "should not emit width: 100.0 for Percent(100)");
    }

    #[test]
    fn skips_zero_margin() {
        let code = emit_swiftui(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(!code.contains(".padding(0.0) /* margin */"), "should not emit zero margin");
    }

    #[test]
    fn flex_grow_emits_infinity_frame() {
        let mut leaf = NodeConfig::new_leaf("A", 80.0, 80.0);
        leaf.flex_grow = 1.0;
        let mut root = NodeConfig::new_container("root");
        root.children = vec![leaf];
        let code = emit_swiftui(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains(".frame(maxWidth: .infinity)"), "flex-grow items should expand");
    }

    #[test]
    fn space_between_emits_spacers() {
        let mut root = test_container();
        root.justify_content = JustifyContent::SpaceBetween;
        let code = emit_swiftui(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains("Spacer(minLength: 0)"), "SpaceBetween should use Spacer()");
    }
}
