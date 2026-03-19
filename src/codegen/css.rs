use std::fmt::Write;

use bevy::prelude::*;

use crate::art::PASTELS;
use crate::config::{NodeConfig, ValueConfig};

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

    let _ = writeln!(css, ".{class} {{");
    css.push_str("  display: flex;\n");
    let _ = writeln!(css, "  flex-direction: {};", css_flex_direction(node.flex_direction));
    let _ = writeln!(css, "  flex-wrap: {};", css_flex_wrap(node.flex_wrap));
    let _ = writeln!(css, "  justify-content: {};", css_justify_content(node.justify_content));
    let _ = writeln!(css, "  align-items: {};", css_align_items(node.align_items));
    let _ = writeln!(css, "  align-content: {};", css_align_content(node.align_content));
    let _ = writeln!(css, "  row-gap: {};", emit_css_value(&node.row_gap));
    let _ = writeln!(css, "  column-gap: {};", emit_css_value(&node.column_gap));
    let _ = writeln!(css, "  flex-grow: {:.1};", node.flex_grow);
    let _ = writeln!(css, "  flex-shrink: {:.1};", node.flex_shrink);
    let _ = writeln!(css, "  flex-basis: {};", emit_css_value(&node.flex_basis));
    let _ = writeln!(css, "  align-self: {};", css_align_self(node.align_self));
    let _ = writeln!(css, "  width: {};", emit_css_value(&node.width));
    let _ = writeln!(css, "  height: {};", emit_css_value(&node.height));
    let _ = writeln!(css, "  min-width: {};", emit_css_value(&node.min_width));
    let _ = writeln!(css, "  min-height: {};", emit_css_value(&node.min_height));
    let _ = writeln!(css, "  max-width: {};", emit_css_value(&node.max_width));
    let _ = writeln!(css, "  max-height: {};", emit_css_value(&node.max_height));
    let _ = writeln!(css, "  padding: {};", emit_css_value(&node.padding));
    let _ = writeln!(css, "  margin: {};", emit_css_value(&node.margin));
    let _ = writeln!(css, "  background: {bg};");
    css.push_str("  box-sizing: border-box;\n");
    if is_leaf {
        css.push_str("  color: rgba(13, 13, 26, 0.85);\n");
        css.push_str("  font-size: 26px;\n");
    }
    css.push_str("}\n\n");

    if is_leaf {
        let _ = writeln!(html, "{pad_html}<div class=\"{class}\">{}</div>", node.label);
    } else {
        let _ = writeln!(html, "{pad_html}<div class=\"{class}\">");
        for child in &node.children {
            emit_html_node(css, html, child, depth + 1, leaf_idx, id_counter);
        }
        let _ = writeln!(html, "{pad_html}</div>");
    }
}
