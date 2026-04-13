//! Headless-ish egui renderer for flexplore golden tests.
//!
//! Reads `testdata/{case}/input.json`, renders the layout with egui via eframe,
//! captures a screenshot via `ViewportCommand::Screenshot`, and saves
//! `rendered_egui.png`.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result};
use clap::Parser;
use eframe::egui;
use egui::{Align, Color32, Layout, Vec2};

mod config;
use config::{
    AlignItems, AlignSelf, ColorPalette, FlexDirection, FlexWrap, JustifyContent, LayoutInput,
    NodeConfig, ValueConfig,
};

/// Render flexplore golden screenshots with egui.
#[derive(Parser)]
#[command(name = "egui-golden")]
struct Arguments {
    /// Path to the testdata directory.
    #[arg(default_value = "testdata")]
    testdata: PathBuf,

    /// Only render these test cases (default: all).
    cases: Vec<String>,
}

const VIEWPORT_W: f32 = 400.0;
const VIEWPORT_H: f32 = 300.0;

// ─── Application state ──────────────────────────────────────────────────────

struct App {
    jobs: Vec<RenderJob>,
    current: usize,
    frames: usize,
    screenshot_requested: bool,
}

struct RenderJob {
    name: String,
    node: NodeConfig,
    palette: ColorPalette,
    output_dir: PathBuf,
}

fn main() -> eframe::Result {
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

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([VIEWPORT_W, VIEWPORT_H])
            .with_resizable(false),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    eframe::run_native(
        "egui-golden",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(App {
                jobs,
                current: 0,
                frames: 0,
                screenshot_requested: false,
            }))
        }),
    )
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.current >= self.jobs.len() {
            eprintln!("All done!");
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        // Check for screenshot result from previous request
        if self.screenshot_requested {
            let screenshot: Option<Arc<egui::ColorImage>> = ctx.input(|i| {
                i.raw.events.iter().find_map(|e| {
                    if let egui::Event::Screenshot { image, .. } = e {
                        Some(image.clone())
                    } else {
                        None
                    }
                })
            });

            if let Some(image) = screenshot {
                let job = &self.jobs[self.current];
                let path = job.output_dir.join(&job.name).join("rendered_egui.png");
                save_color_image(&image, &path);

                self.current += 1;
                self.frames = 0;
                self.screenshot_requested = false;

                if self.current < self.jobs.len() {
                    eprintln!("Rendering: {}", self.jobs[self.current].name);
                }

                ctx.request_repaint();
                return;
            }
        }

        let job = &self.jobs[self.current];

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(28, 28, 43))
                    .inner_margin(0.0),
            )
            .show(ctx, |ui| {
                ui.set_min_size(Vec2::new(VIEWPORT_W, VIEWPORT_H));
                let mut leaf_idx = 0;
                build_widget(ui, &job.node, &mut leaf_idx, job.palette, true, false, true);
            });

        self.frames += 1;

        // After settling, request screenshot
        if self.frames == 4 && !self.screenshot_requested {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
            self.screenshot_requested = true;
        }

        ctx.request_repaint();
    }
}

// ─── Screenshot saving ──────────────────────────────────────────────────────

fn save_color_image(image: &egui::ColorImage, path: &Path) {
    let w = image.size[0] as u32;
    let h = image.size[1] as u32;
    let pixels: Vec<u8> = image
        .pixels
        .iter()
        .flat_map(|c| [c.r(), c.g(), c.b(), c.a()])
        .collect();

    let Some(img) = image::RgbaImage::from_raw(w, h, pixels) else {
        eprintln!("  ERROR: could not create image from screenshot");
        return;
    };

    // Resize to target viewport size to normalize across DPI settings
    let target_w = VIEWPORT_W as u32;
    let target_h = VIEWPORT_H as u32;
    let final_img = if w != target_w || h != target_h {
        image::imageops::resize(
            &img,
            target_w,
            target_h,
            image::imageops::FilterType::Lanczos3,
        )
    } else {
        img
    };

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    match final_img.save(path) {
        Ok(()) => eprintln!("  Saved: {}", path.display()),
        Err(e) => eprintln!("  ERROR saving {}: {e}", path.display()),
    }
}

// ─── Job loading ────────────────────────────────────────────────────────────

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

// ─── Widget building ────────────────────────────────────────────────────────

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
fn child_fixed_main(child: &NodeConfig, is_row: bool) -> f32 {
    if !child.visible {
        return 0.0;
    }
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
    parent_is_row: bool,
    parent_stretch: bool,
    is_root: bool,
) {
    build_widget_sized(
        ui,
        node,
        leaf_idx,
        palette,
        parent_is_row,
        parent_stretch,
        is_root,
        None,
    );
}

