//! Headless-ish egui renderer for flexplore golden tests.
//!
//! Reads `testdata/{case}/input.json`, renders the layout with egui via eframe,
//! captures a screenshot via `ViewportCommand::Screenshot`, and saves
//! `rendered_egui.png`.

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use anyhow::{Result, anyhow, bail};
use clap::Parser;
use eframe::egui;
use egui::{Align, Color32, Layout, Vec2};
use golden_common::{
    RenderJob, VIEWPORT_H, VIEWPORT_W,
    config::{
        AlignItems, AlignSelf, ColorPalette, DisplayMode, FlexDirection, FlexWrap, JustifyContent,
        NodeConfig, ValueConfig,
    },
    effective_justify, palette_rgb8,
    png::save_rgba_png,
    resolve_to_px,
};

/// Render flexplore golden screenshots with egui.
#[derive(Parser)]
#[command(name = "egui-golden")]
struct Arguments {
    /// Testdata directory (default: the repository's testdata/).
    #[arg(long, default_value_os_t = golden_common::default_testdata_dir())]
    testdata: PathBuf,

    /// Only render these test cases (default: all).
    cases: Vec<String>,
}

/// Frames to let the layout settle before requesting a screenshot.
const SETTLE_FRAMES: usize = 4;
/// How long to wait for the compositor to hand back a requested screenshot.
const SCREENSHOT_TIMEOUT: Duration = Duration::from_secs(15);

// ─── Application state ──────────────────────────────────────────────────────

struct App {
    jobs: Vec<RenderJob>,
    current: usize,
    frames: usize,
    /// When the screenshot for the current job was requested.
    screenshot_requested: Option<Instant>,
    /// First fatal error; reported by `main` as a non-zero exit.
    failure: Arc<Mutex<Option<String>>>,
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

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([VIEWPORT_W, VIEWPORT_H])
            .with_resizable(false),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    let failure = Arc::new(Mutex::new(None));
    let app_failure = Arc::clone(&failure);
    eframe::run_native(
        "egui-golden",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(App {
                jobs,
                current: 0,
                frames: 0,
                screenshot_requested: None,
                failure: app_failure,
            }))
        }),
    )
    .map_err(|e| anyhow!("eframe failed: {e}"))?;

    if let Some(msg) = failure.lock().unwrap().take() {
        bail!("{msg}");
    }
    Ok(())
}

impl App {
    fn fail(&mut self, ctx: &egui::Context, msg: String) {
        eprintln!("  ERROR: {msg}");
        self.failure.lock().unwrap().get_or_insert(msg);
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    fn take_screenshot_event(ctx: &egui::Context) -> Option<Arc<egui::ColorImage>> {
        ctx.input(|i| {
            i.raw.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        })
    }
}

impl eframe::App for App {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        if self.failure.lock().unwrap().is_some() {
            return;
        }
        if self.current >= self.jobs.len() {
            eprintln!("All done!");
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        if let Some(requested_at) = self.screenshot_requested {
            if let Some(image) = Self::take_screenshot_event(&ctx) {
                let job = &self.jobs[self.current];
                let path = job.output_path("rendered_egui.png");
                if let Err(e) = save_color_image(&image, &path) {
                    self.fail(&ctx, format!("{}: {e:#}", job.name));
                    return;
                }
                eprintln!("  Saved: {}", path.display());

                self.current += 1;
                self.frames = 0;
                self.screenshot_requested = None;

                if self.current < self.jobs.len() {
                    eprintln!("Rendering: {}", self.jobs[self.current].name);
                }
                ctx.request_repaint();
                return;
            }
            if requested_at.elapsed() > SCREENSHOT_TIMEOUT {
                let name = self.jobs[self.current].name.clone();
                self.fail(
                    &ctx,
                    format!(
                        "timed out after {SCREENSHOT_TIMEOUT:?} waiting for screenshot of {name}"
                    ),
                );
                return;
            }
        }

        let job = &self.jobs[self.current];

        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(Color32::from_rgb(28, 28, 43))
                    .inner_margin(0.0),
            )
            .show(root, |ui| {
                ui.set_min_size(Vec2::new(VIEWPORT_W, VIEWPORT_H));
                let mut leaf_idx = 0;
                build_widget(ui, &job.node, &mut leaf_idx, job.palette, Ctx::root());
            });

