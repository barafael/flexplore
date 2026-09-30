use std::fmt::Write;

use crate::config::*;
use anyhow::Result;

use super::common::{
    grid_tracks, is_full_percent, rust_string_literal, sides_short,
    sorted_children_with_leaf_starts, take_leaf_color,
};
use crate::config::{ColorPalette, NodeConfig, ValueConfig};

fn iced_length(v: &ValueConfig) -> String {
    match v {
        ValueConfig::Auto => "Length::Shrink".into(),
        ValueConfig::Px(n) => format!("Length::Fixed({n:.1})"),
        ValueConfig::Percent(n) if (*n - 100.0).abs() < 0.01 => "Length::Fill".into(),
        ValueConfig::Percent(n) => {
            // FillPortion(0) is meaningless; anything under 1% still gets a share.
            format!("Length::FillPortion({}) /* {n:.0}% */", (*n as u16).max(1))
        }
        ValueConfig::Vw(n) => {
            format!("Length::Fixed({n:.1}) /* {n:.0}vw — no viewport units in Iced */")
        }
        ValueConfig::Vh(n) => {
            format!("Length::Fixed({n:.1}) /* {n:.0}vh — no viewport units in Iced */")
        }
    }
}

/// `Length::Fill`, or `FillPortion(n)` when flex-grow is above 1.
fn iced_grow_length(flex_grow: f32) -> String {
    if flex_grow > 1.0 {
        format!("Length::FillPortion({})", flex_grow as u16)
    } else {
        "Length::Fill".into()
    }
}

fn iced_spacing(v: &ValueConfig) -> Option<String> {
    match v {
        ValueConfig::Px(n) if *n == 0.0 => None,
        ValueConfig::Px(n) => Some(format!("{n:.1}")),
        ValueConfig::Auto => None,
        ValueConfig::Percent(n) => Some(format!(
            "{n:.1} /* {n:.0}% — no percentage spacing in Iced */"
        )),
        ValueConfig::Vw(n) => Some(format!("{n:.1} /* {n:.0}vw — no viewport units in Iced */")),
        ValueConfig::Vh(n) => Some(format!("{n:.1} /* {n:.0}vh — no viewport units in Iced */")),
    }
}

fn iced_padding_value(v: &ValueConfig) -> Option<String> {
    match v {
        ValueConfig::Auto => None,
        ValueConfig::Px(n) if *n == 0.0 => None,
        ValueConfig::Px(n) => Some(format!("{n:.1}")),
        ValueConfig::Percent(n) => Some(format!(
            "{n:.1} /* {n:.0}% — no percentage padding in Iced */"
        )),
        ValueConfig::Vw(n) => Some(format!("{n:.1} /* {n:.0}vw — no viewport units in Iced */")),
        ValueConfig::Vh(n) => Some(format!("{n:.1} /* {n:.0}vh — no viewport units in Iced */")),
    }
}

fn iced_padding(sides: &Sides) -> Option<String> {
    if sides.is_zero() {
        return None;
    }
    if sides.is_uniform() {
        return iced_padding_value(&sides.first());
    }
    // Per-side padding: Padding::from([top, right, bottom, left])
    let t = sides.top.num().unwrap_or(0.0);
    let r = sides.right.num().unwrap_or(0.0);
    let b = sides.bottom.num().unwrap_or(0.0);
    let l = sides.left.num().unwrap_or(0.0);
    Some(format!("Padding::from([{t:.1}, {r:.1}, {b:.1}, {l:.1}])"))
}

fn iced_cross_align_row(a: AlignItems) -> &'static str {
    match a {
        AlignItems::FlexStart | AlignItems::Start => "Vertical::Top",
        AlignItems::FlexEnd | AlignItems::End => "Vertical::Bottom",
        AlignItems::Center => "Vertical::Center",
        AlignItems::Baseline => "Vertical::Top",
        AlignItems::Stretch => "Vertical::Top",
        _ => "Vertical::Center",
    }
}

fn iced_cross_align_col(a: AlignItems) -> &'static str {
    match a {
        AlignItems::FlexStart | AlignItems::Start => "Horizontal::Left",
        AlignItems::FlexEnd | AlignItems::End => "Horizontal::Right",
        AlignItems::Center => "Horizontal::Center",
        AlignItems::Baseline => "Horizontal::Left",
        AlignItems::Stretch => "Horizontal::Left",
        _ => "Horizontal::Center",
    }
}

