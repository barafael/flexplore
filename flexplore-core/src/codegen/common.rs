//! Helpers shared by the code generators: colour conversion, child ordering,
//! CSS keyword tables, and per-target string escaping.

use crate::art::palette_color;
use crate::config::{
    AlignContent, AlignItems, AlignSelf, ColorPalette, FlexDirection, FlexWrap, GridTrackSize,
    JustifyContent, NodeConfig, Sides, ValueConfig,
};

// ─── Colours ─────────────────────────────────────────────────────────────────

/// Take the next palette colour for a leaf and advance the leaf counter.
pub(crate) fn take_leaf_color(palette: ColorPalette, leaf_idx: &mut usize) -> (f32, f32, f32) {
    let rgb = palette_color(palette, *leaf_idx);
    *leaf_idx += 1;
    rgb
}

/// Convert normalised (0.0..=1.0) channels to 8-bit, rounding to nearest so
/// palette bytes round-trip exactly.
pub(crate) fn rgb8(r: f32, g: f32, b: f32) -> (u8, u8, u8) {
    let ch = |v: f32| (v * 255.0).round().clamp(0.0, 255.0) as u8;
    (ch(r), ch(g), ch(b))
}

// ─── Values ──────────────────────────────────────────────────────────────────

/// `Auto` or `Px(0)` — the cases where a gap/spacing property is omitted.
pub(crate) fn is_auto_or_zero(v: &ValueConfig) -> bool {
    matches!(v, ValueConfig::Auto) || v.is_zero_px()
}

/// `Percent(n)` with `n >= 100` — targets map this to "fill".
pub(crate) fn is_full_percent(v: &ValueConfig) -> bool {
    matches!(v, ValueConfig::Percent(n) if *n >= 100.0)
}

/// Space-separated grid track list, e.g. `1.0fr 200px auto`.
pub(crate) fn grid_tracks(tracks: &[GridTrackSize]) -> String {
    tracks
        .iter()
        .map(|t| t.display_short())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Short human-readable form of per-side values for comments:
/// `10px` when uniform, otherwise `top/right/bottom/left`.
pub(crate) fn sides_short(sides: &Sides) -> String {
    if sides.is_uniform() {
        sides.first().display_short()
    } else {
        format!(
            "{}/{}/{}/{}",
            sides.top.display_short(),
            sides.right.display_short(),
            sides.bottom.display_short(),
            sides.left.display_short()
        )
    }
}

// ─── Children ────────────────────────────────────────────────────────────────

/// Children sorted by `order` (stable, so equal orders keep source order).
pub(crate) fn sorted_children(node: &NodeConfig) -> Vec<&NodeConfig> {
    let mut sorted: Vec<&NodeConfig> = node.children.iter().collect();
    sorted.sort_by_key(|c| c.order);
    sorted
}

/// Children sorted by `order`, plus the leaf index each child starts at, so
/// palette colours track the original nodes even when a target reverses or
/// interleaves the children. Advances `leaf_idx` past the whole subtree.
pub(crate) fn sorted_children_with_leaf_starts<'a>(
    node: &'a NodeConfig,
    leaf_idx: &mut usize,
) -> (Vec<&'a NodeConfig>, Vec<usize>) {
    let children = sorted_children(node);
    let mut starts = Vec::with_capacity(children.len());
    let mut acc = *leaf_idx;
    for child in &children {
        starts.push(acc);
        acc += child.count_leaves();
    }
    *leaf_idx = acc;
    (children, starts)
}

// ─── CSS keywords ────────────────────────────────────────────────────────────

pub(crate) fn css_flex_direction(d: FlexDirection) -> &'static str {
    match d {
        FlexDirection::Row => "row",
        FlexDirection::Column => "column",
        FlexDirection::RowReverse => "row-reverse",
        FlexDirection::ColumnReverse => "column-reverse",
    }
}

pub(crate) fn css_flex_wrap(w: FlexWrap) -> &'static str {
    match w {
        FlexWrap::NoWrap => "nowrap",
        FlexWrap::Wrap => "wrap",
        FlexWrap::WrapReverse => "wrap-reverse",
    }
}

pub(crate) fn css_justify_content(j: JustifyContent) -> &'static str {
    match j {
        JustifyContent::FlexStart => "flex-start",
        JustifyContent::FlexEnd => "flex-end",
        JustifyContent::Center => "center",
        JustifyContent::SpaceBetween => "space-between",
        JustifyContent::SpaceAround => "space-around",
        JustifyContent::SpaceEvenly => "space-evenly",
        JustifyContent::Stretch => "stretch",
        JustifyContent::Start => "start",
        JustifyContent::End => "end",
        JustifyContent::Default => "flex-start",
    }
}

pub(crate) fn css_align_items(a: AlignItems) -> &'static str {
    match a {
        AlignItems::FlexStart => "flex-start",
        AlignItems::FlexEnd => "flex-end",
        AlignItems::Center => "center",
        AlignItems::Baseline => "baseline",
        AlignItems::Stretch => "stretch",
        AlignItems::Start => "start",
        AlignItems::End => "end",
        AlignItems::Default => "stretch",
    }
}

pub(crate) fn css_align_content(a: AlignContent) -> &'static str {
    match a {
        AlignContent::FlexStart => "flex-start",
        AlignContent::FlexEnd => "flex-end",
        AlignContent::Center => "center",
        AlignContent::SpaceBetween => "space-between",
        AlignContent::SpaceAround => "space-around",
        AlignContent::SpaceEvenly => "space-evenly",
        AlignContent::Stretch => "stretch",
        AlignContent::Start => "start",
        AlignContent::End => "end",
        AlignContent::Default => "stretch",
    }
}

pub(crate) fn css_align_self(a: AlignSelf) -> &'static str {
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

