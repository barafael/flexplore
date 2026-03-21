use bevy::prelude::*;
use test_case::test_case;

use crate::codegen::{emit_bevy_code, emit_html_css, emit_iced};
use crate::config::{ColorPalette, NodeConfig, ValueConfig};
use crate::templates;

// ─── Procedural graph builders ───────────────────────────────────────────────

fn single_leaf() -> NodeConfig {
    let mut root = NodeConfig::new_container("root");
    root.children = vec![NodeConfig::new_leaf("only", 100.0, 60.0)];
    root
}

fn wide_flat(n: usize) -> NodeConfig {
    let mut root = NodeConfig::new_container("root");
    root.children = (0..n)
        .map(|i| NodeConfig::new_leaf(format!("item-{i}"), 60.0, 60.0))
        .collect();
    root
}

fn deep_chain(depth: usize) -> NodeConfig {
    let mut current = NodeConfig::new_leaf("leaf", 50.0, 50.0);
    for i in (0..depth).rev() {
        let mut parent = NodeConfig::new_container(format!("level-{i}"));
        parent.children = vec![current];
        current = parent;
    }
    current
}

fn direction_test(dir: FlexDirection) -> NodeConfig {
    let mut root = NodeConfig::new_container("root");
    root.flex_direction = dir;
    root.children = vec![
        NodeConfig::new_leaf("A", 80.0, 80.0),
        NodeConfig::new_leaf("B", 80.0, 80.0),
    ];
    root
}

fn justify_test(j: JustifyContent) -> NodeConfig {
    let mut root = NodeConfig::new_container("root");
    root.justify_content = j;
    root.children = vec![
        NodeConfig::new_leaf("A", 80.0, 80.0),
        NodeConfig::new_leaf("B", 80.0, 80.0),
    ];
    root
}

fn align_items_test(a: AlignItems) -> NodeConfig {
    let mut root = NodeConfig::new_container("root");
    root.align_items = a;
    root.children = vec![
        NodeConfig::new_leaf("A", 80.0, 80.0),
        NodeConfig::new_leaf("B", 80.0, 80.0),
    ];
    root
}

fn wrap_test(w: FlexWrap) -> NodeConfig {
    let mut root = NodeConfig::new_container("root");
    root.flex_wrap = w;
    root.children = vec![
        NodeConfig::new_leaf("A", 80.0, 80.0),
        NodeConfig::new_leaf("B", 80.0, 80.0),
    ];
    root
}

fn value_leaf(width: ValueConfig, height: ValueConfig) -> NodeConfig {
    let mut leaf = NodeConfig::new_leaf("sized", 100.0, 100.0);
    leaf.width = width;
    leaf.height = height;
    let mut root = NodeConfig::new_container("root");
    root.children = vec![leaf];
    root
}

fn grow_shrink(grow: f32, shrink: f32) -> NodeConfig {
    let mut leaf = NodeConfig::new_leaf("flex", 80.0, 80.0);
    leaf.flex_grow = grow;
    leaf.flex_shrink = shrink;
    let mut root = NodeConfig::new_container("root");
    root.children = vec![leaf];
    root
}

fn with_gaps(row_gap: ValueConfig, col_gap: ValueConfig) -> NodeConfig {
    let mut root = NodeConfig::new_container("root");
    root.row_gap = row_gap;
    root.column_gap = col_gap;
    root.children = vec![
        NodeConfig::new_leaf("A", 60.0, 60.0),
        NodeConfig::new_leaf("B", 60.0, 60.0),
    ];
    root
}

fn hidden_child() -> NodeConfig {
    let mut hidden = NodeConfig::new_leaf("hidden", 80.0, 80.0);
    hidden.visible = false;
    let mut root = NodeConfig::new_container("root");
    root.children = vec![NodeConfig::new_leaf("visible", 80.0, 80.0), hidden];
    root
}

