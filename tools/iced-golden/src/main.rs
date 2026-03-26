//! Headless-ish Iced renderer for flexplore golden tests.
//!
//! Reads `testdata/{case}/input.json`, renders the layout with Iced,
//! captures a screenshot, and saves `rendered_iced.png`.

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Result};
use clap::Parser;
use iced::{
    Color, Element, Length, Padding, Size, Subscription, Task, Theme,
    widget::{Space, column, container, row, text},
    window,
};

mod config;
use config::{
    AlignItems, AlignSelf, ColorPalette, FlexDirection, FlexWrap, JustifyContent, LayoutInput,
    NodeConfig, ValueConfig,
};

/// Render flexplore golden screenshots with Iced.
#[derive(Parser)]
#[command(name = "iced-golden")]
struct Arguments {
    /// Path to the testdata directory.
    #[arg(default_value = "testdata")]
    testdata: PathBuf,

    /// Only render these test cases (default: all).
    cases: Vec<String>,
}

const VIEWPORT_W: f32 = 400.0;
const VIEWPORT_H: f32 = 300.0;

// --- Application state ---

struct App {
    jobs: Vec<RenderJob>,
    current: usize,
    frames: usize,
}

struct RenderJob {
    name: String,
    node: NodeConfig,
    palette: ColorPalette,
    output_dir: PathBuf,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    Screenshot(window::Screenshot),
}

fn main() -> iced::Result {
    let cli = Arguments::parse();

    let filter: Vec<&str> = cli.cases.iter().map(|s| s.as_str()).collect();

    let jobs = load_jobs(&cli.testdata, &filter).expect("failed to load test jobs");
    if jobs.is_empty() {
        eprintln!("No render jobs found.");
        return Ok(());
    }
    eprintln!(
        "Will render {} case(s) from {}",
        jobs.len(),
        cli.testdata.display()
    );

    iced::application("iced-golden", App::update, App::view)
        .subscription(App::subscription)
        .theme(|_| Theme::Dark)
        .scale_factor(|_| 1.0)
        .window_size(Size::new(VIEWPORT_W, VIEWPORT_H))
        .run_with(move || {
            (
                App {
                    jobs,
                    current: 0,
                    frames: 0,
                },
                Task::none(),
            )
        })
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.frames += 1;
                if self.frames == 4 {
                    return window::get_latest()
                        .and_then(window::screenshot)
                        .map(Message::Screenshot);
                }
                Task::none()
            }
            Message::Screenshot(screenshot) => {
                if let Some(job) = self.jobs.get(self.current) {
                    let path = job.output_dir.join(&job.name).join("rendered_iced.png");
                    save_screenshot(&path, &screenshot);
                }

                self.current += 1;
                self.frames = 0;

                if self.current >= self.jobs.len() {
                    eprintln!("All done!");
                    return window::get_latest().and_then(window::close);
                }

                eprintln!("Rendering: {}", self.jobs[self.current].name);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        if let Some(job) = self.jobs.get(self.current) {
            let mut leaf_idx = 0;
            build_widget(&job.node, &mut leaf_idx, job.palette, true, false, true)
        } else {
            Space::new(Length::Fill, Length::Fill).into()
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.current < self.jobs.len() {
            iced::time::every(Duration::from_millis(50)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }
}

// --- Screenshot saving ---

fn save_screenshot(path: &Path, screenshot: &window::Screenshot) {
    let size = screenshot.size;
    let bytes = screenshot.bytes.clone();

    if let Some(img) = image::RgbaImage::from_raw(size.width, size.height, bytes.to_vec()) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        match img.save(path) {
            Ok(()) => eprintln!("  Saved: {}", path.display()),
            Err(e) => eprintln!("  ERROR saving {}: {e}", path.display()),
        }
    } else {
        eprintln!("  ERROR: could not create image from screenshot bytes");
    }
}

// --- Job loading ---

fn load_jobs(testdata_dir: &Path, filter: &[&str]) -> Result<Vec<RenderJob>> {
    let mut jobs = Vec::new();

    let mut entries: Vec<_> = std::fs::read_dir(testdata_dir)
        .context("cannot read testdata directory")?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();

        if !filter.is_empty() && !filter.iter().any(|f| *f == name) {
            continue;
        }

        let input_path = entry.path().join("input.json");
        if !input_path.exists() {
            continue;
        }

        let json = std::fs::read_to_string(&input_path)
            .with_context(|| format!("failed to read {}", input_path.display()))?;
        let input: LayoutInput = serde_json::from_str(&json)
            .with_context(|| format!("failed to parse {}", input_path.display()))?;

        jobs.push(RenderJob {
            name,
            node: input.node,
            palette: input.palette,
            output_dir: testdata_dir.to_path_buf(),
        });
    }

    Ok(jobs)
}