        self.frames += 1;

        if self.frames == SETTLE_FRAMES && self.screenshot_requested.is_none() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
            self.screenshot_requested = Some(Instant::now());
        }

        ctx.request_repaint();
    }
}

// ─── Screenshot saving ──────────────────────────────────────────────────────

fn save_color_image(image: &egui::ColorImage, path: &std::path::Path) -> Result<()> {
    let w = image.size[0] as u32;
    let h = image.size[1] as u32;
    let pixels: Vec<u8> = image
        .pixels
        .iter()
        .flat_map(|c| [c.r(), c.g(), c.b(), c.a()])
        .collect();
    // Resize to the viewport size to normalize across DPI settings.
    save_rgba_png(
        path,
        w,
        h,
        pixels,
        Some((VIEWPORT_W as u32, VIEWPORT_H as u32)),
    )
}

// ─── Widget building ────────────────────────────────────────────────────────

/// Layout context a node inherits from its parent.
#[derive(Clone, Copy)]
struct Ctx {
    /// Parent's main axis is horizontal.
    parent_is_row: bool,
    /// Parent has `align-items: stretch`.
    parent_stretch: bool,
    is_root: bool,
    /// Main-axis size assigned by the parent's flex-grow distribution.
    main_override: Option<f32>,
    /// This node or an ancestor is `visible: false`: keep its space, paint nothing.
    hidden: bool,
    /// `auto`-sized leaves fit their label (as an `egui::Frame` without a
    /// min size does) instead of the 60×40 flex fallback. Used for grid cells.
    content_sized: bool,
}

impl Ctx {
    fn root() -> Self {
        Self {
            parent_is_row: true,
            parent_stretch: false,
            is_root: true,
            main_override: None,
            hidden: false,
            content_sized: false,
        }
    }
}

/// Allocate space for a widget, applying align-self positioning if needed.
///
/// When `align_self` overrides the parent's cross-axis alignment, we allocate
/// the full cross-axis space and position the child within it.
fn allocate_with_align_self(
    ui: &mut egui::Ui,
    outer_size: Vec2,
    align_self: AlignSelf,
    parent_is_row: bool,
) -> egui::Rect {
    match align_self {
        AlignSelf::Center
        | AlignSelf::FlexStart
        | AlignSelf::Start
        | AlignSelf::FlexEnd
        | AlignSelf::End => {
            if parent_is_row {
                let cross_avail = ui.available_height();
                let alloc_size = Vec2::new(outer_size.x, cross_avail);
                let (alloc_rect, _) = ui.allocate_exact_size(alloc_size, egui::Sense::hover());

                let y_offset = match align_self {
                    AlignSelf::Center => (cross_avail - outer_size.y) / 2.0,
                    AlignSelf::FlexEnd | AlignSelf::End => cross_avail - outer_size.y,
                    _ => 0.0,
                };

                egui::Rect::from_min_size(alloc_rect.min + Vec2::new(0.0, y_offset), outer_size)
            } else {
                let cross_avail = ui.available_width();
                let alloc_size = Vec2::new(cross_avail, outer_size.y);
                let (alloc_rect, _) = ui.allocate_exact_size(alloc_size, egui::Sense::hover());

                let x_offset = match align_self {
                    AlignSelf::Center => (cross_avail - outer_size.x) / 2.0,
                    AlignSelf::FlexEnd | AlignSelf::End => cross_avail - outer_size.x,
                    _ => 0.0,
                };

                egui::Rect::from_min_size(alloc_rect.min + Vec2::new(x_offset, 0.0), outer_size)
            }
        }
        _ => ui.allocate_exact_size(outer_size, egui::Sense::hover()).0,
    }
}

