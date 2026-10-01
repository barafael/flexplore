//! Headless-ish Iced renderer for flexplore golden tests.
//!
//! Reads `testdata/{case}/input.json`, renders the layout with Iced,
//! captures a screenshot, and saves `rendered_iced.png`.

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::{Result, anyhow, bail};
use clap::Parser;
use golden_common::{
    RenderJob, VIEWPORT_H, VIEWPORT_W,
    config::{
        AlignItems, AlignSelf, ColorPalette, DisplayMode, FlexDirection, FlexWrap, JustifyContent,
        NodeConfig, ValueConfig,
    },
    effective_justify, palette_color,
    png::save_rgba_png,
    resolve_to_px,
};
use iced::{
    Color, Element, Length, Padding, Size, Subscription, Task, Theme,
    widget::{Space, column, container, row, text},
    window,
};

/// Render flexplore golden screenshots with Iced.
#[derive(Parser)]
#[command(name = "iced-golden")]
struct Arguments {
    /// Testdata directory (default: the repository's testdata/).
    #[arg(long, default_value_os_t = golden_common::default_testdata_dir())]
    testdata: PathBuf,

    /// Only render these test cases (default: all).
    cases: Vec<String>,
}

/// Interval between layout ticks.
const TICK: Duration = Duration::from_millis(50);
/// Ticks to let the layout settle before requesting a screenshot.
const SETTLE_TICKS: usize = 4;
/// Ticks to wait for the screenshot before giving up (15 s at 50 ms).
const TIMEOUT_TICKS: usize = 300;

// --- Application state ---

struct App {
    jobs: Vec<RenderJob>,
    current: usize,
    frames: usize,
    /// First fatal error; reported by `main` as a non-zero exit.
    failure: Arc<Mutex<Option<String>>>,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    Screenshot(window::Screenshot),
}

fn main() -> Result<()> {
    let cli = Arguments::parse();

    let jobs = golden_common::load_jobs(&cli.testdata, &cli.cases)?;
    if jobs.is_empty() {
        eprintln!("No render jobs found in {}.", cli.testdata.display());
        return Ok(());
    }
    eprintln!(
        "Will render {} case(s) from {}",
        jobs.len(),
        cli.testdata.display()
    );
    eprintln!("Rendering: {}", jobs[0].name);

    let failure = Arc::new(Mutex::new(None));

    // iced 0.14 takes the boot closure as `Fn`, so hand the jobs over through a
    // slot that the (single) boot call takes from.
    let jobs = Mutex::new(Some(jobs));
    let app_failure = Arc::clone(&failure);
    let boot = move || {
        let jobs = jobs.lock().unwrap().take().unwrap_or_default();
        (
            App {
                jobs,
                current: 0,
                frames: 0,
                failure: Arc::clone(&app_failure),
            },
            Task::none(),
        )
    };

    iced::application(boot, App::update, App::view)
        .title("iced-golden")
        .subscription(App::subscription)
        .theme(Theme::Dark)
        .scale_factor(|_| 1.0)
        .window_size(Size::new(VIEWPORT_W, VIEWPORT_H))
        .run()
        .map_err(|e| anyhow!("iced failed: {e}"))?;

    if let Some(msg) = failure.lock().unwrap().take() {
        bail!("{msg}");
    }
    Ok(())
}

