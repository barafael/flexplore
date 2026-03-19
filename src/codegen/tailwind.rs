use std::fmt::Write;

use anyhow::Result;
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
        JustifyContent::Stretch => "justify-stretch",
        JustifyContent::Start => "justify-start",
        JustifyContent::End => "justify-end",
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
        AlignItems::Start => "items-start",
        AlignItems::End => "items-end",
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
        AlignContent::Start => "content-start",
        AlignContent::End => "content-end",
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
        AlignSelf::Start => "self-start",
        AlignSelf::End => "self-end",
    }
}

fn tailwind_value(property: &str, v: &ValueConfig) -> String {
    match v {
        ValueConfig::Auto => {
            match property {
                "w" | "h" | "basis" | "m" => format!("{property}-auto"),
                "max-w" | "max-h" => format!("{property}-none"),
                "p" | "gap-x" | "gap-y" | "min-w" | "min-h" => format!("{property}-0"),
                _ => format!("{property}-auto"),
            }
        }
        ValueConfig::Px(n) => format!("{property}-[{n:.1}px]"),
        ValueConfig::Percent(n) => format!("{property}-[{n:.1}%]"),
        ValueConfig::Vw(n) => format!("{property}-[{n:.1}vw]"),
        ValueConfig::Vh(n) => format!("{property}-[{n:.1}vh]"),
    }
}

pub fn emit_tailwind(root: &NodeConfig) -> Result<String> {
    let mut buf = String::new();
    emit_tailwind_node(&mut buf, root, 0, &mut 0)?;
    Ok(buf)
}

fn emit_tailwind_node(
    buf: &mut String,
    node: &NodeConfig,
    depth: usize,
    leaf_idx: &mut usize,
) -> Result<()> {
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
        (if node.visible { "flex" } else { "hidden" }).into(),
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
    if node.order != 0 {
        classes.push(format!("order-[{}]", node.order));
    }

    if is_leaf {
        classes.push("text-[26px]".into());
        classes.push("text-[rgba(13,13,26,0.85)]".into());
    }

    let cls = classes.join(" ");

    if is_leaf {
        writeln!(buf, "{pad}<div class=\"{cls}\">{}</div>", node.label)?;
    } else {
        writeln!(buf, "{pad}<div class=\"{cls}\">")?;
        let mut sorted: Vec<&NodeConfig> = node.children.iter().collect();
        sorted.sort_by_key(|c| c.order);
        for child in sorted {
            emit_tailwind_node(buf, child, depth + 1, leaf_idx)?;
        }
        writeln!(buf, "{pad}</div>")?;
    }
    Ok(())
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
    fn emits_flex_classes() {
        let code = emit_tailwind(&test_container()).unwrap();
        assert!(code.contains("flex"));
        assert!(code.contains("flex-row"));
    }

    #[test]
    fn emits_column_direction() {
        let mut root = test_container();
        root.flex_direction = FlexDirection::Column;
        let code = emit_tailwind(&root).unwrap();
        assert!(code.contains("flex-col"));
    }

    #[test]
    fn emits_hidden_when_not_visible() {
        let mut node = NodeConfig::new_leaf("A", 80.0, 80.0);
        node.visible = false;
        let mut root = NodeConfig::new_container("root");
        root.children = vec![node];
        let code = emit_tailwind(&root).unwrap();
        assert!(code.contains("hidden"));
    }

    #[test]
    fn emits_order_class() {
        let mut node = NodeConfig::new_leaf("A", 80.0, 80.0);
        node.order = -2;
        let mut root = NodeConfig::new_container("root");
        root.children = vec![node];
        let code = emit_tailwind(&root).unwrap();
        assert!(code.contains("order-[-2]"));
    }

    #[test]
    fn emits_leaf_label() {
        let code = emit_tailwind(&test_container()).unwrap();
        assert!(code.contains(">A</div>"));
        assert!(code.contains(">B</div>"));
    }
}