/// Get the fixed main-axis size of a child (0.0 if the child uses flex-grow).
/// Hidden children still take up space (`visibility: hidden` semantics).
fn child_fixed_main(child: &NodeConfig, is_row: bool) -> f32 {
    let dim = if is_row { &child.width } else { &child.height };
    let size = resolve_to_px(dim, if is_row { VIEWPORT_W } else { VIEWPORT_H });
    let margin = resolve_to_px(&child.margin.first(), 0.0) * 2.0;
    // If child has flex-grow and no explicit main-axis size, it's flexible
    if child.flex_grow > 0.0 && size == 0.0 {
        return 0.0;
    }
    let auto_size = if child.children.is_empty() {
        if is_row { 60.0 } else { 40.0 }
    } else {
        0.0
    };
    let s = if size > 0.0 { size } else { auto_size };
    s + margin
}

fn build_widget(
    ui: &mut egui::Ui,
    node: &NodeConfig,
    leaf_idx: &mut usize,
    palette: ColorPalette,
    ctx: Ctx,
) {
    let ctx = Ctx {
        hidden: ctx.hidden || !node.visible,
        ..ctx
    };
    if node.children.is_empty() {
        build_leaf(ui, node, leaf_idx, palette, ctx);
    } else {
        build_container(ui, node, leaf_idx, palette, ctx);
    }
}

fn build_leaf(
    ui: &mut egui::Ui,
    node: &NodeConfig,
    leaf_idx: &mut usize,
    palette: ColorPalette,
    ctx: Ctx,
) {
    let (r, g, b) = palette_rgb8(palette, *leaf_idx);
    *leaf_idx += 1;
    let bg = Color32::from_rgb(r, g, b);

    let w = resolve_to_px(&node.width, VIEWPORT_W);
    let h = resolve_to_px(&node.height, VIEWPORT_H);
    let padding = resolve_to_px(&node.padding.first(), 0.0);
    let margin = resolve_to_px(&node.margin.first(), 0.0);
    let font = egui::FontId::proportional(26.0);
    let (auto_w, auto_h) = if ctx.content_sized {
        let text = ui
            .painter()
            .layout_no_wrap(node.display_text().to_owned(), font.clone(), Color32::WHITE)
            .size();
        (text.x + padding * 2.0, text.y + padding * 2.0)
    } else {
        (60.0, 40.0)
    };

    // Apply max constraints
    let max_w = match node.max_width {
        ValueConfig::Px(n) if n > 0.0 => Some(n),
        _ => None,
    };
    let max_h = match node.max_height {
        ValueConfig::Px(n) if n > 0.0 => Some(n),
        _ => None,
    };

    // Determine effective size, applying main_override for flex-grow
    let mut eff_w;
    let mut eff_h;

    if ctx.parent_is_row {
        eff_w = ctx.main_override.unwrap_or_else(|| {
            if node.flex_grow > 0.0 && w == 0.0 {
                (ui.available_width() - margin * 2.0).max(0.0)
            } else if w > 0.0 {
                w
            } else {
                auto_w
            }
        });
        eff_h = if ctx.parent_stretch && h == 0.0 {
            (ui.available_height() - margin * 2.0).max(0.0)
        } else if h > 0.0 {
            h
        } else {
            auto_h
        };
    } else {
        eff_w = if ctx.parent_stretch && w == 0.0 {
            (ui.available_width() - margin * 2.0).max(0.0)
        } else if w > 0.0 {
            w
        } else {
            auto_w
        };
        eff_h = ctx.main_override.unwrap_or_else(|| {
            if node.flex_grow > 0.0 && h == 0.0 {
                (ui.available_height() - margin * 2.0).max(0.0)
            } else if h > 0.0 {
                h
            } else {
                auto_h
            }
        });
    }

    if let Some(mw) = max_w {
        eff_w = eff_w.min(mw);
    }
    if let Some(mh) = max_h {
        eff_h = eff_h.min(mh);
    }

    let outer_size = Vec2::new(eff_w + margin * 2.0, eff_h + margin * 2.0);
    let outer_rect = allocate_with_align_self(ui, outer_size, node.align_self, ctx.parent_is_row);
    if ctx.hidden {
        return;
    }
    let inner_rect = outer_rect.shrink(margin);

    ui.painter().rect_filled(inner_rect, 0.0, bg);

    let text_rect = inner_rect.shrink(padding);
    ui.painter().text(
        text_rect.center(),
        egui::Align2::CENTER_CENTER,
        node.display_text(),
        font,
        Color32::from_rgba_premultiplied(13, 13, 26, 217),
    );
}

