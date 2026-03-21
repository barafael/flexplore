use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

pub const PANEL_WIDTH: f32 = 390.0;
pub const ART_TEXTURE_SIZE: u32 = 128;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Display, EnumIter, Serialize, Deserialize)]
pub enum ValueKind {
    Auto,
    Px,
    Percent,
    Vw,
    Vh,
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum ValueConfig {
    Auto,
    Px(f32),
    Percent(f32),
    Vw(f32),
    Vh(f32),
}

impl ValueConfig {
    pub fn to_val(self) -> Val {
        match self {
            ValueConfig::Auto => Val::Auto,
            ValueConfig::Px(v) => Val::Px(v),
            ValueConfig::Percent(v) => Val::Percent(v),
            ValueConfig::Vw(v) => Val::Vw(v),
            ValueConfig::Vh(v) => Val::Vh(v),
        }
    }
    pub fn kind(&self) -> ValueKind {
        match self {
            ValueConfig::Auto => ValueKind::Auto,
            ValueConfig::Px(_) => ValueKind::Px,
            ValueConfig::Percent(_) => ValueKind::Percent,
            ValueConfig::Vw(_) => ValueKind::Vw,
            ValueConfig::Vh(_) => ValueKind::Vh,
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
    pub fn cast(&self, kind: ValueKind) -> Self {
        let n = self.num().unwrap_or(100.0);
        match kind {
            ValueKind::Px => ValueConfig::Px(n),
            ValueKind::Percent => ValueConfig::Percent(n),
            ValueKind::Vw => ValueConfig::Vw(n),
            ValueKind::Vh => ValueConfig::Vh(n),
            ValueKind::Auto => ValueConfig::Auto,
        }
    }
    pub fn display_short(&self) -> String {
        match self {
            ValueConfig::Auto => "auto".into(),
            ValueConfig::Px(n) => format!("{n:.0}px"),
            ValueConfig::Percent(n) => format!("{n:.0}%"),
            ValueConfig::Vw(n) => format!("{n:.0}vw"),
            ValueConfig::Vh(n) => format!("{n:.0}vh"),
        }
    }
}

// ─── Node config (recursive tree) ────────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize)]
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
    pub order: i32,
    #[serde(default = "default_true")]
    pub visible: bool,
    pub children: Vec<NodeConfig>,
}

fn default_true() -> bool {
    true
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
            order: 0,
            visible: true,
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
            order: 0,
            visible: true,
            children: vec![],
        }
    }
}

impl NodeConfig {
    pub fn get(&self, path: &[usize]) -> Option<&NodeConfig> {
        if path.is_empty() {
            Some(self)
        } else {
            self.children.get(path[0])?.get(&path[1..])
        }
    }

    pub fn get_mut(&mut self, path: &[usize]) -> Option<&mut NodeConfig> {
        if path.is_empty() {
            Some(self)
        } else {
            self.children.get_mut(path[0])?.get_mut(&path[1..])
        }
    }

    pub fn count_leaves(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            self.children.iter().map(|c| c.count_leaves()).sum()
        }
    }

    pub fn text_scale(&self) -> f32 {
        fn approx_px(v: &ValueConfig) -> Option<f32> {
            match v {
                ValueConfig::Px(n) => Some(*n),
                ValueConfig::Percent(n) => Some(n / 100.0 * 600.0),
                ValueConfig::Vw(n) | ValueConfig::Vh(n) => Some(n / 100.0 * 800.0),
                ValueConfig::Auto => None,
            }
        }
        let w = approx_px(&self.width);
        let h = approx_px(&self.height);
        let min_dim = match (w, h) {
            (Some(w), Some(h)) => w.min(h),
            (Some(v), None) | (None, Some(v)) => v,
            (None, None) => 120.0,
        };
        (min_dim / 120.0).clamp(0.25, 1.5)
    }

    pub fn info(&self) -> String {
        let g = format_float(self.flex_grow);
        let s = format_float(self.flex_shrink);
        let basis = self.flex_basis.kind();
        let w = self.width.display_short();
        let h = self.height.display_short();
        let o = self.order;
        let vis = if self.visible { "" } else { " [hidden]" };
        format!("g:{g} s:{s}\nbasis:{basis} w:{w} h:{h}\norder:{o}{vis}")
    }
}