fn ordered_children() -> NodeConfig {
    let mut a = NodeConfig::new_leaf("A", 80.0, 80.0);
    a.order = 3;
    let mut b = NodeConfig::new_leaf("B", 80.0, 80.0);
    b.order = -1;
    let c = NodeConfig::new_leaf("C", 80.0, 80.0);
    let mut root = NodeConfig::new_container("root");
    root.children = vec![a, b, c];
    root
}

fn with_padding_margin() -> NodeConfig {
    let mut leaf = NodeConfig::new_leaf("spaced", 80.0, 80.0);
    leaf.padding = ValueConfig::Px(20.0);
    leaf.margin = ValueConfig::Px(10.0);
    let mut root = NodeConfig::new_container("root");
    root.children = vec![leaf];
    root
}

fn align_self_child(align: AlignSelf) -> NodeConfig {
    let mut leaf = NodeConfig::new_leaf("self-aligned", 80.0, 80.0);
    leaf.align_self = align;
    let mut root = NodeConfig::new_container("root");
    root.children = vec![leaf];
    root
}

fn with_flex_basis(basis: ValueConfig) -> NodeConfig {
    let mut leaf = NodeConfig::new_leaf("based", 80.0, 80.0);
    leaf.flex_basis = basis;
    let mut root = NodeConfig::new_container("root");
    root.children = vec![leaf];
    root
}

fn min_max_sizes() -> NodeConfig {
    let mut leaf = NodeConfig::new_leaf("constrained", 80.0, 80.0);
    leaf.min_width = ValueConfig::Px(40.0);
    leaf.max_width = ValueConfig::Px(200.0);
    leaf.min_height = ValueConfig::Px(30.0);
    leaf.max_height = ValueConfig::Px(150.0);
    let mut root = NodeConfig::new_container("root");
    root.children = vec![leaf];
    root
}

fn nested_mixed() -> NodeConfig {
    let mut inner = NodeConfig::new_container("inner");
    inner.flex_direction = FlexDirection::Column;
    inner.children = vec![
        NodeConfig::new_leaf("X", 40.0, 40.0),
        NodeConfig::new_leaf("Y", 40.0, 40.0),
    ];
    let mut root = NodeConfig::new_container("outer");
    root.flex_direction = FlexDirection::Row;
    root.children = vec![
        NodeConfig::new_leaf("A", 80.0, 80.0),
        inner,
        NodeConfig::new_leaf("B", 80.0, 80.0),
    ];
    root
}

fn all_hidden() -> NodeConfig {
    let mut a = NodeConfig::new_leaf("A", 80.0, 80.0);
    a.visible = false;
    let mut b = NodeConfig::new_leaf("B", 80.0, 80.0);
    b.visible = false;
    let mut root = NodeConfig::new_container("root");
    root.children = vec![a, b];
    root
}

fn align_content_test(ac: AlignContent) -> NodeConfig {
    let mut root = NodeConfig::new_container("root");
    root.align_content = ac;
    root.flex_wrap = FlexWrap::Wrap;
    root.children = vec![
        NodeConfig::new_leaf("A", 80.0, 80.0),
        NodeConfig::new_leaf("B", 80.0, 80.0),
    ];
    root
}

// ─── Structural assertions ───────────────────────────────────────────────────

/// All codegen targets must produce non-empty, well-structured output.
fn assert_both_emit(node: &NodeConfig, palette: ColorPalette) {
    let html = emit_html_css(node, palette).unwrap();
    let bevy = emit_bevy_code(node, palette).unwrap();
    let iced = emit_iced(node, palette).unwrap();

    // HTML structure
    assert!(html.contains("<style>"), "HTML missing <style> block");
    assert!(html.contains("</style>"), "HTML missing </style> close");
    assert!(html.contains("<div"), "HTML missing <div> elements");

    // Bevy structure
    assert!(
        bevy.contains("fn spawn_ui(commands: &mut Commands)"),
        "Bevy missing spawn_ui function"
    );
    assert!(bevy.contains("Node {"), "Bevy missing Node struct");

    // Iced structure
    assert!(
        iced.contains("fn view(&self) -> iced::Element<'_, Message>"),
        "Iced missing view function"
    );
    assert!(iced.contains(".into()"), "Iced missing .into() call");

    // Leaf count: each leaf produces a <div> with its label AND a Text::new in Bevy
    let leaf_count = count_leaves(node);
    for i in 0..leaf_count {
        // Each leaf has a background color assignment in both targets
        assert!(
            html.contains("background:"),
            "HTML missing background for leaf {i}"
        );
    }
}

