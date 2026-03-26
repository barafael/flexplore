use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

// ─── Display mode ────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Display, EnumIter, Serialize, Deserialize)]
pub enum DisplayMode {
    #[default]
    Flex,
    Grid,
}

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

// ─── Grid layout types ──────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Display, EnumIter, Serialize, Deserialize)]
pub enum GridAutoFlow {
    #[default]
    Row,
    Column,
    RowDense,
    ColumnDense,
}

#[derive(Clone, Copy, PartialEq, Debug, Display, EnumIter, Serialize, Deserialize)]
pub enum GridTrackSize {
    Auto,
    Px(f32),
    Percent(f32),
    Fr(f32),
    MinContent,
    MaxContent,
}

impl Default for GridTrackSize {
    fn default() -> Self {
        GridTrackSize::Fr(1.0)
    }
}

impl GridTrackSize {
    pub fn display_short(&self) -> String {
        match self {
            GridTrackSize::Auto => "auto".into(),
            GridTrackSize::Px(n) => format!("{n:.0}px"),
            GridTrackSize::Percent(n) => format!("{n:.0}%"),
            GridTrackSize::Fr(n) => format!("{n:.1}fr"),
            GridTrackSize::MinContent => "min-content".into(),
            GridTrackSize::MaxContent => "max-content".into(),
        }
    }

    pub fn kind(&self) -> GridTrackKind {
        match self {
            GridTrackSize::Auto => GridTrackKind::Auto,
            GridTrackSize::Px(_) => GridTrackKind::Px,
            GridTrackSize::Percent(_) => GridTrackKind::Percent,
            GridTrackSize::Fr(_) => GridTrackKind::Fr,
            GridTrackSize::MinContent => GridTrackKind::MinContent,
            GridTrackSize::MaxContent => GridTrackKind::MaxContent,
        }
    }

    pub fn num(&self) -> Option<f32> {
        match self {
            GridTrackSize::Px(v) | GridTrackSize::Percent(v) | GridTrackSize::Fr(v) => Some(*v),
            _ => None,
        }
    }

    pub fn set_num(&mut self, n: f32) {
        match self {
            GridTrackSize::Px(v) | GridTrackSize::Percent(v) | GridTrackSize::Fr(v) => *v = n,
            _ => {}
        }
    }

    pub fn cast(kind: GridTrackKind, n: f32) -> Self {
        match kind {
            GridTrackKind::Auto => GridTrackSize::Auto,
            GridTrackKind::Px => GridTrackSize::Px(n),
            GridTrackKind::Percent => GridTrackSize::Percent(n),
            GridTrackKind::Fr => GridTrackSize::Fr(n),
            GridTrackKind::MinContent => GridTrackSize::MinContent,
            GridTrackKind::MaxContent => GridTrackSize::MaxContent,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Display, EnumIter, Serialize, Deserialize)]
pub enum GridTrackKind {
    Auto,
    Px,
    Percent,
    Fr,
    MinContent,
    MaxContent,
}

/// Grid placement for a child item on one axis.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum GridPlacement {
    #[default]
    Auto,
    /// Place at a specific line (1-based, can be negative).
    Start(i16),
    /// Span a number of tracks.
    Span(u16),
    /// Place at a specific line and span tracks.
    StartSpan(i16, u16),
}

impl GridPlacement {
    pub fn display_short(&self) -> String {
        match self {
            GridPlacement::Auto => "auto".into(),
            GridPlacement::Start(s) => format!("{s}"),
            GridPlacement::Span(n) => format!("span {n}"),
            GridPlacement::StartSpan(s, n) => format!("{s} / span {n}"),
        }
    }
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
    #[serde(default)]
    pub display_mode: DisplayMode,
    // ─ Flex container ─
    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub align_content: AlignContent,
    pub row_gap: ValueConfig,
    pub column_gap: ValueConfig,
    // ─ Flex item ─
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: ValueConfig,
    pub align_self: AlignSelf,
    // ─ Grid container ─
    #[serde(default)]
    pub grid_template_columns: Vec<GridTrackSize>,
    #[serde(default)]
    pub grid_template_rows: Vec<GridTrackSize>,
    #[serde(default)]
    pub grid_auto_columns: Vec<GridTrackSize>,
    #[serde(default)]
    pub grid_auto_rows: Vec<GridTrackSize>,
    #[serde(default)]
    pub grid_auto_flow: GridAutoFlow,
    // ─ Grid item ─
    #[serde(default)]
    pub grid_column: GridPlacement,
    #[serde(default)]
    pub grid_row: GridPlacement,
    // ─ Sizing ─
    pub width: ValueConfig,
    pub height: ValueConfig,
    pub min_width: ValueConfig,
    pub min_height: ValueConfig,
    pub max_width: ValueConfig,
    pub max_height: ValueConfig,
    // ─ Spacing / border ─
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
            display_mode: DisplayMode::Flex,
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
            grid_template_columns: vec![],
            grid_template_rows: vec![],
            grid_auto_columns: vec![],
            grid_auto_rows: vec![],
            grid_auto_flow: GridAutoFlow::Row,
            grid_column: GridPlacement::Auto,
            grid_row: GridPlacement::Auto,
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
            display_mode: DisplayMode::Flex,
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
            grid_template_columns: vec![],
            grid_template_rows: vec![],
            grid_auto_columns: vec![],
            grid_auto_rows: vec![],
            grid_auto_flow: GridAutoFlow::Row,
            grid_column: GridPlacement::Auto,
            grid_row: GridPlacement::Auto,
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