// --- Widget building ---

fn build_widget<'a>(
    node: &NodeConfig,
    leaf_idx: &mut usize,
    palette: ColorPalette,
    parent_is_row: bool,
    parent_stretch: bool,
    is_root: bool,
) -> Element<'a, Message> {
    let is_leaf = node.children.is_empty();

    let inner = if is_leaf {
        build_leaf(node, leaf_idx, palette, parent_is_row, parent_stretch)
    } else {
        build_container(
            node,
            leaf_idx,
            palette,
            parent_is_row,
            parent_stretch,
            is_root,
        )
    };

    // Apply margin as an outer container with padding
    apply_margin(inner, &node.margin.first())
}

fn build_leaf<'a>(
    node: &NodeConfig,
    leaf_idx: &mut usize,
    palette: ColorPalette,
    parent_is_row: bool,
    parent_stretch: bool,
) -> Element<'a, Message> {
    let (r, g, b) = palette_color(palette, *leaf_idx);
    *leaf_idx += 1;
    let bg = Color::from_rgb(r, g, b);

    let label = text(node.label.clone())
        .size(26)
        .color(Color::from_rgba(0.05, 0.05, 0.1, 0.85));

    // Determine effective width
    let basis_overrides_width =
        parent_is_row && matches!(node.flex_basis, ValueConfig::Percent(n) if n > 0.0);
    let grow_overrides_width =
        node.flex_grow > 0.0 && parent_is_row && matches!(node.width, ValueConfig::Auto);
    let stretch_overrides_width =
        parent_stretch && !parent_is_row && matches!(node.width, ValueConfig::Auto);

    let width = if basis_overrides_width {
        flex_basis_length(&node.flex_basis)
    } else if grow_overrides_width {
        fill_portion(node.flex_grow)
    } else if stretch_overrides_width {
        Length::Fill
    } else {
        to_length(&node.width)
    };

    // Determine effective height
    let basis_overrides_height =
        !parent_is_row && matches!(node.flex_basis, ValueConfig::Percent(n) if n > 0.0);
    let grow_overrides_height =
        node.flex_grow > 0.0 && !parent_is_row && matches!(node.height, ValueConfig::Auto);
    let stretch_overrides_height =
        parent_stretch && parent_is_row && matches!(node.height, ValueConfig::Auto);

    let height = if basis_overrides_height {
        flex_basis_length(&node.flex_basis)
    } else if grow_overrides_height {
        fill_portion(node.flex_grow)
    } else if stretch_overrides_height {
        Length::Fill
    } else {
        to_length(&node.height)
    };

    let mut c = container(label)
        .width(width)
        .height(height)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .style(move |_| container::Style {
            background: Some(bg.into()),
            ..Default::default()
        });

    if let Some(p) = to_padding(&node.padding.first()) {
        c = c.padding(p);
    }
    c = apply_min_max(c, node);

    c.into()
}

/// Resolve a ValueConfig to pixels given the parent's resolved size.
fn resolve_to_px(v: &ValueConfig, parent_px: f32) -> f32 {
    match v {
        ValueConfig::Auto => 0.0,
        ValueConfig::Px(n) => *n,
        ValueConfig::Percent(n) => n / 100.0 * parent_px,
        ValueConfig::Vw(n) => n / 100.0 * VIEWPORT_W,
        ValueConfig::Vh(n) => n / 100.0 * VIEWPORT_H,
    }
}