/// Main-axis Space widget: fills along the container direction.
fn space_widget(is_row: bool) -> &'static str {
    if is_row {
        "Space::new(Length::Fill, Length::Shrink)"
    } else {
        "Space::new(Length::Shrink, Length::Fill)"
    }
}

/// The `border: Border { ... }` fragment for a `.style()` closure comment.
fn iced_border_fields(node: &NodeConfig) -> String {
    let mut s = String::new();
    if !node.border_radius.is_zero() {
        let c = &node.border_radius;
        if c.is_uniform() {
            let _ = write!(s, "radius: {:.1}.into(), ", c.top_left);
        } else {
            let _ = write!(
                s,
                "radius: [{:.1}, {:.1}, {:.1}, {:.1}].into(), ",
                c.top_left, c.top_right, c.bottom_right, c.bottom_left
            );
        }
    }
    if !node.border_width.is_zero() {
        let w = node.border_width.first().num().unwrap_or(0.0);
        let _ = write!(s, "width: {w:.1}, color: Color::WHITE, ");
    }
    s
}

/// `// NOTE:` comments for properties Iced cannot express, appended after
/// the widget expression. Leaves and containers share the same order; only
/// the border advice differs (a leaf already is a `container`).
fn emit_iced_unsupported_notes(
    buf: &mut String,
    node: &NodeConfig,
    prefix: &str,
    is_leaf: bool,
) -> Result<()> {
    let mut notes: Vec<String> = Vec::new();
    if !node.margin.is_zero() {
        notes.push(format!(
            "margin: {} — no Iced equivalent",
            sides_short(&node.margin)
        ));
    }
    if !node.border_width.is_zero() || !node.border_radius.is_zero() {
        let how = if is_leaf {
            "apply via .style("
        } else {
            "wrap in container().style("
        };
        notes.push(format!(
            "border — {how}|_| container::Style {{ border: Border {{ {}..Default::default() }}, ..Default::default() }})",
            iced_border_fields(node)
        ));
    }
    if node.flex_shrink != 1.0 {
        notes.push(format!(
            "flex-shrink: {} — no Iced equivalent",
            format_float(node.flex_shrink)
        ));
    }
    if !matches!(node.flex_basis, ValueConfig::Auto) {
        notes.push(format!(
            "flex-basis: {} — no Iced equivalent",
            node.flex_basis.display_short()
        ));
    }
    if node.align_self != AlignSelf::Auto {
        notes.push(format!(
            "align-self: {:?} — no Iced equivalent",
            node.align_self
        ));
    }
    if !node.visible {
        notes.push(
            "hidden — Iced has no visibility modifier; conditionally include this widget".into(),
        );
    }
    if node.order != 0 {
        notes.push(format!(
            "order: {} — children pre-sorted in source",
            node.order
        ));
    }
    for note in notes {
        writeln!(buf)?;
        write!(buf, "{prefix}// NOTE: {note}")?;
    }
    Ok(())
}

pub fn emit_iced(root: &NodeConfig, palette: ColorPalette) -> Result<String> {
    let mut buf = String::from("fn view(&self) -> iced::Element<'_, Message> {\n");
    emit_iced_node(&mut buf, root, 1, &mut 0, palette, true, false)?;
    buf.push_str("\n    .into()\n}\n");
    Ok(buf)
}

/// Layout context a node inherits from its parent.
#[derive(Clone, Copy)]
struct Parent {
    is_row: bool,
    stretch: bool,
}

fn emit_iced_node(
    buf: &mut String,
    node: &NodeConfig,
    depth: usize,
    leaf_idx: &mut usize,
    palette: ColorPalette,
    parent_is_row: bool,
    parent_stretch: bool,
) -> Result<()> {
    let parent = Parent {
        is_row: parent_is_row,
        stretch: parent_stretch,
    };
    if node.children.is_empty() {
        emit_iced_leaf(buf, node, depth, leaf_idx, palette, parent)
    } else {
        emit_iced_container(buf, node, depth, leaf_idx, palette, parent)
    }
}

