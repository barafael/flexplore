use bevy::prelude::*;

pub const PANEL_WIDTH: f32 = 390.0;
pub const ART_TEXTURE_SIZE: u32 = 128;

#[derive(Clone, PartialEq, Debug)]
pub enum ValueConfig {
    Auto,
    Px(f32),
    Percent(f32),
    Vw(f32),
    Vh(f32),
}

impl ValueConfig {
    pub fn to_val(&self) -> Val {
        match self {
            ValueConfig::Auto => Val::Auto,
            ValueConfig::Px(v) => Val::Px(*v),
            ValueConfig::Percent(v) => Val::Percent(*v),
            ValueConfig::Vw(v) => Val::Vw(*v),
            ValueConfig::Vh(v) => Val::Vh(*v),
        }
    }
    pub fn variant(&self) -> &'static str {
        match self {
            ValueConfig::Auto => "Auto",
            ValueConfig::Px(_) => "Px",
            ValueConfig::Percent(_) => "Percent",
            ValueConfig::Vw(_) => "Vw",
            ValueConfig::Vh(_) => "Vh",
        }
    }
    pub fn num(&self) -> Option<f32> {
        match self {
            ValueConfig::Auto => None,
            ValueConfig::Px(v)
            | ValueConfig::Percent(v)
            | ValueConfig::Vw(v)
            | ValueConfig::Vh(v) => Some(*v),
        }
    }
    pub fn set_num(&mut self, n: f32) {
        match self {
            ValueConfig::Px(v)
            | ValueConfig::Percent(v)
            | ValueConfig::Vw(v)
            | ValueConfig::Vh(v) => *v = n,
            _ => {}
        }
    }
    pub fn cast(&self, variant: &str) -> Self {
        let n = self.num().unwrap_or(100.0);
        match variant {
            "Px" => ValueConfig::Px(n),
            "Percent" => ValueConfig::Percent(n),
            "Vw" => ValueConfig::Vw(n),
            "Vh" => ValueConfig::Vh(n),
            _ => ValueConfig::Auto,
        }
    }
}

// ─── Node config (recursive tree) ────────────────────────────────────────────

#[derive(Clone)]
pub struct NodeConfig {
    pub label: String,
    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub align_content: AlignContent,
    pub row_gap: ValueConfig,
    pub column_gap: ValueConfig,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: ValueConfig,
    pub align_self: AlignSelf,
    pub width: ValueConfig,
    pub height: ValueConfig,
    pub min_width: ValueConfig,
    pub min_height: ValueConfig,
    pub max_width: ValueConfig,
    pub max_height: ValueConfig,
    pub padding: ValueConfig,
    pub margin: ValueConfig,
    pub children: Vec<NodeConfig>,
}

impl NodeConfig {
    pub fn new_leaf(label: impl Into<String>, w: f32, h: f32) -> Self {
        Self {
            label: label.into(),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            align_content: AlignContent::FlexStart,
            row_gap: ValueConfig::Px(4.0),
            column_gap: ValueConfig::Px(4.0),
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: ValueConfig::Auto,
            align_self: AlignSelf::Auto,
            width: ValueConfig::Px(w),
            height: ValueConfig::Px(h),
            min_width: ValueConfig::Auto,
            min_height: ValueConfig::Auto,
            max_width: ValueConfig::Auto,
            max_height: ValueConfig::Auto,
            padding: ValueConfig::Px(8.0),
            margin: ValueConfig::Px(0.0),
            children: vec![],
        }
    }

    pub fn new_container(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            align_content: AlignContent::FlexStart,
            row_gap: ValueConfig::Px(8.0),
            column_gap: ValueConfig::Px(8.0),
            flex_grow: 1.0,
            flex_shrink: 1.0,
            flex_basis: ValueConfig::Auto,
            align_self: AlignSelf::Auto,
            width: ValueConfig::Percent(100.0),
            height: ValueConfig::Auto,
            min_width: ValueConfig::Auto,
            min_height: ValueConfig::Px(0.0),
            max_width: ValueConfig::Auto,
            max_height: ValueConfig::Auto,
            padding: ValueConfig::Px(12.0),
            margin: ValueConfig::Px(0.0),
            children: vec![],
        }
    }
}

pub fn get_node<'a>(root: &'a NodeConfig, path: &[usize]) -> &'a NodeConfig {
    if path.is_empty() {
        root
    } else {
        get_node(&root.children[path[0]], &path[1..])
    }
}