/// Get the main-axis pixel size of a child for wrap line-breaking.
fn child_main_axis_px(child: &NodeConfig, parent_is_row: bool, parent_main: f32) -> f32 {
    let dim = if parent_is_row {
        &child.width
    } else {
        &child.height
    };
    resolve_to_px(dim, parent_main)
}

fn build_container<'a>(
    node: &NodeConfig,
    leaf_idx: &mut usize,
    palette: ColorPalette,
    parent_is_row: bool,
    parent_stretch: bool,
    is_root: bool,
) -> Element<'a, Message> {
    let is_row = matches!(
        node.flex_direction,
        FlexDirection::Row | FlexDirection::RowReverse
    );
    let is_reversed = matches!(
        node.flex_direction,
        FlexDirection::RowReverse | FlexDirection::ColumnReverse
    );
    let stretch = node.align_items == AlignItems::Stretch;
    let wraps = matches!(node.flex_wrap, FlexWrap::Wrap | FlexWrap::WrapReverse);

    // Sort children by order and pre-compute leaf_idx starts so palette
    // colours track with original nodes even when reversed.
    let mut children: Vec<&NodeConfig> = node.children.iter().collect();
    children.sort_by_key(|c| c.order);
    let mut starts: Vec<usize> = Vec::with_capacity(children.len());
    let mut acc = *leaf_idx;
    for child in &children {
        starts.push(acc);
        acc += leaf_count(child);
    }
    *leaf_idx = acc;

    // For non-wrapping layouts, reverse children + swap justify to approximate
    // reversed direction. For wrapping, handle direction in the line builder.
    if is_reversed && !wraps {
        children.reverse();
        starts.reverse();
    }
    let jc = if wraps {
        // Wrapping + reversed: direction handled by reversing items within
        // each line, so no justify swap needed.
        node.justify_content.clone()
    } else {
        effective_justify(&node.justify_content, is_reversed)
    };
    let uses_space_justification = matches!(
        jc,
        JustifyContent::SpaceBetween
            | JustifyContent::SpaceEvenly
            | JustifyContent::SpaceAround
            | JustifyContent::Center
            | JustifyContent::FlexEnd
            | JustifyContent::End
    );

    // Gap values (main-axis and cross-axis)
    let main_gap = if is_row {
        &node.column_gap
    } else {
        &node.row_gap
    };
    let main_gap_px = match main_gap {
        ValueConfig::Px(n) => *n,
        _ => 0.0,
    };
    let cross_gap = if is_row {
        &node.row_gap
    } else {
        &node.column_gap
    };
    let cross_gap_px = match cross_gap {
        ValueConfig::Px(n) => *n,
        _ => 0.0,
    };

    let layout: Element<'a, Message> = if wraps {
        // --- Wrapping layout ---
        // Compute available main-axis space for line breaking
        let parent_main = if is_row { VIEWPORT_W } else { VIEWPORT_H };
        let container_main =
            resolve_to_px(if is_row { &node.width } else { &node.height }, parent_main);
        // Use viewport if the container has no explicit size
        let container_main = if container_main > 0.0 {
            container_main
        } else {
            parent_main
        };
        let padding_px = resolve_to_px(&node.padding.first(), 0.0);
        let inner_main = (container_main - padding_px * 2.0).max(0.0);

        // First pass: compute line assignments for visible children
        let mut line_breaks: Vec<usize> = Vec::new(); // line index per visible child
        let mut current_line = 0usize;
        let mut line_used = 0.0f32;
        let mut visible_count_on_line = 0usize;

        for child in &children {
            if !child.visible {
                continue;
            }
            let size = child_main_axis_px(child, is_row, inner_main);
            let margin_extra = resolve_to_px(&child.margin.first(), 0.0) * 2.0;
            let total = size + margin_extra;

            if visible_count_on_line > 0 && line_used + main_gap_px + total > inner_main {
                current_line += 1;
                line_used = 0.0;
                visible_count_on_line = 0;
            }
            if visible_count_on_line > 0 {
                line_used += main_gap_px;
            }
            line_used += total;
            visible_count_on_line += 1;
            line_breaks.push(current_line);
        }
        let num_lines = if line_breaks.is_empty() {
            0
        } else {
            line_breaks.iter().copied().max().unwrap_or(0) + 1
        };

        // Second pass: build widgets and distribute to lines
        let mut lines: Vec<Vec<Element<'a, Message>>> =
            (0..num_lines).map(|_| Vec::new()).collect();
        let mut visible_idx = 0usize;

        for (child, start) in children.iter().zip(starts.iter()) {
            if !child.visible {
                continue;
            }
            let mut idx = *start;
            let widget = build_widget(child, &mut idx, palette, is_row, stretch, false);
            let widget = apply_align_self(widget, child, is_row);
            if let Some(&line_idx) = line_breaks.get(visible_idx) {
                lines[line_idx].push(widget);
            }
            visible_idx += 1;
        }

        if matches!(node.flex_wrap, FlexWrap::WrapReverse) {
            lines.reverse();
        }

        // Build each line as a Row/Column, then stack lines in the cross direction.
        // For reversed direction, reverse items within each line and right-align
        // (a Space widget pushes items to the end, matching CSS row-reverse flex-start).
        let line_widgets: Vec<Element<'a, Message>> = lines
            .into_iter()
            .map(|mut line_elements| {
                if is_reversed {
                    line_elements.reverse();
                }
                if is_row {
                    let mut elements: Vec<Element<'a, Message>> = Vec::new();
                    if is_reversed {
                        elements.push(Space::new(Length::Fill, Length::Shrink).into());
                    }
                    elements.extend(line_elements);
                    let mut r = row(elements).spacing(main_gap_px).height(Length::Fill);
                    r = apply_row_align(&node.align_items, r);
                    r.into()
                } else {
                    let mut elements: Vec<Element<'a, Message>> = Vec::new();
                    if is_reversed {
                        elements.push(Space::new(Length::Shrink, Length::Fill).into());
                    }
                    elements.extend(line_elements);
                    let mut c = column(elements)
                        .spacing(main_gap_px)
                        .width(Length::Fill);
                    c = apply_column_align(&node.align_items, c);
                    c.into()
                }
            })
            .collect();

        // Stack lines in the cross-axis direction
        if is_row {
            column(line_widgets).spacing(cross_gap_px).into()
        } else {
            row(line_widgets).spacing(cross_gap_px).into()
        }
    } else {
        // --- Single-line layout ---

        // Pre-compute flex-shrink: if children overflow, shrink them proportionally.
        let parent_main = if is_row { VIEWPORT_W } else { VIEWPORT_H };
        let padding_px = resolve_to_px(&node.padding.first(), 0.0);
        let available = (parent_main - padding_px * 2.0).max(0.0);
        let visible: Vec<&&NodeConfig> = children.iter().filter(|c| c.visible).collect();
        let num_gaps = if visible.len() > 1 && !uses_space_justification {
            (visible.len() - 1) as f32
        } else {
            0.0
        };
        let total_main: f32 = visible
            .iter()
            .map(|c| {
                let dim = if is_row { &c.width } else { &c.height };
                resolve_to_px(dim, parent_main) + resolve_to_px(&c.margin.first(), 0.0) * 2.0
            })
            .sum::<f32>()
            + num_gaps * main_gap_px;

        let shrink_ratio = if total_main > available && total_main > 0.0 {
            available / total_main
        } else {
            1.0
        };

        // Build child widgets, skipping invisible ones
        let child_widgets: Vec<Element<'a, Message>> = children
            .iter()
            .zip(starts.iter())
            .filter_map(|(child, start)| {
                if !child.visible {
                    return None;
                }
                let mut idx = *start;
                let widget = build_widget(child, &mut idx, palette, is_row, stretch, false);
                let widget = apply_align_self(widget, child, is_row);
                // Apply flex-shrink by wrapping in a fixed-size container
                let widget = if shrink_ratio < 1.0 && child.flex_shrink > 0.0 {
                    let dim = if is_row { &child.width } else { &child.height };
                    let orig = resolve_to_px(dim, parent_main);
                    if orig > 0.0 {
                        let shrunk = orig * shrink_ratio;
                        if is_row {
                            container(widget).width(Length::Fixed(shrunk)).into()
                        } else {
                            container(widget).height(Length::Fixed(shrunk)).into()
                        }
                    } else {
                        widget
                    }
                } else {
                    widget
                };
                Some(widget)
            })
            .collect();

        // Build elements list with Space widgets for justify-content
        let space_widget = || -> Element<'a, Message> {
            if is_row {
                Space::new(Length::Fill, Length::Shrink).into()
            } else {
                Space::new(Length::Shrink, Length::Fill).into()
            }
        };

        let mut elements: Vec<Element<'a, Message>> = Vec::new();

        match &jc {
            JustifyContent::SpaceBetween => {
                for (i, widget) in child_widgets.into_iter().enumerate() {
                    if i > 0 {
                        elements.push(space_widget());
                    }
                    elements.push(widget);
                }
            }
            JustifyContent::Center => {
                elements.push(space_widget());
                for widget in child_widgets {
                    elements.push(widget);
                }
                elements.push(space_widget());
            }
            JustifyContent::SpaceEvenly | JustifyContent::SpaceAround => {
                for widget in child_widgets {
                    elements.push(space_widget());
                    elements.push(widget);
                }
                elements.push(space_widget());
            }
            JustifyContent::FlexEnd | JustifyContent::End => {
                elements.push(space_widget());
                for widget in child_widgets {
                    elements.push(widget);
                }
            }
            _ => {
                elements = child_widgets;
            }
        }

        let spacing = if uses_space_justification {
            0.0
        } else {
            main_gap_px
        };

        // Build the row or column — fill parent so cross-axis alignment works
        if is_row {
            let mut r = row(elements).spacing(spacing).height(Length::Fill);
            r = apply_row_align(&node.align_items, r);
            r.into()
        } else {
            let mut c = column(elements).spacing(spacing).width(Length::Fill);
            c = apply_column_align(&node.align_items, c);
            c.into()
        }
    };

    // Wrap in container for sizing, padding, background
    let full_w = matches!(node.width, ValueConfig::Percent(n) if n >= 100.0);
    let full_h = matches!(node.height, ValueConfig::Percent(n) if n >= 100.0);

    let basis_w = parent_is_row && matches!(node.flex_basis, ValueConfig::Percent(n) if n > 0.0);
    let basis_h = !parent_is_row && matches!(node.flex_basis, ValueConfig::Percent(n) if n > 0.0);

    let width = if basis_w {
        flex_basis_length(&node.flex_basis)
    } else if full_w {
        Length::Fill
    } else if grow_overrides(node.flex_grow, parent_is_row, &node.width) {
        fill_portion(node.flex_grow)
    } else if !parent_is_row && parent_stretch && matches!(node.width, ValueConfig::Auto) && !full_w
    {
        Length::Fill
    } else {
        to_length(&node.width)
    };

    // Root always fills viewport height, matching HTML body { height: 100% }
    let height = if is_root {
        Length::Fill
    } else if basis_h {
        flex_basis_length(&node.flex_basis)
    } else if full_h {
        Length::Fill
    } else if grow_overrides(node.flex_grow, !parent_is_row, &node.height) {
        fill_portion(node.flex_grow)
    } else if parent_is_row && parent_stretch && matches!(node.height, ValueConfig::Auto) && !full_h
    {
        Length::Fill
    } else {
        to_length(&node.height)
    };

    let bg = Color::from_rgba(0.11, 0.11, 0.17, 1.0);

    let mut c = container(layout)
        .width(width)
        .height(height)
        .style(move |_| container::Style {
            background: Some(bg.into()),
            ..Default::default()
        });

    if let Some(p) = to_padding(&node.padding.first()) {
        c = c.padding(p);
    }

    c = apply_min_max(c, node);

    c.into()
}