fn emit_iced_leaf(
    buf: &mut String,
    node: &NodeConfig,
    depth: usize,
    leaf_idx: &mut usize,
    palette: ColorPalette,
    parent: Parent,
) -> Result<()> {
    let pad = "    ".repeat(depth);
    let (r, g, b) = take_leaf_color(palette, leaf_idx);

    writeln!(
        buf,
        "{pad}container(text({}).size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))",
        rust_string_literal(node.display_text())
    )?;

    // Determine effective width: flex-grow or stretch may override Auto
    let width_auto = matches!(node.width, ValueConfig::Auto);
    let height_auto = matches!(node.height, ValueConfig::Auto);
    let grow_overrides_width = node.flex_grow > 0.0 && parent.is_row && width_auto;
    let stretch_overrides_width = parent.stretch && !parent.is_row && width_auto;
    let grow_overrides_height = node.flex_grow > 0.0 && !parent.is_row && height_auto;
    let stretch_overrides_height = parent.stretch && parent.is_row && height_auto;

    // Width
    if grow_overrides_width {
        writeln!(buf, "{pad}    .width({})", iced_grow_length(node.flex_grow))?;
    } else if stretch_overrides_width {
        writeln!(buf, "{pad}    .width(Length::Fill)")?;
    } else {
        writeln!(buf, "{pad}    .width({})", iced_length(&node.width))?;
    }

    // Height
    if grow_overrides_height {
        writeln!(
            buf,
            "{pad}    .height({})",
            iced_grow_length(node.flex_grow)
        )?;
    } else if stretch_overrides_height {
        writeln!(buf, "{pad}    .height(Length::Fill)")?;
    } else {
        writeln!(buf, "{pad}    .height({})", iced_length(&node.height))?;
    }

    // Min/max constraints
    if !matches!(node.min_width, ValueConfig::Auto) && !node.min_width.is_zero_px() {
        writeln!(
            buf,
            "{pad}    // NOTE: min-width: {} — no Iced equivalent",
            node.min_width.display_short()
        )?;
    }
    if !matches!(node.min_height, ValueConfig::Auto) && !node.min_height.is_zero_px() {
        writeln!(
            buf,
            "{pad}    // NOTE: min-height: {} — no Iced equivalent",
            node.min_height.display_short()
        )?;
    }
    if !matches!(node.max_width, ValueConfig::Auto) {
        writeln!(
            buf,
            "{pad}    .max_width({})",
            match &node.max_width {
                ValueConfig::Px(n) => format!("{n:.1}"),
                other => format!(
                    "{} /* {} — approximated */",
                    other.num().unwrap_or(0.0),
                    other.display_short()
                ),
            }
        )?;
    }
    if !matches!(node.max_height, ValueConfig::Auto) {
        writeln!(
            buf,
            "{pad}    // NOTE: max-height: {} — use Container wrapper for max_height",
            node.max_height.display_short()
        )?;
    }

    // Padding
    if let Some(p) = iced_padding(&node.padding) {
        writeln!(buf, "{pad}    .padding({p})")?;
    }

    // Center the text content. `Container::center(len)` would also reset
    // width/height to `len`, clobbering the explicit sizes above, so set the
    // alignments directly.
    writeln!(buf, "{pad}    .align_x(Horizontal::Center)")?;
    writeln!(buf, "{pad}    .align_y(Vertical::Center)")?;

    // Background color
    writeln!(buf, "{pad}    .style(|_| container::Style {{")?;
    writeln!(
        buf,
        "{pad}        background: Some(Color::from_rgb({r:.2}, {g:.2}, {b:.2}).into()),"
    )?;
    writeln!(buf, "{pad}        ..Default::default()")?;
    write!(buf, "{pad}    }})")?;

    emit_iced_unsupported_notes(buf, node, &format!("{pad}    "), true)
}