    /// Create a new grid container with the given template columns.
    pub fn new_grid(label: impl Into<String>, cols: Vec<GridTrackSize>) -> Self {
        let mut node = Self::new_container(label);
        node.display_mode = DisplayMode::Grid;
        node.grid_template_columns = cols;
        node.row_gap = ValueConfig::Px(8.0);
        node.column_gap = ValueConfig::Px(8.0);
        node
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

    impl GridTrackSize {
        pub fn to_bevy_grid_track(self) -> bevy::prelude::GridTrack {
            match self {
                GridTrackSize::Auto => bevy::prelude::GridTrack::auto(),
                GridTrackSize::Px(n) => bevy::prelude::GridTrack::px(n),
                GridTrackSize::Percent(n) => bevy::prelude::GridTrack::percent(n),
                GridTrackSize::Fr(n) => bevy::prelude::GridTrack::fr(n),
                GridTrackSize::MinContent => bevy::prelude::GridTrack::min_content(),
                GridTrackSize::MaxContent => bevy::prelude::GridTrack::max_content(),
            }
        }

        pub fn to_bevy_repeated_grid_track(self) -> bevy::prelude::RepeatedGridTrack {
            match self {
                GridTrackSize::Auto => bevy::prelude::RepeatedGridTrack::auto(1),
                GridTrackSize::Px(n) => bevy::prelude::RepeatedGridTrack::px(1, n),
                GridTrackSize::Percent(n) => bevy::prelude::RepeatedGridTrack::percent(1, n),
                GridTrackSize::Fr(n) => bevy::prelude::RepeatedGridTrack::fr(1, n),
                GridTrackSize::MinContent => bevy::prelude::RepeatedGridTrack::min_content(1),
                GridTrackSize::MaxContent => bevy::prelude::RepeatedGridTrack::max_content(1),
            }
        }
    }

    impl GridAutoFlow {
        pub fn to_bevy(self) -> bevy::prelude::GridAutoFlow {
            match self {
                GridAutoFlow::Row => bevy::prelude::GridAutoFlow::Row,
                GridAutoFlow::Column => bevy::prelude::GridAutoFlow::Column,
                GridAutoFlow::RowDense => bevy::prelude::GridAutoFlow::RowDense,
                GridAutoFlow::ColumnDense => bevy::prelude::GridAutoFlow::ColumnDense,
            }
        }
    }

    impl GridPlacement {
        pub fn to_bevy(self) -> bevy::prelude::GridPlacement {
            match self {
                GridPlacement::Auto => bevy::prelude::GridPlacement::default(),
                GridPlacement::Start(s) => bevy::prelude::GridPlacement::start(s),
                GridPlacement::Span(n) => bevy::prelude::GridPlacement::span(n),
                GridPlacement::StartSpan(s, n) => {
                    bevy::prelude::GridPlacement::start_span(s, n)
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_track_display_short() {
        assert_eq!(GridTrackSize::Auto.display_short(), "auto");
        assert_eq!(GridTrackSize::Px(100.0).display_short(), "100px");
        assert_eq!(GridTrackSize::Percent(50.0).display_short(), "50%");
        assert_eq!(GridTrackSize::Fr(1.0).display_short(), "1.0fr");
        assert_eq!(GridTrackSize::MinContent.display_short(), "min-content");
        assert_eq!(GridTrackSize::MaxContent.display_short(), "max-content");
    }

    #[test]
    fn grid_track_kind_roundtrip() {
        let track = GridTrackSize::Px(42.0);
        assert_eq!(track.kind(), GridTrackKind::Px);
        assert_eq!(track.num(), Some(42.0));

        let cast = GridTrackSize::cast(GridTrackKind::Fr, 2.0);
        assert_eq!(cast, GridTrackSize::Fr(2.0));
    }

    #[test]
    fn grid_track_cast_preserves_unitless() {
        let auto = GridTrackSize::cast(GridTrackKind::Auto, 99.0);
        assert_eq!(auto, GridTrackSize::Auto);
        assert_eq!(auto.num(), None);

        let mc = GridTrackSize::cast(GridTrackKind::MinContent, 99.0);
        assert_eq!(mc, GridTrackSize::MinContent);
    }

    #[test]
    fn grid_placement_display_short() {
        assert_eq!(GridPlacement::Auto.display_short(), "auto");
        assert_eq!(GridPlacement::Start(2).display_short(), "2");
        assert_eq!(GridPlacement::Span(3).display_short(), "span 3");
        assert_eq!(GridPlacement::StartSpan(1, 3).display_short(), "1 / span 3");
    }

    #[test]
    fn new_grid_sets_display_mode() {
        let g = NodeConfig::new_grid("test", vec![GridTrackSize::Fr(1.0)]);
        assert_eq!(g.display_mode, DisplayMode::Grid);
        assert_eq!(g.grid_template_columns.len(), 1);
    }

    #[test]
    fn new_leaf_defaults_to_flex() {
        let l = NodeConfig::new_leaf("test", 80.0, 80.0);
        assert_eq!(l.display_mode, DisplayMode::Flex);
        assert!(l.grid_template_columns.is_empty());
    }

    #[test]
    fn grid_placement_default_is_auto() {
        assert_eq!(GridPlacement::default(), GridPlacement::Auto);
    }

    #[test]
    fn display_mode_default_is_flex() {
        assert_eq!(DisplayMode::default(), DisplayMode::Flex);
    }

    #[test]
    fn grid_auto_flow_default_is_row() {
        assert_eq!(GridAutoFlow::default(), GridAutoFlow::Row);
    }

    #[test]
    fn grid_track_set_num() {
        let mut track = GridTrackSize::Px(10.0);
        track.set_num(42.0);
        assert_eq!(track, GridTrackSize::Px(42.0));

        let mut auto = GridTrackSize::Auto;
        auto.set_num(99.0);
        assert_eq!(auto, GridTrackSize::Auto); // no-op for Auto
    }

    #[test]
    fn grid_node_serialization_roundtrip() {
        let mut root = NodeConfig::new_grid(
            "grid",
            vec![GridTrackSize::Fr(1.0), GridTrackSize::Px(200.0)],
        );
        root.grid_auto_flow = GridAutoFlow::ColumnDense;
        root.grid_auto_rows = vec![GridTrackSize::Px(100.0)];
        let mut child = NodeConfig::new_leaf("A", 80.0, 80.0);
        child.grid_column = GridPlacement::StartSpan(1, 2);
        child.grid_row = GridPlacement::Span(3);
        root.children = vec![child];

        let json = serde_json::to_string(&root).unwrap();
        let deser: NodeConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deser.display_mode, DisplayMode::Grid);
        assert_eq!(deser.grid_template_columns.len(), 2);
        assert_eq!(deser.grid_auto_flow, GridAutoFlow::ColumnDense);
        assert_eq!(deser.grid_auto_rows.len(), 1);
        assert_eq!(deser.children[0].grid_column, GridPlacement::StartSpan(1, 2));
        assert_eq!(deser.children[0].grid_row, GridPlacement::Span(3));
    }

    #[test]
    fn old_json_without_grid_fields_deserializes() {
        // Simulates loading a pre-grid JSON file — grid fields should default
        let json = r#"{
            "label": "root",
            "flex_direction": "Row",
            "flex_wrap": "Wrap",
            "justify_content": "FlexStart",
            "align_items": "FlexStart",
            "align_content": "FlexStart",
            "row_gap": {"Px": 8.0},
            "column_gap": {"Px": 8.0},
            "flex_grow": 1.0,
            "flex_shrink": 1.0,
            "flex_basis": "Auto",
            "align_self": "Auto",
            "width": {"Percent": 100.0},
            "height": "Auto",
            "min_width": "Auto",
            "min_height": {"Px": 0.0},
            "max_width": "Auto",
            "max_height": "Auto",
            "padding": {"Px": 12.0},
            "margin": {"Px": 0.0},
            "order": 0,
            "children": []
        }"#;
        let node: NodeConfig = serde_json::from_str(json).unwrap();
        assert_eq!(node.display_mode, DisplayMode::Flex);
        assert!(node.grid_template_columns.is_empty());
        assert_eq!(node.grid_column, GridPlacement::Auto);
    }
}
