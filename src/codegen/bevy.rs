use crate::art::PASTELS;
use crate::config::{NodeConfig, ValueConfig};

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
