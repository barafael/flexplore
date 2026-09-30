//! Static HTML+CSS for the web-flavoured backends (Dioxus, React Native).
//!
//! Both backends ultimately lay out `<div>`s with a browser engine, so the
//! goldens are produced by generating one document with inline styles and
//! screenshotting it. The property mapping mirrors
//! `flexplore_core::codegen::css` (including CSS grid), and an
//! [`HtmlFlavor`] captures where the two backends differ.

use std::fmt::Write;

use flexplore_core::config::{
    AlignContent, AlignItems, AlignSelf, ColorPalette, Corners, DisplayMode, FlexDirection,
    FlexWrap, GridAutoFlow, GridPlacement, GridTrackSize, JustifyContent, NodeConfig, Sides,
    ValueConfig,
};

use crate::palette_rgb8;

/// How a `display: grid` container is laid out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridMode {
    /// Real CSS grid, mirroring `codegen/css.rs`.
    CssGrid,
    /// What `codegen/react_native.rs` emits for grid nodes: a
    /// `flexDirection: 'row'` + `flexWrap: 'wrap'` container, with
    /// `grid-column`/`grid-row` placement only mentioned in comments.
    FlexWrap,
}

/// Backend-specific rendering rules.
#[derive(Clone, Copy, Debug)]
pub struct HtmlFlavor {
    /// Extra CSS for `body` (the viewport container).
    pub body_css: &'static str,
    /// Declaration used for `visible: false`. Space is always kept.
    pub hidden_css: &'static str,
    /// Reproduce Yoga (React Native) defaults instead of CSS ones:
    /// `flex-direction: column`, `flex-shrink: 0`, `align-content: flex-start`
    /// — the relevant properties are always emitted explicitly.
    pub yoga_defaults: bool,
    /// Emit the CSS `order` property. Children are sorted by `order` in
    /// source regardless, so this only matters for engines that also honour
    /// the property.
    pub emit_order: bool,
    /// Force leaf text to be centred (as the Dioxus codegen does), instead of
    /// relying on the leaf's own justify/align settings.
    pub center_leaf_text: bool,
    /// Grid container strategy.
    pub grid: GridMode,
}

/// Dioxus renders through a webview with plain CSS semantics.
pub const DIOXUS: HtmlFlavor = HtmlFlavor {
    body_css: "display:flex;flex-direction:column;align-items:flex-start;",
    hidden_css: "visibility: hidden",
    yoga_defaults: false,
    emit_order: true,
    center_leaf_text: true,
    grid: GridMode::CssGrid,
};

/// React Native (via react-native-web) uses Yoga defaults, has no `order`,
/// hides with `opacity: 0` and approximates grids with a wrapping row
/// (all matching the RN codegen).
pub const REACT_NATIVE: HtmlFlavor = HtmlFlavor {
    body_css: "",
    hidden_css: "opacity: 0",
    yoga_defaults: true,
    emit_order: false,
    center_leaf_text: false,
    grid: GridMode::FlexWrap,
};

const BACKGROUND: &str = "rgba(28, 28, 43, 1)";

/// A complete HTML document rendering `root`.
pub fn generate_html(root: &NodeConfig, palette: ColorPalette, flavor: &HtmlFlavor) -> String {
    let mut buf = format!(
        "<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"UTF-8\">\n\
         <style>html,body{{margin:0;padding:0;height:100%;width:100%}}\
         body{{{}background:{BACKGROUND}}}</style>\n</head>\n<body>\n",
        flavor.body_css
    );
    emit_node(&mut buf, root, 0, &mut 0, palette, flavor);
    buf.push_str("</body>\n</html>\n");
    buf
}

