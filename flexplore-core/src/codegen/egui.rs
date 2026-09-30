use std::fmt::Write;

use crate::config::*;
use anyhow::Result;

use super::common::{
    rgb8, rust_string_literal, sides_short, sorted_children_with_leaf_starts, take_leaf_color,
};
use crate::config::{ColorPalette, NodeConfig, ValueConfig};

fn egui_align_self_main(a: AlignSelf) -> Option<&'static str> {
    match a {
        AlignSelf::Center => Some("egui::Align::Center"),
        AlignSelf::FlexStart | AlignSelf::Start => Some("egui::Align::Min"),
        AlignSelf::FlexEnd | AlignSelf::End => Some("egui::Align::Max"),
        _ => None,
    }
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

fn egui_margin_value(v: &ValueConfig) -> Option<String> {
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

fn egui_margin(sides: &Sides) -> Option<String> {
    if sides.is_zero() {
        return None;
    }
    if sides.is_uniform() {
        return egui_margin_value(&sides.first());
    }
    // Per-side margin: Margin { top, bottom, left, right }
    let t = sides.top.num().unwrap_or(0.0);
    let r = sides.right.num().unwrap_or(0.0);
    let b = sides.bottom.num().unwrap_or(0.0);
    let l = sides.left.num().unwrap_or(0.0);
    Some(format!(
        "Margin {{ top: {t:.1}, bottom: {b:.1}, left: {l:.1}, right: {r:.1} }}"
    ))
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

/// `// NOTE:` comments for properties egui cannot express directly.
///
/// The two node kinds list the notes in a different order and with slightly
/// different advice (leaves live inside a `Frame`, containers own a layout),
/// so the messages are built once here and ordered per kind.
fn egui_unsupported_notes(node: &NodeConfig, is_leaf: bool) -> Vec<String> {
    let border = (!node.border_width.is_zero()).then(|| {
        let w = node.border_width.first().num().unwrap_or(0.0);
        format!("border — add .stroke(Stroke::new({w:.1}, Color32::WHITE)) to Frame")
    });
    let radius = (!node.border_radius.is_zero()).then(|| {
        let c = &node.border_radius;
        if c.is_uniform() {
            format!("border-radius — add .rounding({:.1}) to Frame", c.top_left)
        } else {
            format!(
                "border-radius — add .rounding(Rounding {{ nw: {:.1}, ne: {:.1}, se: {:.1}, sw: {:.1} }}) to Frame",
                c.top_left, c.top_right, c.bottom_right, c.bottom_left
            )
        }
    });
    let margin = (!node.margin.is_zero()).then(|| {
        let advice = if is_leaf {
            "use outer_margin() on Frame or add spacing"
        } else {
            "use outer_margin() on Frame"
        };
        format!("margin: {} — {advice}", sides_short(&node.margin))
    });
    let grow = (node.flex_grow > 0.0).then(|| {
        let advice = if is_leaf {
            "no egui equivalent; use ui.available_size()"
        } else {
            "use ui.available_size() to fill parent"
        };
        format!("flex-grow: {} — {advice}", format_float(node.flex_grow))
    });
    let shrink = (node.flex_shrink != 1.0).then(|| {
        format!(
            "flex-shrink: {} — no egui equivalent",
            format_float(node.flex_shrink)
        )
    });
    let basis = (!matches!(node.flex_basis, ValueConfig::Auto)).then(|| {
        format!(
            "flex-basis: {} — no egui equivalent",
            node.flex_basis.display_short()
        )
    });
    let align_self = (egui_align_self_main(node.align_self).is_none()
        && node.align_self != AlignSelf::Auto)
        .then(|| {
            let what = if is_leaf {
                "no per-child cross-axis override in egui"
            } else {
                "no per-child override in egui"
            };
            format!("align-self: {:?} — {what}", node.align_self)
        });
    let hidden = (!node.visible).then(|| "hidden — conditionally include this widget".to_string());
    let order =
        (node.order != 0).then(|| format!("order: {} — children pre-sorted in source", node.order));

    let ordered = if is_leaf {
        [
            border, radius, margin, grow, shrink, basis, align_self, hidden, order,
        ]
    } else {
        [
            grow, shrink, basis, align_self, margin, border, radius, hidden, order,
        ]
    };
    ordered.into_iter().flatten().collect()
}

pub fn emit_egui(root: &NodeConfig, palette: ColorPalette) -> Result<String> {
    let mut buf = String::from("fn build_ui(ui: &mut egui::Ui) {\n");
    emit_egui_node(&mut buf, root, 1, &mut 0, palette, true, false, true)?;
    buf.push_str("\n}\n");
    Ok(buf)
}

/// Layout context a node inherits from its parent.
#[derive(Clone, Copy)]
struct Parent {
    is_row: bool,
    stretch: bool,
}

#[allow(clippy::too_many_arguments)] // recursive tree-walker; leaf_idx varies per call site
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
    let parent = Parent {
        is_row: parent_is_row,
        stretch: parent_stretch,
    };
    if node.children.is_empty() {
        emit_egui_leaf(buf, node, depth, leaf_idx, palette)
    } else {
        emit_egui_container(buf, node, depth, leaf_idx, palette, parent, is_root)
    }
}

fn emit_egui_leaf(
    buf: &mut String,
    node: &NodeConfig,
    depth: usize,
    leaf_idx: &mut usize,
    palette: ColorPalette,
) -> Result<()> {
    let pad = "    ".repeat(depth);
    let (r, g, b) = take_leaf_color(palette, leaf_idx);
    let (r8, g8, b8) = rgb8(r, g, b);

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
    if !matches!(node.min_width, ValueConfig::Auto) && !node.min_width.is_zero_px() {
        writeln!(
            buf,
            "{pad}        ui.set_min_width({});",
            egui_size(&node.min_width, "width")
        )?;
    }
    if !matches!(node.min_height, ValueConfig::Auto) && !node.min_height.is_zero_px() {
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
        "{pad}            ui.label(egui::RichText::new({}).size(26.0).color(egui::Color32::from_rgba_premultiplied(13, 13, 26, 217)));",
        rust_string_literal(node.display_text())
    )?;
    writeln!(buf, "{pad}        }});")?;
    write!(buf, "{pad}    }})")?;

    // Notes for unsupported features — each on its own line after the
    // expression, without a trailing newline (the caller appends `;`).
    for note in egui_unsupported_notes(node, true) {
        writeln!(buf)?;
        write!(buf, "{pad}    // NOTE: {note}")?;
    }
    Ok(())
}

fn emit_egui_container(
    buf: &mut String,
    node: &NodeConfig,
    depth: usize,
    leaf_idx: &mut usize,
    palette: ColorPalette,
    parent: Parent,
    is_root: bool,
) -> Result<()> {
    let pad = "    ".repeat(depth);
    let is_grid = node.display_mode == DisplayMode::Grid;
    let is_row = matches!(
        node.flex_direction,
        FlexDirection::Row | FlexDirection::RowReverse
    );
    let stretch = node.align_items == AlignItems::Stretch;

    // Build the Frame (background + padding)
    writeln!(buf, "{pad}egui::Frame::none()")?;
    writeln!(buf, "{pad}    .fill(egui::Color32::from_rgb(28, 28, 43))")?;
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
        if parent.stretch {
            if parent.is_row && !has_h {
                writeln!(
                    buf,
                    "{pad}        ui.set_min_height(ui.available_height());"
                )?;
            } else if !parent.is_row && !has_w {
                writeln!(buf, "{pad}        ui.set_min_width(ui.available_width());")?;
            }
        }
    }

    let (children, starts) = sorted_children_with_leaf_starts(node, leaf_idx);

    if is_grid {
        emit_egui_grid(buf, node, &pad, depth, &children, &starts, palette, stretch)?;
    } else {
        emit_egui_flex(
            buf, node, &pad, depth, &children, &starts, palette, is_row, stretch,
        )?;
    }

    // Container-level notes
    for note in egui_unsupported_notes(node, false) {
        writeln!(buf, "{pad}        // NOTE: {note}")?;
    }

    write!(buf, "{pad}    }})")?;
    Ok(())
}