pub fn get_node_mut<'a>(root: &'a mut NodeConfig, path: &[usize]) -> &'a mut NodeConfig {
    if path.is_empty() {
        root
    } else {
        get_node_mut(&mut root.children[path[0]], &path[1..])
    }
}

pub fn path_valid(root: &NodeConfig, path: &[usize]) -> bool {
    if path.is_empty() {
        return true;
    }
    if path[0] >= root.children.len() {
        return false;
    }
    path_valid(&root.children[path[0]], &path[1..])
}

pub fn count_leaves(node: &NodeConfig) -> usize {
    if node.children.is_empty() {
        1
    } else {
        node.children.iter().map(count_leaves).sum()
    }
}

// ─── Background mode + art style ─────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug)]
pub enum BackgroundMode {
    Pastel,
    RandomArt,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ArtStyle {
    ExprTree,
    Voronoi,
    FlowField,
    Crackle,
    OpArt,
}

impl ArtStyle {
    pub const ALL: &'static [(&'static str, ArtStyle)] = &[
        ("Expr Tree", ArtStyle::ExprTree),
        ("Voronoi", ArtStyle::Voronoi),
        ("Flow Field", ArtStyle::FlowField),
        ("Crackle", ArtStyle::Crackle),
        ("Op Art", ArtStyle::OpArt),
    ];
}

// ─── Main resource ────────────────────────────────────────────────────────────

#[derive(Resource, Clone)]
pub struct FlexConfig {
    pub root: NodeConfig,
    pub selected: Vec<usize>,
    pub bg_mode: BackgroundMode,
    pub art_style: ArtStyle,
    pub art_seed: u64,
    pub art_depth: u32,
    pub art_anim: f32,
    pub needs_rebuild: bool,
}

impl Default for FlexConfig {
    fn default() -> Self {
        let mut root = NodeConfig::new_container("root");
        root.min_height = ValueConfig::Px(200.0);
        root.children = vec![
            NodeConfig::new_leaf("A", 80.0, 80.0),
            NodeConfig::new_leaf("B", 120.0, 100.0),
            NodeConfig::new_leaf("C", 60.0, 60.0),
            NodeConfig::new_leaf("D", 100.0, 80.0),
        ];
        Self {
            root,
            selected: vec![],
            bg_mode: BackgroundMode::Pastel,
            art_style: ArtStyle::ExprTree,
            art_seed: 137,
            art_depth: 5,
            art_anim: 0.0,
            needs_rebuild: true,
        }
    }
}

// ─── Display helpers ──────────────────────────────────────────────────────────

pub fn text_scale(node: &NodeConfig) -> f32 {
    fn approx_px(v: &ValueConfig) -> Option<f32> {
        match v {
            ValueConfig::Px(n) => Some(*n),
            ValueConfig::Percent(n) => Some(n / 100.0 * 600.0),
            ValueConfig::Vw(n) | ValueConfig::Vh(n) => Some(n / 100.0 * 800.0),
            ValueConfig::Auto => None,
        }
    }
    let w = approx_px(&node.width);
    let h = approx_px(&node.height);
    let min_dim = match (w, h) {
        (Some(w), Some(h)) => w.min(h),
        (Some(v), None) | (None, Some(v)) => v,
        (None, None) => 80.0,
    };
    (min_dim / 80.0).clamp(0.25, 2.0)
}

pub fn node_info(node: &NodeConfig) -> String {
    format!(
        "g:{} s:{}\nbasis:{} w:{} h:{}",
        format_float(node.flex_grow),
        format_float(node.flex_shrink),
        node.flex_basis.variant(),
        format_value(&node.width),
        format_value(&node.height)
    )
}

pub fn format_float(v: f32) -> String {
    if (v - v.round()).abs() < 0.005 {
        format!("{}", v as i32)
    } else {
        format!("{v:.1}")
    }
}

pub fn format_value(v: &ValueConfig) -> String {
    match v {
        ValueConfig::Auto => "auto".into(),
        ValueConfig::Px(n) => format!("{n:.0}px"),
        ValueConfig::Percent(n) => format!("{n:.0}%"),
        ValueConfig::Vw(n) => format!("{n:.0}vw"),
        ValueConfig::Vh(n) => format!("{n:.0}vh"),
    }
}
