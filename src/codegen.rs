use bevy::prelude::*;

use crate::art::PASTELS;
use crate::config::{NodeConfig, ValueConfig};

// ─── Bevy code generation ────────────────────────────────────────────────────

fn emit_bevy_value(v: &ValueConfig) -> String {
    match v {
        ValueConfig::Auto => "Val::Auto".into(),
        ValueConfig::Px(n) => format!("Val::Px({n:.1})"),
        ValueConfig::Percent(n) => format!("Val::Percent({n:.1})"),
        ValueConfig::Vw(n) => format!("Val::Vw({n:.1})"),
        ValueConfig::Vh(n) => format!("Val::Vh({n:.1})"),
    }
}

pub fn emit_bevy_code(root: &NodeConfig) -> String {
    let mut buf = String::from("fn spawn_ui(commands: &mut Commands) {\n");
    emit_node(&mut buf, root, 1, &mut 0, true);
    buf.push_str("}\n");
    buf
}

fn emit_node(
    buf: &mut String,
    node: &NodeConfig,
    depth: usize,
    leaf_idx: &mut usize,
    is_root: bool,
) {
    let pad = "    ".repeat(depth);
    let is_leaf = node.children.is_empty();

    let bg = if is_leaf {
        let (r, g, b) = PASTELS[*leaf_idx % PASTELS.len()];
        *leaf_idx += 1;
        format!("Color::srgb({r:.2}, {g:.2}, {b:.2})")
    } else {
        "Color::srgba(0.11, 0.11, 0.17, 1.0)".into()
    };

    let spawner = if is_root { "commands" } else { "parent" };
    buf.push_str(&format!("{pad}// {}\n", node.label));
    buf.push_str(&format!("{pad}{spawner}.spawn((\n"));

    buf.push_str(&format!("{pad}    Node {{\n"));
    buf.push_str(&format!("{pad}        display: Display::Flex,\n"));
    emit_field(
        buf,
        &pad,
        "flex_direction",
        &format!("FlexDirection::{:?}", node.flex_direction),
    );
    emit_field(
        buf,
        &pad,
        "flex_wrap",
        &format!("FlexWrap::{:?}", node.flex_wrap),
    );
    emit_field(
        buf,
        &pad,
        "justify_content",
        &format!("JustifyContent::{:?}", node.justify_content),
    );
    emit_field(
        buf,
        &pad,
        "align_items",
        &format!("AlignItems::{:?}", node.align_items),
    );
    emit_field(
        buf,
        &pad,
        "align_content",
        &format!("AlignContent::{:?}", node.align_content),
    );
    emit_field(buf, &pad, "row_gap", &emit_bevy_value(&node.row_gap));
    emit_field(buf, &pad, "column_gap", &emit_bevy_value(&node.column_gap));
    emit_field(buf, &pad, "flex_grow", &format!("{:.1}", node.flex_grow));
    emit_field(
        buf,
        &pad,
        "flex_shrink",
        &format!("{:.1}", node.flex_shrink),
    );
    emit_field(buf, &pad, "flex_basis", &emit_bevy_value(&node.flex_basis));
    emit_field(
        buf,
        &pad,
        "align_self",
        &format!("AlignSelf::{:?}", node.align_self),
    );
    emit_field(buf, &pad, "width", &emit_bevy_value(&node.width));
    emit_field(buf, &pad, "height", &emit_bevy_value(&node.height));
    emit_field(buf, &pad, "min_width", &emit_bevy_value(&node.min_width));
    emit_field(buf, &pad, "min_height", &emit_bevy_value(&node.min_height));
    emit_field(buf, &pad, "max_width", &emit_bevy_value(&node.max_width));
    emit_field(buf, &pad, "max_height", &emit_bevy_value(&node.max_height));
    emit_field(
        buf,
        &pad,
        "padding",
        &format!("UiRect::all({})", emit_bevy_value(&node.padding)),
    );
    emit_field(
        buf,
        &pad,
        "margin",
        &format!("UiRect::all({})", emit_bevy_value(&node.margin)),
    );
    buf.push_str(&format!("{pad}        ..default()\n"));
    buf.push_str(&format!("{pad}    }},\n"));

    buf.push_str(&format!("{pad}    BackgroundColor({bg}),\n"));
    buf.push_str(&format!("{pad}))"));

    if is_leaf {
        buf.push_str(".with_children(|parent| {\n");
        buf.push_str(&format!("{pad}    parent.spawn((\n"));
        buf.push_str(&format!("{pad}        Text::new({:?}),\n", node.label));
        buf.push_str(&format!(
            "{pad}        TextFont {{ font_size: 26.0, ..default() }},\n"
        ));
        buf.push_str(&format!(
            "{pad}        TextColor(Color::srgba(0.05, 0.05, 0.1, 0.85)),\n"
        ));
        buf.push_str(&format!("{pad}    ));\n"));
        buf.push_str(&format!("{pad}}});\n"));
    } else if node.children.is_empty() {
        buf.push_str(";\n");
    } else {
        buf.push_str(".with_children(|parent| {\n");
        for child in &node.children {
            emit_node(buf, child, depth + 1, leaf_idx, false);
        }
        buf.push_str(&format!("{pad}}});\n"));
    }
}