/// CSS Grid → `egui::Grid`, inserting `ui.end_row()` after every `num_cols` items.
#[allow(clippy::too_many_arguments)]
fn emit_egui_grid(
    buf: &mut String,
    node: &NodeConfig,
    pad: &str,
    depth: usize,
    children: &[&NodeConfig],
    starts: &[usize],
    palette: ColorPalette,
    stretch: bool,
) -> Result<()> {
    let num_cols = if node.grid_template_columns.is_empty() {
        1
    } else {
        node.grid_template_columns.len()
    };

    // Set gap via item_spacing (row_gap, column_gap)
    let col_gap = egui_gap(&node.column_gap);
    let row_gap = egui_gap(&node.row_gap);
    match (&col_gap, &row_gap) {
        (Some(cg), Some(rg)) => {
            writeln!(
                buf,
                "{pad}        ui.spacing_mut().item_spacing = egui::vec2({cg}, {rg});"
            )?;
        }
        (Some(cg), None) => {
            writeln!(
                buf,
                "{pad}        ui.spacing_mut().item_spacing = egui::vec2({cg}, 0.0);"
            )?;
        }
        (None, Some(rg)) => {
            writeln!(
                buf,
                "{pad}        ui.spacing_mut().item_spacing = egui::vec2(0.0, {rg});"
            )?;
        }
        (None, None) => {
            writeln!(
                buf,
                "{pad}        ui.spacing_mut().item_spacing = egui::Vec2::ZERO;"
            )?;
        }
    }

    writeln!(
        buf,
        "{pad}        egui::Grid::new({})",
        rust_string_literal(&node.label)
    )?;
    writeln!(buf, "{pad}            .num_columns({num_cols})")?;
    writeln!(buf, "{pad}            .show(ui, |ui| {{")?;

    for (i, (child, start)) in children.iter().zip(starts.iter()).enumerate() {
        let mut idx = *start;
        emit_egui_node(
            buf,
            child,
            depth + 4,
            &mut idx,
            palette,
            true, // grid children laid out in rows
            stretch,
            false,
        )?;
        writeln!(buf, ";")?;
        if (i + 1) % num_cols == 0 {
            writeln!(buf, "{pad}                ui.end_row();")?;
        }
    }

    writeln!(buf, "{pad}            }});")?;
    Ok(())
}

