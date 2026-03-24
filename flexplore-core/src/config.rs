use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

// ─── Flex layout enums (Bevy-compatible variant names) ───────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum FlexDirection {
    #[default]
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum FlexWrap {
    NoWrap,
    #[default]
    Wrap,
    WrapReverse,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum JustifyContent {
    #[default]
    Default,
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
    Stretch,
    Start,
    End,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum AlignItems {
    #[default]
    Default,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
    Start,
    End,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum AlignContent {
    #[default]
    Default,
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
    Stretch,
    Start,
    End,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum AlignSelf {
    #[default]
    Auto,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
    Start,
    End,
}

// ─── Value types ─────────────────────────────────────────────────────────────

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
    pub fn is_zero_px(&self) -> bool {
        matches!(self, ValueConfig::Px(v) if *v == 0.0)
    }
}

// ─── Per-side values ────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug, Serialize)]
pub struct Sides {
    pub top: ValueConfig,
    pub right: ValueConfig,
    pub bottom: ValueConfig,
    pub left: ValueConfig,
}

impl Sides {
    pub fn uniform(v: ValueConfig) -> Self {
        Self { top: v, right: v, bottom: v, left: v }
    }

    pub fn zero() -> Self {
        Self::uniform(ValueConfig::Px(0.0))
    }

    pub fn is_uniform(&self) -> bool {
        self.top == self.right && self.right == self.bottom && self.bottom == self.left
    }

    pub fn is_zero(&self) -> bool {
        self.top.is_zero_px() && self.right.is_zero_px() && self.bottom.is_zero_px() && self.left.is_zero_px()
    }

    /// First side value — use when only uniform values are supported.
    pub fn first(&self) -> ValueConfig {
        self.top
    }
}

impl Default for Sides {
    fn default() -> Self {
        Self::zero()
    }
}

impl<'de> Deserialize<'de> for Sides {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Helper {
            PerSide {
                top: ValueConfig,
                right: ValueConfig,
                bottom: ValueConfig,
                left: ValueConfig,
            },
            Uniform(ValueConfig),
        }
        match Helper::deserialize(deserializer)? {
            Helper::PerSide { top, right, bottom, left } => Ok(Sides { top, right, bottom, left }),
            Helper::Uniform(v) => Ok(Sides::uniform(v)),
        }
    }
}

// ─── Per-corner values ──────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug, Serialize)]
pub struct Corners {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl Corners {
    pub fn uniform(v: f32) -> Self {
        Self { top_left: v, top_right: v, bottom_right: v, bottom_left: v }
    }

    pub fn is_uniform(&self) -> bool {
        self.top_left == self.top_right
            && self.top_right == self.bottom_right
            && self.bottom_right == self.bottom_left
    }

    pub fn is_zero(&self) -> bool {
        self.top_left == 0.0
            && self.top_right == 0.0
            && self.bottom_right == 0.0
            && self.bottom_left == 0.0
    }
}

impl Default for Corners {
    fn default() -> Self {
        Self::uniform(0.0)
    }
}

impl<'de> Deserialize<'de> for Corners {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Helper {
            PerCorner {
                top_left: f32,
                top_right: f32,
                bottom_right: f32,
                bottom_left: f32,
            },
            Uniform(f32),
        }
        match Helper::deserialize(deserializer)? {
            Helper::PerCorner { top_left, top_right, bottom_right, bottom_left } => {
                Ok(Corners { top_left, top_right, bottom_right, bottom_left })
            }
            Helper::Uniform(v) => Ok(Corners::uniform(v)),
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
    pub padding: Sides,
    pub margin: Sides,
    #[serde(default)]
    pub border_width: Sides,
    #[serde(default)]
    pub border_radius: Corners,
    pub order: i32,
    #[serde(default = "default_true")]
    pub visible: bool,
    #[serde(default)]
    pub text_content: String,
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
            padding: Sides::uniform(ValueConfig::Px(8.0)),
            margin: Sides::zero(),
            border_width: Sides::zero(),
            border_radius: Corners::uniform(0.0),
            order: 0,
            visible: true,
            text_content: String::new(),
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
            padding: Sides::uniform(ValueConfig::Px(12.0)),
            margin: Sides::zero(),
            border_width: Sides::zero(),
            border_radius: Corners::uniform(0.0),
            order: 0,
            visible: true,
            text_content: String::new(),
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

    /// The text to display in the node. Uses label if text_content is empty.
    pub fn display_text(&self) -> &str {
        if self.text_content.is_empty() {
            &self.label
        } else {
            &self.text_content
        }
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

pub fn default_palette() -> ColorPalette {
    ColorPalette::Pastel1
}

pub fn format_float(v: f32) -> String {
    if (v - v.round()).abs() < 0.005 {
        format!("{}", v as i32)
    } else {
        format!("{v:.1}")
    }
}

// ─── Optional Bevy conversions ───────────────────────────────────────────────

#[cfg(feature = "bevy")]
mod bevy_bridge {
    use super::*;

    macro_rules! into_bevy {
        ($core:ty, $bevy:ty, [$($variant:ident),+ $(,)?]) => {
            #[allow(clippy::from_over_into)]
            impl Into<$bevy> for $core {
                fn into(self) -> $bevy {
                    match self {
                        $( Self::$variant => <$bevy>::$variant, )+
                    }
                }
            }
        };
    }

    into_bevy!(
        FlexDirection,
        bevy::prelude::FlexDirection,
        [Row, Column, RowReverse, ColumnReverse]
    );
    into_bevy!(
        FlexWrap,
        bevy::prelude::FlexWrap,
        [NoWrap, Wrap, WrapReverse]
    );
    into_bevy!(
        JustifyContent,
        bevy::prelude::JustifyContent,
        [
            Default,
            FlexStart,
            FlexEnd,
            Center,
            SpaceBetween,
            SpaceAround,
            SpaceEvenly,
            Stretch,
            Start,
            End
        ]
    );
    into_bevy!(
        AlignItems,
        bevy::prelude::AlignItems,
        [
            Default, FlexStart, FlexEnd, Center, Baseline, Stretch, Start, End
        ]
    );
    into_bevy!(
        AlignContent,
        bevy::prelude::AlignContent,
        [
            Default,
            FlexStart,
            FlexEnd,
            Center,
            SpaceBetween,
            SpaceAround,
            SpaceEvenly,
            Stretch,
            Start,
            End
        ]
    );
    into_bevy!(
        AlignSelf,
        bevy::prelude::AlignSelf,
        [
            Auto, FlexStart, FlexEnd, Center, Baseline, Stretch, Start, End
        ]
    );

    impl ValueConfig {
        pub fn to_bevy_val(self) -> bevy::prelude::Val {
            match self {
                ValueConfig::Auto => bevy::prelude::Val::Auto,
                ValueConfig::Px(n) => bevy::prelude::Val::Px(n),
                ValueConfig::Percent(n) => bevy::prelude::Val::Percent(n),
                ValueConfig::Vw(n) => bevy::prelude::Val::Vw(n),
                ValueConfig::Vh(n) => bevy::prelude::Val::Vh(n),
            }
        }
    }

    impl Sides {
        pub fn to_bevy_ui_rect(&self) -> bevy::prelude::UiRect {
            bevy::prelude::UiRect {
                top: self.top.to_bevy_val(),
                right: self.right.to_bevy_val(),
                bottom: self.bottom.to_bevy_val(),
                left: self.left.to_bevy_val(),
            }
        }
    }

    impl Corners {
        pub fn to_bevy_border_radius(&self) -> bevy::prelude::BorderRadius {
            bevy::prelude::BorderRadius {
                top_left: bevy::prelude::Val::Px(self.top_left),
                top_right: bevy::prelude::Val::Px(self.top_right),
                bottom_right: bevy::prelude::Val::Px(self.bottom_right),
                bottom_left: bevy::prelude::Val::Px(self.bottom_left),
            }
        }
    }
}