fn emit_field(buf: &mut String, pad: &str, name: &str, value: &str) {
    buf.push_str(&format!("{pad}        {name}: {value},\n"));
}

// ─── HTML/CSS code generation ────────────────────────────────────────────────

fn emit_css_value(v: &ValueConfig) -> String {
    match v {
        ValueConfig::Auto => "auto".into(),
        ValueConfig::Px(n) => format!("{n:.1}px"),
        ValueConfig::Percent(n) => format!("{n:.1}%"),
        ValueConfig::Vw(n) => format!("{n:.1}vw"),
        ValueConfig::Vh(n) => format!("{n:.1}vh"),
    }
}

fn css_flex_direction(d: FlexDirection) -> &'static str {
    match d {
        FlexDirection::Row => "row",
        FlexDirection::Column => "column",
        FlexDirection::RowReverse => "row-reverse",
        FlexDirection::ColumnReverse => "column-reverse",
    }
}

fn css_flex_wrap(w: FlexWrap) -> &'static str {
    match w {
        FlexWrap::NoWrap => "nowrap",
        FlexWrap::Wrap => "wrap",
        FlexWrap::WrapReverse => "wrap-reverse",
    }
}

fn css_justify_content(j: JustifyContent) -> &'static str {
    match j {
        JustifyContent::FlexStart => "flex-start",
        JustifyContent::FlexEnd => "flex-end",
        JustifyContent::Center => "center",
        JustifyContent::SpaceBetween => "space-between",
        JustifyContent::SpaceAround => "space-around",
        JustifyContent::SpaceEvenly => "space-evenly",
        _ => "flex-start",
    }
}

fn css_align_items(a: AlignItems) -> &'static str {
    match a {
        AlignItems::FlexStart => "flex-start",
        AlignItems::FlexEnd => "flex-end",
        AlignItems::Center => "center",
        AlignItems::Baseline => "baseline",
        AlignItems::Stretch => "stretch",
        _ => "stretch",
    }
}

fn css_align_content(a: AlignContent) -> &'static str {
    match a {
        AlignContent::FlexStart => "flex-start",
        AlignContent::FlexEnd => "flex-end",
        AlignContent::Center => "center",
        AlignContent::SpaceBetween => "space-between",
        AlignContent::SpaceAround => "space-around",
        AlignContent::SpaceEvenly => "space-evenly",
        AlignContent::Stretch => "stretch",
        _ => "stretch",
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
        _ => "auto",
    }
}

pub fn emit_html_css(root: &NodeConfig) -> String {
    let mut css = String::new();
    let mut html = String::new();
    emit_html_node(&mut css, &mut html, root, 0, &mut 0, &mut 0);
    format!("<style>\n{css}</style>\n\n{html}")
}

