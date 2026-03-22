use std::fmt::Write;

use crate::config::*;
use anyhow::Result;

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

fn egui_size(v: &ValueConfig, axis: &str) -> String {
    match v {
        ValueConfig::Auto => format!("/* auto {axis} */"),
        ValueConfig::Px(n) => format!("{n:.1}"),
        ValueConfig::Percent(n) => format!(
            "{:.1} /* {n:.0}% — compute from parent size */",
            n / 100.0 * if axis == "width" { 400.0 } else { 300.0 }
        ),
        ValueConfig::Vw(n) => format!("{:.1} /* {n:.0}vw */", n / 100.0 * 400.0),
        ValueConfig::Vh(n) => format!("{:.1} /* {n:.0}vh */", n / 100.0 * 300.0),
    }
}

fn egui_margin(v: &ValueConfig) -> Option<String> {
    match v {
        ValueConfig::Auto => None,
        ValueConfig::Px(n) if *n == 0.0 => None,
        ValueConfig::Px(n) => Some(format!("{n:.1}")),
        ValueConfig::Percent(n) => Some(format!(
            "{n:.1} /* {n:.0}% — no percentage margin in egui */"
        )),
        ValueConfig::Vw(n) => Some(format!("{n:.1} /* {n:.0}vw */")),
        ValueConfig::Vh(n) => Some(format!("{n:.1} /* {n:.0}vh */")),
    }
}

fn egui_direction(dir: FlexDirection) -> &'static str {
    match dir {
        FlexDirection::Row => "egui::Layout::left_to_right",
        FlexDirection::RowReverse => "egui::Layout::right_to_left",
        FlexDirection::Column => "egui::Layout::top_down",
        FlexDirection::ColumnReverse => "egui::Layout::bottom_up",
    }
}

fn egui_cross_align(a: AlignItems) -> &'static str {
    match a {
        AlignItems::FlexStart | AlignItems::Start => "egui::Align::Min",
        AlignItems::FlexEnd | AlignItems::End => "egui::Align::Max",
        AlignItems::Center => "egui::Align::Center",
        AlignItems::Baseline => "egui::Align::Min",
        AlignItems::Stretch => "egui::Align::Min",
        _ => "egui::Align::Min",
    }
}

fn egui_gap(v: &ValueConfig) -> Option<String> {
    match v {
        ValueConfig::Px(n) if *n == 0.0 => None,
        ValueConfig::Px(n) => Some(format!("{n:.1}")),
        ValueConfig::Auto => None,
        ValueConfig::Percent(n) => Some(format!(
            "{n:.1} /* {n:.0}% — no percentage spacing in egui */"
        )),
        ValueConfig::Vw(n) => Some(format!("{n:.1} /* {n:.0}vw */")),
        ValueConfig::Vh(n) => Some(format!("{n:.1} /* {n:.0}vh */")),
    }
}

pub fn emit_egui(root: &NodeConfig, palette: ColorPalette) -> Result<String> {
    let mut buf = String::from("fn build_ui(ui: &mut egui::Ui) {\n");
    emit_egui_node(&mut buf, root, 1, &mut 0, palette, true, false, true)?;
    buf.push_str("\n}\n");
    Ok(buf)
}