fn count_leaves(node: &NodeConfig) -> usize {
    if node.children.is_empty() {
        1
    } else {
        node.children.iter().map(count_leaves).sum()
    }
}

/// Verify that every leaf label appears in all outputs.
fn assert_labels_present(node: &NodeConfig, palette: ColorPalette) {
    let html = emit_html_css(node, palette).unwrap();
    let bevy = emit_bevy_code(node, palette).unwrap();
    let iced = emit_iced(node, palette).unwrap();
    check_labels_in(&html, &bevy, &iced, node);
}

fn check_labels_in(html: &str, bevy: &str, iced: &str, node: &NodeConfig) {
    if node.children.is_empty() {
        assert!(
            html.contains(&node.label),
            "HTML missing leaf label {:?}",
            node.label,
        );
        assert!(
            bevy.contains(&format!("Text::new({:?})", node.label)),
            "Bevy missing Text::new for {:?}",
            node.label,
        );
        assert!(
            iced.contains(&format!("text({:?})", node.label)),
            "Iced missing text() for {:?}",
            node.label,
        );
    }
    for child in &node.children {
        check_labels_in(html, bevy, iced, child);
    }
}

/// Verify matching property consistency across HTML, Bevy, and Iced outputs.
fn assert_property_consistency(node: &NodeConfig, palette: ColorPalette) {
    let html = emit_html_css(node, palette).unwrap();
    let bevy = emit_bevy_code(node, palette).unwrap();
    let iced = emit_iced(node, palette).unwrap();

    // flex-direction: if non-default, all must mention it
    if node.flex_direction != FlexDirection::Row {
        let css_dir = match node.flex_direction {
            FlexDirection::Column => "column",
            FlexDirection::RowReverse => "row-reverse",
            FlexDirection::ColumnReverse => "column-reverse",
            _ => unreachable!(),
        };
        assert!(
            html.contains(&format!("flex-direction: {css_dir}")),
            "HTML missing flex-direction: {css_dir}"
        );
        assert!(
            bevy.contains(&format!("FlexDirection::{:?}", node.flex_direction)),
            "Bevy missing FlexDirection::{:?}",
            node.flex_direction,
        );
        // Iced uses row![] / column![] — column direction means column![ must appear
        let is_col = matches!(
            node.flex_direction,
            FlexDirection::Column | FlexDirection::ColumnReverse
        );
        if is_col {
            assert!(
                iced.contains("column!["),
                "Iced missing column![ for {:?}",
                node.flex_direction
            );
        }
    }

    // visibility: if hidden, all targets should indicate it
    if !node.visible {
        assert!(
            html.contains("visibility: hidden"),
            "HTML missing visibility: hidden"
        );
        assert!(
            bevy.contains("Visibility::Hidden"),
            "Bevy missing Visibility::Hidden"
        );
        assert!(
            iced.contains("// NOTE: hidden"),
            "Iced missing hidden comment"
        );
    }
}

// ─── Parameterized tests: flex-direction ─────────────────────────────────────

#[test_case(FlexDirection::Row ; "row")]
#[test_case(FlexDirection::Column ; "column")]
#[test_case(FlexDirection::RowReverse ; "row_reverse")]
#[test_case(FlexDirection::ColumnReverse ; "column_reverse")]
fn flex_direction_codegen(dir: FlexDirection) {
    let node = direction_test(dir);
    assert_both_emit(&node, ColorPalette::Pastel1);
    assert_labels_present(&node, ColorPalette::Pastel1);
    assert_property_consistency(&node, ColorPalette::Pastel1);
}