impl App {
    fn fail(&mut self, msg: String) -> Task<Message> {
        eprintln!("  ERROR: {msg}");
        self.failure.lock().unwrap().get_or_insert(msg);
        // Stop ticking and close.
        self.current = self.jobs.len();
        window::latest().and_then(window::close)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.frames += 1;
                if self.frames == SETTLE_TICKS {
                    return window::latest()
                        .and_then(window::screenshot)
                        .map(Message::Screenshot);
                }
                if self.frames > TIMEOUT_TICKS {
                    let name = self.jobs[self.current].name.clone();
                    return self.fail(format!(
                        "timed out after {:?} waiting for screenshot of {name}",
                        TICK * TIMEOUT_TICKS as u32
                    ));
                }
                Task::none()
            }
            Message::Screenshot(screenshot) => {
                if let Some(job) = self.jobs.get(self.current) {
                    let path = job.output_path("rendered_iced.png");
                    if let Err(e) = save_screenshot(&path, &screenshot) {
                        return self.fail(format!("{}: {e:#}", job.name));
                    }
                    eprintln!("  Saved: {}", path.display());
                }

                self.current += 1;
                self.frames = 0;

                if self.current >= self.jobs.len() {
                    eprintln!("All done!");
                    return window::latest().and_then(window::close);
                }

                eprintln!("Rendering: {}", self.jobs[self.current].name);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        if let Some(job) = self.jobs.get(self.current) {
            let mut leaf_idx = 0;
            build_widget(&job.node, &mut leaf_idx, job.palette, Ctx::root())
        } else {
            Space::new().width(Length::Fill).height(Length::Fill).into()
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.current < self.jobs.len() {
            iced::time::every(TICK).map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }
}

// --- Screenshot saving ---

fn save_screenshot(path: &std::path::Path, screenshot: &window::Screenshot) -> Result<()> {
    let size = screenshot.size;
    // Resize to the viewport size to normalize across DPI settings.
    save_rgba_png(
        path,
        size.width,
        size.height,
        screenshot.rgba.to_vec(),
        Some((VIEWPORT_W as u32, VIEWPORT_H as u32)),
    )
}

// --- Widget building ---

/// Layout context a node inherits from its parent.
#[derive(Clone, Copy)]
struct Ctx {
    /// Parent's main axis is horizontal.
    parent_is_row: bool,
    /// Parent has `align-items: stretch`.
    parent_stretch: bool,
    is_root: bool,
    /// This node or an ancestor is `visible: false`: keep its space, paint nothing.
    hidden: bool,
}

impl Ctx {
    fn root() -> Self {
        Self {
            parent_is_row: true,
            parent_stretch: false,
            is_root: true,
            hidden: false,
        }
    }

    fn child(self, parent_is_row: bool, parent_stretch: bool) -> Self {
        Self {
            parent_is_row,
            parent_stretch,
            is_root: false,
            hidden: self.hidden,
        }
    }
}

fn build_widget<'a>(
    node: &NodeConfig,
    leaf_idx: &mut usize,
    palette: ColorPalette,
    ctx: Ctx,
) -> Element<'a, Message> {
    let ctx = Ctx {
        hidden: ctx.hidden || !node.visible,
        ..ctx
    };

    let inner = if node.children.is_empty() {
        build_leaf(node, leaf_idx, palette, ctx)
    } else {
        build_container(node, leaf_idx, palette, ctx)
    };

    // Apply margin as an outer container with padding
    apply_margin(inner, &node.margin.first())
}