fn emit_html_node(
    css: &mut String,
    html: &mut String,
    node: &NodeConfig,
    depth: usize,
    leaf_idx: &mut usize,
    id_counter: &mut usize,
) {
    let id = *id_counter;
    *id_counter += 1;
    let is_leaf = node.children.is_empty();
    let pad_html = "  ".repeat(depth);
    let class = format!("node-{id}");

    let bg = if is_leaf {
        let (r, g, b) = PASTELS[*leaf_idx % PASTELS.len()];
        *leaf_idx += 1;
        format!(
            "rgb({}, {}, {})",
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
        )
    } else {
        "rgba(28, 28, 43, 1)".into()
    };

    css.push_str(&format!(".{class} {{\n"));
    css.push_str("  display: flex;\n");
    css.push_str(&format!(
        "  flex-direction: {};\n",
        css_flex_direction(node.flex_direction)
    ));
    css.push_str(&format!(
        "  flex-wrap: {};\n",
        css_flex_wrap(node.flex_wrap)
    ));
    css.push_str(&format!(
        "  justify-content: {};\n",
        css_justify_content(node.justify_content)
    ));
    css.push_str(&format!(
        "  align-items: {};\n",
        css_align_items(node.align_items)
    ));
    css.push_str(&format!(
        "  align-content: {};\n",
        css_align_content(node.align_content)
    ));
    css.push_str(&format!("  row-gap: {};\n", emit_css_value(&node.row_gap)));
    css.push_str(&format!(
        "  column-gap: {};\n",
        emit_css_value(&node.column_gap)
    ));
    css.push_str(&format!("  flex-grow: {:.1};\n", node.flex_grow));
    css.push_str(&format!("  flex-shrink: {:.1};\n", node.flex_shrink));
    css.push_str(&format!(
        "  flex-basis: {};\n",
        emit_css_value(&node.flex_basis)
    ));
    css.push_str(&format!(
        "  align-self: {};\n",
        css_align_self(node.align_self)
    ));
    css.push_str(&format!("  width: {};\n", emit_css_value(&node.width)));
    css.push_str(&format!("  height: {};\n", emit_css_value(&node.height)));
    css.push_str(&format!(
        "  min-width: {};\n",
        emit_css_value(&node.min_width)
    ));
    css.push_str(&format!(
        "  min-height: {};\n",
        emit_css_value(&node.min_height)
    ));
    css.push_str(&format!(
        "  max-width: {};\n",
        emit_css_value(&node.max_width)
    ));
    css.push_str(&format!(
        "  max-height: {};\n",
        emit_css_value(&node.max_height)
    ));
    css.push_str(&format!("  padding: {};\n", emit_css_value(&node.padding)));
    css.push_str(&format!("  margin: {};\n", emit_css_value(&node.margin)));
    css.push_str(&format!("  background: {bg};\n"));
    css.push_str("  box-sizing: border-box;\n");
    if is_leaf {
        css.push_str("  color: rgba(13, 13, 26, 0.85);\n");
        css.push_str("  font-size: 26px;\n");
    }
    css.push_str("}\n\n");

    if is_leaf {
        html.push_str(&format!(
            "{pad_html}<div class=\"{class}\">{}</div>\n",
            node.label
        ));
    } else {
        html.push_str(&format!("{pad_html}<div class=\"{class}\">\n"));
        for child in &node.children {
            emit_html_node(css, html, child, depth + 1, leaf_idx, id_counter);
        }
        html.push_str(&format!("{pad_html}</div>\n"));
    }
}

// ─── Tailwind HTML code generation ───────────────────────────────────────────

fn tailwind_flex_direction(d: FlexDirection) -> &'static str {
    match d {
        FlexDirection::Row => "flex-row",
        FlexDirection::Column => "flex-col",
        FlexDirection::RowReverse => "flex-row-reverse",
        FlexDirection::ColumnReverse => "flex-col-reverse",
    }
}

fn tailwind_flex_wrap(w: FlexWrap) -> &'static str {
    match w {
        FlexWrap::NoWrap => "flex-nowrap",
        FlexWrap::Wrap => "flex-wrap",
        FlexWrap::WrapReverse => "flex-wrap-reverse",
    }
}

fn tailwind_justify_content(j: JustifyContent) -> &'static str {
    match j {
        JustifyContent::FlexStart => "justify-start",
        JustifyContent::FlexEnd => "justify-end",
        JustifyContent::Center => "justify-center",
        JustifyContent::SpaceBetween => "justify-between",
        JustifyContent::SpaceAround => "justify-around",
        JustifyContent::SpaceEvenly => "justify-evenly",
        _ => "justify-start",
    }
}

fn tailwind_align_items(a: AlignItems) -> &'static str {
    match a {
        AlignItems::FlexStart => "items-start",
        AlignItems::FlexEnd => "items-end",
        AlignItems::Center => "items-center",
        AlignItems::Baseline => "items-baseline",
        AlignItems::Stretch => "items-stretch",
        _ => "items-stretch",
    }
}

