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

#[derive(Clone, Deserialize)]
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

#[derive(Clone, Copy, PartialEq, Debug)]
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
