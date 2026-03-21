//! Headless-ish Iced renderer for flexplain golden tests.
//!
//! Reads `testdata/{case}/input.json`, renders the layout with Iced,
//! captures a screenshot, and saves `rendered_iced.png`.

use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use iced::widget::{column, container, row, text, Space};
use iced::window;
use iced::{Color, Element, Length, Padding, Size, Subscription, Task, Theme};

mod config;
use config::{
    AlignItems, AlignSelf, ColorPalette, FlexDirection, JustifyContent, LayoutInput, NodeConfig,
    ValueConfig,
};

// ─── Application state ──────────────────────────────────────────────────────

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
    let args: Vec<String> = std::env::args().collect();

    let testdata_dir = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from("testdata")
    };

    let filter: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();

    let jobs = load_jobs(&testdata_dir, &filter).expect("failed to load test jobs");
    if jobs.is_empty() {
        eprintln!("No render jobs found.");
        return Ok(());
    }
    eprintln!(
        "Will render {} case(s) from {}",
        jobs.len(),
        testdata_dir.display()
    );

    iced::application("iced-golden", App::update, App::view)
        .subscription(App::subscription)
        .theme(|_| Theme::Dark)
        .window_size(Size::new(400.0, 300.0))
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

// ─── Screenshot saving ──────────────────────────────────────────────────────

fn save_screenshot(path: &PathBuf, screenshot: &window::Screenshot) {
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

// ─── Job loading ────────────────────────────────────────────────────────────

fn load_jobs(testdata_dir: &PathBuf, filter: &[&str]) -> Result<Vec<RenderJob>> {
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

// ─── Widget building ────────────────────────────────────────────────────────

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
        build_container(node, leaf_idx, palette, parent_is_row, parent_stretch, is_root)
    };

    // Apply margin as an outer container with padding
    apply_margin(inner, &node.margin)
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
    let basis_overrides_width = parent_is_row && matches!(node.flex_basis, ValueConfig::Percent(n) if n > 0.0);
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
    let basis_overrides_height = !parent_is_row && matches!(node.flex_basis, ValueConfig::Percent(n) if n > 0.0);
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

    if let Some(p) = to_padding(&node.padding) {
        c = c.padding(p);
    }
    c = apply_min_max(c, node);

    c.into()
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

    let jc = &node.justify_content;
    let uses_space_justification = matches!(
        jc,
        JustifyContent::SpaceBetween
            | JustifyContent::SpaceEvenly
            | JustifyContent::SpaceAround
            | JustifyContent::Center
            | JustifyContent::FlexEnd
            | JustifyContent::End
    );

    // Sort children by order
    let mut children: Vec<&NodeConfig> = node.children.iter().collect();
    children.sort_by_key(|c| c.order);
    if is_reversed {
        children.reverse();
    }

    // Build child widgets, skipping invisible ones
    let child_widgets: Vec<Element<'a, Message>> = children
        .iter()
        .filter_map(|child| {
            if !child.visible {
                // Still advance leaf_idx for correct color assignment
                count_leaves(child, leaf_idx);
                return None;
            }
            let widget = build_widget(child, leaf_idx, palette, is_row, stretch, false);
            // Apply align_self override
            let widget = apply_align_self(widget, child, is_row);
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

    match jc {
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

    // Gap (main-axis spacing)
    let gap = if is_row {
        &node.column_gap
    } else {
        &node.row_gap
    };
    let spacing = if uses_space_justification {
        0.0
    } else {
        match gap {
            ValueConfig::Px(n) => *n,
            _ => 0.0,
        }
    };

    // Build the row or column
    let layout: Element<'a, Message> = if is_row {
        let mut r = row(elements).spacing(spacing);
        r = match node.align_items {
            AlignItems::FlexStart | AlignItems::Start | AlignItems::Stretch => {
                r.align_y(iced::Alignment::Start)
            }
            AlignItems::FlexEnd | AlignItems::End => r.align_y(iced::Alignment::End),
            AlignItems::Center => r.align_y(iced::Alignment::Center),
            _ => r.align_y(iced::Alignment::Start),
        };
        r.into()
    } else {
        let mut c = column(elements).spacing(spacing);
        c = match node.align_items {
            AlignItems::FlexStart | AlignItems::Start | AlignItems::Stretch => {
                c.align_x(iced::Alignment::Start)
            }
            AlignItems::FlexEnd | AlignItems::End => c.align_x(iced::Alignment::End),
            AlignItems::Center => c.align_x(iced::Alignment::Center),
            _ => c.align_x(iced::Alignment::Start),
        };
        c.into()
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

    if let Some(p) = to_padding(&node.padding) {
        c = c.padding(p);
    }

    c.into()
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

// ─── Value conversion helpers ───────────────────────────────────────────────

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

/// Count leaf nodes without building widgets (for color index bookkeeping).
fn count_leaves(node: &NodeConfig, leaf_idx: &mut usize) {
    if node.children.is_empty() {
        *leaf_idx += 1;
    } else {
        for child in &node.children {
            count_leaves(child, leaf_idx);
        }
    }
}

/// Convert a flex_basis percentage to a Length::FillPortion.
fn flex_basis_length(basis: &ValueConfig) -> Length {
    match basis {
        ValueConfig::Percent(n) => Length::FillPortion(n.round() as u16),
        _ => Length::Shrink,
    }
}

// ─── Palette colors (mirrors flexplore's art::palette_color) ────────────────

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