fn emit_node(
    buf: &mut String,
    node: &NodeConfig,
    depth: usize,
    leaf_idx: &mut usize,
    palette: ColorPalette,
    flavor: &HtmlFlavor,
) {
    let pad = "  ".repeat(depth);
    let is_leaf = node.children.is_empty();
    let grid_node = !is_leaf && node.display_mode == DisplayMode::Grid;
    let is_grid = grid_node && flavor.grid == GridMode::CssGrid;
    let grid_as_flex_wrap = grid_node && flavor.grid == GridMode::FlexWrap;

    let bg = if is_leaf {
        let (r, g, b) = palette_rgb8(palette, *leaf_idx);
        *leaf_idx += 1;
        format!("rgb({r}, {g}, {b})")
    } else {
        BACKGROUND.to_string()
    };

    let mut s: Vec<String> = Vec::new();
    s.push(format!(
        "display: {}",
        if is_grid { "grid" } else { "flex" }
    ));
    if !node.visible {
        s.push(flavor.hidden_css.into());
    }

    if is_grid {
        push_tracks(&mut s, "grid-template-columns", &node.grid_template_columns);
        push_tracks(&mut s, "grid-template-rows", &node.grid_template_rows);
        push_tracks(&mut s, "grid-auto-columns", &node.grid_auto_columns);
        push_tracks(&mut s, "grid-auto-rows", &node.grid_auto_rows);
        if node.grid_auto_flow != GridAutoFlow::Row {
            s.push(format!(
                "grid-auto-flow: {}",
                node.grid_auto_flow.to_css_str()
            ));
        }
    } else if grid_as_flex_wrap {
        s.push("flex-direction: row".into());
        s.push("flex-wrap: wrap".into());
    } else {
        // Yoga's default direction is column, so RN always spells it out.
        if flavor.yoga_defaults || node.flex_direction != FlexDirection::Row {
            s.push(format!(
                "flex-direction: {}",
                css_direction(node.flex_direction)
            ));
        }
        if node.flex_wrap != FlexWrap::NoWrap {
            s.push(format!("flex-wrap: {}", css_wrap(node.flex_wrap)));
        }
    }

    if !matches!(
        node.justify_content,
        JustifyContent::Default | JustifyContent::FlexStart | JustifyContent::Start
    ) {
        s.push(format!(
            "justify-content: {}",
            css_justify(node.justify_content)
        ));
    }
    if !matches!(node.align_items, AlignItems::Default | AlignItems::Stretch) {
        s.push(format!(
            "align-items: {}",
            css_align_items(node.align_items)
        ));
    }
    if flavor.yoga_defaults {
        // Yoga defaults align-content to flex-start where CSS uses stretch.
        s.push(format!(
            "align-content: {}",
            css_align_content(node.align_content, "flex-start")
        ));
    } else if !matches!(
        node.align_content,
        AlignContent::Default | AlignContent::Stretch
    ) {
        s.push(format!(
            "align-content: {}",
            css_align_content(node.align_content, "stretch")
        ));
    }
    if !is_zero_or_auto(&node.row_gap) {
        s.push(format!("row-gap: {}", css_value(&node.row_gap)));
    }
    if !is_zero_or_auto(&node.column_gap) {
        s.push(format!("column-gap: {}", css_value(&node.column_gap)));
    }

    // Flex item
    if node.flex_grow != 0.0 {
        s.push(format!("flex-grow: {}", format_num(node.flex_grow)));
    }
    if flavor.yoga_defaults || node.flex_shrink != 1.0 {
        s.push(format!("flex-shrink: {}", format_num(node.flex_shrink)));
    }
    if !matches!(node.flex_basis, ValueConfig::Auto) {
        s.push(format!("flex-basis: {}", css_value(&node.flex_basis)));
    }
    if node.align_self != AlignSelf::Auto {
        s.push(format!("align-self: {}", css_align_self(node.align_self)));
    }
    // Grid item (placement is comment-only in the RN codegen, so skipped there)
    if flavor.grid == GridMode::CssGrid {
        if node.grid_column != GridPlacement::Auto {
            s.push(format!("grid-column: {}", node.grid_column.display_short()));
        }
        if node.grid_row != GridPlacement::Auto {
            s.push(format!("grid-row: {}", node.grid_row.display_short()));
        }
    }

    // Sizing
    for (prop, v) in [
        ("width", &node.width),
        ("height", &node.height),
        ("min-width", &node.min_width),
        ("min-height", &node.min_height),
        ("max-width", &node.max_width),
        ("max-height", &node.max_height),
    ] {
        if !matches!(v, ValueConfig::Auto) {
            s.push(format!("{prop}: {}", css_value(v)));
        }
    }

    // Spacing / border. `auto` is meaningless for padding and border widths,
    // so it is emitted as 0 there; `margin: auto` is valid CSS and kept.
    push_sides(&mut s, "padding", &node.padding, false);
    push_sides(&mut s, "margin", &node.margin, true);
    push_sides(&mut s, "border-width", &node.border_width, false);
    push_corners(&mut s, &node.border_radius);
    if !node.border_width.is_zero() {
        s.push("border-style: solid".into());
        s.push("border-color: rgba(13, 13, 26, 0.85)".into());
    }
    if flavor.emit_order && node.order != 0 {
        s.push(format!("order: {}", node.order));
    }

    s.push(format!("background: {bg}"));
    s.push("box-sizing: border-box".into());
    if is_leaf {
        s.push("color: rgba(13, 13, 26, 0.85)".into());
        s.push("font-size: 26px".into());
        if flavor.center_leaf_text {
            s.push("justify-content: center".into());
            s.push("align-items: center".into());
        }
    }

    let style = s.join("; ");
    if is_leaf {
        let _ = writeln!(
            buf,
            "{pad}<div style=\"{style}\">{}</div>",
            escape_html(node.display_text())
        );
    } else {
        let _ = writeln!(buf, "{pad}<div style=\"{style}\">");
        let mut sorted: Vec<&NodeConfig> = node.children.iter().collect();
        sorted.sort_by_key(|c| c.order);
        for child in sorted {
            emit_node(buf, child, depth + 1, leaf_idx, palette, flavor);
        }
        let _ = writeln!(buf, "{pad}</div>");
    }
}

