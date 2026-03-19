use bevy::prelude::*;

use crate::art::PASTELS;
use crate::config::{NodeConfig, ValueConfig};

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
