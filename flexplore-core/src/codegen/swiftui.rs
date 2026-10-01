use std::fmt::Write;

use crate::config::*;
use anyhow::Result;

use super::common::{
    is_full_percent, sorted_children_with_leaf_starts, swift_string_literal, take_leaf_color,
};
use crate::config::{ColorPalette, Corners, NodeConfig, Sides, ValueConfig};

/// Whether any flex container in the tree wraps, and therefore needs the
/// `FlowLayout` helper struct. Grid containers become `LazyVGrid`/`LazyHGrid`
/// and never use it, whatever their `flex_wrap` says.
fn needs_wrap(node: &NodeConfig) -> bool {
    if !node.children.is_empty()
        && node.display_mode != DisplayMode::Grid
        && node.flex_wrap != FlexWrap::NoWrap
    {
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

/// Sum of the `Px` components of two padding sides (other units add 0).
fn px_sum(a: &ValueConfig, b: &ValueConfig) -> f32 {
    let px = |v: &ValueConfig| match v {
        ValueConfig::Px(n) => *n,
        _ => 0.0,
    };
    px(a) + px(b)
}

/// Append `- pad` to a Swift size expression (no-op for zero padding).
fn swift_minus_pad(expr: String, pad: f32) -> String {
    if pad > 0.0 {
        format!("{expr} - {pad:.1}")
    } else {
        expr
    }
}

/// Move a fixed main-axis size into the `max` slot so an `HStack`/`VStack`
/// can shrink the view (see the flex-shrink note in `emit_swiftui_leaf`).
/// An explicit smaller `max-*` wins, as in CSS; percent sizes stay fixed
/// because they carry a comment rather than a plain expression.
fn shrink_main_axis(
    size: &mut Option<String>,
    max: &mut Option<String>,
    size_cfg: &ValueConfig,
    max_cfg: &ValueConfig,
) {
    let plain = matches!(
        size_cfg,
        ValueConfig::Px(_) | ValueConfig::Vw(_) | ValueConfig::Vh(_)
    );
    let Some(expr) = (plain).then(|| size.take()).flatten() else {
        return;
    };
    let max_smaller = matches!((max_cfg, size_cfg),
        (ValueConfig::Px(m), ValueConfig::Px(n)) if m < n);
    if !max_smaller {
        *max = Some(expr);
    }
}

/// Content-box size for a CSS border-box size. SwiftUI applies `.frame`
/// before `.padding`, so the padding must come off an explicit size for the
/// painted box to match the other backends (`box-sizing: border-box`).
/// Percent values carry an explanatory comment and are left untouched.
fn swift_inner_value(v: &ValueConfig, pad: f32) -> Option<String> {
    match v {
        ValueConfig::Auto => None,
        ValueConfig::Px(n) => Some(format!("{:.1}", (n - pad).max(0.0))),
        ValueConfig::Vw(_) | ValueConfig::Vh(_) => Some(swift_minus_pad(swift_value(v), pad)),
        ValueConfig::Percent(_) => Some(swift_value(v)),
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
    let viewport = Parent {
        is_row: true,
        stretch: false,
        wraps: false,
    };
    emit_swiftui_node(&mut buf, root, 2, &mut 0, palette, viewport, true)?;
    buf.push_str("    }\n}\n");
    if needs_wrap(root) {
        buf.push('\n');
        buf.push_str(FLOW_LAYOUT_STRUCT);
    }
    Ok(buf)
}

/// Layout context a node inherits from its parent.
#[derive(Clone, Copy)]
struct Parent {
    is_row: bool,
    stretch: bool,
    /// Parent is a `FlowLayout` or `Lazy*Grid` (children keep fixed frames)
    /// rather than an `HStack`/`VStack` (children may shrink like CSS flex items).
    wraps: bool,
}

fn emit_swiftui_node(
    buf: &mut String,
    node: &NodeConfig,
    depth: usize,
    leaf_idx: &mut usize,
    palette: ColorPalette,
    parent: Parent,
    is_root: bool,
) -> Result<()> {
    if node.children.is_empty() {
        emit_swiftui_leaf(buf, node, depth, leaf_idx, palette, parent)
    } else {
        emit_swiftui_container(buf, node, depth, leaf_idx, palette, parent, is_root)
    }
}

fn emit_swiftui_leaf(
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
        "{pad}Text({})",
        swift_string_literal(node.display_text())
    )?;
    writeln!(buf, "{pad}    .font(.system(size: 26))")?;
    writeln!(
        buf,
        "{pad}    .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))"
    )?;

    // All CSS sizes are border-box; `.padding()` is applied after `.frame()`,
    // so the padding comes off every explicit size.
    let pad_x = px_sum(&node.padding.left, &node.padding.right);
    let pad_y = px_sum(&node.padding.top, &node.padding.bottom);

    // Apply flex-basis percentage as width/height when no explicit size is set
    let basis_w = if parent.is_row && matches!(node.width, ValueConfig::Auto) {
        swift_flex_basis_value(&node.flex_basis, true).map(|v| swift_minus_pad(v, pad_x))
    } else {
        None
    };
    let basis_h = if !parent.is_row && matches!(node.height, ValueConfig::Auto) {
        swift_flex_basis_value(&node.flex_basis, false).map(|v| swift_minus_pad(v, pad_y))
    } else {
        None
    };

    let mut w = basis_w.or_else(|| swift_inner_value(&node.width, pad_x));
    let mut h = basis_h.or_else(|| swift_inner_value(&node.height, pad_y));
    let min_w = swift_inner_value(&node.min_width, pad_x);
    let min_h = swift_inner_value(&node.min_height, pad_y);
    let mut max_w = swift_inner_value(&node.max_width, pad_x);
    let mut max_h = swift_inner_value(&node.max_height, pad_y);

    // flex-shrink: inside an HStack/VStack a fixed `.frame(width:)` can never
    // shrink, so an overflowing row is clipped where CSS would squeeze the
    // items. Turn the main-axis size into an upper bound (`maxWidth`) instead;
    // the stack hands the view its full size while it fits and less when it
    // does not, which is what flex-shrink > 0 means. Grow items keep their
    // `.infinity` bound below. Not applied inside FlowLayout/grids, which
    // measure children with an unspecified proposal.
    if !parent.wraps && node.flex_shrink > 0.0 && node.flex_grow <= 0.0 {
        if parent.is_row {
            shrink_main_axis(&mut w, &mut max_w, &node.width, &node.max_width);
        } else {
            shrink_main_axis(&mut h, &mut max_h, &node.height, &node.max_height);
        }
    }

    if w.is_some() || h.is_some() {
        let w_str = w.as_deref().unwrap_or("nil");
        let h_str = h.as_deref().unwrap_or("nil");
        writeln!(buf, "{pad}    .frame(width: {w_str}, height: {h_str})")?;
    }
    // Flex-grow: merge into max constraints
    if node.flex_grow > 0.0 {
        if parent.is_row && max_w.is_none() {
            max_w = Some(".infinity".to_string());
        } else if !parent.is_row && max_h.is_none() {
            max_h = Some(".infinity".to_string());
        }
    }
    // align-items: Stretch from parent: merge into max constraints
    if parent.stretch {
        if parent.is_row && matches!(node.height, ValueConfig::Auto) && max_h.is_none() {
            max_h = Some(".infinity".to_string());
        } else if !parent.is_row && matches!(node.width, ValueConfig::Auto) && max_w.is_none() {
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
    if let Some(alignment) = swift_align_self(node.align_self, parent.is_row) {
        if parent.is_row {
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
    Ok(())
}

fn emit_swiftui_container(
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
    let is_reversed = matches!(
        node.flex_direction,
        FlexDirection::RowReverse | FlexDirection::ColumnReverse
    );
    let is_wrapping = node.flex_wrap != FlexWrap::NoWrap;
    let child_stretch = node.align_items == AlignItems::Stretch;

    // Sort children by order and pre-compute leaf_idx starts for palette colours.
    let (mut children, mut starts) = sorted_children_with_leaf_starts(node, leaf_idx);

    // Only reverse children for HStack/VStack (non-wrapping).
    // FlowLayout handles reversal natively via mainReversed.
    if is_reversed && !is_wrapping && !is_grid {
        children.reverse();
        starts.reverse();
    }

    // Emit one child, colouring from its pre-computed leaf start.
    let emit_child =
        |buf: &mut String, child: &NodeConfig, start: usize, child_is_row: bool, wraps: bool| {
            let mut idx = start;
            let ctx = Parent {
                is_row: child_is_row,
                stretch: child_stretch,
                wraps,
            };
            emit_swiftui_node(buf, child, depth + 1, &mut idx, palette, ctx, false)
        };

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
        let spacing_arg = swift_spacing_value(if is_column_flow {
            &node.column_gap
        } else {
            &node.row_gap
        })
        .map(|s| format!(", spacing: {s}"))
        .unwrap_or_default();
        // LazyHGrid uses `rows:`, LazyVGrid uses `columns:`. The track array
        // is inlined so sibling grids in one ViewBuilder never redeclare a
        // shared `let`.
        let (grid_type, param_name) = if is_column_flow {
            ("LazyHGrid", "rows")
        } else {
            ("LazyVGrid", "columns")
        };
        writeln!(
            buf,
            "{pad}// NOTE: CSS Grid approximated with {grid_type} — tracks map to GridItems, items flow in order; grid-column/grid-row spans and explicit placement are not supported"
        )?;
        writeln!(
            buf,
            "{pad}{grid_type}({param_name}: [{}]{spacing_arg}) {{",
            items.join(", ")
        )?;

        for (child, start) in children.iter().zip(starts.iter()) {
            if child.grid_column != GridPlacement::Auto {
                writeln!(
                    buf,
                    "{pad}    // grid-column: {} — not expressible in {grid_type}; item takes one cell",
                    child.grid_column.display_short()
                )?;
            }
            if child.grid_row != GridPlacement::Auto {
                writeln!(
                    buf,
                    "{pad}    // grid-row: {} — not expressible in {grid_type}; item takes one cell",
                    child.grid_row.display_short()
                )?;
            }
            // grid children flow like rows
            emit_child(buf, child, *start, true, true)?;
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
            emit_child(buf, child, *start, is_row, true)?;
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

        // Spacers before/between/after children approximate justify-content.
        let (spacer_before, spacer_between, spacer_after) = match jc {
            JustifyContent::SpaceBetween => (false, true, false),
            JustifyContent::Center => (true, false, true),
            JustifyContent::SpaceEvenly | JustifyContent::SpaceAround => (true, true, true),
            JustifyContent::FlexEnd | JustifyContent::End => (true, false, false),
            _ => (false, false, false),
        };
        for (i, (child, start)) in children.iter().zip(starts.iter()).enumerate() {
            if (i == 0 && spacer_before) || (i > 0 && spacer_between) {
                writeln!(buf, "{pad}    Spacer(minLength: 0)")?;
            }
            emit_child(buf, child, *start, is_row, false)?;
        }
        if spacer_after {
            writeln!(buf, "{pad}    Spacer(minLength: 0)")?;
        }
    }

    writeln!(buf, "{pad}}}")?;

    // Container frame: map Percent(100%) to maxWidth/maxHeight: .infinity.
    // Sizes are border-box, so the container's own padding comes off them.
    let pad_x = px_sum(&node.padding.left, &node.padding.right);
    let pad_y = px_sum(&node.padding.top, &node.padding.bottom);
    let full_w = is_full_percent(&node.width);
    let full_h = is_full_percent(&node.height);
    let w = if full_w {
        None
    } else {
        swift_inner_value(&node.width, pad_x)
    };
    // Root with flex_grow fills the viewport height (matching CSS body { height: 100% }),
    // and Percent(100%) maps to .infinity in the min/max frame below — skip both.
    let root_fills_height = is_root && node.flex_grow > 0.0;
    let h = if full_h || root_fills_height {
        None
    } else {
        swift_inner_value(&node.height, pad_y)
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
    let min_w = if node.min_width.is_zero_px() {
        None
    } else {
        swift_inner_value(&node.min_width, pad_x)
    };
    let min_h = if node.min_height.is_zero_px() {
        None
    } else {
        swift_inner_value(&node.min_height, pad_y)
    };

    let mut max_w = if full_w {
        Some(".infinity".to_string())
    } else {
        swift_inner_value(&node.max_width, pad_x)
    };
    let mut max_h = if full_h || root_fills_height {
        Some(".infinity".to_string())
    } else {
        swift_inner_value(&node.max_height, pad_y)
    };

    // Flex-grow expansion: merge into max constraints
    if node.flex_grow > 0.0 {
        if parent.is_row && !full_w && max_w.is_none() {
            max_w = Some(".infinity".to_string());
        } else if !parent.is_row && !full_h && max_h.is_none() {
            max_h = Some(".infinity".to_string());
        }
    }
    // align-items: Stretch from parent: merge into max constraints
    if parent.stretch {
        if parent.is_row && !full_h && matches!(node.height, ValueConfig::Auto) && max_h.is_none() {
            max_h = Some(".infinity".to_string());
        } else if !parent.is_row
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
        // flex-wrap: wrap-reverse flips the cross axis: the first line sits at
        // the cross end (bottom for a horizontal flow) and later lines stack
        // towards the cross start, so the line order *and* the start/end
        // anchoring of lineAlignment are mirrored.
        if reversed { lines.reverse() }
        let effectiveAlignment: LineAlignment
        switch (reversed, lineAlignment) {
        case (true, .start): effectiveAlignment = .end
        case (true, .end): effectiveAlignment = .start
        default: effectiveAlignment = lineAlignment
        }

        let totalCross = lines.map(\.crossLength).reduce(0, +)
        let remaining = maxCross - totalCross
        let n = CGFloat(lines.count)
        var crossStart: CGFloat = 0
        var gap = lineSpacing

        switch effectiveAlignment {
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
    fn escapes_leaf_text() {
        let mut root = test_container();
        root.children = vec![NodeConfig::new_leaf("say \"\\(hi)\"", 80.0, 80.0)];
        let code = emit_swiftui(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains(r#"Text("say \"\\(hi)\"")"#), "{code}");
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

    #[test]
    fn explicit_sizes_are_border_box() {
        // 100x60 leaf with 8px padding → inner frame 84x44 so the painted
        // box (frame + padding) is 100x60 like every other backend.
        let mut root = test_container();
        let mut leaf = NodeConfig::new_leaf("A", 100.0, 60.0);
        leaf.flex_shrink = 0.0; // keep a fixed frame
        root.children = vec![leaf];
        let code = emit_swiftui(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains(".frame(width: 84.0, height: 44.0)"), "{code}");
        assert!(code.contains("            .padding(8.0)\n"), "{code}");
        // Container: 200px wide with 12px padding → 176 inner.
        let mut inner = NodeConfig::new_container("inner");
        inner.flex_wrap = FlexWrap::NoWrap;
        inner.width = ValueConfig::Px(200.0);
        inner.children = vec![NodeConfig::new_leaf("X", 40.0, 40.0)];
        root.children = vec![inner];
        let code = emit_swiftui(&root, ColorPalette::Pastel1).unwrap();
        assert!(
            code.contains(".frame(width: 176.0, height: nil, alignment: .topLeading)"),
            "{code}"
        );
    }

    #[test]
    fn vw_sizes_subtract_padding_as_expression() {
        let mut root = test_container();
        let mut leaf = NodeConfig::new_leaf("A", 100.0, 60.0);
        leaf.flex_shrink = 0.0;
        leaf.width = ValueConfig::Vw(50.0);
        root.children = vec![leaf];
        let code = emit_swiftui(&root, ColorPalette::Pastel1).unwrap();
        assert!(
            code.contains(".frame(width: UIScreen.main.bounds.width * 0.500 - 16.0, height: 44.0)"),
            "{code}"
        );
    }

    #[test]
    fn shrinkable_leaf_in_stack_uses_max_width() {
        // Default leaf has flex-shrink 1: in an HStack its width becomes an
        // upper bound so the stack can squeeze it instead of clipping.
        let code = emit_swiftui(&test_container(), ColorPalette::Pastel1).unwrap();
        assert!(code.contains(".frame(width: nil, height: 64.0)"), "{code}");
        assert!(
            code.contains(".frame(minWidth: nil, maxWidth: 64.0, minHeight: nil, maxHeight: nil)"),
            "{code}"
        );
        // Inside a FlowLayout the frame stays fixed.
        let mut root = test_container();
        root.flex_wrap = FlexWrap::Wrap;
        let code = emit_swiftui(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains(".frame(width: 64.0, height: 64.0)"), "{code}");
        assert!(!code.contains("maxWidth: 64.0"), "{code}");
    }

    #[test]
    fn grow_leaf_keeps_infinity_bound() {
        let mut leaf = NodeConfig::new_leaf("A", 80.0, 80.0);
        leaf.flex_grow = 1.0;
        let mut root = test_container();
        root.children = vec![leaf];
        let code = emit_swiftui(&root, ColorPalette::Pastel1).unwrap();
        assert!(code.contains(".frame(width: 64.0, height: 64.0)"), "{code}");
        assert!(code.contains("maxWidth: .infinity"), "{code}");
    }

    #[test]
    fn flow_layout_mirrors_line_anchoring_when_reversed() {
        assert!(FLOW_LAYOUT_STRUCT.contains("case (true, .start): effectiveAlignment = .end"));
        assert!(FLOW_LAYOUT_STRUCT.contains("switch effectiveAlignment {"));
    }

    #[test]
    fn grid_spans_get_a_note() {
        let mut root = NodeConfig::new_grid("g", vec![GridTrackSize::Fr(1.0); 3]);
        let mut wide = NodeConfig::new_leaf("wide", 80.0, 60.0);
        wide.grid_column = GridPlacement::Span(2);
        root.children = vec![wide, NodeConfig::new_leaf("cell", 80.0, 60.0)];
        let code = emit_swiftui(&root, ColorPalette::Pastel1).unwrap();
        assert!(
            code.contains("// NOTE: CSS Grid approximated with LazyVGrid"),
            "{code}"
        );
        assert!(
            code.contains("// grid-column: span 2 — not expressible in LazyVGrid"),
            "{code}"
        );
    }

    #[test]
    fn grid_does_not_pull_in_flow_layout() {
        // new_grid inherits flex_wrap: Wrap from new_container, but grids
        // never use FlowLayout.
        let mut root = NodeConfig::new_grid("g", vec![GridTrackSize::Fr(1.0)]);
        root.children = vec![NodeConfig::new_leaf("A", 80.0, 80.0)];
        let code = emit_swiftui(&root, ColorPalette::Pastel1).unwrap();
        assert!(!code.contains("FlowLayout"), "{code}");
    }

    #[test]
    fn sibling_grids_do_not_redeclare_columns() {
        let mut root = NodeConfig::new_container("root");
        root.flex_wrap = FlexWrap::NoWrap;
        let mut g1 = NodeConfig::new_grid("g1", vec![GridTrackSize::Fr(1.0)]);
        g1.children = vec![NodeConfig::new_leaf("A", 80.0, 80.0)];
        let mut g2 = NodeConfig::new_grid("g2", vec![GridTrackSize::Px(50.0)]);
        g2.grid_auto_flow = GridAutoFlow::Column;
        g2.grid_template_rows = vec![GridTrackSize::Fr(1.0)];
        g2.children = vec![NodeConfig::new_leaf("B", 80.0, 80.0)];
        root.children = vec![g1, g2];
        let code = emit_swiftui(&root, ColorPalette::Pastel1).unwrap();
        assert!(!code.contains("let columns"), "{code}");
        assert!(
            code.contains("LazyVGrid(columns: [GridItem(.flexible())]"),
            "{code}"
        );
        assert!(
            code.contains("LazyHGrid(rows: [GridItem(.flexible())]"),
            "{code}"
        );
    }
}