/// Flex container → `egui::Layout` with `ui.with_layout`.
#[allow(clippy::too_many_arguments)]
fn emit_egui_flex(
    buf: &mut String,
    node: &NodeConfig,
    pad: &str,
    depth: usize,
    children: &[&NodeConfig],
    starts: &[usize],
    palette: ColorPalette,
    is_row: bool,
    stretch: bool,
) -> Result<()> {
    let is_reversed = matches!(
        node.flex_direction,
        FlexDirection::RowReverse | FlexDirection::ColumnReverse
    );
    let wraps = matches!(node.flex_wrap, FlexWrap::Wrap | FlexWrap::WrapReverse);
    let dir_fn = egui_direction(node.flex_direction);
    let cross = egui_cross_align(node.align_items);

    // Gap: main-axis, set via item_spacing
    let gap = if is_row {
        &node.column_gap
    } else {
        &node.row_gap
    };
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
        JustifyContent::SpaceBetween | JustifyContent::SpaceEvenly | JustifyContent::SpaceAround
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
        writeln!(
            buf,
            "{pad}            .with_main_justify(true); // approximate {:?}",
            jc
        )?;
    } else {
        writeln!(buf, ";")?;
    }

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
        if let Some(main_align) = egui_align_self_main(child.align_self) {
            let child_pad = "    ".repeat(depth + 3);
            let wrapper_dir = if is_row {
                "egui::Layout::top_down(egui::Align::Min)"
            } else {
                "egui::Layout::left_to_right(egui::Align::Min)"
            };
            let fill_axis = if is_row {
                "ui.set_min_height(ui.available_height());"
            } else {
                "ui.set_min_width(ui.available_width());"
            };
            writeln!(
                buf,
                "{child_pad}ui.with_layout({wrapper_dir}.with_main_align({main_align}), |ui| {{"
            )?;
            writeln!(buf, "{child_pad}    {fill_axis}")?;
            emit_egui_node(
                buf,
                child,
                depth + 4,
                &mut idx,
                palette,
                is_row,
                stretch,
                false,
            )?;
            writeln!(buf, ";")?;
            write!(buf, "{child_pad}}})")?;
        } else {
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
        }
        writeln!(buf, ";")?;
    }

    writeln!(buf, "{pad}        }});")?;
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
    fn escapes_label_and_grid_id() {
        let mut root = NodeConfig::new_grid("my \"grid\"", vec![GridTrackSize::Fr(1.0)]);
        root.children = vec![NodeConfig::new_leaf("a\\b\"c", 80.0, 80.0)];
        let code = emit_egui(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains(r#"egui::Grid::new("my \"grid\"")"#), "{code}");
        assert!(code.contains(r#"RichText::new("a\\b\"c")"#), "{code}");
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

    #[test]
    fn note_order_differs_per_kind() {
        let mut leaf = NodeConfig::new_leaf("A", 80.0, 80.0);
        leaf.margin = Sides::uniform(ValueConfig::Px(4.0));
        leaf.flex_grow = 1.0;
        leaf.border_width = Sides::uniform(ValueConfig::Px(1.0));
        let leaf_notes = egui_unsupported_notes(&leaf, true);
        assert!(leaf_notes[0].starts_with("border"), "{leaf_notes:?}");
        assert!(leaf_notes[1].starts_with("margin"), "{leaf_notes:?}");
        assert!(leaf_notes[2].starts_with("flex-grow"), "{leaf_notes:?}");
        let container_notes = egui_unsupported_notes(&leaf, false);
        assert!(container_notes[0].starts_with("flex-grow"));
        assert!(container_notes[1].starts_with("margin"));
        assert!(container_notes[2].starts_with("border"));
    }
}