// ─── Background mode + art style ─────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum BackgroundMode {
    Pastel,
    RandomArt,
}

#[derive(Clone, Copy, PartialEq, Debug, Display, EnumIter, Serialize, Deserialize)]
pub enum ArtStyle {
    #[strum(serialize = "Expr Tree")]
    ExprTree,
    Voronoi,
    #[strum(serialize = "Flow Field")]
    FlowField,
    Crackle,
    #[strum(serialize = "Op Art")]
    OpArt,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Display, EnumIter, Serialize, Deserialize)]
pub enum Theme {
    Latte,
    #[strum(serialize = "Frappé")]
    Frappe,
    Macchiato,
    Mocha,
}

impl Theme {
    pub fn is_light(self) -> bool {
        self == Theme::Latte
    }
}

fn deserialize_theme<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Theme, D::Error> {
    use serde::Deserialize;
    let s = String::deserialize(d)?;
    Ok(match s.as_str() {
        "Dark" | "Mocha" => Theme::Mocha,
        "Light" | "Latte" => Theme::Latte,
        "Frappe" | "Frappé" => Theme::Frappe,
        "Macchiato" => Theme::Macchiato,
        _ => Theme::Mocha,
    })
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Display, EnumIter, Serialize, Deserialize)]
pub enum ColorPalette {
    Pastel1,
    Pastel2,
    Set1,
    Set2,
    Set3,
    Tableau10,
    Category10,
    Accent,
    Dark2,
    Paired,
}

// ─── Serialisable layout + palette bundle ────────────────────────────────────

/// Self-contained input for codegen: the node tree plus the colour palette.
/// Uses `#[serde(flatten)]` so existing JSON files without a `palette` field
/// still deserialise (defaulting to [`ColorPalette::Pastel1`]).
#[derive(Clone, Serialize, Deserialize)]
pub struct LayoutInput {
    #[serde(flatten)]
    pub node: NodeConfig,
    #[serde(default = "default_palette")]
    pub palette: ColorPalette,
}

// ─── Main resource ────────────────────────────────────────────────────────────

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct FlexConfig {
    pub root: NodeConfig,
    #[serde(skip)]
    selected: Vec<usize>,
    pub bg_mode: BackgroundMode,
    pub art_style: ArtStyle,
    pub art_seed: u64,
    pub art_depth: u32,
    pub art_anim: f32,
    #[serde(deserialize_with = "deserialize_theme")]
    pub theme: Theme,
    #[serde(default = "default_palette")]
    pub palette: ColorPalette,
    #[serde(skip)]
    needs_rebuild: bool,
}

fn default_palette() -> ColorPalette {
    ColorPalette::Pastel1
}

impl FlexConfig {
    pub fn selected(&self) -> &[usize] {
        &self.selected
    }

    /// Set the selected node path and mark for rebuild.
    pub fn select(&mut self, path: Vec<usize>) {
        self.selected = path;
        self.needs_rebuild = true;
    }

    /// Deselect towards root until the path is valid.
    pub fn sanitize_selection(&mut self) {
        while self.root.get(&self.selected).is_none() && !self.selected.is_empty() {
            self.selected.pop();
        }
    }

    pub fn request_rebuild(&mut self) {
        self.needs_rebuild = true;
    }

    /// Returns true (and resets the flag) if a rebuild was requested.
    pub fn take_rebuild(&mut self) -> bool {
        std::mem::replace(&mut self.needs_rebuild, false)
    }
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
            theme: Theme::Mocha,
            palette: ColorPalette::Pastel1,
            needs_rebuild: true,
        }
    }
}

fn format_float(v: f32) -> String {
    if (v - v.round()).abs() < 0.005 {
        format!("{}", v as i32)
    } else {
        format!("{v:.1}")
    }
}
