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

fn is_zero_px(v: &ValueConfig) -> bool {
    matches!(v, ValueConfig::Px(n) if *n == 0.0)
}

fn is_full_percent(v: &ValueConfig) -> bool {
    matches!(v, ValueConfig::Percent(n) if *n >= 100.0)
}

fn needs_wrap(node: &NodeConfig) -> bool {
    if !node.children.is_empty() && node.flex_wrap != FlexWrap::NoWrap {
        return true;
    }
    node.children.iter().any(needs_wrap)
}

fn swift_line_alignment(ac: AlignContent) -> &'static str {
    match ac {
        AlignContent::FlexStart | AlignContent::Start | AlignContent::Stretch => ".start",
        AlignContent::FlexEnd | AlignContent::End => ".end",
        AlignContent::Center => ".center",
        AlignContent::SpaceBetween => ".spaceBetween",
        AlignContent::SpaceAround => ".spaceAround",
        AlignContent::SpaceEvenly => ".spaceEvenly",
        _ => ".start",
    }
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

fn emit_swift_padding(
    buf: &mut String,
    prefix: &str,
    sides: &Sides,
    comment: &str,
) -> Result<(), std::fmt::Error> {
    if sides.is_zero() {
        return Ok(());
    }
    if sides.is_uniform() {
        if let Some(v) = swift_optional_value(&sides.first()) {
            writeln!(buf, "{prefix}.padding({v}){comment}")?;
        }
    } else {
        let edge_map: &[(&str, &ValueConfig)] = &[
            (".top", &sides.top),
            (".leading", &sides.left),
            (".bottom", &sides.bottom),
            (".trailing", &sides.right),
        ];
        for (edge, val) in edge_map {
            if let Some(v) = swift_optional_value(val) {
                writeln!(buf, "{prefix}.padding({edge}, {v}){comment}")?;
            }
        }
    }
    Ok(())
}

fn emit_swift_border(
    buf: &mut String,
    prefix: &str,
    border_width: &Sides,
    border_radius: &Corners,
) -> Result<(), std::fmt::Error> {
    if !border_radius.is_zero() {
        if border_radius.is_uniform() {
            writeln!(buf, "{prefix}.cornerRadius({:.1})", border_radius.top_left)?;
        } else {
            writeln!(
                buf,
                "{prefix}.clipShape(RoundedRectangle(cornerSize: CGSize(width: {:.1}, height: {:.1}))) /* per-corner: tl={:.1} tr={:.1} br={:.1} bl={:.1} */",
                border_radius.top_left,
                border_radius.top_left,
                border_radius.top_left,
                border_radius.top_right,
                border_radius.bottom_right,
                border_radius.bottom_left,
            )?;
        }
    }
    if !border_width.is_zero() {
        if border_width.is_uniform() {
            if let Some(w) = swift_optional_value(&border_width.first()) {
                writeln!(buf, "{prefix}.border(Color.primary, width: {w})")?;
            }
        } else {
            writeln!(buf, "{prefix}.overlay(")?;
            writeln!(buf, "{prefix}    Rectangle().inset(by: 0)")?;
            let edge_map: &[(&str, &ValueConfig)] = &[
                ("top", &border_width.top),
                ("leading", &border_width.left),
                ("bottom", &border_width.bottom),
                ("trailing", &border_width.right),
            ];
            for (edge, val) in edge_map {
                if let Some(w) = swift_optional_value(val) {
                    writeln!(buf, "{prefix}        /* {edge}: {w} */")?;
                }
            }
            writeln!(buf, "{prefix}        .stroke(Color.primary)")?;
            writeln!(buf, "{prefix})")?;
        }
    }
    Ok(())
}

fn swift_flex_basis_value(v: &ValueConfig, is_width: bool) -> Option<String> {
    match v {
        ValueConfig::Percent(n) if *n > 0.0 => {
            let screen_dim = if is_width {
                "UIScreen.main.bounds.width"
            } else {
                "UIScreen.main.bounds.height"
            };
            Some(format!("{screen_dim} * {:.3}", n / 100.0))
        }
        _ => None,
    }
}