/// When direction is reversed, flex-start/end swap so items anchor to the
/// correct end of the main axis (CSS reverses the axis, not just child order).
fn effective_justify(jc: &JustifyContent, is_reversed: bool) -> JustifyContent {
    if !is_reversed {
        return *jc;
    }
    match jc {
        JustifyContent::FlexStart => JustifyContent::FlexEnd,
        JustifyContent::FlexEnd => JustifyContent::FlexStart,
        JustifyContent::Start => JustifyContent::End,
        JustifyContent::End => JustifyContent::Start,
        other => *other,
    }
}

fn apply_row_align<'a>(
    align: &AlignItems,
    r: iced::widget::Row<'a, Message>,
) -> iced::widget::Row<'a, Message> {
    match align {
        AlignItems::FlexStart | AlignItems::Start | AlignItems::Stretch => {
            r.align_y(iced::Alignment::Start)
        }
        AlignItems::FlexEnd | AlignItems::End => r.align_y(iced::Alignment::End),
        AlignItems::Center => r.align_y(iced::Alignment::Center),
        _ => r.align_y(iced::Alignment::Start),
    }
}

fn apply_column_align<'a>(
    align: &AlignItems,
    c: iced::widget::Column<'a, Message>,
) -> iced::widget::Column<'a, Message> {
    match align {
        AlignItems::FlexStart | AlignItems::Start | AlignItems::Stretch => {
            c.align_x(iced::Alignment::Start)
        }
        AlignItems::FlexEnd | AlignItems::End => c.align_x(iced::Alignment::End),
        AlignItems::Center => c.align_x(iced::Alignment::Center),
        _ => c.align_x(iced::Alignment::Start),
    }
}