// ─── String escaping ─────────────────────────────────────────────────────────
//
// User-provided labels end up inside generated source. Each target has its own
// literal syntax, so each gets its own escaper; none of these add quotes unless
// the name says `literal`.

/// Escape for HTML text content (`&`, `<`, `>`, `"`, `'`).
pub(crate) fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// Escape for JSX text children: HTML entities plus `{`/`}`, which would
/// otherwise open an expression.
pub(crate) fn jsx_text_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            '{' => out.push_str("&#123;"),
            '}' => out.push_str("&#125;"),
            _ => out.push(c),
        }
    }
    out
}

/// A quoted Rust string literal. `Debug` for `str` escapes `"`, `\` and
/// control characters and leaves printable Unicode alone.
pub(crate) fn rust_string_literal(s: &str) -> String {
    format!("{s:?}")
}

/// A quoted Dioxus `rsx!` string literal. These are format strings, so
/// `{`/`}` must be doubled in addition to normal Rust escaping.
pub(crate) fn dioxus_string_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '{' => out.push_str("{{"),
            '}' => out.push_str("}}"),
            _ => push_c_style_escape(&mut out, c, '"'),
        }
    }
    out.push('"');
    out
}

/// A single-quoted Dart string literal (`\`, `'`, `$` and control chars).
pub(crate) fn dart_string_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for c in s.chars() {
        match c {
            '$' => out.push_str("\\$"),
            _ => push_c_style_escape(&mut out, c, '\''),
        }
    }
    out.push('\'');
    out
}

/// A double-quoted Swift string literal. Swift's escapes match Rust's for
/// `\\`, `\"`, `\n`, `\r`, `\t`, `\0` and `\u{...}`, and `\(` would start an
/// interpolation, so the backslash is escaped first.
pub(crate) fn swift_string_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        push_c_style_escape(&mut out, c, '"');
    }
    out.push('"');
    out
}

/// Shared escaping for C-family string literals: backslash, the given quote,
/// and control characters (as `\n`, `\r`, `\t`, `\0` or `\u{..}`).
fn push_c_style_escape(out: &mut String, c: char, quote: char) {
    match c {
        '\\' => out.push_str("\\\\"),
        '\n' => out.push_str("\\n"),
        '\r' => out.push_str("\\r"),
        '\t' => out.push_str("\\t"),
        '\0' => out.push_str("\\0"),
        c if c == quote => {
            out.push('\\');
            out.push(c);
        }
        c if c.is_control() => {
            out.push_str(&format!("\\u{{{:x}}}", c as u32));
        }
        c => out.push(c),
    }
}

/// Collapse a label onto one line for use inside a `//` comment, so a
/// newline in a label cannot terminate the comment early.
pub(crate) fn single_line(s: &str) -> String {
    s.chars()
        .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb8_rounds_to_nearest() {
        // 127.5 → 128, 0.4 → 0, 254.6 → 255
        assert_eq!(rgb8(0.5, 0.0016, 0.9984), (128, 0, 255));
        // Palette round-trip: colorous bytes / 255 * 255 must give the byte back.
        let (r, g, b) = crate::art::palette_color(ColorPalette::Pastel1, 0);
        assert_eq!(rgb8(r, g, b), (251, 180, 174));
    }

    #[test]
    fn html_and_jsx_escape() {
        assert_eq!(
            html_escape("<a & \"b\" 'c'>"),
            "&lt;a &amp; &quot;b&quot; &#39;c&#39;&gt;"
        );
        assert_eq!(jsx_text_escape("{x} <y>"), "&#123;x&#125; &lt;y&gt;");
    }

    #[test]
    fn rust_and_dioxus_literals() {
        assert_eq!(rust_string_literal("a\"b\\c\n"), "\"a\\\"b\\\\c\\n\"");
        assert_eq!(dioxus_string_literal("{x} \"y\""), "\"{{x}} \\\"y\\\"\"");
        assert_eq!(dioxus_string_literal("\u{7f}"), "\"\\u{7f}\"");
    }

    #[test]
    fn dart_and_swift_literals() {
        assert_eq!(dart_string_literal("it's $5\\"), "'it\\'s \\$5\\\\'");
        assert_eq!(
            swift_string_literal("say \"hi\"\\(x)"),
            "\"say \\\"hi\\\"\\\\(x)\""
        );
        assert_eq!(swift_string_literal("tab\there"), "\"tab\\there\"");
    }

    #[test]
    fn single_line_strips_newlines() {
        assert_eq!(single_line("a\nb\r\nc"), "a b  c");
    }

    #[test]
    fn sides_short_forms() {
        assert_eq!(sides_short(&Sides::uniform(ValueConfig::Px(10.0))), "10px");
        let mut s = Sides::uniform(ValueConfig::Px(1.0));
        s.left = ValueConfig::Percent(5.0);
        assert_eq!(sides_short(&s), "1px/1px/1px/5%");
    }

    #[test]
    fn leaf_starts_follow_sorted_order() {
        let mut root = NodeConfig::new_container("root");
        let mut a = NodeConfig::new_container("a");
        a.children = vec![
            NodeConfig::new_leaf("a1", 1.0, 1.0),
            NodeConfig::new_leaf("a2", 1.0, 1.0),
        ];
        a.order = 1;
        let b = NodeConfig::new_leaf("b", 1.0, 1.0);
        root.children = vec![a, b];
        let mut idx = 3;
        let (children, starts) = sorted_children_with_leaf_starts(&root, &mut idx);
        assert_eq!(
            children
                .iter()
                .map(|c| c.label.as_str())
                .collect::<Vec<_>>(),
            ["b", "a"]
        );
        assert_eq!(starts, [3, 4]);
        assert_eq!(idx, 6);
    }
}