fn tailwind_align_content(a: AlignContent) -> &'static str {
    match a {
        AlignContent::FlexStart => "content-start",
        AlignContent::FlexEnd => "content-end",
        AlignContent::Center => "content-center",
        AlignContent::SpaceBetween => "content-between",
        AlignContent::SpaceAround => "content-around",
        AlignContent::SpaceEvenly => "content-evenly",
        AlignContent::Stretch => "content-stretch",
        _ => "content-stretch",
    }
}

fn tailwind_align_self(a: AlignSelf) -> &'static str {
    match a {
        AlignSelf::Auto => "self-auto",
        AlignSelf::FlexStart => "self-start",
        AlignSelf::FlexEnd => "self-end",
        AlignSelf::Center => "self-center",
        AlignSelf::Baseline => "self-baseline",
        AlignSelf::Stretch => "self-stretch",
        _ => "self-auto",
    }
}

fn tailwind_value(property: &str, v: &ValueConfig) -> String {
    match v {
        ValueConfig::Auto => format!("{property}-auto"),
        ValueConfig::Px(n) => format!("{property}-[{n:.1}px]"),
        ValueConfig::Percent(n) => format!("{property}-[{n:.1}%]"),
        ValueConfig::Vw(n) => format!("{property}-[{n:.1}vw]"),
        ValueConfig::Vh(n) => format!("{property}-[{n:.1}vh]"),
    }
}

pub fn emit_tailwind(root: &NodeConfig) -> String {
    let mut buf = String::new();
    emit_tailwind_node(&mut buf, root, 0, &mut 0);
    buf
}

fn emit_tailwind_node(buf: &mut String, node: &NodeConfig, depth: usize, leaf_idx: &mut usize) {
    let pad = "  ".repeat(depth);
    let is_leaf = node.children.is_empty();

    let bg = if is_leaf {
        let (r, g, b) = PASTELS[*leaf_idx % PASTELS.len()];
        *leaf_idx += 1;
        format!(
            "bg-[rgb({},{},{})]",
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
        )
    } else {
        "bg-[rgba(28,28,43,1)]".into()
    };

    let mut classes = vec![
        "flex".into(),
        tailwind_flex_direction(node.flex_direction).into(),
        tailwind_flex_wrap(node.flex_wrap).into(),
        tailwind_justify_content(node.justify_content).into(),
        tailwind_align_items(node.align_items).into(),
        tailwind_align_content(node.align_content).into(),
        tailwind_value("gap-x", &node.column_gap),
        tailwind_value("gap-y", &node.row_gap),
        format!("grow-[{:.1}]", node.flex_grow),
        format!("shrink-[{:.1}]", node.flex_shrink),
        tailwind_value("basis", &node.flex_basis),
        tailwind_align_self(node.align_self).into(),
        tailwind_value("w", &node.width),
        tailwind_value("h", &node.height),
        tailwind_value("min-w", &node.min_width),
        tailwind_value("min-h", &node.min_height),
        tailwind_value("max-w", &node.max_width),
        tailwind_value("max-h", &node.max_height),
        tailwind_value("p", &node.padding),
        tailwind_value("m", &node.margin),
        bg,
        "box-border".into(),
    ];

    if is_leaf {
        classes.push("text-[26px]".into());
        classes.push("text-[rgba(13,13,26,0.85)]".into());
    }

    let cls = classes.join(" ");

    if is_leaf {
        buf.push_str(&format!("{pad}<div class=\"{cls}\">{}</div>\n", node.label));
    } else {
        buf.push_str(&format!("{pad}<div class=\"{cls}\">\n"));
        for child in &node.children {
            emit_tailwind_node(buf, child, depth + 1, leaf_idx);
        }
        buf.push_str(&format!("{pad}</div>\n"));
    }
}

// ─── SwiftUI code generation ─────────────────────────────────────────────────

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

fn swift_alignment(a: AlignItems) -> &'static str {
    match a {
        AlignItems::FlexStart => ".top",
        AlignItems::FlexEnd => ".bottom",
        AlignItems::Center => ".center",
        AlignItems::Baseline => ".firstTextBaseline",
        AlignItems::Stretch => ".center",
        _ => ".center",
    }
}

fn swift_h_alignment(a: AlignItems) -> &'static str {
    match a {
        AlignItems::FlexStart => ".leading",
        AlignItems::FlexEnd => ".trailing",
        AlignItems::Center => ".center",
        _ => ".center",
    }
}

