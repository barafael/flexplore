//! Minimal config types that deserialize from flexplore's `input.json`.
//! These mirror the Bevy-derived enums in `flexplore::config` but use plain
//! serde without a Bevy dependency.

#![allow(dead_code)]

use serde::Deserialize;

#[derive(Clone, Copy, PartialEq, Debug, Deserialize)]
pub enum ValueConfig {
    Auto,
    Px(f32),
    Percent(f32),
    Vw(f32),
    Vh(f32),
}

#[derive(Clone, Copy, PartialEq, Debug, Deserialize)]
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

#[derive(Clone, Copy, PartialEq, Debug, Deserialize)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

#[derive(Clone, Copy, PartialEq, Debug, Deserialize)]
pub enum JustifyContent {
    Default,
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
    Start,
    End,
    Stretch,
}

#[derive(Clone, Copy, PartialEq, Debug, Deserialize)]
pub enum AlignItems {
    Default,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
    Start,
    End,
}

#[derive(Clone, Copy, PartialEq, Debug, Deserialize)]
pub enum AlignContent {
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

#[derive(Clone, Copy, PartialEq, Debug, Deserialize)]
pub enum AlignSelf {
    Auto,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
    Start,
    End,
}

#[derive(Clone, Copy, PartialEq, Debug, Default, Deserialize)]
pub enum DisplayMode {
    #[default]
    Flex,
    Grid,
}

#[derive(Clone, Copy, PartialEq, Debug, Default, Deserialize)]
pub enum GridAutoFlow {
    #[default]
    Row,
    Column,
    RowDense,
    ColumnDense,
}

#[derive(Clone, Copy, PartialEq, Debug, Deserialize)]
pub enum GridTrackSize {
    Auto,
    Px(f32),
    Percent(f32),
    Fr(f32),
    MinContent,
    MaxContent,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Deserialize)]
pub enum GridPlacement {
    #[default]
    Auto,
    Start(i16),
    Span(u16),
    StartSpan(i16, u16),
}

#[derive(Clone, Copy, PartialEq, Debug, Deserialize)]
pub struct Sides {
    pub top: ValueConfig,
    pub right: ValueConfig,
    pub bottom: ValueConfig,
    pub left: ValueConfig,
}

impl Default for Sides {
    fn default() -> Self {
        Self {
            top: ValueConfig::Px(0.0),
            right: ValueConfig::Px(0.0),
            bottom: ValueConfig::Px(0.0),
            left: ValueConfig::Px(0.0),
        }
    }
}

impl Sides {
    pub fn first(&self) -> ValueConfig {
        self.top
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default, Deserialize)]
pub struct Corners {
    #[serde(default)]
    pub top_left: f32,
    #[serde(default)]
    pub top_right: f32,
    #[serde(default)]
    pub bottom_right: f32,
    #[serde(default)]
    pub bottom_left: f32,
}

#[derive(Clone, Deserialize)]
pub struct NodeConfig {
    pub label: String,
    #[serde(default)]
    pub display_mode: DisplayMode,
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
    #[serde(default)]
    pub grid_column: GridPlacement,
    #[serde(default)]
    pub grid_row: GridPlacement,
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

#[derive(Clone, Copy, PartialEq, Debug, Deserialize)]
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

fn default_palette() -> ColorPalette {
    ColorPalette::Pastel1
}

/// Top-level JSON structure: a NodeConfig plus an optional palette field.
#[derive(Clone, Deserialize)]
pub struct LayoutInput {
    #[serde(flatten)]
    pub node: NodeConfig,
    #[serde(default = "default_palette")]
    pub palette: ColorPalette,
}