fn emit_egui_node(
    buf: &mut String,
    node: &NodeConfig,
    depth: usize,
    leaf_idx: &mut usize,
    palette: ColorPalette,
    parent_is_row: bool,
    parent_stretch: bool,
    is_root: bool,
) -> Result<()> {
    let pad = "    ".repeat(depth);
    let is_leaf = node.children.is_empty();

    if is_leaf {
        let (r, g, b) = palette_color(palette, *leaf_idx);
        *leaf_idx += 1;
        let r8 = (r * 255.0).round() as u8;
        let g8 = (g * 255.0).round() as u8;
        let b8 = (b * 255.0).round() as u8;

        writeln!(buf, "{pad}egui::Frame::none()")?;
        writeln!(
            buf,
            "{pad}    .fill(egui::Color32::from_rgb({r8}, {g8}, {b8}))"
        )?;

        if let Some(p) = egui_margin(&node.padding) {
            writeln!(buf, "{pad}    .inner_margin({p})")?;
        }

        writeln!(buf, "{pad}    .show(ui, |ui| {{")?;

        // Size
        let has_w = !matches!(node.width, ValueConfig::Auto);
        let has_h = !matches!(node.height, ValueConfig::Auto);
        if has_w || has_h {
            let w_str = if has_w {
                egui_size(&node.width, "width")
            } else {
                "40.0".into()
            };
            let h_str = if has_h {
                egui_size(&node.height, "height")
            } else {
                "40.0".into()
            };
            writeln!(
                buf,
                "{pad}        ui.set_min_size(egui::vec2({w_str}, {h_str}));"
            )?;
        }

        // Min/max constraints
        if !matches!(node.min_width, ValueConfig::Auto) && !is_zero_px(&node.min_width) {
            writeln!(
                buf,
                "{pad}        ui.set_min_width({});",
                egui_size(&node.min_width, "width")
            )?;
        }
        if !matches!(node.min_height, ValueConfig::Auto) && !is_zero_px(&node.min_height) {
            writeln!(
                buf,
                "{pad}        ui.set_min_height({});",
                egui_size(&node.min_height, "height")
            )?;
        }
        if !matches!(node.max_width, ValueConfig::Auto) {
            writeln!(
                buf,
                "{pad}        ui.set_max_width({});",
                egui_size(&node.max_width, "width")
            )?;
        }
        if !matches!(node.max_height, ValueConfig::Auto) {
            writeln!(
                buf,
                "{pad}        ui.set_max_height({});",
                egui_size(&node.max_height, "height")
            )?;
        }

        writeln!(buf, "{pad}        ui.centered_and_justified(|ui| {{")?;
        writeln!(
            buf,
            "{pad}            ui.label(egui::RichText::new({:?}).size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));",
            node.label
        )?;
        writeln!(buf, "{pad}        }});")?;
        write!(buf, "{pad}    }})")?;

        // Notes for unsupported features
        if !is_zero_px(&node.margin) && !matches!(node.margin, ValueConfig::Auto) {
            writeln!(buf)?;
            write!(
                buf,
                "{pad}    // NOTE: margin: {} — use outer_margin() on Frame or add spacing",
                node.margin.display_short()
            )?;
        }
        if node.flex_grow > 0.0 {
            writeln!(buf)?;
            write!(
                buf,
                "{pad}    // NOTE: flex-grow: {} — no egui equivalent; use ui.available_size()",
                format_float(node.flex_grow)
            )?;
        }
        if node.flex_shrink != 1.0 {
            writeln!(buf)?;
            write!(
                buf,
                "{pad}    // NOTE: flex-shrink: {} — no egui equivalent",
                format_float(node.flex_shrink)
            )?;
        }
        if !matches!(node.flex_basis, ValueConfig::Auto) {
            writeln!(buf)?;
            write!(
                buf,
                "{pad}    // NOTE: flex-basis: {} — no egui equivalent",
                node.flex_basis.display_short()
            )?;
        }
        if node.align_self != AlignSelf::Auto {
            writeln!(buf)?;
            write!(
                buf,
                "{pad}    // NOTE: align-self: {:?} — no per-child cross-axis override in egui",
                node.align_self
            )?;
        }
        if !node.visible {
            writeln!(buf)?;
            write!(
                buf,
                "{pad}    // NOTE: hidden — conditionally include this widget"
            )?;
        }
        if node.order != 0 {
            writeln!(buf)?;
            write!(
                buf,
                "{pad}    // NOTE: order: {} — children pre-sorted in source",
                node.order
            )?;
        }
    } else {
        // ── Container node ──────────────────────────────────────────────
        let is_row = matches!(
            node.flex_direction,
            FlexDirection::Row | FlexDirection::RowReverse
        );
        let is_reversed = matches!(
            node.flex_direction,
            FlexDirection::RowReverse | FlexDirection::ColumnReverse
        );
        let stretch = node.align_items == AlignItems::Stretch;
        let wraps = matches!(node.flex_wrap, FlexWrap::Wrap | FlexWrap::WrapReverse);

        let dir_fn = egui_direction(node.flex_direction);
        let cross = egui_cross_align(node.align_items);

        // Gap: main-axis
        let gap = if is_row {
            &node.column_gap
        } else {
            &node.row_gap
        };

        // Build the Frame (background + padding)
        writeln!(buf, "{pad}egui::Frame::none()")?;
        if !is_leaf {
            writeln!(buf, "{pad}    .fill(egui::Color32::from_rgb(28, 28, 43))")?;
        }
        if let Some(p) = egui_margin(&node.padding) {
            writeln!(buf, "{pad}    .inner_margin({p})")?;
        }
        writeln!(buf, "{pad}    .show(ui, |ui| {{")?;

        // Root fills viewport
        if is_root {
            writeln!(buf, "{pad}        ui.set_min_size(ui.available_size());")?;
        } else {
            // Explicit sizing
            let has_w = !matches!(node.width, ValueConfig::Auto);
            let has_h = !matches!(node.height, ValueConfig::Auto);
            if has_w {
                writeln!(
                    buf,
                    "{pad}        ui.set_min_width({});",
                    egui_size(&node.width, "width")
                )?;
            }
            if has_h {
                writeln!(
                    buf,
                    "{pad}        ui.set_min_height({});",
                    egui_size(&node.height, "height")
                )?;
            }

            // Stretch from parent
            if parent_stretch {
                if parent_is_row && !has_h {
                    writeln!(
                        buf,
                        "{pad}        ui.set_min_height(ui.available_height());"
                    )?;
                } else if !parent_is_row && !has_w {
                    writeln!(buf, "{pad}        ui.set_min_width(ui.available_width());")?;
                }
            }
        }

        // Set gap via item_spacing
        if let Some(g) = egui_gap(gap) {
            if is_row {
                writeln!(
                    buf,
                    "{pad}        ui.spacing_mut().item_spacing = egui::vec2({g}, 0.0);"
                )?;
            } else {
                writeln!(
                    buf,
                    "{pad}        ui.spacing_mut().item_spacing = egui::vec2(0.0, {g});"
                )?;
            }
        } else {
            writeln!(
                buf,
                "{pad}        ui.spacing_mut().item_spacing = egui::Vec2::ZERO;"
            )?;
        }

        // Build layout
        let jc = node.justify_content;
        let needs_justify = matches!(
            jc,
            JustifyContent::SpaceBetween
                | JustifyContent::SpaceEvenly
                | JustifyContent::SpaceAround
        );
        let needs_center = matches!(jc, JustifyContent::Center);
        let needs_end = matches!(jc, JustifyContent::FlexEnd | JustifyContent::End);

        write!(buf, "{pad}        let layout = {dir_fn}({cross})")?;
        if stretch {
            writeln!(buf)?;
            write!(buf, "{pad}            .with_cross_justify(true)")?;
        }
        if wraps {
            writeln!(buf)?;
            write!(buf, "{pad}            .with_main_wrap(true)")?;
        }
        if needs_justify {
            writeln!(buf)?;
            write!(
                buf,
                "{pad}            .with_main_justify(true) // approximate {:?}",
                jc
            )?;
        }
        writeln!(buf, ";")?;

        if needs_center {
            writeln!(
                buf,
                "{pad}        // NOTE: justify-content: Center — egui Layout lacks main_align; center manually or use custom layout"
            )?;
        }
        if needs_end {
            writeln!(
                buf,
                "{pad}        // NOTE: justify-content: {:?} — egui Layout lacks main_align; reverse child order or use custom layout",
                jc
            )?;
        }

        // Wrap note
        if node.flex_wrap == FlexWrap::WrapReverse {
            writeln!(
                buf,
                "{pad}        // NOTE: flex-wrap: WrapReverse — egui has main_wrap but no reverse wrap"
            )?;
        }

        // Align-content note
        if !matches!(
            node.align_content,
            AlignContent::Default | AlignContent::FlexStart | AlignContent::Start
        ) {
            writeln!(
                buf,
                "{pad}        // NOTE: align-content: {:?} — no egui equivalent",
                node.align_content
            )?;
        }

        if node.align_items == AlignItems::Baseline {
            writeln!(
                buf,
                "{pad}        // NOTE: align-items: Baseline — approximated as Min; egui has no baseline alignment"
            )?;
        }

        writeln!(buf, "{pad}        ui.with_layout(layout, |ui| {{")?;

        // Sort children by order
        let mut children: Vec<&NodeConfig> = node.children.iter().collect();
        children.sort_by_key(|c| c.order);

        // Pre-compute leaf_idx starts
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
            writeln!(
                buf,
                "{pad}            // flex-direction: {dir_label} — handled by Layout direction"
            )?;
        }

        for (child, start) in children.iter().zip(starts.iter()) {
            let mut idx = *start;
            emit_egui_node(
                buf,
                child,
                depth + 3,
                &mut idx,
                palette,
                is_row,
                stretch,
                false,
            )?;
            writeln!(buf, ";")?;
        }

        writeln!(buf, "{pad}        }});")?;

        // Container-level notes
        if node.flex_grow > 0.0 {
            writeln!(
                buf,
                "{pad}        // NOTE: flex-grow: {} — use ui.available_size() to fill parent",
                format_float(node.flex_grow)
            )?;
        }
        if node.flex_shrink != 1.0 {
            writeln!(
                buf,
                "{pad}        // NOTE: flex-shrink: {} — no egui equivalent",
                format_float(node.flex_shrink)
            )?;
        }
        if !matches!(node.flex_basis, ValueConfig::Auto) {
            writeln!(
                buf,
                "{pad}        // NOTE: flex-basis: {} — no egui equivalent",
                node.flex_basis.display_short()
            )?;
        }
        if node.align_self != AlignSelf::Auto {
            writeln!(
                buf,
                "{pad}        // NOTE: align-self: {:?} — no per-child override in egui",
                node.align_self
            )?;
        }
        if !is_zero_px(&node.margin) && !matches!(node.margin, ValueConfig::Auto) {
            writeln!(
                buf,
                "{pad}        // NOTE: margin: {} — use outer_margin() on Frame",
                node.margin.display_short()
            )?;
        }
        if !node.visible {
            writeln!(
                buf,
                "{pad}        // NOTE: hidden — conditionally include this widget"
            )?;
        }
        if node.order != 0 {
            writeln!(
                buf,
                "{pad}        // NOTE: order: {} — children pre-sorted in source",
                node.order
            )?;
        }

        write!(buf, "{pad}    }})")?;
    }
    Ok(())
}