// ─── Parameterized tests: justify-content ────────────────────────────────────

#[test_case(JustifyContent::FlexStart ; "flex_start")]
#[test_case(JustifyContent::FlexEnd ; "flex_end")]
#[test_case(JustifyContent::Center ; "center")]
#[test_case(JustifyContent::SpaceBetween ; "space_between")]
#[test_case(JustifyContent::SpaceAround ; "space_around")]
#[test_case(JustifyContent::SpaceEvenly ; "space_evenly")]
fn justify_content_codegen(j: JustifyContent) {
    let node = justify_test(j);
    assert_both_emit(&node, ColorPalette::Set1);
    assert_labels_present(&node, ColorPalette::Set1);
}

// ─── Parameterized tests: align-items ────────────────────────────────────────

#[test_case(AlignItems::FlexStart ; "flex_start")]
#[test_case(AlignItems::FlexEnd ; "flex_end")]
#[test_case(AlignItems::Center ; "center")]
#[test_case(AlignItems::Baseline ; "baseline")]
#[test_case(AlignItems::Stretch ; "stretch")]
fn align_items_codegen(a: AlignItems) {
    let node = align_items_test(a);
    assert_both_emit(&node, ColorPalette::Tableau10);
    assert_labels_present(&node, ColorPalette::Tableau10);
}

// ─── Parameterized tests: flex-wrap ──────────────────────────────────────────

#[test_case(FlexWrap::NoWrap ; "nowrap")]
#[test_case(FlexWrap::Wrap ; "wrap")]
#[test_case(FlexWrap::WrapReverse ; "wrap_reverse")]
fn flex_wrap_codegen(w: FlexWrap) {
    let node = wrap_test(w);
    assert_both_emit(&node, ColorPalette::Pastel1);
}

// ─── Parameterized tests: align-content ──────────────────────────────────────

#[test_case(AlignContent::FlexStart ; "flex_start")]
#[test_case(AlignContent::FlexEnd ; "flex_end")]
#[test_case(AlignContent::Center ; "center")]
#[test_case(AlignContent::SpaceBetween ; "space_between")]
#[test_case(AlignContent::Stretch ; "stretch")]
fn align_content_codegen(ac: AlignContent) {
    let node = align_content_test(ac);
    assert_both_emit(&node, ColorPalette::Pastel1);
}

// ─── Parameterized tests: value types ────────────────────────────────────────

#[test_case(ValueConfig::Px(100.0), ValueConfig::Px(50.0) ; "px_px")]
#[test_case(ValueConfig::Percent(50.0), ValueConfig::Percent(25.0) ; "percent_percent")]
#[test_case(ValueConfig::Vw(80.0), ValueConfig::Vh(60.0) ; "vw_vh")]
#[test_case(ValueConfig::Auto, ValueConfig::Px(100.0) ; "auto_px")]
#[test_case(ValueConfig::Px(0.0), ValueConfig::Auto ; "zero_auto")]
fn value_types_codegen(w: ValueConfig, h: ValueConfig) {
    let node = value_leaf(w, h);
    let html = emit_html_css(&node, ColorPalette::Pastel1).unwrap();
    let bevy = emit_bevy_code(&node, ColorPalette::Pastel1).unwrap();

    // Non-auto values must appear in both outputs
    match w {
        ValueConfig::Px(n) if n != 0.0 => {
            assert!(html.contains("width:"), "HTML missing width for Px({n})");
            assert!(bevy.contains("width:"), "Bevy missing width for Px({n})");
        }
        ValueConfig::Percent(n) => {
            assert!(
                html.contains(&format!("{n:.1}%")),
                "HTML missing percent width"
            );
            assert!(
                bevy.contains(&format!("Val::Percent({n:.1})")),
                "Bevy missing Val::Percent"
            );
        }
        ValueConfig::Vw(n) => {
            assert!(html.contains(&format!("{n:.1}vw")), "HTML missing vw width");
            assert!(
                bevy.contains(&format!("Val::Vw({n:.1})")),
                "Bevy missing Val::Vw"
            );
        }
        _ => {}
    }
    match h {
        ValueConfig::Vh(n) => {
            assert!(
                html.contains(&format!("{n:.1}vh")),
                "HTML missing vh height"
            );
            assert!(
                bevy.contains(&format!("Val::Vh({n:.1})")),
                "Bevy missing Val::Vh"
            );
        }
        _ => {}
    }
}