// ─── CSS value helpers ───────────────────────────────────────────────────────

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn is_zero_or_auto(v: &ValueConfig) -> bool {
    matches!(v, ValueConfig::Auto) || v.is_zero_px()
}

fn format_num(v: f32) -> String {
    flexplore_core::config::format_float(v)
}

fn css_value(v: &ValueConfig) -> String {
    match v {
        ValueConfig::Auto => "auto".into(),
        ValueConfig::Px(n) => format!("{n:.1}px"),
        ValueConfig::Percent(n) => format!("{n:.1}%"),
        ValueConfig::Vw(n) => format!("{n:.1}vw"),
        ValueConfig::Vh(n) => format!("{n:.1}vh"),
    }
}

/// Like [`css_value`] but `auto` becomes `0px` (for padding / border-width).
fn css_length(v: &ValueConfig) -> String {
    match v {
        ValueConfig::Auto => "0px".into(),
        other => css_value(other),
    }
}

fn push_sides(s: &mut Vec<String>, prop: &str, sides: &Sides, auto_allowed: bool) {
    if sides.is_zero() {
        return;
    }
    let conv = |v: &ValueConfig| {
        if auto_allowed {
            css_value(v)
        } else {
            css_length(v)
        }
    };
    if sides.is_uniform() {
        s.push(format!("{prop}: {}", conv(&sides.first())));
    } else {
        s.push(format!(
            "{prop}: {} {} {} {}",
            conv(&sides.top),
            conv(&sides.right),
            conv(&sides.bottom),
            conv(&sides.left)
        ));
    }
}

fn push_corners(s: &mut Vec<String>, c: &Corners) {
    if c.is_zero() {
        return;
    }
    if c.is_uniform() {
        s.push(format!("border-radius: {:.1}px", c.top_left));
    } else {
        s.push(format!(
            "border-radius: {:.1}px {:.1}px {:.1}px {:.1}px",
            c.top_left, c.top_right, c.bottom_right, c.bottom_left
        ));
    }
}

fn push_tracks(s: &mut Vec<String>, prop: &str, tracks: &[GridTrackSize]) {
    if tracks.is_empty() {
        return;
    }
    let list = tracks
        .iter()
        .map(|t| t.display_short())
        .collect::<Vec<_>>()
        .join(" ");
    s.push(format!("{prop}: {list}"));
}

fn css_direction(d: FlexDirection) -> &'static str {
    match d {
        FlexDirection::Row => "row",
        FlexDirection::Column => "column",
        FlexDirection::RowReverse => "row-reverse",
        FlexDirection::ColumnReverse => "column-reverse",
    }
}

fn css_wrap(w: FlexWrap) -> &'static str {
    match w {
        FlexWrap::NoWrap => "nowrap",
        FlexWrap::Wrap => "wrap",
        FlexWrap::WrapReverse => "wrap-reverse",
    }
}

fn css_justify(j: JustifyContent) -> &'static str {
    match j {
        JustifyContent::Default | JustifyContent::FlexStart => "flex-start",
        JustifyContent::FlexEnd => "flex-end",
        JustifyContent::Center => "center",
        JustifyContent::SpaceBetween => "space-between",
        JustifyContent::SpaceAround => "space-around",
        JustifyContent::SpaceEvenly => "space-evenly",
        JustifyContent::Stretch => "stretch",
        JustifyContent::Start => "start",
        JustifyContent::End => "end",
    }
}

fn css_align_items(a: AlignItems) -> &'static str {
    match a {
        AlignItems::FlexStart => "flex-start",
        AlignItems::FlexEnd => "flex-end",
        AlignItems::Center => "center",
        AlignItems::Baseline => "baseline",
        AlignItems::Default | AlignItems::Stretch => "stretch",
        AlignItems::Start => "start",
        AlignItems::End => "end",
    }
}

