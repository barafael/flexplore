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