// ─── Parameterized tests: grow / shrink ──────────────────────────────────────

#[test_case(0.0, 1.0 ; "defaults")]
#[test_case(1.0, 1.0 ; "grow_1")]
#[test_case(2.5, 0.0 ; "grow_2_5_shrink_0")]
#[test_case(0.0, 0.0 ; "no_shrink")]
#[test_case(3.0, 2.0 ; "both_nondefault")]
fn grow_shrink_codegen(grow: f32, shrink: f32) {
    let node = grow_shrink(grow, shrink);
    let html = emit_html_css(&node, ColorPalette::Pastel1).unwrap();
    let bevy = emit_bevy_code(&node, ColorPalette::Pastel1).unwrap();
    let iced = emit_iced(&node, ColorPalette::Pastel1).unwrap();

    if grow != 0.0 {
        assert!(
            html.contains("flex-grow:"),
            "HTML missing flex-grow for {grow}"
        );
        assert!(
            bevy.contains("flex_grow:"),
            "Bevy missing flex_grow for {grow}"
        );
        // Iced maps flex-grow to Length::Fill or Length::FillPortion
        assert!(
            iced.contains("Length::Fill"),
            "Iced missing Length::Fill for flex-grow {grow}"
        );
    }
    if shrink != 1.0 {
        assert!(
            html.contains("flex-shrink:"),
            "HTML missing flex-shrink for {shrink}"
        );
        assert!(
            bevy.contains("flex_shrink:"),
            "Bevy missing flex_shrink for {shrink}"
        );
        // Iced has no flex-shrink — should emit a comment
        assert!(
            iced.contains("flex-shrink"),
            "Iced missing flex-shrink comment for {shrink}"
        );
    }
}

// ─── Parameterized tests: align-self ─────────────────────────────────────────

#[test_case(AlignSelf::Auto ; "auto")]
#[test_case(AlignSelf::FlexStart ; "flex_start")]
#[test_case(AlignSelf::Center ; "center")]
#[test_case(AlignSelf::Stretch ; "stretch")]
fn align_self_codegen(align: AlignSelf) {
    let node = align_self_child(align);
    let html = emit_html_css(&node, ColorPalette::Pastel1).unwrap();
    let bevy = emit_bevy_code(&node, ColorPalette::Pastel1).unwrap();
    let iced = emit_iced(&node, ColorPalette::Pastel1).unwrap();

    if align != AlignSelf::Auto {
        assert!(
            html.contains("align-self:"),
            "HTML missing align-self for {align:?}"
        );
        assert!(
            bevy.contains("align_self:"),
            "Bevy missing align_self for {align:?}"
        );
        assert!(
            iced.contains("align-self"),
            "Iced missing align-self comment for {align:?}"
        );
    }
}

// ─── Parameterized tests: flex-basis ─────────────────────────────────────────

#[test_case(ValueConfig::Auto ; "auto")]
#[test_case(ValueConfig::Px(100.0) ; "px_100")]
#[test_case(ValueConfig::Percent(50.0) ; "percent_50")]
fn flex_basis_codegen(basis: ValueConfig) {
    let node = with_flex_basis(basis);
    let html = emit_html_css(&node, ColorPalette::Pastel1).unwrap();
    let bevy = emit_bevy_code(&node, ColorPalette::Pastel1).unwrap();
    let iced = emit_iced(&node, ColorPalette::Pastel1).unwrap();

    if !matches!(basis, ValueConfig::Auto) {
        assert!(html.contains("flex-basis:"), "HTML missing flex-basis");
        assert!(bevy.contains("flex_basis:"), "Bevy missing flex_basis");
        assert!(
            iced.contains("flex-basis"),
            "Iced missing flex-basis comment"
        );
    }
}