fn css_align_content(a: AlignContent, default: &'static str) -> &'static str {
    match a {
        AlignContent::Default => default,
        AlignContent::FlexStart => "flex-start",
        AlignContent::FlexEnd => "flex-end",
        AlignContent::Center => "center",
        AlignContent::SpaceBetween => "space-between",
        AlignContent::SpaceAround => "space-around",
        AlignContent::SpaceEvenly => "space-evenly",
        AlignContent::Stretch => "stretch",
        AlignContent::Start => "start",
        AlignContent::End => "end",
    }
}

fn css_align_self(a: AlignSelf) -> &'static str {
    match a {
        AlignSelf::Auto => "auto",
        AlignSelf::FlexStart => "flex-start",
        AlignSelf::FlexEnd => "flex-end",
        AlignSelf::Center => "center",
        AlignSelf::Baseline => "baseline",
        AlignSelf::Stretch => "stretch",
        AlignSelf::Start => "start",
        AlignSelf::End => "end",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid_root() -> NodeConfig {
        let mut root = NodeConfig::new_grid(
            "grid",
            vec![
                GridTrackSize::Fr(1.0),
                GridTrackSize::Fr(1.0),
                GridTrackSize::Fr(1.0),
            ],
        );
        let mut wide = NodeConfig::new_leaf("wide", 80.0, 80.0);
        wide.grid_column = GridPlacement::Span(2);
        root.children = vec![wide, NodeConfig::new_leaf("B", 80.0, 80.0)];
        root
    }

    #[test]
    fn dioxus_grid_container_and_placement() {
        let html = generate_html(&grid_root(), ColorPalette::Pastel1, &DIOXUS);
        assert!(html.contains("display: grid"), "{html}");
        assert!(html.contains("grid-template-columns: 1.0fr 1.0fr 1.0fr"));
        assert!(html.contains("grid-column: span 2"));
        let root_style = html.lines().nth(7).unwrap();
        assert!(!root_style.contains("flex-direction"), "{root_style}");
    }

    #[test]
    fn react_native_grid_is_a_wrapping_row() {
        let html = generate_html(&grid_root(), ColorPalette::Pastel1, &REACT_NATIVE);
        assert!(!html.contains("grid"), "{html}");
        let root_style = html.lines().nth(7).unwrap();
        assert!(
            root_style.contains("display: flex; flex-direction: row; flex-wrap: wrap"),
            "{root_style}"
        );
    }

    #[test]
    fn hidden_keeps_space_per_flavor() {
        let mut root = NodeConfig::new_container("root");
        let mut leaf = NodeConfig::new_leaf("h", 80.0, 80.0);
        leaf.visible = false;
        root.children = vec![leaf];
        let d = generate_html(&root, ColorPalette::Pastel1, &DIOXUS);
        assert!(d.contains("visibility: hidden") && !d.contains("display: none"));
        let rn = generate_html(&root, ColorPalette::Pastel1, &REACT_NATIVE);
        assert!(rn.contains("opacity: 0") && !rn.contains("display: none"));
    }

    #[test]
    fn per_side_spacing_border_and_radius() {
        let mut root = NodeConfig::new_container("root");
        let mut leaf = NodeConfig::new_leaf("h", 80.0, 80.0);
        leaf.padding = Sides {
            top: ValueConfig::Px(1.0),
            right: ValueConfig::Auto,
            bottom: ValueConfig::Px(3.0),
            left: ValueConfig::Px(4.0),
        };
        leaf.border_width = Sides::uniform(ValueConfig::Px(2.0));
        leaf.border_radius = Corners::uniform(6.0);
        root.children = vec![leaf];
        let html = generate_html(&root, ColorPalette::Pastel1, &DIOXUS);
        assert!(html.contains("padding: 1.0px 0px 3.0px 4.0px"), "{html}");
        assert!(!html.contains("padding: auto"));
        assert!(html.contains("border-width: 2.0px; border-radius: 6.0px"));
        assert!(html.contains("border-style: solid"));
    }

    #[test]
    fn uses_display_text() {
        let mut root = NodeConfig::new_container("root");
        let mut leaf = NodeConfig::new_leaf("label", 80.0, 80.0);
        leaf.text_content = "shown <text>".into();
        root.children = vec![leaf];
        let html = generate_html(&root, ColorPalette::Pastel1, &DIOXUS);
        assert!(html.contains(">shown &lt;text&gt;</div>"));
    }
}