fn format_float(v: f32) -> String {
    if (v - v.round()).abs() < 0.005 {
        format!("{}", v as i32)
    } else {
        format!("{v:.1}")
    }
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
    fn emits_build_ui_function() {
        let code = emit_egui(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains("fn build_ui(ui: &mut egui::Ui)"));
    }

    #[test]
    fn emits_frame_for_leaves() {
        let code = emit_egui(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains("egui::Frame::none()"));
        assert!(code.contains("RichText::new(\"A\")"));
        assert!(code.contains("RichText::new(\"B\")"));
    }

    #[test]
    fn emits_layout_for_row() {
        let code = emit_egui(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains("left_to_right"));
    }

    #[test]
    fn emits_layout_for_column() {
        let mut root = test_container();
        root.flex_direction = FlexDirection::Column;
        let code = emit_egui(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains("top_down"));
    }

    #[test]
    fn emits_cross_justify_for_stretch() {
        let mut root = test_container();
        root.align_items = AlignItems::Stretch;
        for child in &mut root.children {
            child.height = ValueConfig::Auto;
        }
        let code = emit_egui(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains("with_cross_justify(true)"));
    }

    #[test]
    fn emits_main_wrap() {
        let mut root = test_container();
        root.flex_wrap = FlexWrap::Wrap;
        let code = emit_egui(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains("with_main_wrap(true)"));
    }

    #[test]
    fn hidden_emits_comment() {
        let mut node = NodeConfig::new_leaf("A", 80.0, 80.0);
        node.visible = false;
        let mut root = NodeConfig::new_container("root");
        root.children = vec![node];
        let code = emit_egui(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains("// NOTE: hidden"));
    }

    #[test]
    fn space_between_emits_main_justify() {
        let mut root = test_container();
        root.justify_content = JustifyContent::SpaceBetween;
        let code = emit_egui(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains("with_main_justify(true)"));
    }

    #[test]
    fn emits_item_spacing() {
        let code = emit_egui(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains("item_spacing"));
    }
}