// ─── Parameterized tests: gap values ─────────────────────────────────────────

#[test_case(ValueConfig::Px(0.0), ValueConfig::Px(0.0) ; "no_gaps")]
#[test_case(ValueConfig::Px(16.0), ValueConfig::Px(16.0) ; "uniform_16")]
#[test_case(ValueConfig::Px(8.0), ValueConfig::Percent(5.0) ; "mixed_types")]
fn gap_codegen(row: ValueConfig, col: ValueConfig) {
    let node = with_gaps(row, col);
    assert_both_emit(&node, ColorPalette::Pastel1);
}

// ─── Parameterized tests: color palettes ─────────────────────────────────────

#[test_case(ColorPalette::Pastel1 ; "pastel1")]
#[test_case(ColorPalette::Set1 ; "set1")]
#[test_case(ColorPalette::Tableau10 ; "tableau10")]
#[test_case(ColorPalette::Dark2 ; "dark2")]
#[test_case(ColorPalette::Category10 ; "category10")]
#[test_case(ColorPalette::Paired ; "paired")]
fn palette_codegen(palette: ColorPalette) {
    let node = wide_flat(5);
    assert_both_emit(&node, palette);
    assert_labels_present(&node, palette);
}

// ─── Parameterized tests: tree shapes ────────────────────────────────────────

fn tree_shapes() -> Vec<(&'static str, NodeConfig)> {
    vec![
        ("single_leaf", single_leaf()),
        ("wide_2", wide_flat(2)),
        ("wide_10", wide_flat(10)),
        ("wide_20", wide_flat(20)),
        ("deep_1", deep_chain(1)),
        ("deep_3", deep_chain(3)),
        ("deep_6", deep_chain(6)),
        ("nested_mixed", nested_mixed()),
    ]
}

#[test]
fn tree_shape_codegen() {
    for (name, node) in tree_shapes() {
        assert_both_emit(&node, ColorPalette::Pastel1);
        assert_labels_present(&node, ColorPalette::Pastel1);
        // Verify consistency too
        assert_property_consistency(&node, ColorPalette::Pastel1);
        eprintln!("  tree_shape OK: {name}");
    }
}

// ─── Parameterized tests: visibility ─────────────────────────────────────────

#[test]
fn visibility_codegen() {
    for node in [hidden_child(), all_hidden()] {
        let html = emit_html_css(&node, ColorPalette::Pastel1).unwrap();
        let bevy = emit_bevy_code(&node, ColorPalette::Pastel1).unwrap();
        let iced = emit_iced(&node, ColorPalette::Pastel1).unwrap();
        assert!(
            html.contains("visibility: hidden"),
            "HTML missing visibility: hidden"
        );
        assert!(
            bevy.contains("Visibility::Hidden"),
            "Bevy missing Visibility::Hidden"
        );
        assert!(
            iced.contains("// NOTE: hidden"),
            "Iced missing hidden comment"
        );
    }
}

// ─── Tests: ordering ─────────────────────────────────────────────────────────

#[test]
fn order_reflected_in_html() {
    let node = ordered_children();
    let html = emit_html_css(&node, ColorPalette::Pastel1).unwrap();
    // CSS order property should be emitted for non-zero
    assert!(
        html.contains("order: 3"),
        "HTML missing order: 3 for node A"
    );
    assert!(
        html.contains("order: -1"),
        "HTML missing order: -1 for node B"
    );
    // The HTML div order follows sort-by-order: B(-1), C(0), A(3)
    let pos_b = html.find(">B<").expect("missing B div");
    let pos_c = html.find(">C<").expect("missing C div");
    let pos_a = html.find(">A<").expect("missing A div");
    assert!(pos_b < pos_c, "B should appear before C in HTML");
    assert!(pos_c < pos_a, "C should appear before A in HTML");
}