fn build_container(
    ui: &mut egui::Ui,
    node: &NodeConfig,
    leaf_idx: &mut usize,
    palette: ColorPalette,
    ctx: Ctx,
) {
    // Grid containers become an `egui::Grid` (see `codegen/egui.rs`), whose
    // children are laid out in rows.
    let is_grid = node.display_mode == DisplayMode::Grid;
    let is_row = is_grid
        || matches!(
            node.flex_direction,
            FlexDirection::Row | FlexDirection::RowReverse
        );
    let stretch = node.align_items == AlignItems::Stretch;
    let wraps = matches!(node.flex_wrap, FlexWrap::Wrap | FlexWrap::WrapReverse);

    let main_dir = match node.flex_direction {
        FlexDirection::Row => egui::Direction::LeftToRight,
        FlexDirection::RowReverse => egui::Direction::RightToLeft,
        FlexDirection::Column => egui::Direction::TopDown,
        FlexDirection::ColumnReverse => egui::Direction::BottomUp,
    };

    let cross_align = match node.align_items {
        AlignItems::FlexStart | AlignItems::Start | AlignItems::Default => Align::Min,
        AlignItems::FlexEnd | AlignItems::End => Align::Max,
        AlignItems::Center => Align::Center,
        AlignItems::Baseline => Align::Min,
        AlignItems::Stretch => Align::Min,
    };

    let mut layout = Layout::from_main_dir_and_cross_align(main_dir, cross_align);
    if stretch {
        layout = layout.with_cross_justify(true);
    }
    if wraps {
        layout = layout.with_main_wrap(true);
    }

    let jc = effective_justify(
        node.justify_content,
        matches!(
            node.flex_direction,
            FlexDirection::RowReverse | FlexDirection::ColumnReverse
        ),
    );
    let main_gap = if is_row {
        resolve_to_px(&node.column_gap, 0.0)
    } else {
        resolve_to_px(&node.row_gap, 0.0)
    };

    let padding = resolve_to_px(&node.padding.first(), 0.0);
    let margin = resolve_to_px(&node.margin.first(), 0.0);
    let w = resolve_to_px(&node.width, VIEWPORT_W);
    let h = resolve_to_px(&node.height, VIEWPORT_H);
    let bg = Color32::from_rgb(28, 28, 43);

    // Determine container size — always compute both dimensions so that
    // egui's layout (main_justify, etc.) has room to distribute items.
    // Use available space as fallback for auto dimensions.
    let avail_w = (ui.available_width() - margin * 2.0).max(0.0);
    let avail_h = (ui.available_height() - margin * 2.0).max(0.0);

    let container_w = if ctx.is_root {
        VIEWPORT_W
    } else if ctx.parent_is_row {
        ctx.main_override
            .unwrap_or(if w > 0.0 { w } else { avail_w })
    } else if w > 0.0 {
        w
    } else {
        avail_w
    };

    let container_h = if ctx.is_root {
        VIEWPORT_H
    } else if !ctx.parent_is_row {
        ctx.main_override
            .unwrap_or(if h > 0.0 { h } else { avail_h })
    } else if h > 0.0 {
        h
    } else {
        avail_h
    };

    let inner_w = (container_w - padding * 2.0).max(0.0);
    let inner_h = (container_h - padding * 2.0).max(0.0);

    // Reserve space including margin
    let outer_size = Vec2::new(container_w + margin * 2.0, container_h + margin * 2.0);
    let outer_rect = allocate_with_align_self(ui, outer_size, node.align_self, ctx.parent_is_row);
    let inner_rect = outer_rect.shrink(margin + padding);

    // Paint background (hidden subtrees keep their space but paint nothing)
    if !ctx.hidden {
        let bg_rect = outer_rect.shrink(margin);
        ui.painter().rect_filled(bg_rect, 0.0, bg);
    }

    // Create a child UI within the inner rect
    let mut child_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner_rect)
            .layout(Layout::top_down(Align::Min)),
    );
    child_ui.set_min_size(Vec2::new(inner_w, inner_h));

    // Sort children by order
    let mut children: Vec<&NodeConfig> = node.children.iter().collect();
    children.sort_by_key(|c| c.order);

    if is_grid {
        // Mirror `codegen/egui.rs`: `egui::Grid::new(label).num_columns(n)`
        // with `item_spacing = (column_gap, row_gap)` and `end_row()` after
        // every n children. Spans, template rows and auto-flow are ignored
        // there, so they are ignored here too.
        let num_cols = node.grid_template_columns.len().max(1);
        let spacing = Vec2::new(
            resolve_to_px(&node.column_gap, 0.0),
            resolve_to_px(&node.row_gap, 0.0),
        );
        child_ui.spacing_mut().item_spacing = spacing;
        egui::Grid::new((node.label.as_str(), *leaf_idx))
            .num_columns(num_cols)
            .spacing(spacing)
            .show(&mut child_ui, |ui| {
                for (i, child) in children.iter().enumerate() {
                    build_widget(
                        ui,
                        child,
                        leaf_idx,
                        palette,
                        Ctx {
                            parent_is_row: true,
                            parent_stretch: stretch,
                            is_root: false,
                            main_override: None,
                            hidden: ctx.hidden,
                            content_sized: true,
                        },
                    );
                    if (i + 1) % num_cols == 0 {
                        ui.end_row();
                    }
                }
            });
        return;
    }

    // Pre-compute flex-grow distribution: calculate how much main-axis space
    // each flex-grow child gets, so they don't greedily consume everything.
    // Hidden children participate like visible ones (visibility: hidden).
    let grows_on_main = |c: &NodeConfig| {
        let dim = if is_row { &c.width } else { &c.height };
        c.flex_grow > 0.0 && resolve_to_px(dim, if is_row { VIEWPORT_W } else { VIEWPORT_H }) == 0.0
    };
    let num_gaps = children.len().saturating_sub(1);
    let total_gap = num_gaps as f32 * main_gap;
    let total_fixed: f32 = children.iter().map(|c| child_fixed_main(c, is_row)).sum();
    let total_grow: f32 = children
        .iter()
        .filter(|c| grows_on_main(c))
        .map(|c| c.flex_grow)
        .sum();
    let main_axis_total = if is_row { inner_w } else { inner_h };
    let remaining_for_grow = (main_axis_total - total_fixed - total_gap).max(0.0);

    // justify-content: space-* — distribute the free main-axis space the way
    // CSS does (as a leading offset plus extra spacing between items).
    // egui's `Layout::with_main_justify` is not usable for this: it stretches
    // every item to the full axis and pushes the rest off-screen.
    let free = if total_grow > 0.0 || wraps {
        0.0
    } else {
        remaining_for_grow
    };
    let n = children.len() as f32;
    let (leading, extra_between) = match jc {
        JustifyContent::SpaceBetween if n > 1.0 => (0.0, free / (n - 1.0)),
        JustifyContent::SpaceAround if n > 0.0 => (free / (2.0 * n), free / n),
        JustifyContent::SpaceEvenly if n > 0.0 => (free / (n + 1.0), free / (n + 1.0)),
        _ => (0.0, 0.0),
    };

    // Set gap
    let spacing = main_gap + extra_between;
    if is_row {
        child_ui.spacing_mut().item_spacing = Vec2::new(spacing, 0.0);
    } else {
        child_ui.spacing_mut().item_spacing = Vec2::new(0.0, spacing);
    }

    child_ui.with_layout(layout, |ui| {
        if leading > 0.0 {
            ui.add_space(leading);
        }
        for child in &children {
            // Calculate main-axis override for flex-grow children
            let main_override = if total_grow > 0.0 && grows_on_main(child) {
                let margin = resolve_to_px(&child.margin.first(), 0.0) * 2.0;
                Some((remaining_for_grow * child.flex_grow / total_grow - margin).max(0.0))
            } else {
                None
            };
            build_widget(
                ui,
                child,
                leaf_idx,
                palette,
                Ctx {
                    parent_is_row: is_row,
                    parent_stretch: stretch,
                    is_root: false,
                    main_override,
                    hidden: ctx.hidden,
                    content_sized: false,
                },
            );
        }
    });
}