fn grow_overrides(flex_grow: f32, axis_matches: bool, dim: &ValueConfig) -> bool {
    flex_grow > 0.0 && axis_matches && matches!(dim, ValueConfig::Auto)
}

fn fill_portion(grow: f32) -> Length {
    if grow > 1.0 {
        Length::FillPortion(grow as u16)
    } else {
        Length::Fill
    }
}

// --- Value conversion helpers ---

fn to_length(v: &ValueConfig) -> Length {
    match v {
        ValueConfig::Auto => Length::Shrink,
        ValueConfig::Px(n) => Length::Fixed(*n),
        ValueConfig::Percent(n) if *n >= 100.0 => Length::Fill,
        ValueConfig::Percent(n) => Length::FillPortion(*n as u16),
        // Viewport units: approximate as fixed pixels for 400x300 viewport
        ValueConfig::Vw(n) => Length::Fixed(n / 100.0 * 400.0),
        ValueConfig::Vh(n) => Length::Fixed(n / 100.0 * 300.0),
    }
}

fn to_padding(v: &ValueConfig) -> Option<Padding> {
    match v {
        ValueConfig::Auto => None,
        ValueConfig::Px(n) if *n == 0.0 => None,
        ValueConfig::Px(n) => Some(Padding::from(*n)),
        ValueConfig::Percent(n) => Some(Padding::from(*n)),
        ValueConfig::Vw(n) => Some(Padding::from(n / 100.0 * 400.0)),
        ValueConfig::Vh(n) => Some(Padding::from(n / 100.0 * 300.0)),
    }
}