#[test]
fn order_reflected_in_bevy() {
    let node = ordered_children();
    let bevy = emit_bevy_code(&node, ColorPalette::Pastel1).unwrap();
    // Bevy sorts children by order; order itself is a comment
    assert!(
        bevy.contains("order: 3"),
        "Bevy missing order comment for 3"
    );
    assert!(
        bevy.contains("order: -1"),
        "Bevy missing order comment for -1"
    );
    // Text spawn order should be B, C, A
    let pos_b = bevy.find("Text::new(\"B\")").expect("missing B text");
    let pos_c = bevy.find("Text::new(\"C\")").expect("missing C text");
    let pos_a = bevy.find("Text::new(\"A\")").expect("missing A text");
    assert!(pos_b < pos_c, "B should appear before C in Bevy output");
    assert!(pos_c < pos_a, "C should appear before A in Bevy output");
}

#[test]
fn order_reflected_in_iced() {
    let node = ordered_children();
    let iced = emit_iced(&node, ColorPalette::Pastel1).unwrap();
    // Iced sorts children by order and emits comments for non-zero
    assert!(
        iced.contains("order: 3"),
        "Iced missing order comment for 3"
    );
    assert!(
        iced.contains("order: -1"),
        "Iced missing order comment for -1"
    );
    // text() call order should be B, C, A
    let pos_b = iced.find("text(\"B\")").expect("missing B text");
    let pos_c = iced.find("text(\"C\")").expect("missing C text");
    let pos_a = iced.find("text(\"A\")").expect("missing A text");
    assert!(pos_b < pos_c, "B should appear before C in Iced output");
    assert!(pos_c < pos_a, "C should appear before A in Iced output");
}

// ─── Tests: padding and margin ───────────────────────────────────────────────

#[test]
fn padding_and_margin_emitted() {
    let node = with_padding_margin();
    let html = emit_html_css(&node, ColorPalette::Pastel1).unwrap();
    let bevy = emit_bevy_code(&node, ColorPalette::Pastel1).unwrap();
    let iced = emit_iced(&node, ColorPalette::Pastel1).unwrap();
    assert!(html.contains("padding: 20.0px"), "HTML missing padding");
    assert!(html.contains("margin: 10.0px"), "HTML missing margin");
    assert!(
        bevy.contains("UiRect::all(Val::Px(20.0))"),
        "Bevy missing padding UiRect"
    );
    assert!(
        bevy.contains("UiRect::all(Val::Px(10.0))"),
        "Bevy missing margin UiRect"
    );
    assert!(iced.contains(".padding(20.0)"), "Iced missing padding");
    assert!(
        iced.contains("// NOTE: margin: 10px"),
        "Iced missing margin comment"
    );
}

// ─── Tests: min/max sizes ────────────────────────────────────────────────────

#[test]
fn min_max_sizes_emitted() {
    let node = min_max_sizes();
    let html = emit_html_css(&node, ColorPalette::Pastel1).unwrap();
    let bevy = emit_bevy_code(&node, ColorPalette::Pastel1).unwrap();
    let iced = emit_iced(&node, ColorPalette::Pastel1).unwrap();
    assert!(html.contains("min-width: 40.0px"), "HTML missing min-width");
    assert!(
        html.contains("max-width: 200.0px"),
        "HTML missing max-width"
    );
    assert!(
        html.contains("min-height: 30.0px"),
        "HTML missing min-height"
    );
    assert!(
        html.contains("max-height: 150.0px"),
        "HTML missing max-height"
    );
    assert!(
        bevy.contains("Val::Px(40.0)"),
        "Bevy missing min_width value"
    );
    assert!(
        bevy.contains("Val::Px(200.0)"),
        "Bevy missing max_width value"
    );
    assert!(
        bevy.contains("Val::Px(30.0)"),
        "Bevy missing min_height value"
    );
    assert!(
        bevy.contains("Val::Px(150.0)"),
        "Bevy missing max_height value"
    );
    // Iced: max_width is supported, min/max height via comments
    assert!(iced.contains(".max_width(200.0)"), "Iced missing max_width");
    assert!(
        iced.contains("min-width: 40px"),
        "Iced missing min-width comment"
    );
    assert!(
        iced.contains("min-height: 30px"),
        "Iced missing min-height comment"
    );
    assert!(
        iced.contains("max-height: 150px"),
        "Iced missing max-height comment"
    );
}