pub fn emit_swiftui(root: &NodeConfig) -> String {
    let mut buf = String::from("struct ContentView: View {\n    var body: some View {\n");
    emit_swiftui_node(&mut buf, root, 2, &mut 0);
    buf.push_str("    }\n}\n");
    buf
}

fn emit_swiftui_node(buf: &mut String, node: &NodeConfig, depth: usize, leaf_idx: &mut usize) {
    let pad = "    ".repeat(depth);
    let is_leaf = node.children.is_empty();

    if is_leaf {
        let (r, g, b) = PASTELS[*leaf_idx % PASTELS.len()];
        *leaf_idx += 1;

        buf.push_str(&format!("{pad}Text({:?})\n", node.label));
        buf.push_str(&format!("{pad}    .font(.system(size: 26))\n"));
        buf.push_str(&format!(
            "{pad}    .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))\n"
        ));

        let w = swift_optional_value(&node.width);
        let h = swift_optional_value(&node.height);
        if w.is_some() || h.is_some() {
            let w_str = w.as_deref().unwrap_or("nil");
            let h_str = h.as_deref().unwrap_or("nil");
            buf.push_str(&format!(
                "{pad}    .frame(width: {w_str}, height: {h_str})\n"
            ));
        }
        let min_w = swift_optional_value(&node.min_width);
        let min_h = swift_optional_value(&node.min_height);
        let max_w = swift_optional_value(&node.max_width);
        let max_h = swift_optional_value(&node.max_height);
        if min_w.is_some() || min_h.is_some() || max_w.is_some() || max_h.is_some() {
            buf.push_str(&format!(
                "{pad}    .frame(minWidth: {}, minHeight: {}, maxWidth: {}, maxHeight: {})\n",
                min_w.as_deref().unwrap_or("nil"),
                min_h.as_deref().unwrap_or("nil"),
                max_w.as_deref().unwrap_or("nil"),
                max_h.as_deref().unwrap_or("nil"),
            ));
        }
        if let Some(p) = swift_optional_value(&node.padding) {
            buf.push_str(&format!("{pad}    .padding({p})\n"));
        }
        buf.push_str(&format!(
            "{pad}    .background(Color(red: {r:.2}, green: {g:.2}, blue: {b:.2}))\n"
        ));
    } else {
        let is_row = matches!(
            node.flex_direction,
            FlexDirection::Row | FlexDirection::RowReverse
        );

        let spacing = match &node.column_gap {
            ValueConfig::Px(n) if is_row => format!(", spacing: {n:.1}"),
            _ => match &node.row_gap {
                ValueConfig::Px(n) if !is_row => format!(", spacing: {n:.1}"),
                _ => String::new(),
            },
        };

        let alignment = if is_row {
            swift_alignment(node.align_items)
        } else {
            swift_h_alignment(node.align_items)
        };

        let stack = if is_row { "HStack" } else { "VStack" };
        buf.push_str(&format!(
            "{pad}{stack}(alignment: {alignment}{spacing}) {{\n"
        ));

        for child in &node.children {
            emit_swiftui_node(buf, child, depth + 1, leaf_idx);
        }

        buf.push_str(&format!("{pad}}}\n"));

        let w = swift_optional_value(&node.width);
        let h = swift_optional_value(&node.height);
        if w.is_some() || h.is_some() {
            let w_str = w.as_deref().unwrap_or("nil");
            let h_str = h.as_deref().unwrap_or("nil");
            buf.push_str(&format!("{pad}.frame(width: {w_str}, height: {h_str})\n"));
        }
        let min_w = swift_optional_value(&node.min_width);
        let min_h = swift_optional_value(&node.min_height);
        let max_w = swift_optional_value(&node.max_width);
        let max_h = swift_optional_value(&node.max_height);
        if min_w.is_some() || min_h.is_some() || max_w.is_some() || max_h.is_some() {
            buf.push_str(&format!(
                "{pad}.frame(minWidth: {}, minHeight: {}, maxWidth: {}, maxHeight: {})\n",
                min_w.as_deref().unwrap_or("nil"),
                min_h.as_deref().unwrap_or("nil"),
                max_w.as_deref().unwrap_or("nil"),
                max_h.as_deref().unwrap_or("nil"),
            ));
        }
        if let Some(p) = swift_optional_value(&node.padding) {
            buf.push_str(&format!("{pad}.padding({p})\n"));
        }
        buf.push_str(&format!(
            "{pad}.background(Color(red: 0.11, green: 0.11, blue: 0.17))\n"
        ));
    }
}
