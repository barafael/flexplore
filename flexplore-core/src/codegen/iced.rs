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

fn is_full_percent(v: &ValueConfig) -> bool {
    matches!(v, ValueConfig::Percent(n) if *n >= 100.0)
}

fn iced_length(v: &ValueConfig) -> String {
    match v {
        ValueConfig::Auto => "Length::Shrink".into(),
        ValueConfig::Px(n) => format!("Length::Fixed({n:.1})"),
        ValueConfig::Percent(n) if (*n - 100.0).abs() < 0.01 => "Length::Fill".into(),
        ValueConfig::Percent(n) => {
            format!("Length::FillPortion({}) /* {n:.0}% */", *n as u16)
        }
        ValueConfig::Vw(n) => {
            format!("Length::Fixed({n:.1}) /* {n:.0}vw — no viewport units in Iced */")
        }
        ValueConfig::Vh(n) => {
            format!("Length::Fixed({n:.1}) /* {n:.0}vh — no viewport units in Iced */")
        }
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

fn iced_padding(v: &ValueConfig) -> Option<String> {
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

pub fn emit_iced(root: &NodeConfig, palette: ColorPalette) -> Result<String> {
    let mut buf = String::from("fn view(&self) -> iced::Element<'_, Message> {\n");
    emit_iced_node(&mut buf, root, 1, &mut 0, palette, true, false)?;
    buf.push_str("\n    .into()\n}\n");
    Ok(buf)
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
    let pad = "    ".repeat(depth);
    let is_leaf = node.children.is_empty();

    if is_leaf {
        let (r, g, b) = palette_color(palette, *leaf_idx);
        *leaf_idx += 1;

        writeln!(
            buf,
            "{pad}container(text({:?}).size(26).color(Color::from_rgba(0.05, 0.05, 0.1, 0.85)))",
            node.label
        )?;

        // Determine effective width: flex-grow or stretch may override Auto
        let grow_overrides_width =
            node.flex_grow > 0.0 && parent_is_row && matches!(node.width, ValueConfig::Auto);
        let stretch_overrides_width =
            parent_stretch && !parent_is_row && matches!(node.width, ValueConfig::Auto);
        let grow_overrides_height =
            node.flex_grow > 0.0 && !parent_is_row && matches!(node.height, ValueConfig::Auto);
        let stretch_overrides_height =
            parent_stretch && parent_is_row && matches!(node.height, ValueConfig::Auto);

        // Width
        if grow_overrides_width {
            let fill = if node.flex_grow > 1.0 {
                format!("Length::FillPortion({})", node.flex_grow as u16)
            } else {
                "Length::Fill".into()
            };
            writeln!(buf, "{pad}    .width({fill})")?;
        } else if stretch_overrides_width {
            writeln!(buf, "{pad}    .width(Length::Fill)")?;
        } else {
            writeln!(buf, "{pad}    .width({})", iced_length(&node.width))?;
        }

        // Height
        if grow_overrides_height {
            let fill = if node.flex_grow > 1.0 {
                format!("Length::FillPortion({})", node.flex_grow as u16)
            } else {
                "Length::Fill".into()
            };
            writeln!(buf, "{pad}    .height({fill})")?;
        } else if stretch_overrides_height {
            writeln!(buf, "{pad}    .height(Length::Fill)")?;
        } else {
            writeln!(buf, "{pad}    .height({})", iced_length(&node.height))?;
        }

        // Min/max constraints
        if !matches!(node.min_width, ValueConfig::Auto) && !is_zero_px(&node.min_width) {
            writeln!(
                buf,
                "{pad}    // NOTE: min-width: {} — no Iced equivalent",
                node.min_width.display_short()
            )?;
        }
        if !matches!(node.min_height, ValueConfig::Auto) && !is_zero_px(&node.min_height) {
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

        // Center the text content
        writeln!(buf, "{pad}    .center(Length::Fill)")?;

        // Background color
        writeln!(buf, "{pad}    .style(|_| container::Style {{")?;
        writeln!(
            buf,
            "{pad}        background: Some(Color::from_rgb({r:.2}, {g:.2}, {b:.2}).into()),"
        )?;
        writeln!(buf, "{pad}        ..Default::default()")?;
        write!(buf, "{pad}    }})")?;

        // Margin — no Iced equivalent
        if !is_zero_px(&node.margin) && !matches!(node.margin, ValueConfig::Auto) {
            writeln!(buf)?;
            write!(
                buf,
                "{pad}    // NOTE: margin: {} — no Iced equivalent",
                node.margin.display_short()
            )?;
        }

        // Flex-shrink — no Iced equivalent
        if node.flex_shrink != 1.0 {
            writeln!(buf)?;
            write!(
                buf,
                "{pad}    // NOTE: flex-shrink: {} — no Iced equivalent",
                format_float(node.flex_shrink)
            )?;
        }

        // Flex-basis — no Iced equivalent
        if !matches!(node.flex_basis, ValueConfig::Auto) {
            writeln!(buf)?;
            write!(
                buf,
                "{pad}    // NOTE: flex-basis: {} — no Iced equivalent",
                node.flex_basis.display_short()
            )?;
        }

        // Align-self — no Iced equivalent
        if node.align_self != AlignSelf::Auto {
            writeln!(buf)?;
            write!(
                buf,
                "{pad}    // NOTE: align-self: {:?} — no Iced equivalent",
                node.align_self
            )?;
        }

        // Visibility
        if !node.visible {
            writeln!(buf)?;
            write!(
                buf,
                "{pad}    // NOTE: hidden — Iced has no visibility modifier; conditionally include this widget"
            )?;
        }

        // Order
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

        writeln!(buf, "{pad}{macro_name}[")?;

        // Flex-wrap note
        if node.flex_wrap != FlexWrap::NoWrap {
            if is_row {
                writeln!(
                    buf,
                    "{pad}    // NOTE: flex-wrap: {:?} — call .wrap() on the Row for wrapping support",
                    node.flex_wrap
                )?;
            } else {
                writeln!(
                    buf,
                    "{pad}    // NOTE: flex-wrap: {:?} — Iced Column does not support wrapping",
                    node.flex_wrap
                )?;
            }
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

        // Sort children by order
        let mut children: Vec<&NodeConfig> = node.children.iter().collect();
        children.sort_by_key(|c| c.order);

        // Pre-compute leaf_idx starts for each child in sorted order
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
                "{pad}    // NOTE: flex-direction: {dir_label} — children reversed in source; Iced has no reverse direction"
            )?;
            children.reverse();
            starts.reverse();
        }

        let stretch = node.align_items == AlignItems::Stretch;
        let space = space_widget(is_row);

        match jc {
            JustifyContent::SpaceBetween => {
                for (i, (child, start)) in children.iter().zip(starts.iter()).enumerate() {
                    if i > 0 {
                        writeln!(buf, "{pad}    {space},")?;
                    }
                    let mut idx = *start;
                    emit_iced_node(buf, child, depth + 1, &mut idx, palette, is_row, stretch)?;
                    writeln!(buf, ",")?;
                }
            }
            JustifyContent::Center => {
                writeln!(buf, "{pad}    {space},")?;
                for (child, start) in children.iter().zip(starts.iter()) {
                    let mut idx = *start;
                    emit_iced_node(buf, child, depth + 1, &mut idx, palette, is_row, stretch)?;
                    writeln!(buf, ",")?;
                }
                writeln!(buf, "{pad}    {space},")?;
            }
            JustifyContent::SpaceEvenly | JustifyContent::SpaceAround => {
                for (child, start) in children.iter().zip(starts.iter()) {
                    writeln!(buf, "{pad}    {space},")?;
                    let mut idx = *start;
                    emit_iced_node(buf, child, depth + 1, &mut idx, palette, is_row, stretch)?;
                    writeln!(buf, ",")?;
                }
                writeln!(buf, "{pad}    {space},")?;
            }
            JustifyContent::FlexEnd | JustifyContent::End => {
                writeln!(buf, "{pad}    {space},")?;
                for (child, start) in children.iter().zip(starts.iter()) {
                    let mut idx = *start;
                    emit_iced_node(buf, child, depth + 1, &mut idx, palette, is_row, stretch)?;
                    writeln!(buf, ",")?;
                }
            }
            _ => {
                // FlexStart / Start / Default / Stretch
                for (child, start) in children.iter().zip(starts.iter()) {
                    let mut idx = *start;
                    emit_iced_node(buf, child, depth + 1, &mut idx, palette, is_row, stretch)?;
                    writeln!(buf, ",")?;
                }
            }
        }

        write!(buf, "{pad}]")?;

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
        if !matches!(node.min_width, ValueConfig::Auto) && !is_zero_px(&node.min_width) {
            writeln!(buf)?;
            write!(
                buf,
                "{pad}// NOTE: min-width: {} — no Iced equivalent on Row/Column",
                node.min_width.display_short()
            )?;
        }
        if !matches!(node.min_height, ValueConfig::Auto) && !is_zero_px(&node.min_height) {
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
        if node.flex_grow > 0.0 {
            if parent_is_row && !full_w && matches!(node.width, ValueConfig::Auto) {
                let fill = if node.flex_grow > 1.0 {
                    format!("Length::FillPortion({})", node.flex_grow as u16)
                } else {
                    "Length::Fill".into()
                };
                writeln!(buf)?;
                write!(buf, "{pad}.width({fill})")?;
            } else if !parent_is_row && !full_h && matches!(node.height, ValueConfig::Auto) {
                let fill = if node.flex_grow > 1.0 {
                    format!("Length::FillPortion({})", node.flex_grow as u16)
                } else {
                    "Length::Fill".into()
                };
                writeln!(buf)?;
                write!(buf, "{pad}.height({fill})")?;
            }
        }

        // align-items: Stretch from parent — expand along cross axis
        if parent_stretch {
            if parent_is_row && !full_h && matches!(node.height, ValueConfig::Auto) {
                writeln!(buf)?;
                write!(buf, "{pad}.height(Length::Fill)")?;
            } else if !parent_is_row && !full_w && matches!(node.width, ValueConfig::Auto) {
                writeln!(buf)?;
                write!(buf, "{pad}.width(Length::Fill)")?;
            }
        }

        // Padding
        if let Some(p) = iced_padding(&node.padding) {
            writeln!(buf)?;
            write!(buf, "{pad}.padding({p})")?;
        }

        // Margin — no Iced equivalent
        if !is_zero_px(&node.margin) && !matches!(node.margin, ValueConfig::Auto) {
            writeln!(buf)?;
            write!(
                buf,
                "{pad}// NOTE: margin: {} — no Iced equivalent",
                node.margin.display_short()
            )?;
        }

        // Flex-shrink — no Iced equivalent
        if node.flex_shrink != 1.0 {
            writeln!(buf)?;
            write!(
                buf,
                "{pad}// NOTE: flex-shrink: {} — no Iced equivalent",
                format_float(node.flex_shrink)
            )?;
        }

        // Flex-basis — no Iced equivalent
        if !matches!(node.flex_basis, ValueConfig::Auto) {
            writeln!(buf)?;
            write!(
                buf,
                "{pad}// NOTE: flex-basis: {} — no Iced equivalent",
                node.flex_basis.display_short()
            )?;
        }

        // Align-self — no Iced equivalent
        if node.align_self != AlignSelf::Auto {
            writeln!(buf)?;
            write!(
                buf,
                "{pad}// NOTE: align-self: {:?} — no Iced equivalent",
                node.align_self
            )?;
        }

        // Visibility
        if !node.visible {
            writeln!(buf)?;
            write!(
                buf,
                "{pad}// NOTE: hidden — Iced has no visibility modifier; conditionally include this widget"
            )?;
        }

        // Order
        if node.order != 0 {
            writeln!(buf)?;
            write!(
                buf,
                "{pad}// NOTE: order: {} — children pre-sorted in source",
                node.order
            )?;
        }
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
    fn emits_spacing() {
        let code = emit_iced(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains(".spacing("));
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
        leaf.margin = ValueConfig::Px(16.0);
        let mut root = NodeConfig::new_container("root");
        root.children = vec![leaf];
        let code = emit_iced(&root, ColorPalette::Pastel1).unwrap();
        assert!(
            code.contains("// NOTE: margin"),
            "margin should emit a comment"
        );
    }
}