// ─── Tests: template layouts ─────────────────────────────────────────────────

#[test]
fn template_codegen() {
    let templates: Vec<(&str, NodeConfig)> = vec![
        ("holy_grail", templates::holy_grail()),
        ("sidebar_content", templates::sidebar_content()),
        ("card_grid", templates::card_grid()),
        ("nav_bar", templates::nav_bar()),
    ];
    for (name, node) in templates {
        assert_both_emit(&node, ColorPalette::Pastel1);
        assert_labels_present(&node, ColorPalette::Pastel1);
        eprintln!("  template OK: {name}");
    }
}

// ─── Tests: default elision (non-default properties emitted, defaults omitted)

#[test]
fn defaults_elided_in_html() {
    // A simple leaf with defaults should NOT have flex-direction, flex-wrap etc
    let node = single_leaf();
    let html = emit_html_css(&node, ColorPalette::Pastel1).unwrap();
    // The root has flex_wrap: Wrap (non-default for CSS), so that SHOULD appear
    assert!(html.contains("flex-wrap: wrap"), "root wrap should appear");
    // flex-shrink is 1.0 (CSS default), should NOT appear for the leaf
    // Count occurrences: at most the root might have it
    let leaf_section = html.split("node-1").nth(1).unwrap_or("");
    assert!(
        !leaf_section.contains("flex-shrink"),
        "default flex-shrink should not appear for leaf"
    );
}

#[test]
fn defaults_elided_in_bevy() {
    let node = single_leaf();
    let bevy = emit_bevy_code(&node, ColorPalette::Pastel1).unwrap();
    // flex_direction Row is default in Bevy, should not appear for a row container
    // (root is Row by default)
    // Count "FlexDirection" - should only appear if explicitly non-default
    assert!(
        !bevy.contains("FlexDirection::Row"),
        "default FlexDirection::Row should not appear"
    );
}

// ─── Tests: leaf count matches div/text count ────────────────────────────────

#[test_case(1 ; "one_leaf")]
#[test_case(5 ; "five_leaves")]
fn leaf_count_wide_flat(n: usize) {
    let node = wide_flat(n);
    let expected_leaves = n;
    leaf_count_check(&node, expected_leaves);
}

#[test]
fn leaf_count_nested_mixed() {
    leaf_count_check(&nested_mixed(), 4);
}

#[test]
fn leaf_count_holy_grail() {
    leaf_count_check(&templates::holy_grail(), 5);
}

#[test]
fn leaf_count_card_grid() {
    leaf_count_check(&templates::card_grid(), 6);
}

fn leaf_count_check(node: &NodeConfig, expected_leaves: usize) {
    let html = emit_html_css(&node, ColorPalette::Pastel1).unwrap();
    let bevy = emit_bevy_code(&node, ColorPalette::Pastel1).unwrap();
    let iced = emit_iced(&node, ColorPalette::Pastel1).unwrap();

    let text_new_count = bevy.matches("Text::new(").count();
    assert_eq!(
        text_new_count, expected_leaves,
        "Bevy Text::new count mismatch"
    );

    // Count leaf divs: divs that don't contain other divs = divs with </div> on same line
    // Simpler: count "rgb(" in background which only appears on leaves in HTML
    let rgb_count = html.matches("rgb(").count();
    assert_eq!(
        rgb_count, expected_leaves,
        "HTML leaf background rgb() count mismatch"
    );

    // Iced: count text() calls (leaves only) — exclude "text(" in "container::Style" etc.
    let iced_text_count = iced.matches("text(\"").count();
    assert_eq!(
        iced_text_count, expected_leaves,
        "Iced text() count mismatch"
    );
}