fn emit_iced_container(
    buf: &mut String,
    node: &NodeConfig,
    depth: usize,
    leaf_idx: &mut usize,
    palette: ColorPalette,
    parent: Parent,
) -> Result<()> {
    let pad = "    ".repeat(depth);
    let is_grid = node.display_mode == DisplayMode::Grid;
    let grid_col_count = if is_grid && !node.grid_template_columns.is_empty() {
        node.grid_template_columns.len()
    } else if is_grid {
        1
    } else {
        0
    };

    let is_row = if is_grid {
        true // Grid approximation uses row-based layout
    } else {
        matches!(
            node.flex_direction,
            FlexDirection::Row | FlexDirection::RowReverse
        )
    };
    let is_reversed = matches!(
        node.flex_direction,
        FlexDirection::RowReverse | FlexDirection::ColumnReverse
    );

    let macro_name = if is_row { "row!" } else { "column!" };
    let jc = node.justify_content;

    let uses_space_justification = matches!(
        jc,
        JustifyContent::SpaceBetween
            | JustifyContent::SpaceEvenly
            | JustifyContent::SpaceAround
            | JustifyContent::Center
            | JustifyContent::FlexEnd
            | JustifyContent::End
    );

    // Gap: main-axis gap
    let gap = if is_row {
        &node.column_gap
    } else {
        &node.row_gap
    };

    // CSS Grid comment
    if is_grid {
        writeln!(
            buf,
            "{pad}// CSS Grid: {grid_col_count} column{plural}",
            plural = if grid_col_count == 1 { "" } else { "s" }
        )?;
        if !node.grid_template_columns.is_empty() {
            writeln!(
                buf,
                "{pad}// grid-template-columns: {}",
                grid_tracks(&node.grid_template_columns)
            )?;
        }
        if !node.grid_template_rows.is_empty() {
            writeln!(
                buf,
                "{pad}// grid-template-rows: {}",
                grid_tracks(&node.grid_template_rows)
            )?;
        }
        writeln!(
            buf,
            "{pad}// Approximated with Row/Column — Iced has no CSS Grid support"
        )?;
    }

    writeln!(buf, "{pad}{macro_name}[")?;

    // Flex-wrap note
    if node.flex_wrap != FlexWrap::NoWrap && !is_row {
        writeln!(
            buf,
            "{pad}    // NOTE: flex-wrap: {:?} — Iced Column does not support wrapping",
            node.flex_wrap
        )?;
    }

    // Align-content note (no Iced equivalent)
    if !matches!(
        node.align_content,
        AlignContent::Default | AlignContent::FlexStart | AlignContent::Start
    ) {
        writeln!(
            buf,
            "{pad}    // NOTE: align-content: {:?} — no Iced equivalent",
            node.align_content
        )?;
    }

    let (mut children, mut starts) = sorted_children_with_leaf_starts(node, leaf_idx);

    if is_reversed {
        let dir_label = match node.flex_direction {
            FlexDirection::RowReverse => "RowReverse",
            FlexDirection::ColumnReverse => "ColumnReverse",
            _ => unreachable!(),
        };
        writeln!(
            buf,
            "{pad}    // NOTE: flex-direction: {dir_label} — children reversed in source; Iced has no reverse direction"
        )?;
        children.reverse();
        starts.reverse();
    }

    let stretch = node.align_items == AlignItems::Stretch;
    let space = space_widget(is_row);

    // Space widgets before/between/after children approximate justify-content.
    let (space_before, space_between, space_after) = match jc {
        JustifyContent::SpaceBetween => (false, true, false),
        JustifyContent::Center => (true, false, true),
        JustifyContent::SpaceEvenly | JustifyContent::SpaceAround => (true, true, true),
        JustifyContent::FlexEnd | JustifyContent::End => (true, false, false),
        _ => (false, false, false), // FlexStart / Start / Default / Stretch
    };
    for (i, (child, start)) in children.iter().zip(starts.iter()).enumerate() {
        if (i == 0 && space_before) || (i > 0 && space_between) {
            writeln!(buf, "{pad}    {space},")?;
        }
        let mut idx = *start;
        emit_iced_node(buf, child, depth + 1, &mut idx, palette, is_row, stretch)?;
        writeln!(buf, ",")?;
    }
    if space_after {
        writeln!(buf, "{pad}    {space},")?;
    }

    write!(buf, "{pad}]")?;

    // Wrapping — emit .wrap() on Row
    if (is_grid || node.flex_wrap != FlexWrap::NoWrap) && is_row {
        writeln!(buf)?;
        write!(buf, "{pad}.wrap()")?;
    }

    // Spacing
    if uses_space_justification {
        // When using Space widgets for justification, suppress gap
        // but note the original gap value if nonzero.
        if let Some(g) = iced_spacing(gap) {
            writeln!(buf)?;
            write!(
                buf,
                "{pad}.spacing(0) // original gap: {g}; suppressed for Space-based justification"
            )?;
        }
    } else if let Some(g) = iced_spacing(gap) {
        writeln!(buf)?;
        write!(buf, "{pad}.spacing({g})")?;
    }

    // Cross-axis alignment
    let align = if is_row {
        iced_cross_align_row(node.align_items)
    } else {
        iced_cross_align_col(node.align_items)
    };
    let default_align = if is_row {
        "Vertical::Center"
    } else {
        "Horizontal::Center"
    };
    if align != default_align {
        writeln!(buf)?;
        if is_row {
            write!(buf, "{pad}.align_y({align})")?;
        } else {
            write!(buf, "{pad}.align_x({align})")?;
        }
    }

    if node.align_items == AlignItems::Baseline {
        writeln!(buf)?;
        write!(
            buf,
            "{pad}// NOTE: align-items: Baseline — approximated as Top/Left; Iced has no baseline alignment"
        )?;
    }

    // Width
    let full_w = is_full_percent(&node.width);
    if full_w {
        writeln!(buf)?;
        write!(buf, "{pad}.width(Length::Fill)")?;
    } else if !matches!(node.width, ValueConfig::Auto) {
        writeln!(buf)?;
        write!(buf, "{pad}.width({})", iced_length(&node.width))?;
    }

    // Height
    let full_h = is_full_percent(&node.height);
    if full_h {
        writeln!(buf)?;
        write!(buf, "{pad}.height(Length::Fill)")?;
    } else if !matches!(node.height, ValueConfig::Auto) {
        writeln!(buf)?;
        write!(buf, "{pad}.height({})", iced_length(&node.height))?;
    }

    // Min/max constraints
    if !matches!(node.min_width, ValueConfig::Auto) && !node.min_width.is_zero_px() {
        writeln!(buf)?;
        write!(
            buf,
            "{pad}// NOTE: min-width: {} — no Iced equivalent on Row/Column",
            node.min_width.display_short()
        )?;
    }
    if !matches!(node.min_height, ValueConfig::Auto) && !node.min_height.is_zero_px() {
        writeln!(buf)?;
        write!(
            buf,
            "{pad}// NOTE: min-height: {} — no Iced equivalent on Row/Column",
            node.min_height.display_short()
        )?;
    }
    if !matches!(node.max_width, ValueConfig::Auto) {
        writeln!(buf)?;
        write!(
            buf,
            "{pad}// NOTE: max-width: {} — wrap in Container for .max_width()",
            node.max_width.display_short()
        )?;
    }
    if !matches!(node.max_height, ValueConfig::Auto) {
        writeln!(buf)?;
        write!(
            buf,
            "{pad}// NOTE: max-height: {} — wrap in Container for .max_height()",
            node.max_height.display_short()
        )?;
    }

    // Flex-grow: expand along parent's main axis
    let width_auto = matches!(node.width, ValueConfig::Auto);
    let height_auto = matches!(node.height, ValueConfig::Auto);
    if node.flex_grow > 0.0 {
        if parent.is_row && !full_w && width_auto {
            writeln!(buf)?;
            write!(buf, "{pad}.width({})", iced_grow_length(node.flex_grow))?;
        } else if !parent.is_row && !full_h && height_auto {
            writeln!(buf)?;
            write!(buf, "{pad}.height({})", iced_grow_length(node.flex_grow))?;
        }
    }

    // align-items: Stretch from parent — expand along cross axis
    if parent.stretch {
        if parent.is_row && !full_h && height_auto {
            writeln!(buf)?;
            write!(buf, "{pad}.height(Length::Fill)")?;
        } else if !parent.is_row && !full_w && width_auto {
            writeln!(buf)?;
            write!(buf, "{pad}.width(Length::Fill)")?;
        }
    }

    // Padding
    if let Some(p) = iced_padding(&node.padding) {
        writeln!(buf)?;
        write!(buf, "{pad}.padding({p})")?;
    }

    emit_iced_unsupported_notes(buf, node, &pad, false)
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
    fn emits_view_function() {
        let code = emit_iced(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains("fn view(&self) -> iced::Element<'_, Message>"));
    }

    #[test]
    fn emits_row_for_row() {
        let code = emit_iced(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains("row!["));
    }

    #[test]
    fn emits_column_for_column() {
        let mut root = test_container();
        root.flex_direction = FlexDirection::Column;
        let code = emit_iced(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains("column!["));
    }

    #[test]
    fn emits_text_for_leaves() {
        let code = emit_iced(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains("text(\"A\")"));
        assert!(code.contains("text(\"B\")"));
    }

    #[test]
    fn escapes_leaf_text() {
        let mut root = NodeConfig::new_container("root");
        root.children = vec![NodeConfig::new_leaf("a\"b\\c", 80.0, 80.0)];
        let code = emit_iced(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains(r#"text("a\"b\\c")"#), "{code}");
    }

    #[test]
    fn emits_spacing() {
        let code = emit_iced(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains(".spacing("));
    }

    #[test]
    fn leaf_centers_without_clobbering_size() {
        // `.center(Length::Fill)` resets width/height in iced 0.14; the leaf
        // must keep its explicit size and set the alignments directly.
        let code = emit_iced(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(!code.contains(".center("), "{code}");
        assert!(code.contains(".align_x(Horizontal::Center)"), "{code}");
        assert!(code.contains(".align_y(Vertical::Center)"), "{code}");
        assert!(code.contains(".width(Length::Fixed(80.0))"), "{code}");
    }

    #[test]
    fn fill_portion_never_zero() {
        assert_eq!(
            iced_length(&ValueConfig::Percent(0.5)),
            "Length::FillPortion(1) /* 0% */"
        );
        assert_eq!(
            iced_length(&ValueConfig::Percent(25.0)),
            "Length::FillPortion(25) /* 25% */"
        );
    }

    #[test]
    fn flex_grow_emits_fill() {
        let mut leaf = NodeConfig::new_leaf("A", 80.0, 80.0);
        leaf.flex_grow = 1.0;
        leaf.width = ValueConfig::Auto;
        let mut root = NodeConfig::new_container("root");
        root.children = vec![leaf];
        let code = emit_iced(&root, ColorPalette::Pastel1).unwrap();
        assert!(
            code.contains("Length::Fill"),
            "flex-grow items should use Length::Fill"
        );
    }

    #[test]
    fn space_between_emits_space() {
        let mut root = test_container();
        root.justify_content = JustifyContent::SpaceBetween;
        let code = emit_iced(&root, ColorPalette::Pastel1).unwrap();
        assert!(
            code.contains("Space::new("),
            "SpaceBetween should use Space widgets"
        );
    }

    #[test]
    fn stretch_emits_fill_cross() {
        let mut root = test_container();
        root.align_items = AlignItems::Stretch;
        root.height = ValueConfig::Px(300.0);
        // Give children Auto height so stretch applies
        for child in &mut root.children {
            child.height = ValueConfig::Auto;
        }
        let code = emit_iced(&root, ColorPalette::Pastel1).unwrap();
        assert!(
            code.contains(".height(Length::Fill)"),
            "Stretch should set cross-axis to Length::Fill"
        );
    }

    #[test]
    fn hidden_emits_comment() {
        let mut node = NodeConfig::new_leaf("A", 80.0, 80.0);
        node.visible = false;
        let mut root = NodeConfig::new_container("root");
        root.children = vec![node];
        let code = emit_iced(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains("// NOTE: hidden"));
    }

    #[test]
    fn percent_100_becomes_fill() {
        let code = emit_iced(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(
            code.contains(".width(Length::Fill)"),
            "Percent(100) should map to Length::Fill"
        );
    }

    #[test]
    fn margin_emits_comment() {
        let mut leaf = NodeConfig::new_leaf("A", 80.0, 80.0);
        leaf.margin = Sides::uniform(ValueConfig::Px(16.0));
        let mut root = NodeConfig::new_container("root");
        root.children = vec![leaf];
        let code = emit_iced(&root, ColorPalette::Pastel1).unwrap();
        assert!(
            code.contains("// NOTE: margin"),
            "margin should emit a comment"
        );
    }
}