fn swift_spacing_value(v: &ValueConfig) -> Option<String> {
    match v {
        ValueConfig::Px(n) => Some(format!("{n:.1}")),
        ValueConfig::Vw(n) => Some(format!("UIScreen.main.bounds.width * {:.3}", n / 100.0)),
        ValueConfig::Vh(n) => Some(format!("UIScreen.main.bounds.height * {:.3}", n / 100.0)),
        ValueConfig::Percent(n) => Some(format!(
            "{n:.1} /* {n:.1}% — no direct SwiftUI equivalent for percentage spacing */"
        )),
        ValueConfig::Auto => None,
    }
}

fn swift_grid_item(track: &GridTrackSize) -> String {
    match track {
        GridTrackSize::Px(n) => format!("GridItem(.fixed({n:.1}))"),
        GridTrackSize::Fr(_) => "GridItem(.flexible())".into(),
        GridTrackSize::Auto => "GridItem(.flexible())".into(),
        GridTrackSize::Percent(_) => {
            "GridItem(.flexible()) /* percentage track — use .flexible() as approximation */".into()
        }
        GridTrackSize::MinContent => "GridItem(.flexible(minimum: 0)) /* min-content */".into(),
        GridTrackSize::MaxContent => "GridItem(.flexible()) /* max-content */".into(),
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

fn swift_align_self(a: AlignSelf, parent_is_row: bool) -> Option<&'static str> {
    match (a, parent_is_row) {
        (AlignSelf::Auto, _) => None,
        (AlignSelf::Center, _) => Some(".center"),
        (AlignSelf::FlexStart | AlignSelf::Start, true) => Some(".top"),
        (AlignSelf::FlexStart | AlignSelf::Start, false) => Some(".leading"),
        (AlignSelf::FlexEnd | AlignSelf::End, true) => Some(".bottom"),
        (AlignSelf::FlexEnd | AlignSelf::End, false) => Some(".trailing"),
        _ => None,
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
    emit_swiftui_node(&mut buf, root, 2, &mut 0, palette, true, false, true)?;
    buf.push_str("    }\n}\n");
    if needs_wrap(root) {
        buf.push('\n');
        buf.push_str(FLOW_LAYOUT_STRUCT);
    }
    Ok(buf)
}

#[allow(clippy::too_many_arguments)] // recursive tree-walker; leaf_idx varies per call site
fn emit_swiftui_node(
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

        writeln!(buf, "{pad}Text({:?})", node.label)?;
        writeln!(buf, "{pad}    .font(.system(size: 26))")?;
        writeln!(
            buf,
            "{pad}    .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))"
        )?;

        // Apply flex-basis percentage as width/height when no explicit size is set
        let basis_w = if parent_is_row && matches!(node.width, ValueConfig::Auto) {
            swift_flex_basis_value(&node.flex_basis, true)
        } else {
            None
        };
        let basis_h = if !parent_is_row && matches!(node.height, ValueConfig::Auto) {
            swift_flex_basis_value(&node.flex_basis, false)
        } else {
            None
        };

        let w = basis_w.or_else(|| swift_optional_value(&node.width));
        let h = basis_h.or_else(|| swift_optional_value(&node.height));
        if w.is_some() || h.is_some() {
            let w_str = w.as_deref().unwrap_or("nil");
            let h_str = h.as_deref().unwrap_or("nil");
            writeln!(buf, "{pad}    .frame(width: {w_str}, height: {h_str})")?;
        }
        let min_w = swift_optional_value(&node.min_width);
        let min_h = swift_optional_value(&node.min_height);
        let mut max_w = swift_optional_value(&node.max_width);
        let mut max_h = swift_optional_value(&node.max_height);
        // Flex-grow: merge into max constraints
        if node.flex_grow > 0.0 {
            if parent_is_row && max_w.is_none() {
                max_w = Some(".infinity".to_string());
            } else if !parent_is_row && max_h.is_none() {
                max_h = Some(".infinity".to_string());
            }
        }
        // align-items: Stretch from parent: merge into max constraints
        if parent_stretch {
            if parent_is_row && matches!(node.height, ValueConfig::Auto) && max_h.is_none() {
                max_h = Some(".infinity".to_string());
            } else if !parent_is_row && matches!(node.width, ValueConfig::Auto) && max_w.is_none() {
                max_w = Some(".infinity".to_string());
            }
        }
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
        let leaf_prefix = format!("{pad}    ");
        emit_swift_padding(buf, &leaf_prefix, &node.padding, "")?;
        writeln!(
            buf,
            "{pad}    .background(Color(red: {r:.2}, green: {g:.2}, blue: {b:.2}))"
        )?;
        emit_swift_border(buf, &leaf_prefix, &node.border_width, &node.border_radius)?;
        emit_swift_padding(buf, &leaf_prefix, &node.margin, " /* margin */")?;
        if let Some(alignment) = swift_align_self(node.align_self, parent_is_row) {
            if parent_is_row {
                writeln!(
                    buf,
                    "{pad}    .frame(maxHeight: .infinity, alignment: {alignment})"
                )?;
            } else {
                writeln!(
                    buf,
                    "{pad}    .frame(maxWidth: .infinity, alignment: {alignment})"
                )?;
            }
        } else if node.align_self != AlignSelf::Auto {
            writeln!(
                buf,
                "{pad}    /* align-self: {:?} — override manually with .alignmentGuide() */",
                node.align_self
            )?;
        }
        if !node.visible {
            writeln!(buf, "{pad}    .hidden()")?;
        }
        if node.order != 0 {
            writeln!(
                buf,
                "{pad}    // order: {} (no SwiftUI equivalent)",
                node.order
            )?;
        }
    } else {
        let is_grid = node.display_mode == DisplayMode::Grid;
        let is_row = matches!(
            node.flex_direction,
            FlexDirection::Row | FlexDirection::RowReverse
        );
        let is_reversed = matches!(
            node.flex_direction,
            FlexDirection::RowReverse | FlexDirection::ColumnReverse
        );
        let is_wrapping = node.flex_wrap != FlexWrap::NoWrap;

        // Sort children by order and pre-compute leaf_idx starts for palette colours.
        let mut children: Vec<&NodeConfig> = node.children.iter().collect();
        children.sort_by_key(|c| c.order);
        let mut starts = Vec::with_capacity(children.len());
        let mut acc = *leaf_idx;
        for child in &children {
            starts.push(acc);
            acc += count_leaves(child);
        }
        *leaf_idx = acc;

        // Only reverse children for HStack/VStack (non-wrapping).
        // FlowLayout handles reversal natively via mainReversed.
        if is_reversed && !is_wrapping && !is_grid {
            children.reverse();
            starts.reverse();
        }

        if is_grid {
            // --- LazyVGrid / LazyHGrid (CSS Grid) ---
            let is_column_flow = matches!(
                node.grid_auto_flow,
                GridAutoFlow::Column | GridAutoFlow::ColumnDense
            );
            let tracks = if is_column_flow {
                &node.grid_template_rows
            } else {
                &node.grid_template_columns
            };
            let items: Vec<String> = if tracks.is_empty() {
                vec!["GridItem(.flexible())".into()]
            } else {
                tracks.iter().map(swift_grid_item).collect()
            };
            writeln!(buf, "{pad}let columns = [{}]", items.join(", "))?;
            let spacing_arg = swift_spacing_value(if is_column_flow {
                &node.column_gap
            } else {
                &node.row_gap
            })
            .map(|s| format!(", spacing: {s}"))
            .unwrap_or_default();
            let grid_type = if is_column_flow {
                "LazyHGrid"
            } else {
                "LazyVGrid"
            };
            // LazyHGrid uses `rows:`, LazyVGrid uses `columns:`
            let param_name = if is_column_flow { "rows" } else { "columns" };
            writeln!(
                buf,
                "{pad}{grid_type}({param_name}: columns{spacing_arg}) {{"
            )?;

            for (child, start) in children.iter().zip(starts.iter()) {
                let mut idx = *start;
                emit_swiftui_node(
                    buf,
                    child,
                    depth + 1,
                    &mut idx,
                    palette,
                    true, // grid children flow like rows
                    node.align_items == AlignItems::Stretch,
                    false,
                )?;
            }
        } else if is_wrapping {
            // --- FlowLayout (custom wrapping layout) ---
            let axis = if is_row { ".horizontal" } else { ".vertical" };
            let item_gap = if is_row {
                &node.column_gap
            } else {
                &node.row_gap
            };
            let line_gap = if is_row {
                &node.row_gap
            } else {
                &node.column_gap
            };
            let line_align = swift_line_alignment(node.align_content);
            let wrap_reversed = node.flex_wrap == FlexWrap::WrapReverse;

            let mut args = vec![format!("axis: {axis}")];
            if let Some(s) = swift_spacing_value(item_gap) {
                args.push(format!("spacing: {s}"));
            }
            if let Some(s) = swift_spacing_value(line_gap) {
                args.push(format!("lineSpacing: {s}"));
            }
            if line_align != ".start" {
                args.push(format!("lineAlignment: {line_align}"));
            }
            if is_reversed {
                args.push("mainReversed: true".to_string());
            }
            if wrap_reversed {
                args.push("reversed: true".to_string());
            }
            writeln!(buf, "{pad}FlowLayout({}) {{", args.join(", "))?;

            for (child, start) in children.iter().zip(starts.iter()) {
                let mut idx = *start;
                emit_swiftui_node(
                    buf,
                    child,
                    depth + 1,
                    &mut idx,
                    palette,
                    is_row,
                    node.align_items == AlignItems::Stretch,
                    false,
                )?;
            }
        } else {
            // --- HStack / VStack (non-wrapping) ---
            let gap = if is_row {
                &node.column_gap
            } else {
                &node.row_gap
            };
            let jc = effective_justify(node.justify_content, is_reversed);
            let uses_zero_spacing = matches!(
                jc,
                JustifyContent::SpaceBetween
                    | JustifyContent::SpaceEvenly
                    | JustifyContent::SpaceAround
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

            if is_reversed {
                let dir_label = match node.flex_direction {
                    FlexDirection::RowReverse => "RowReverse",
                    FlexDirection::ColumnReverse => "ColumnReverse",
                    _ => unreachable!(),
                };
                writeln!(
                    buf,
                    "{pad}    // NOTE: flex-direction: {dir_label} — children reversed in source to approximate visual order"
                )?;
            }

            match jc {
                JustifyContent::SpaceBetween => {
                    for (i, (child, start)) in children.iter().zip(starts.iter()).enumerate() {
                        if i > 0 {
                            writeln!(buf, "{pad}    Spacer(minLength: 0)")?;
                        }
                        let mut idx = *start;
                        emit_swiftui_node(
                            buf,
                            child,
                            depth + 1,
                            &mut idx,
                            palette,
                            is_row,
                            node.align_items == AlignItems::Stretch,
                            false,
                        )?;
                    }
                }
                JustifyContent::Center => {
                    writeln!(buf, "{pad}    Spacer(minLength: 0)")?;
                    for (child, start) in children.iter().zip(starts.iter()) {
                        let mut idx = *start;
                        emit_swiftui_node(
                            buf,
                            child,
                            depth + 1,
                            &mut idx,
                            palette,
                            is_row,
                            node.align_items == AlignItems::Stretch,
                            false,
                        )?;
                    }
                    writeln!(buf, "{pad}    Spacer(minLength: 0)")?;
                }
                JustifyContent::SpaceEvenly | JustifyContent::SpaceAround => {
                    for (child, start) in children.iter().zip(starts.iter()) {
                        writeln!(buf, "{pad}    Spacer(minLength: 0)")?;
                        let mut idx = *start;
                        emit_swiftui_node(
                            buf,
                            child,
                            depth + 1,
                            &mut idx,
                            palette,
                            is_row,
                            node.align_items == AlignItems::Stretch,
                            false,
                        )?;
                    }
                    writeln!(buf, "{pad}    Spacer(minLength: 0)")?;
                }
                JustifyContent::FlexEnd | JustifyContent::End => {
                    writeln!(buf, "{pad}    Spacer(minLength: 0)")?;
                    for (child, start) in children.iter().zip(starts.iter()) {
                        let mut idx = *start;
                        emit_swiftui_node(
                            buf,
                            child,
                            depth + 1,
                            &mut idx,
                            palette,
                            is_row,
                            node.align_items == AlignItems::Stretch,
                            false,
                        )?;
                    }
                }
                _ => {
                    for (child, start) in children.iter().zip(starts.iter()) {
                        let mut idx = *start;
                        emit_swiftui_node(
                            buf,
                            child,
                            depth + 1,
                            &mut idx,
                            palette,
                            is_row,
                            node.align_items == AlignItems::Stretch,
                            false,
                        )?;
                    }
                }
            }
        }

        writeln!(buf, "{pad}}}")?;

        // Container frame: map Percent(100%) to maxWidth/maxHeight: .infinity
        let full_w = is_full_percent(&node.width);
        let full_h = is_full_percent(&node.height);
        let w = if full_w {
            None
        } else {
            swift_optional_value(&node.width)
        };
        // Root with flex_grow fills the viewport height (matching CSS body { height: 100% }),
        // and Percent(100%) maps to .infinity in the min/max frame below — skip both.
        let h = if full_h || (is_root && node.flex_grow > 0.0) {
            None
        } else {
            swift_optional_value(&node.height)
        };

        if w.is_some() || h.is_some() {
            let w_str = w.as_deref().unwrap_or("nil");
            let h_str = h.as_deref().unwrap_or("nil");
            writeln!(
                buf,
                "{pad}.frame(width: {w_str}, height: {h_str}, alignment: .topLeading)"
            )?;
        }

        // Collect all min/max constraints into a single .frame() call.
        // This merges explicit min/max, 100% → .infinity, flex-grow, and
        // parent stretch so later modifiers don't override earlier ones.
        let min_w = if is_zero_px(&node.min_width) {
            None
        } else {
            swift_optional_value(&node.min_width)
        };
        let min_h = if is_zero_px(&node.min_height) {
            None
        } else {
            swift_optional_value(&node.min_height)
        };

        let mut max_w = if full_w {
            Some(".infinity".to_string())
        } else {
            swift_optional_value(&node.max_width)
        };
        let mut max_h = if full_h || (is_root && node.flex_grow > 0.0) {
            Some(".infinity".to_string())
        } else {
            swift_optional_value(&node.max_height)
        };

        // Flex-grow expansion: merge into max constraints
        if node.flex_grow > 0.0 {
            if parent_is_row && !full_w && max_w.is_none() {
                max_w = Some(".infinity".to_string());
            } else if !parent_is_row && !full_h && max_h.is_none() {
                max_h = Some(".infinity".to_string());
            }
        }
        // align-items: Stretch from parent: merge into max constraints
        if parent_stretch {
            if parent_is_row
                && !full_h
                && matches!(node.height, ValueConfig::Auto)
                && max_h.is_none()
            {
                max_h = Some(".infinity".to_string());
            } else if !parent_is_row
                && !full_w
                && matches!(node.width, ValueConfig::Auto)
                && max_w.is_none()
            {
                max_w = Some(".infinity".to_string());
            }
        }

        if min_w.is_some() || min_h.is_some() || max_w.is_some() || max_h.is_some() {
            // Derive frame alignment from the container's cross-axis alignment
            let frame_align = if is_row {
                match node.align_items {
                    AlignItems::Center => ".leading",
                    AlignItems::FlexEnd | AlignItems::End => ".bottomLeading",
                    _ => ".topLeading",
                }
            } else {
                match node.align_items {
                    AlignItems::Center => ".top",
                    AlignItems::FlexEnd | AlignItems::End => ".topTrailing",
                    _ => ".topLeading",
                }
            };
            writeln!(
                buf,
                "{pad}.frame(minWidth: {}, maxWidth: {}, minHeight: {}, maxHeight: {}, alignment: {frame_align})",
                min_w.as_deref().unwrap_or("nil"),
                max_w.as_deref().unwrap_or("nil"),
                min_h.as_deref().unwrap_or("nil"),
                max_h.as_deref().unwrap_or("nil"),
            )?;
        }

        emit_swift_padding(buf, &pad, &node.padding, "")?;
        writeln!(
            buf,
            "{pad}.background(Color(red: 0.11, green: 0.11, blue: 0.17))"
        )?;
        emit_swift_border(buf, &pad, &node.border_width, &node.border_radius)?;
        emit_swift_padding(buf, &pad, &node.margin, " /* margin */")?;
        if !node.visible {
            writeln!(buf, "{pad}.hidden()")?;
        }
        if node.order != 0 {
            writeln!(buf, "{pad}// order: {} (no SwiftUI equivalent)", node.order)?;
        }
    }
    Ok(())
}

const FLOW_LAYOUT_STRUCT: &str = r#"struct FlowLayout: Layout {
    var axis: Axis = .horizontal
    var spacing: CGFloat = 0
    var lineSpacing: CGFloat = 0
    var lineAlignment: LineAlignment = .start
    var mainReversed: Bool = false
    var reversed: Bool = false

    enum LineAlignment: Sendable {
        case start, center, end, spaceBetween, spaceAround, spaceEvenly
    }

    private struct FlowLine {
        var range: Range<Int>
        var mainLength: CGFloat
        var crossLength: CGFloat
    }

    private func mainLength(_ s: CGSize) -> CGFloat {
        axis == .horizontal ? s.width : s.height
    }

    private func crossLength(_ s: CGSize) -> CGFloat {
        axis == .horizontal ? s.height : s.width
    }

    private func breakLines(sizes: [CGSize], maxMain: CGFloat) -> [FlowLine] {
        var lines: [FlowLine] = []
        var start = 0, main: CGFloat = 0, cross: CGFloat = 0
        for (i, size) in sizes.enumerated() {
            let m = mainLength(size)
            if main + m > maxMain && main > 0 {
                lines.append(FlowLine(range: start..<i, mainLength: main - spacing, crossLength: cross))
                start = i; main = 0; cross = 0
            }
            main += m + spacing
            cross = max(cross, crossLength(size))
        }
        if start < sizes.count {
            lines.append(FlowLine(range: start..<sizes.count, mainLength: main - spacing, crossLength: cross))
        }
        return lines
    }

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let sizes = subviews.map { $0.sizeThatFits(.unspecified) }
        let maxMain = axis == .horizontal ? (proposal.width ?? .infinity) : (proposal.height ?? .infinity)
        let lines = breakLines(sizes: sizes, maxMain: maxMain)
        let mainMax = lines.map(\.mainLength).max() ?? 0
        let crossTotal = lines.map(\.crossLength).reduce(0, +)
            + CGFloat(max(lines.count - 1, 0)) * lineSpacing
        return axis == .horizontal
            ? CGSize(width: mainMax, height: crossTotal)
            : CGSize(width: crossTotal, height: mainMax)
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        let sizes = subviews.map { $0.sizeThatFits(.unspecified) }
        let maxMain = axis == .horizontal ? bounds.width : bounds.height
        let maxCross = axis == .horizontal ? bounds.height : bounds.width
        var lines = breakLines(sizes: sizes, maxMain: maxMain)
        if reversed { lines.reverse() }

        let totalCross = lines.map(\.crossLength).reduce(0, +)
        let remaining = maxCross - totalCross
        let n = CGFloat(lines.count)
        var crossStart: CGFloat = 0
        var gap = lineSpacing

        switch lineAlignment {
        case .start: break
        case .center:
            crossStart = (remaining - CGFloat(max(lines.count - 1, 0)) * lineSpacing) / 2
        case .end:
            crossStart = remaining - CGFloat(max(lines.count - 1, 0)) * lineSpacing
        case .spaceBetween:
            gap = n > 1 ? remaining / (n - 1) : 0
        case .spaceAround:
            gap = n > 0 ? remaining / n : 0
            crossStart = gap / 2
        case .spaceEvenly:
            gap = n > 0 ? remaining / (n + 1) : 0
            crossStart = gap
        }

        var cross = crossStart
        for line in lines {
            var main: CGFloat = mainReversed ? maxMain : 0
            for idx in line.range {
                if mainReversed { main -= mainLength(sizes[idx]) }
                let pt = axis == .horizontal
                    ? CGPoint(x: bounds.minX + main, y: bounds.minY + cross)
                    : CGPoint(x: bounds.minX + cross, y: bounds.minY + main)
                subviews[idx].place(at: pt, proposal: .unspecified)
                if mainReversed {
                    main -= spacing
                } else {
                    main += mainLength(sizes[idx]) + spacing
                }
            }
            cross += line.crossLength + gap
        }
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn test_container() -> NodeConfig {
        let mut root = NodeConfig::new_container("root");
        root.flex_wrap = FlexWrap::NoWrap;
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
        assert!(
            code.contains("maxWidth: .infinity"),
            "Percent(100) should map to maxWidth: .infinity"
        );
        assert!(
            !code.contains("width: 100.0"),
            "should not emit width: 100.0 for Percent(100)"
        );
    }

    #[test]
    fn skips_zero_margin() {
        let code = emit_swiftui(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(
            !code.contains(".padding(0.0) /* margin */"),
            "should not emit zero margin"
        );
    }

    #[test]
    fn flex_grow_emits_infinity_frame() {
        let mut leaf = NodeConfig::new_leaf("A", 80.0, 80.0);
        leaf.flex_grow = 1.0;
        let mut root = NodeConfig::new_container("root");
        root.children = vec![leaf];
        let code = emit_swiftui(&root, ColorPalette::Pastel1).unwrap();
        assert!(
            code.contains("maxWidth: .infinity"),
            "flex-grow items should expand"
        );
    }

    #[test]
    fn space_between_emits_spacers() {
        let mut root = test_container();
        root.justify_content = JustifyContent::SpaceBetween;
        let code = emit_swiftui(&root, ColorPalette::Pastel1).unwrap();
        assert!(
            code.contains("Spacer(minLength: 0)"),
            "SpaceBetween should use Spacer()"
        );
    }

    #[test]
    fn wrapping_emits_flow_layout() {
        let mut root = test_container();
        root.flex_wrap = FlexWrap::Wrap;
        let code = emit_swiftui(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains("FlowLayout("), "Wrap should emit FlowLayout");
        assert!(
            code.contains("struct FlowLayout: Layout"),
            "Should include FlowLayout definition"
        );
        assert!(
            !code.contains("HStack"),
            "Should not emit HStack when wrapping"
        );
    }

    #[test]
    fn wrapping_with_space_between_sets_line_alignment() {
        let mut root = test_container();
        root.flex_wrap = FlexWrap::Wrap;
        root.align_content = AlignContent::SpaceBetween;
        let code = emit_swiftui(&root, ColorPalette::Pastel1).unwrap();
        assert!(
            code.contains("lineAlignment: .spaceBetween"),
            "SpaceBetween align_content should set lineAlignment"
        );
    }

    #[test]
    fn no_flow_layout_without_wrap() {
        let code = emit_swiftui(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(
            !code.contains("FlowLayout"),
            "Non-wrapping should not include FlowLayout"
        );
    }
}