/// Apply min/max width/height constraints from the node config.
fn apply_min_max<'a>(
    mut c: iced::widget::Container<'a, Message>,
    node: &NodeConfig,
) -> iced::widget::Container<'a, Message> {
    if let ValueConfig::Px(n) = node.max_width
        && n > 0.0
    {
        c = c.max_width(n);
    }
    if let ValueConfig::Px(n) = node.max_height
        && n > 0.0
    {
        c = c.max_height(n);
    }
    // Iced doesn't have min_width/min_height on Container, but we can
    // approximate via a minimum-sized Space inside a wrapper if needed.
    // For now, skip min constraints as Iced lacks direct API support.
    c
}

/// Wrap a widget in a container with padding to simulate CSS margin.
fn apply_margin<'a>(widget: Element<'a, Message>, margin: &ValueConfig) -> Element<'a, Message> {
    match to_padding(margin) {
        Some(p) => container(widget).padding(p).into(),
        None => widget,
    }
}

/// Apply align_self by wrapping the child in a container with appropriate alignment.
fn apply_align_self<'a>(
    widget: Element<'a, Message>,
    child: &NodeConfig,
    parent_is_row: bool,
) -> Element<'a, Message> {
    match child.align_self {
        AlignSelf::Auto => widget,
        AlignSelf::Center => {
            if parent_is_row {
                container(widget)
                    .align_y(iced::alignment::Vertical::Center)
                    .height(Length::Fill)
                    .into()
            } else {
                container(widget)
                    .align_x(iced::alignment::Horizontal::Center)
                    .width(Length::Fill)
                    .into()
            }
        }
        AlignSelf::FlexStart | AlignSelf::Start => {
            if parent_is_row {
                container(widget)
                    .align_y(iced::alignment::Vertical::Top)
                    .height(Length::Fill)
                    .into()
            } else {
                container(widget)
                    .align_x(iced::alignment::Horizontal::Left)
                    .width(Length::Fill)
                    .into()
            }
        }
        AlignSelf::FlexEnd | AlignSelf::End => {
            if parent_is_row {
                container(widget)
                    .align_y(iced::alignment::Vertical::Bottom)
                    .height(Length::Fill)
                    .into()
            } else {
                container(widget)
                    .align_x(iced::alignment::Horizontal::Right)
                    .width(Length::Fill)
                    .into()
            }
        }
        AlignSelf::Stretch => {
            if parent_is_row {
                container(widget).height(Length::Fill).into()
            } else {
                container(widget).width(Length::Fill).into()
            }
        }
        AlignSelf::Baseline => widget, // Iced has no baseline alignment
    }
}