/// Build a widget with an optional override for its main-axis size (used for flex-grow distribution).
fn build_widget_sized(
    ui: &mut egui::Ui,
    node: &NodeConfig,
    leaf_idx: &mut usize,
    palette: ColorPalette,
    parent_is_row: bool,
    parent_stretch: bool,
    is_root: bool,
    main_override: Option<f32>,
) {
    if !node.visible {
        *leaf_idx += leaf_count(node);
        return;
    }

    if node.children.is_empty() {
        build_leaf(
            ui,
            node,
            leaf_idx,
            palette,
            parent_is_row,
            parent_stretch,
            main_override,
        );
    } else {
        build_container(
            ui,
            node,
            leaf_idx,
            palette,
            parent_is_row,
            parent_stretch,
            is_root,
            main_override,
        );
    }
}

fn build_leaf(
    ui: &mut egui::Ui,
    node: &NodeConfig,
    leaf_idx: &mut usize,
    palette: ColorPalette,
    parent_is_row: bool,
    parent_stretch: bool,
    main_override: Option<f32>,
) {
    let (r, g, b) = palette_color(palette, *leaf_idx);
    *leaf_idx += 1;
    let bg = Color32::from_rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8);

    let w = resolve_to_px(&node.width, VIEWPORT_W);
    let h = resolve_to_px(&node.height, VIEWPORT_H);
    let padding = resolve_to_px(&node.padding.first(), 0.0);
    let margin = resolve_to_px(&node.margin.first(), 0.0);

    // Apply max constraints
    let max_w = if let ValueConfig::Px(n) = node.max_width {
        if n > 0.0 { Some(n) } else { None }
    } else {
        None
    };
    let max_h = if let ValueConfig::Px(n) = node.max_height {
        if n > 0.0 { Some(n) } else { None }
    } else {
        None
    };

    // Determine effective size, applying main_override for flex-grow
    let mut eff_w;
    let mut eff_h;

    if parent_is_row {
        eff_w = main_override.unwrap_or_else(|| {
            if node.flex_grow > 0.0 && w == 0.0 {
                (ui.available_width() - margin * 2.0).max(0.0)
            } else if parent_stretch && !parent_is_row && w == 0.0 {
                (ui.available_width() - margin * 2.0).max(0.0)
            } else if w > 0.0 {
                w
            } else {
                60.0
            }
        });
        eff_h = if node.flex_grow > 0.0 && !parent_is_row && h == 0.0 {
            (ui.available_height() - margin * 2.0).max(0.0)
        } else if parent_stretch && h == 0.0 {
            (ui.available_height() - margin * 2.0).max(0.0)
        } else if h > 0.0 {
            h
        } else {
            40.0
        };
    } else {
        eff_w = if node.flex_grow > 0.0 && parent_is_row && w == 0.0 {
            (ui.available_width() - margin * 2.0).max(0.0)
        } else if parent_stretch && w == 0.0 {
            (ui.available_width() - margin * 2.0).max(0.0)
        } else if w > 0.0 {
            w
        } else {
            60.0
        };
        eff_h = main_override.unwrap_or_else(|| {
            if node.flex_grow > 0.0 && h == 0.0 {
                (ui.available_height() - margin * 2.0).max(0.0)
            } else if parent_stretch && parent_is_row && h == 0.0 {
                (ui.available_height() - margin * 2.0).max(0.0)
            } else if h > 0.0 {
                h
            } else {
                40.0
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
    let outer_rect = allocate_with_align_self(ui, outer_size, node.align_self, parent_is_row);
    let inner_rect = outer_rect.shrink(margin);

    ui.painter().rect_filled(inner_rect, 0.0, bg);

    let text_rect = inner_rect.shrink(padding);
    ui.painter().text(
        text_rect.center(),
        egui::Align2::CENTER_CENTER,
        &node.label,
        egui::FontId::proportional(26.0),
        Color32::from_rgba_premultiplied(13, 13, 26, 217),
    );
}

fn build_container(
    ui: &mut egui::Ui,
    node: &NodeConfig,
    leaf_idx: &mut usize,
    palette: ColorPalette,
    parent_is_row: bool,
    parent_stretch: bool,
    is_root: bool,
    main_override: Option<f32>,
) {
    let is_row = matches!(
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
        &node.justify_content,
        matches!(
            node.flex_direction,
            FlexDirection::RowReverse | FlexDirection::ColumnReverse
        ),
    );
    if matches!(
        jc,
        JustifyContent::SpaceBetween | JustifyContent::SpaceEvenly | JustifyContent::SpaceAround
    ) {
        layout = layout.with_main_justify(true);
    }

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

    let container_w = if is_root {
        VIEWPORT_W
    } else if parent_is_row {
        main_override.unwrap_or({
            if w > 0.0 {
                w
            } else if node.flex_grow > 0.0 {
                avail_w
            } else {
                avail_w
            }
        })
    } else if w > 0.0 {
        w
    } else if parent_stretch && !parent_is_row {
        avail_w
    } else {
        avail_w
    };

    let container_h = if is_root {
        VIEWPORT_H
    } else if !parent_is_row {
        main_override.unwrap_or({
            if h > 0.0 {
                h
            } else if node.flex_grow > 0.0 {
                avail_h
            } else {
                avail_h
            }
        })
    } else if h > 0.0 {
        h
    } else if parent_stretch && parent_is_row {
        avail_h
    } else {
        avail_h
    };

    let inner_w = (container_w - padding * 2.0).max(0.0);
    let inner_h = (container_h - padding * 2.0).max(0.0);

    // Reserve space including margin
    let outer_size = Vec2::new(container_w + margin * 2.0, container_h + margin * 2.0);
    let outer_rect = allocate_with_align_self(ui, outer_size, node.align_self, parent_is_row);
    let inner_rect = outer_rect.shrink(margin + padding);

    // Paint background
    let bg_rect = outer_rect.shrink(margin);
    ui.painter().rect_filled(bg_rect, 0.0, bg);

    // Create a child UI within the inner rect
    let mut child_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner_rect)
            .layout(Layout::top_down(Align::Min)),
    );
    child_ui.set_min_size(Vec2::new(inner_w, inner_h));

    // Set gap
    if is_row {
        child_ui.spacing_mut().item_spacing = Vec2::new(main_gap, 0.0);
    } else {
        child_ui.spacing_mut().item_spacing = Vec2::new(0.0, main_gap);
    }

    // Sort children by order
    let mut children: Vec<&NodeConfig> = node.children.iter().collect();
    children.sort_by_key(|c| c.order);

    // Pre-compute flex-grow distribution: calculate how much main-axis space
    // each flex-grow child gets, so they don't greedily consume everything.
    let visible: Vec<&&NodeConfig> = children.iter().filter(|c| c.visible).collect();
    let num_gaps = if visible.len() > 1 {
        visible.len() - 1
    } else {
        0
    };
    let total_gap = num_gaps as f32 * main_gap;
    let total_fixed: f32 = visible.iter().map(|c| child_fixed_main(c, is_row)).sum();
    let total_grow: f32 = visible
        .iter()
        .filter(|c| {
            let dim = if is_row { &c.width } else { &c.height };
            c.flex_grow > 0.0
                && resolve_to_px(dim, if is_row { VIEWPORT_W } else { VIEWPORT_H }) == 0.0
        })
        .map(|c| c.flex_grow)
        .sum();
    let main_axis_total = if is_row { inner_w } else { inner_h };
    let remaining_for_grow = (main_axis_total - total_fixed - total_gap).max(0.0);

    child_ui.with_layout(layout, |ui| {
        for child in &children {
            // Calculate main-axis override for flex-grow children
            let main_override = if child.visible && child.flex_grow > 0.0 && total_grow > 0.0 {
                let dim = if is_row { &child.width } else { &child.height };
                if resolve_to_px(dim, if is_row { VIEWPORT_W } else { VIEWPORT_H }) == 0.0 {
                    let margin = resolve_to_px(&child.margin.first(), 0.0) * 2.0;
                    Some((remaining_for_grow * child.flex_grow / total_grow - margin).max(0.0))
                } else {
                    None
                }
            } else {
                None
            };
            build_widget_sized(
                ui,
                child,
                leaf_idx,
                palette,
                is_row,
                stretch,
                false,
                main_override,
            );
        }
    });
}

/// When direction is reversed, flex-start/end swap.
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

fn resolve_to_px(v: &ValueConfig, parent_px: f32) -> f32 {
    match v {
        ValueConfig::Auto => 0.0,
        ValueConfig::Px(n) => *n,
        ValueConfig::Percent(n) => n / 100.0 * parent_px,
        ValueConfig::Vw(n) => n / 100.0 * VIEWPORT_W,
        ValueConfig::Vh(n) => n / 100.0 * VIEWPORT_H,
    }
}

fn leaf_count(node: &NodeConfig) -> usize {
    if node.children.is_empty() {
        1
    } else {
        node.children.iter().map(leaf_count).sum()
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