fn build_leaf<'a>(
    node: &NodeConfig,
    leaf_idx: &mut usize,
    palette: ColorPalette,
    ctx: Ctx,
) -> Element<'a, Message> {
    let (r, g, b) = palette_color(palette, *leaf_idx);
    *leaf_idx += 1;
    // Hidden nodes keep their layout box but paint nothing.
    let (bg, fg) = if ctx.hidden {
        (Color::TRANSPARENT, Color::TRANSPARENT)
    } else {
        (
            Color::from_rgb(r, g, b),
            Color::from_rgba(0.05, 0.05, 0.1, 0.85),
        )
    };

    let label = text(node.display_text().to_owned()).size(26).color(fg);

    // Determine effective width
    let basis_overrides_width =
        ctx.parent_is_row && matches!(node.flex_basis, ValueConfig::Percent(n) if n > 0.0);
    let grow_overrides_width =
        node.flex_grow > 0.0 && ctx.parent_is_row && matches!(node.width, ValueConfig::Auto);
    let stretch_overrides_width =
        ctx.parent_stretch && !ctx.parent_is_row && matches!(node.width, ValueConfig::Auto);

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
        !ctx.parent_is_row && matches!(node.flex_basis, ValueConfig::Percent(n) if n > 0.0);
    let grow_overrides_height =
        node.flex_grow > 0.0 && !ctx.parent_is_row && matches!(node.height, ValueConfig::Auto);
    let stretch_overrides_height =
        ctx.parent_stretch && ctx.parent_is_row && matches!(node.height, ValueConfig::Auto);

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
    ctx: Ctx,
) -> Element<'a, Message> {
    // Grid containers mirror `codegen/iced.rs`: a wrapping `row![…]` of the
    // children in source order (spans, template rows and auto-flow ignored).
    let is_grid = node.display_mode == DisplayMode::Grid;
    let is_row = is_grid
        || matches!(
            node.flex_direction,
            FlexDirection::Row | FlexDirection::RowReverse
        );
    let is_reversed = !is_grid
        && matches!(
            node.flex_direction,
            FlexDirection::RowReverse | FlexDirection::ColumnReverse
        );
    let stretch = node.align_items == AlignItems::Stretch;
    let wraps = !is_grid && matches!(node.flex_wrap, FlexWrap::Wrap | FlexWrap::WrapReverse);
    let child_ctx = ctx.child(is_row, stretch);

    // Sort children by order and pre-compute leaf_idx starts so palette
    // colours track with original nodes even when reversed.
    let mut children: Vec<&NodeConfig> = node.children.iter().collect();
    children.sort_by_key(|c| c.order);
    let mut starts: Vec<usize> = Vec::with_capacity(children.len());
    let mut acc = *leaf_idx;
    for child in &children {
        starts.push(acc);
        acc += child.count_leaves();
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
        node.justify_content
    } else {
        effective_justify(node.justify_content, is_reversed)
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

    // Hidden children keep their space (`visibility: hidden`), so every
    // child takes part in line breaking, shrinking and justification.
    let layout: Element<'a, Message> = if is_grid {
        // --- Grid approximation: `row![children].wrap().spacing(gap)` ---
        let widgets: Vec<Element<'a, Message>> = children
            .iter()
            .zip(&starts)
            .map(|(child, start)| {
                let mut idx = *start;
                let widget = build_widget(child, &mut idx, palette, child_ctx);
                apply_align_self(widget, child, true)
            })
            .collect();
        let mut r = row(widgets).spacing(main_gap_px);
        r = apply_row_align(&node.align_items, r);
        r.wrap().vertical_spacing(cross_gap_px).into()
    } else if wraps {
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

        // First pass: compute line assignments
        let mut line_breaks: Vec<usize> = Vec::with_capacity(children.len());
        let mut current_line = 0usize;
        let mut line_used = 0.0f32;
        let mut count_on_line = 0usize;

        for child in &children {
            let size = child_main_axis_px(child, is_row, inner_main);
            let margin_extra = resolve_to_px(&child.margin.first(), 0.0) * 2.0;
            let total = size + margin_extra;

            if count_on_line > 0 && line_used + main_gap_px + total > inner_main {
                current_line += 1;
                line_used = 0.0;
                count_on_line = 0;
            }
            if count_on_line > 0 {
                line_used += main_gap_px;
            }
            line_used += total;
            count_on_line += 1;
            line_breaks.push(current_line);
        }
        let num_lines = line_breaks.iter().copied().max().map_or(0, |m| m + 1);

        // Second pass: build widgets and distribute to lines
        let mut lines: Vec<Vec<Element<'a, Message>>> =
            (0..num_lines).map(|_| Vec::new()).collect();

        for ((child, start), line_idx) in children.iter().zip(&starts).zip(&line_breaks) {
            let mut idx = *start;
            let widget = build_widget(child, &mut idx, palette, child_ctx);
            let widget = apply_align_self(widget, child, is_row);
            lines[*line_idx].push(widget);
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
                        elements.push(
                            Space::new()
                                .width(Length::Fill)
                                .height(Length::Shrink)
                                .into(),
                        );
                    }
                    elements.extend(line_elements);
                    let mut r = row(elements).spacing(main_gap_px).height(Length::Fill);
                    r = apply_row_align(&node.align_items, r);
                    r.into()
                } else {
                    let mut elements: Vec<Element<'a, Message>> = Vec::new();
                    if is_reversed {
                        elements.push(
                            Space::new()
                                .width(Length::Shrink)
                                .height(Length::Fill)
                                .into(),
                        );
                    }
                    elements.extend(line_elements);
                    let mut c = column(elements).spacing(main_gap_px).width(Length::Fill);
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
        let num_gaps = if children.len() > 1 && !uses_space_justification {
            (children.len() - 1) as f32
        } else {
            0.0
        };
        let total_main: f32 = children
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

        let child_widgets: Vec<Element<'a, Message>> = children
            .iter()
            .zip(&starts)
            .map(|(child, start)| {
                let mut idx = *start;
                let widget = build_widget(child, &mut idx, palette, child_ctx);
                let widget = apply_align_self(widget, child, is_row);
                // Apply flex-shrink by wrapping in a fixed-size container
                if shrink_ratio < 1.0 && child.flex_shrink > 0.0 {
                    let dim = if is_row { &child.width } else { &child.height };
                    let orig = resolve_to_px(dim, parent_main);
                    if orig > 0.0 {
                        let shrunk = orig * shrink_ratio;
                        return if is_row {
                            container(widget).width(Length::Fixed(shrunk)).into()
                        } else {
                            container(widget).height(Length::Fixed(shrunk)).into()
                        };
                    }
                }
                widget
            })
            .collect();

        // Build elements list with Space widgets for justify-content
        let space_widget = || -> Element<'a, Message> {
            if is_row {
                Space::new()
                    .width(Length::Fill)
                    .height(Length::Shrink)
                    .into()
            } else {
                Space::new()
                    .width(Length::Shrink)
                    .height(Length::Fill)
                    .into()
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

    let basis_w =
        ctx.parent_is_row && matches!(node.flex_basis, ValueConfig::Percent(n) if n > 0.0);
    let basis_h =
        !ctx.parent_is_row && matches!(node.flex_basis, ValueConfig::Percent(n) if n > 0.0);

    let width = if basis_w {
        flex_basis_length(&node.flex_basis)
    } else if full_w {
        Length::Fill
    } else if grow_overrides(node.flex_grow, ctx.parent_is_row, &node.width) {
        fill_portion(node.flex_grow)
    } else if !ctx.parent_is_row
        && ctx.parent_stretch
        && matches!(node.width, ValueConfig::Auto)
        && !full_w
    {
        Length::Fill
    } else {
        to_length(&node.width)
    };

    // Root always fills viewport height, matching HTML body { height: 100% }
    let height = if ctx.is_root {
        Length::Fill
    } else if basis_h {
        flex_basis_length(&node.flex_basis)
    } else if full_h {
        Length::Fill
    } else if grow_overrides(node.flex_grow, !ctx.parent_is_row, &node.height) {
        fill_portion(node.flex_grow)
    } else if ctx.parent_is_row
        && ctx.parent_stretch
        && matches!(node.height, ValueConfig::Auto)
        && !full_h
    {
        Length::Fill
    } else {
        to_length(&node.height)
    };

    let bg = if ctx.hidden {
        Color::TRANSPARENT
    } else {
        Color::from_rgba(0.11, 0.11, 0.17, 1.0)
    };

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

/// `FillPortion(0)` is invalid in iced, so portions are clamped to at least 1.
fn portion(n: f32) -> Length {
    Length::FillPortion((n as u16).max(1))
}

fn fill_portion(grow: f32) -> Length {
    if grow > 1.0 {
        portion(grow)
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
        ValueConfig::Percent(n) => portion(*n),
        ValueConfig::Vw(_) | ValueConfig::Vh(_) => Length::Fixed(resolve_to_px(v, 0.0)),
    }
}

fn to_padding(v: &ValueConfig) -> Option<Padding> {
    match v {
        ValueConfig::Auto => None,
        ValueConfig::Px(n) if *n == 0.0 => None,
        ValueConfig::Px(n) => Some(Padding::from(*n)),
        ValueConfig::Percent(n) => Some(Padding::from(*n)),
        ValueConfig::Vw(_) | ValueConfig::Vh(_) => Some(Padding::from(resolve_to_px(v, 0.0))),
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

/// Convert a flex_basis percentage to a Length::FillPortion.
fn flex_basis_length(basis: &ValueConfig) -> Length {
    match basis {
        ValueConfig::Percent(n) => portion(n.round()),
        _ => Length::Shrink,
    }
}