/// Return the number of leaves under a node.
fn leaf_count(node: &NodeConfig) -> usize {
    if node.children.is_empty() {
        1
    } else {
        node.children.iter().map(leaf_count).sum()
    }
}

/// Convert a flex_basis percentage to a Length::FillPortion.
fn flex_basis_length(basis: &ValueConfig) -> Length {
    match basis {
        ValueConfig::Percent(n) => Length::FillPortion(n.round() as u16),
        _ => Length::Shrink,
    }
}

// --- Palette colors (mirrors flexplore's art::palette_color) ---

fn palette_color(palette: ColorPalette, idx: usize) -> (f32, f32, f32) {
    let c = match palette {
        ColorPalette::Pastel1 => colorous::PASTEL1[idx % colorous::PASTEL1.len()],
        ColorPalette::Pastel2 => colorous::PASTEL2[idx % colorous::PASTEL2.len()],
        ColorPalette::Set1 => colorous::SET1[idx % colorous::SET1.len()],
        ColorPalette::Set2 => colorous::SET2[idx % colorous::SET2.len()],
        ColorPalette::Set3 => colorous::SET3[idx % colorous::SET3.len()],
        ColorPalette::Tableau10 => colorous::TABLEAU10[idx % colorous::TABLEAU10.len()],
        ColorPalette::Category10 => colorous::CATEGORY10[idx % colorous::CATEGORY10.len()],
        ColorPalette::Accent => colorous::ACCENT[idx % colorous::ACCENT.len()],
        ColorPalette::Dark2 => colorous::DARK2[idx % colorous::DARK2.len()],
        ColorPalette::Paired => colorous::PAIRED[idx % colorous::PAIRED.len()],
    };
    (c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0)
}
