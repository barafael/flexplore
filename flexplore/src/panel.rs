use bevy::prelude::{Local, ResMut, Result, Vec2};
use bevy_egui::{EguiContexts, egui};
use strum::IntoEnumIterator;

use crate::history::UndoHistory;
use crate::viz::ArrowNav;
use flexplore::codegen::{
    emit_bevy_code, emit_dioxus, emit_egui, emit_flutter, emit_html_css, emit_iced, emit_react,
    emit_react_native, emit_swiftui, emit_tailwind,
};
use flexplore::config::*;

type TemplateFn = fn() -> NodeConfig;
type CodegenFn = fn(&NodeConfig, ColorPalette) -> anyhow::Result<String>;

const CODEGEN_TARGETS: &[(&str, CodegenFn)] = &[
    ("Bevy", emit_bevy_code),
    ("HTML/CSS", emit_html_css),
    ("Tailwind", emit_tailwind),
    ("React", emit_react),
    ("SwiftUI", emit_swiftui),
    ("Flutter", emit_flutter),
    ("Iced", emit_iced),
    ("egui", emit_egui),
    ("React Native", emit_react_native),
    ("Dioxus", emit_dioxus),
];

// ─── Left-panel tab ──────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum LeftTab {
    Flexbox,
    CssGrid,
}

// ─── Codegen right panel state ───────────────────────────────────────────────

pub(crate) struct CodegenState {
    framework_idx: usize,
    preview_open: bool,
    cached_code: String,
    dirty: bool,
}

impl Default for CodegenState {
    fn default() -> Self {
        Self {
            framework_idx: 0,
            preview_open: false,
            cached_code: String::new(),
            dirty: true,
        }
    }
}

// ─── Drag-to-reorder state ──────────────────────────────────────────────────

#[derive(Default)]
pub(crate) struct DragState {
    /// Path of the node being dragged.
    dragging: Option<Vec<usize>>,
    /// (parent_path, child_index) where the node would be inserted.
    drop_target: Option<(Vec<usize>, usize)>,
}

// ─── Tree UI helper ───────────────────────────────────────────────────────────

fn draw_tree_ui(
    ui: &mut egui::Ui,
    node: &mut NodeConfig,
    path: &mut Vec<usize>,
    selected: &[usize],
    changed: &mut bool,
    drag: &mut DragState,
) -> (Option<Vec<usize>>, bool) {
    let mut clicked = None;
    let mut remove = false;
    let is_selected = path.as_slice() == selected;
    let is_root = path.is_empty();

    // ── Drop target indicator (before this node) ──
    let is_drop_here = drag.drop_target.as_ref().is_some_and(|(dp, di)| {
        if let Some(last) = path.last() {
            !path.is_empty() && &path[..path.len() - 1] == dp.as_slice() && *di == *last
        } else {
            false
        }
    });
    if is_drop_here {
        ui.horizontal(|ui| {
            ui.add_space(path.len() as f32 * 14.0);
            let (rect, _) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), 2.0),
                egui::Sense::hover(),
            );
            ui.painter()
                .rect_filled(rect, 0.0, egui::Color32::from_rgb(0x60, 0xA0, 0xFF));
        });
    }

    let row_resp = ui.horizontal(|ui| {
        ui.add_space(path.len() as f32 * 14.0);
        let icon = if node.children.is_empty() {
            "□"
        } else {
            "▣"
        };
        if is_selected {
            let _ = ui.selectable_label(true, icon);
            let r = ui.add(egui::TextEdit::singleline(&mut node.label).desired_width(80.0));
            if r.changed() {
                *changed = true;
            }
            if !is_root && ui.small_button("x").clicked() {
                remove = true;
            }
        } else if ui
            .selectable_label(false, format!("{icon} {}", node.label))
            .clicked()
        {
            clicked = Some(path.clone());
        }
    });

    // ── Drag source / drop target logic ──
    let row_rect = row_resp.response.rect;
    let pointer = ui.ctx().input(|i| i.pointer.hover_pos());
    if !is_root {
        // Start drag on this row
        if ui.ctx().input(|i| i.pointer.any_pressed()) && row_rect.contains(ui.ctx().input(|i| i.pointer.press_origin().unwrap_or_default())) && drag.dragging.is_none() {
            // Only start drag after a small movement
            if let Some(pos) = pointer {
                if let Some(origin) = ui.ctx().input(|i| i.pointer.press_origin()) {
                    if (pos - origin).length() > 4.0 {
                        drag.dragging = Some(path.clone());
                    }
                }
            }
        }

        // This row is a potential drop target
        if drag.dragging.is_some() && drag.dragging.as_deref() != Some(path.as_slice()) {
            if let Some(pos) = pointer {
                if row_rect.contains(pos) {
                    let parent_path = path[..path.len() - 1].to_vec();
                    let idx = *path.last().unwrap();
                    // Drop above or below based on pointer position relative to row center
                    let insert_idx = if pos.y < row_rect.center().y { idx } else { idx + 1 };
                    drag.drop_target = Some((parent_path, insert_idx));
                }
            }
        }
    }

    for i in 0..node.children.len() {
        path.push(i);
        let (r, rem) = draw_tree_ui(ui, &mut node.children[i], path, selected, changed, drag);
        path.pop();
        if r.is_some() {
            clicked = r;
        }
        if rem {
            remove = true;
        }
    }

    // ── Drop target indicator (after last child, at end) ──
    if !node.children.is_empty() {
        let after_last = drag.drop_target.as_ref().is_some_and(|(dp, di)| {
            dp.as_slice() == path.as_slice() && *di == node.children.len()
        });
        if after_last {
            ui.horizontal(|ui| {
                ui.add_space((path.len() + 1) as f32 * 14.0);
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 2.0),
                    egui::Sense::hover(),
                );
                ui.painter()
                    .rect_filled(rect, 0.0, egui::Color32::from_rgb(0x60, 0xA0, 0xFF));
            });
        }
    }

    (clicked, remove)
}

// ─── Hover preview ────────────────────────────────────────────────────────────

fn apply_hover<T: PartialEq + Clone>(
    opt: Option<T>,
    cfg: &mut FlexConfig,
    preview: &mut Option<FlexConfig>,
    path: &[usize],
    get: impl Fn(&NodeConfig) -> T,
    set: impl FnOnce(&mut NodeConfig, T),
) -> bool {
    let Some(v) = opt else { return false };
    let Some(node) = cfg.root.get(path) else {
        return false;
    };
    if get(node) != v {
        if preview.is_none() {
            *preview = Some(cfg.clone());
        }
        if let Some(node) = cfg.root.get_mut(path) {
            set(node, v);
        }
        true
    } else {
        false
    }
}

// ─── Panel system ─────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub fn panel_system(
    mut contexts: EguiContexts,
    mut cfg: ResMut<FlexConfig>,
    mut history: ResMut<UndoHistory>,
    mut arrow_nav: ResMut<ArrowNav>,
    mut right_panel_open: ResMut<RightPanelOpen>,
    mut preview: Local<Option<FlexConfig>>,
    mut applied_theme: Local<Option<Theme>>,
    mut import_buf: Local<String>,
    mut toast: Local<Option<(String, f64)>>,
    mut left_tab: Local<Option<LeftTab>>,
    mut codegen: Local<Option<CodegenState>>,
    mut drag: Local<DragState>,
    mut show_help: Local<bool>,
    #[cfg(feature = "multiplayer")] mut pending_edits: Option<ResMut<crate::net::PendingEdits>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    let tab = left_tab.get_or_insert(LeftTab::Flexbox);
    let cg = codegen.get_or_insert_with(CodegenState::default);
    let mut net_dirty = false;

    // ── Global shortcuts ──────────────────────────────────────────────────────
    let undo_pressed = ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Z));
    let redo_pressed = ctx.input_mut(|i| {
        i.consume_key(egui::Modifiers::COMMAND, egui::Key::Y)
            || i.consume_key(
                egui::Modifiers::COMMAND.plus(egui::Modifiers::SHIFT),
                egui::Key::Z,
            )
    });
    if undo_pressed && let Some(snapshot) = history.undo() {
        *cfg = snapshot.clone();
        cfg.request_rebuild();
        *preview = None;
        cg.dirty = true;
    }
    if redo_pressed && let Some(snapshot) = history.redo() {
        *cfg = snapshot.clone();
        cfg.request_rebuild();
        *preview = None;
        cg.dirty = true;
    }

    // ── Tree navigation shortcuts ─────────────────────────────────────────────
    let key_add_child =
        ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Enter));
    let key_add_sibling =
        ctx.input_mut(|i| i.consume_key(egui::Modifiers::SHIFT, egui::Key::Enter));
    let key_delete = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Delete));
    let key_parent = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
    let key_duplicate =
        ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::D));
    let key_save = ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::S));
    let key_open = ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::O));
    let key_help =
        ctx.input_mut(|i| i.consume_key(egui::Modifiers::SHIFT, egui::Key::Slash));

    // Arrow keys → spatial navigation
    let arrow_up = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp));
    let arrow_down = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown));
    let arrow_left = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft));
    let arrow_right =
        ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight));
    arrow_nav.0 = if arrow_up {
        Some(Vec2::new(0.0, -1.0))
    } else if arrow_down {
        Some(Vec2::new(0.0, 1.0))
    } else if arrow_left {
        Some(Vec2::new(-1.0, 0.0))
    } else if arrow_right {
        Some(Vec2::new(1.0, 0.0))
    } else {
        None
    };

    if *applied_theme != Some(cfg.theme) {
        apply_theme(ctx, cfg.theme);
        *applied_theme = Some(cfg.theme);
    }

    // ── File picker (Ctrl+S / Ctrl+O) ─────────────────────────────────────────
    if key_save {
        save_file(&cfg, &mut toast, ctx);
    }
    if key_open {
        if let Some(loaded) = open_file() {
            *cfg = loaded;
            cfg.request_rebuild();
            *preview = None;
            history.push(cfg.clone());
            cg.dirty = true;
            net_dirty = true;
        }
    }

    if key_help {
        *show_help = !*show_help;
    }

    let mut changed = false;
    let mut any_hovered = false;

    let mut hover_direction: Option<FlexDirection> = None;
    let mut hover_wrap: Option<FlexWrap> = None;
    let mut hover_justify: Option<JustifyContent> = None;
    let mut hover_align_items: Option<AlignItems> = None;
    let mut hover_align_content: Option<AlignContent> = None;
    let mut hover_row_gap: Option<ValueConfig> = None;
    let mut hover_column_gap: Option<ValueConfig> = None;
    let mut hover_width: Option<ValueConfig> = None;
    let mut hover_height: Option<ValueConfig> = None;
    let mut hover_min_width: Option<ValueConfig> = None;
    let mut hover_min_height: Option<ValueConfig> = None;
    let mut hover_max_width: Option<ValueConfig> = None;
    let mut hover_max_height: Option<ValueConfig> = None;
    let mut hover_basis: Option<ValueConfig> = None;
    let mut hover_align_self: Option<AlignSelf> = None;

    let mut sel_path = cfg.selected().to_vec();
    let mut is_root = sel_path.is_empty();

    // ── Apply tree navigation shortcuts ───────────────────────────────────────
    if !ctx.wants_keyboard_input() {
        if key_add_child {
            let n = cfg.root.count_leaves();
            let lbl = format!("node{}", n + 1);
            if let Some(node) = cfg.root.get_mut(&sel_path) {
                node.children.push(NodeConfig::new_leaf(&lbl, 80.0, 80.0));
                changed = true;
            }
        }
        if key_add_sibling && !is_root {
            let pidx = sel_path.len() - 1;
            let sibling_idx = sel_path[pidx];
            let n = cfg.root.count_leaves();
            let lbl = format!("node{}", n + 1);
            if let Some(parent) = cfg.root.get_mut(&sel_path[..pidx]) {
                let insert_at = (sibling_idx + 1).min(parent.children.len());
                parent
                    .children
                    .insert(insert_at, NodeConfig::new_leaf(&lbl, 80.0, 80.0));
                changed = true;
            }
        }
        if key_duplicate && !is_root {
            let pidx = sel_path.len() - 1;
            let sibling_idx = sel_path[pidx];
            if let Some(parent) = cfg.root.get_mut(&sel_path[..pidx]) {
                let clone = parent.children[sibling_idx].clone();
                let insert_at = (sibling_idx + 1).min(parent.children.len());
                parent.children.insert(insert_at, clone);
                changed = true;
            }
        }
        if key_delete && !is_root {
            let pidx = sel_path.len() - 1;
            let idx = sel_path[pidx];
            if let Some(parent) = cfg.root.get_mut(&sel_path[..pidx]) {
                parent.children.remove(idx);
            }
            sel_path.truncate(pidx);
            is_root = sel_path.is_empty();
            cfg.select(sel_path.clone());
            changed = true;
        }
        if key_parent && !is_root {
            sel_path.pop();
            is_root = sel_path.is_empty();
            cfg.select(sel_path.clone());
        }
    }

    // ══════════════════════════════════════════════════════════════════════════
    // LEFT PANEL
    // ══════════════════════════════════════════════════════════════════════════
    egui::SidePanel::left("flex_panel")
        .exact_width(PANEL_WIDTH)
        .resizable(false)
        .show_separator_line(false)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(4.0);

                // ── Toolbar ───────────────────────────────────────────────
                ui.horizontal(|ui| {
                    let mut hover_theme: Option<Theme> = None;
                    let theme_resp = egui::ComboBox::from_id_salt("theme_sel")
                        .selected_text(cfg.theme.to_string())
                        .width(90.0)
                        .show_ui(ui, |ui| {
                            for t in Theme::iter() {
                                let r = ui.selectable_label(cfg.theme == t, t.to_string());
                                if r.clicked() { cfg.theme = t; changed = true; }
                                else if r.hovered() { hover_theme = Some(t); }
                            }
                        });
                    if theme_resp.inner.is_some() { any_hovered = true; }
                    if let Some(t) = hover_theme {
                        any_hovered = true;
                        if cfg.theme != t {
                            if preview.is_none() { *preview = Some(cfg.clone()); }
                            cfg.theme = t;
                            *applied_theme = None;
                        }
                    }
                    ui.separator();
                    if ui.add_enabled(history.can_undo(), egui::Button::new("⟲ Undo")).clicked()
                        && let Some(snapshot) = history.undo()
                    {
                        *cfg = snapshot.clone();
                        cfg.request_rebuild();
                        *preview = None;
                        cg.dirty = true;
                    }
                    if ui.add_enabled(history.can_redo(), egui::Button::new("⟳ Redo")).clicked()
                        && let Some(snapshot) = history.redo()
                    {
                        *cfg = snapshot.clone();
                        cfg.request_rebuild();
                        *preview = None;
                        cg.dirty = true;
                    }
                    if ui.button("?").on_hover_text("Keyboard shortcuts (Shift+/)").clicked() {
                        *show_help = !*show_help;
                    }
                });
                ui.add_space(4.0);

                // Auto-sync tab to selected node's display mode (before
                // drawing widgets so the user's click can still override).
                if let Some(node) = cfg.root.get(&sel_path) {
                    *tab = match node.display_mode {
                        DisplayMode::Flex => LeftTab::Flexbox,
                        DisplayMode::Grid => LeftTab::CssGrid,
                    };
                }

                // ── Tabs: Flexbox / CSS Grid ──────────────────────────────
                ui.horizontal(|ui| {
                    ui.selectable_value(tab, LeftTab::Flexbox, "Flexbox");
                    ui.selectable_value(tab, LeftTab::CssGrid, "CSS Grid");
                });
                ui.separator();

                match *tab {
                    LeftTab::CssGrid | LeftTab::Flexbox => {
                        // When user clicks a tab, switch the selected node's display mode
                        let target_mode = match *tab {
                            LeftTab::CssGrid => DisplayMode::Grid,
                            LeftTab::Flexbox => DisplayMode::Flex,
                        };
                        if let Some(node) = cfg.root.get_mut(&sel_path) {
                            if node.display_mode != target_mode {
                                node.display_mode = target_mode;
                                changed = true;
                            }
                        }
                        draw_layout_panel(
                            ui,
                            &mut cfg,
                            &mut history,
                            &mut preview,
                            &mut import_buf,
                            &mut toast,
                            &mut changed,
                            &mut any_hovered,
                            &mut sel_path,
                            &mut is_root,
                            &mut hover_direction,
                            &mut hover_wrap,
                            &mut hover_justify,
                            &mut hover_align_items,
                            &mut hover_align_content,
                            &mut hover_row_gap,
                            &mut hover_column_gap,
                            &mut hover_width,
                            &mut hover_height,
                            &mut hover_min_width,
                            &mut hover_min_height,
                            &mut hover_max_width,
                            &mut hover_max_height,
                            &mut hover_basis,
                            &mut hover_align_self,
                            cg,
                            &mut drag,
                            &mut net_dirty,
                        );
                    }
                }
            });
        });

    // ══════════════════════════════════════════════════════════════════════════
    // RIGHT PANEL (codegen preview)
    // ══════════════════════════════════════════════════════════════════════════
    right_panel_open.0 = cg.preview_open;
    if cg.preview_open {
        egui::SidePanel::right("codegen_panel")
            .exact_width(RIGHT_PANEL_WIDTH)
            .resizable(false)
            .show_separator_line(false)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("Framework:");
                    egui::ComboBox::from_id_salt("cg_fw")
                        .selected_text(CODEGEN_TARGETS[cg.framework_idx].0)
                        .width(120.0)
                        .show_ui(ui, |ui| {
                            for (i, (name, _)) in CODEGEN_TARGETS.iter().enumerate() {
                                if ui.selectable_label(cg.framework_idx == i, *name).clicked() {
                                    cg.framework_idx = i;
                                    cg.dirty = true;
                                }
                            }
                        });
                    if ui.button("Copy").on_hover_text("Copy to clipboard").clicked() {
                        if !cg.cached_code.is_empty() {
                            ui.ctx().copy_text(cg.cached_code.clone());
                            let now = ui.ctx().input(|i| i.time);
                            *toast = Some(("Copied!".into(), now + 2.0));
                        }
                    }
                    if ui.button("x").on_hover_text("Close preview").clicked() {
                        cg.preview_open = false;
                    }
                });
                ui.separator();

                // Regenerate if dirty
                if cg.dirty {
                    let (_, emitter) = CODEGEN_TARGETS[cg.framework_idx];
                    cg.cached_code = match emitter(&cfg.root, cfg.palette) {
                        Ok(code) => code,
                        Err(e) => format!("Error: {e}"),
                    };
                    cg.dirty = false;
                }

                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let lang = crate::highlight::lang_for_framework(cg.framework_idx);
                        let font = egui::FontId::monospace(12.0);
                        let job = crate::highlight::highlight(&cg.cached_code, lang, font);
                        let response = ui.label(job);
                        // Allow text selection via right-click context menu
                        response.context_menu(|ui| {
                            if ui.button("Copy all").clicked() {
                                ui.ctx().copy_text(cg.cached_code.clone());
                                ui.close();
                            }
                        });
                    });
            });
    }

    // ── Commit or revert ──────────────────────────────────────────────────────
    if changed {
        *preview = None;
        cfg.request_rebuild();
        history.push(cfg.clone());
        cg.dirty = true;
        net_dirty = true;
    } else if !any_hovered && let Some(saved) = preview.take() {
        *cfg = saved;
        cfg.sanitize_selection();
        cfg.request_rebuild();
        *applied_theme = None;
        cg.dirty = true;
    }

    // ── Drag-to-reorder commit ────────────────────────────────────────────────
    if drag.dragging.is_some() && !ctx.input(|i| i.pointer.any_down()) {
        // Pointer released — commit the reorder
        if let (Some(src_path), Some((dst_parent, dst_idx))) =
            (drag.dragging.take(), drag.drop_target.take())
        {
            if !src_path.is_empty() {
                let src_parent = &src_path[..src_path.len() - 1];
                let src_idx = *src_path.last().unwrap();
                // Only reorder within the same parent for now
                if src_parent == dst_parent.as_slice() && src_idx != dst_idx {
                    if let Some(parent) = cfg.root.get_mut(src_parent) {
                        let node = parent.children.remove(src_idx);
                        let adjusted_idx = if dst_idx > src_idx {
                            (dst_idx - 1).min(parent.children.len())
                        } else {
                            dst_idx.min(parent.children.len())
                        };
                        parent.children.insert(adjusted_idx, node);
                        cfg.request_rebuild();
                        history.push(cfg.clone());
                        cg.dirty = true;
                        net_dirty = true;
                    }
                }
            }
        }
        drag.dragging = None;
        drag.drop_target = None;
    }
    if !ctx.input(|i| i.pointer.any_down()) {
        drag.dragging = None;
        drag.drop_target = None;
    }

    // ── Toast overlay ─────────────────────────────────────────────────────────
    if let Some((msg, expiry)) = &*toast {
        let now = ctx.input(|i| i.time);
        if now < *expiry {
            egui::Area::new(egui::Id::new("toast"))
                .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-16.0, -16.0))
                .show(ctx, |ui| {
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgb(0x30, 0x80, 0x40))
                        .corner_radius(4.0)
                        .inner_margin(egui::Margin::same(10))
                        .show(ui, |ui| {
                            ui.colored_label(egui::Color32::WHITE, msg);
                        });
                });
            ctx.request_repaint();
        } else {
            *toast = None;
        }
    }

    // ── Help overlay ──────────────────────────────────────────────────────────
    if *show_help {
        egui::Window::new("Keyboard Shortcuts")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                egui::Grid::new("help_grid")
                    .num_columns(2)
                    .spacing([20.0, 4.0])
                    .show(ui, |ui| {
                        let shortcuts = [
                            ("Ctrl+Z", "Undo"),
                            ("Ctrl+Y / Ctrl+Shift+Z", "Redo"),
                            ("Ctrl+Enter", "Add child node"),
                            ("Shift+Enter", "Add sibling node"),
                            ("Ctrl+D", "Duplicate selected node"),
                            ("Delete", "Delete selected node"),
                            ("Escape", "Select parent"),
                            ("Arrow keys", "Spatial navigation"),
                            ("Ctrl+S", "Save layout to file"),
                            ("Ctrl+O", "Open layout from file"),
                            ("Shift+/", "Toggle this help"),
                        ];
                        for (key, desc) in shortcuts {
                            ui.strong(key);
                            ui.label(desc);
                            ui.end_row();
                        }
                    });
                ui.add_space(8.0);
                if ui.button("Close").clicked() {
                    *show_help = false;
                }
            });
    }

    // ── Send accumulated edits to the network ─────────────────────────────────
    #[cfg(feature = "multiplayer")]
    if net_dirty {
        if let Some(ref mut edits) = pending_edits {
            edits.0.push(flexplore_protocol::LayoutEdit::ReplaceRoot(
                cfg.root.clone(),
            ));
            edits.0.push(flexplore_protocol::LayoutEdit::UpdateSettings {
                bg_mode: cfg.bg_mode,
                art_style: cfg.art_style,
                art_seed: cfg.art_seed,
                art_depth: cfg.art_depth,
                theme: cfg.theme,
                palette: cfg.palette,
            });
        }
    }

    #[cfg(not(feature = "multiplayer"))]
    let _ = net_dirty;

    Ok(())
}

// ─── Layout panel contents (shared by Flexbox and CSS Grid tabs) ─────────────

#[allow(clippy::too_many_arguments)]
fn draw_layout_panel(
    ui: &mut egui::Ui,
    cfg: &mut ResMut<FlexConfig>,
    history: &mut ResMut<UndoHistory>,
    preview: &mut Local<Option<FlexConfig>>,
    import_buf: &mut Local<String>,
    toast: &mut Local<Option<(String, f64)>>,
    changed: &mut bool,
    any_hovered: &mut bool,
    sel_path: &mut Vec<usize>,
    is_root: &mut bool,
    hover_direction: &mut Option<FlexDirection>,
    hover_wrap: &mut Option<FlexWrap>,
    hover_justify: &mut Option<JustifyContent>,
    hover_align_items: &mut Option<AlignItems>,
    hover_align_content: &mut Option<AlignContent>,
    hover_row_gap: &mut Option<ValueConfig>,
    hover_column_gap: &mut Option<ValueConfig>,
    hover_width: &mut Option<ValueConfig>,
    hover_height: &mut Option<ValueConfig>,
    hover_min_width: &mut Option<ValueConfig>,
    hover_min_height: &mut Option<ValueConfig>,
    hover_max_width: &mut Option<ValueConfig>,
    hover_max_height: &mut Option<ValueConfig>,
    hover_basis: &mut Option<ValueConfig>,
    hover_align_self: &mut Option<AlignSelf>,
    cg: &mut CodegenState,
    drag: &mut DragState,
    net_dirty: &mut bool,
) {
    // ── Tree ──────────────────────────────────────────────────────────────
    egui::CollapsingHeader::new("Tree")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .button("+ Child")
                    .on_hover_text("Add a new child node inside the selected node")
                    .clicked()
                {
                    let n = cfg.root.count_leaves();
                    let lbl = format!("node{}", n + 1);
                    if let Some(node) = cfg.root.get_mut(sel_path) {
                        node.children
                            .push(NodeConfig::new_leaf(&lbl, 80.0, 80.0));
                        *changed = true;
                    }
                }
                if !*is_root
                    && ui
                        .button("+ Sibling")
                        .on_hover_text(
                            "Add a new node next to the selected node (same parent)",
                        )
                        .clicked()
                {
                    let pidx = sel_path.len() - 1;
                    let sibling_idx = sel_path[pidx];
                    let n = cfg.root.count_leaves();
                    let lbl = format!("node{}", n + 1);
                    if let Some(parent) = cfg.root.get_mut(&sel_path[..pidx]) {
                        let insert_at = (sibling_idx + 1).min(parent.children.len());
                        parent
                            .children
                            .insert(insert_at, NodeConfig::new_leaf(&lbl, 80.0, 80.0));
                        *changed = true;
                    }
                }
                if !*is_root
                    && ui
                        .button("Dup")
                        .on_hover_text("Duplicate selected node (Ctrl+D)")
                        .clicked()
                {
                    let pidx = sel_path.len() - 1;
                    let sibling_idx = sel_path[pidx];
                    if let Some(parent) = cfg.root.get_mut(&sel_path[..pidx]) {
                        let clone = parent.children[sibling_idx].clone();
                        let insert_at = (sibling_idx + 1).min(parent.children.len());
                        parent.children.insert(insert_at, clone);
                        *changed = true;
                    }
                }
            });
            ui.add_space(2.0);
            let sel_snapshot = cfg.selected().to_vec();
            let (clicked, remove_req) =
                draw_tree_ui(ui, &mut cfg.root, &mut vec![], &sel_snapshot, changed, drag);
            if remove_req && !sel_path.is_empty() {
                let pidx = sel_path.len() - 1;
                let idx = sel_path[pidx];
                if let Some(parent) = cfg.root.get_mut(&sel_path[..pidx]) {
                    parent.children.remove(idx);
                }
                let new_path = sel_path[..pidx].to_vec();
                *sel_path = new_path.clone();
                *is_root = sel_path.is_empty();
                cfg.select(new_path);
                *changed = true;
            }
            if let Some(p) = clicked
                && p != cfg.selected()
            {
                *sel_path = p.clone();
                *is_root = sel_path.is_empty();
                cfg.select(p);
                **preview = None;
            }
        });

    ui.add_space(6.0);

    if let Some(n) = cfg.root.get_mut(sel_path) {
        ui.horizontal(|ui| {
            if ui
                .checkbox(&mut n.visible, "Visible")
                .on_hover_text("Whether this node is displayed in the layout")
                .changed()
            {
                *changed = true;
            }
        });
        ui.add_space(4.0);

        // ── Text content ──────────────────────────────────────────────
        {
            let Some(n) = cfg.root.get_mut(sel_path) else {
                return;
            };
            ui.horizontal(|ui| {
                label_with_help(
                    ui,
                    "text",
                    "Text displayed inside this node (empty = use label)",
                );
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut n.text_content)
                            .desired_width(180.0)
                            .hint_text(&n.label),
                    )
                    .changed()
                {
                    *changed = true;
                }
            });
        }
        ui.add_space(4.0);

        let sel_display_mode = cfg.root.get(sel_path).map(|n| n.display_mode).unwrap_or(DisplayMode::Flex);

        if sel_display_mode == DisplayMode::Grid {
            // ── Grid Container ───────────────────────────────────────
            draw_grid_container_section(ui, cfg, sel_path, changed, any_hovered, hover_row_gap, hover_column_gap, hover_justify, hover_align_items, hover_align_content, preview);
        } else {

        // ── Flex Container ────────────────────────────────────────────
        egui::CollapsingHeader::new("Flex Container")
            .default_open(true)
            .show(ui, |ui| {
                ui.add_space(4.0);
                egui::Grid::new("cg1")
                    .num_columns(2)
                    .spacing([10.0, 6.0])
                    .show(ui, |ui| {
                        {
                            let Some(n) = cfg.root.get_mut(sel_path) else {
                                return;
                            };
                            label_with_help(
                                ui,
                                "direction",
                                "The main axis along which children are laid out",
                            );
                            *hover_direction = combo(
                                ui,
                                "fd",
                                &mut n.flex_direction,
                                &[
                                    ("Row", FlexDirection::Row),
                                    ("Column", FlexDirection::Column),
                                    ("RowReverse", FlexDirection::RowReverse),
                                    ("ColumnReverse", FlexDirection::ColumnReverse),
                                ],
                                changed,
                                any_hovered,
                            );
                            ui.end_row();

                            label_with_help(
                                ui,
                                "wrap",
                                "Whether children wrap to new lines when they overflow",
                            );
                            *hover_wrap = combo(
                                ui,
                                "fw",
                                &mut n.flex_wrap,
                                &[
                                    ("NoWrap", FlexWrap::NoWrap),
                                    ("Wrap", FlexWrap::Wrap),
                                    ("WrapReverse", FlexWrap::WrapReverse),
                                ],
                                changed,
                                any_hovered,
                            );
                            ui.end_row();

                            label_with_help(
                                ui,
                                "justify",
                                "How children are distributed along the main axis",
                            );
                            *hover_justify = combo(
                                ui,
                                "jc",
                                &mut n.justify_content,
                                &[
                                    ("Default", JustifyContent::Default),
                                    ("FlexStart", JustifyContent::FlexStart),
                                    ("FlexEnd", JustifyContent::FlexEnd),
                                    ("Center", JustifyContent::Center),
                                    ("SpaceBetween", JustifyContent::SpaceBetween),
                                    ("SpaceAround", JustifyContent::SpaceAround),
                                    ("SpaceEvenly", JustifyContent::SpaceEvenly),
                                    ("Stretch", JustifyContent::Stretch),
                                    ("Start", JustifyContent::Start),
                                    ("End", JustifyContent::End),
                                ],
                                changed,
                                any_hovered,
                            );
                            ui.end_row();

                            label_with_help(
                                ui,
                                "align-items",
                                "How children are aligned along the cross axis",
                            );
                            *hover_align_items = combo(
                                ui,
                                "ai",
                                &mut n.align_items,
                                &[
                                    ("Default", AlignItems::Default),
                                    ("FlexStart", AlignItems::FlexStart),
                                    ("FlexEnd", AlignItems::FlexEnd),
                                    ("Center", AlignItems::Center),
                                    ("Baseline", AlignItems::Baseline),
                                    ("Stretch", AlignItems::Stretch),
                                    ("Start", AlignItems::Start),
                                    ("End", AlignItems::End),
                                ],
                                changed,
                                any_hovered,
                            );
                            ui.end_row();

                            label_with_help(
                                ui,
                                "align-content",
                                "How wrapped lines are distributed along the cross axis",
                            );
                            *hover_align_content = combo(
                                ui,
                                "ac",
                                &mut n.align_content,
                                &[
                                    ("Default", AlignContent::Default),
                                    ("FlexStart", AlignContent::FlexStart),
                                    ("FlexEnd", AlignContent::FlexEnd),
                                    ("Center", AlignContent::Center),
                                    ("SpaceBetween", AlignContent::SpaceBetween),
                                    ("SpaceAround", AlignContent::SpaceAround),
                                    ("SpaceEvenly", AlignContent::SpaceEvenly),
                                    ("Stretch", AlignContent::Stretch),
                                    ("Start", AlignContent::Start),
                                    ("End", AlignContent::End),
                                ],
                                changed,
                                any_hovered,
                            );
                            ui.end_row();
                        }
                    });
                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);
                egui::Grid::new("cg2")
                    .num_columns(2)
                    .spacing([10.0, 6.0])
                    .show(ui, |ui| {
                        {
                            let Some(n) = cfg.root.get_mut(sel_path) else {
                                return;
                            };
                            label_with_help(ui, "row-gap", "Spacing between rows of children");
                            *hover_row_gap =
                                val_row(ui, "rg", &mut n.row_gap, changed, any_hovered);
                            ui.end_row();
                            label_with_help(ui, "column-gap", "Spacing between columns of children");
                            *hover_column_gap =
                                val_row(ui, "cgap", &mut n.column_gap, changed, any_hovered);
                            ui.end_row();
                        }
                    });
                ui.add_space(2.0);

                let has_container_hover = hover_direction.is_some()
                    || hover_wrap.is_some()
                    || hover_justify.is_some()
                    || hover_align_items.is_some()
                    || hover_align_content.is_some()
                    || hover_row_gap.is_some()
                    || hover_column_gap.is_some();
                if has_container_hover {
                    *any_hovered = true;
                    let p = &mut **preview;
                    let sp = sel_path.as_slice();
                    let needs_rebuild = apply_hover(*hover_direction, cfg, p, sp, |n| n.flex_direction, |n, v| n.flex_direction = v)
                        | apply_hover(*hover_wrap, cfg, p, sp, |n| n.flex_wrap, |n, v| n.flex_wrap = v)
                        | apply_hover(*hover_justify, cfg, p, sp, |n| n.justify_content, |n, v| n.justify_content = v)
                        | apply_hover(*hover_align_items, cfg, p, sp, |n| n.align_items, |n, v| n.align_items = v)
                        | apply_hover(*hover_align_content, cfg, p, sp, |n| n.align_content, |n, v| n.align_content = v)
                        | apply_hover(*hover_row_gap, cfg, p, sp, |n| n.row_gap, |n, v| n.row_gap = v)
                        | apply_hover(*hover_column_gap, cfg, p, sp, |n| n.column_gap, |n, v| n.column_gap = v);
                    if needs_rebuild {
                        cfg.request_rebuild();
                    }
                }
            });

        } // end else (Flex Container)

        ui.add_space(6.0);

        // ── Sizing ────────────────────────────────────────────────────
        egui::CollapsingHeader::new("Sizing")
            .default_open(true)
            .show(ui, |ui| {
                ui.add_space(4.0);
                egui::Grid::new("sg")
                    .num_columns(2)
                    .spacing([10.0, 6.0])
                    .show(ui, |ui| {
                        {
                            let Some(n) = cfg.root.get_mut(sel_path) else {
                                return;
                            };
                            label_with_help(ui, "width", "The preferred width of this node");
                            *hover_width = val_row(ui, "sw", &mut n.width, changed, any_hovered);
                            ui.end_row();
                            label_with_help(ui, "height", "The preferred height of this node");
                            *hover_height = val_row(ui, "sh", &mut n.height, changed, any_hovered);
                            ui.end_row();
                            label_with_help(ui, "min-width", "The minimum width this node can shrink to");
                            *hover_min_width = val_row(ui, "sminw", &mut n.min_width, changed, any_hovered);
                            ui.end_row();
                            label_with_help(ui, "min-height", "The minimum height this node can shrink to");
                            *hover_min_height = val_row(ui, "sminh", &mut n.min_height, changed, any_hovered);
                            ui.end_row();
                            label_with_help(ui, "max-width", "The maximum width this node can grow to");
                            *hover_max_width = val_row(ui, "smaxw", &mut n.max_width, changed, any_hovered);
                            ui.end_row();
                            label_with_help(ui, "max-height", "The maximum height this node can grow to");
                            *hover_max_height = val_row(ui, "smaxh", &mut n.max_height, changed, any_hovered);
                            ui.end_row();
                        }
                    });
                ui.add_space(2.0);

                let has_sizing_hover = hover_width.is_some()
                    || hover_height.is_some()
                    || hover_min_width.is_some()
                    || hover_min_height.is_some()
                    || hover_max_width.is_some()
                    || hover_max_height.is_some();
                if has_sizing_hover {
                    *any_hovered = true;
                    let p = &mut **preview;
                    let sp = sel_path.as_slice();
                    let needs_rebuild = apply_hover(*hover_width, cfg, p, sp, |n| n.width, |n, v| n.width = v)
                        | apply_hover(*hover_height, cfg, p, sp, |n| n.height, |n, v| n.height = v)
                        | apply_hover(*hover_min_width, cfg, p, sp, |n| n.min_width, |n, v| n.min_width = v)
                        | apply_hover(*hover_min_height, cfg, p, sp, |n| n.min_height, |n, v| n.min_height = v)
                        | apply_hover(*hover_max_width, cfg, p, sp, |n| n.max_width, |n, v| n.max_width = v)
                        | apply_hover(*hover_max_height, cfg, p, sp, |n| n.max_height, |n, v| n.max_height = v);
                    if needs_rebuild {
                        cfg.request_rebuild();
                    }
                }
            });

        ui.add_space(6.0);

        // ── Spacing (per-side padding / margin) ───────────────────────
        egui::CollapsingHeader::new("Spacing")
            .default_open(true)
            .show(ui, |ui| {
                ui.add_space(4.0);
                {
                    let Some(n) = cfg.root.get_mut(sel_path) else {
                        return;
                    };
                    sides_editor(ui, "padding", "pad", "Space between border and children", &mut n.padding, changed);
                    ui.add_space(4.0);
                    sides_editor(ui, "margin", "mar", "Space outside the border", &mut n.margin, changed);
                }
            });

        ui.add_space(6.0);

        // ── Border ────────────────────────────────────────────────────
        egui::CollapsingHeader::new("Border")
            .default_open(false)
            .show(ui, |ui| {
                ui.add_space(4.0);
                {
                    let Some(n) = cfg.root.get_mut(sel_path) else {
                        return;
                    };
                    sides_editor(ui, "border-width", "bw", "Border thickness per side", &mut n.border_width, changed);
                    ui.add_space(4.0);
                    corners_editor(ui, "border-radius", "br", "Corner rounding", &mut n.border_radius, changed);
                }
            });

        ui.add_space(6.0);

        // ── Item properties (non-root) ──────────────────────────────
        // Whether to show grid-item or flex-item controls depends on the
        // *parent's* display mode, not the node's own.
        let parent_display_mode = if sel_path.is_empty() {
            DisplayMode::Flex
        } else {
            cfg.root
                .get(&sel_path[..sel_path.len() - 1])
                .map(|p| p.display_mode)
                .unwrap_or(DisplayMode::Flex)
        };
        if !*is_root && parent_display_mode == DisplayMode::Grid {
            draw_grid_item_section(ui, cfg, sel_path, changed, hover_align_self, any_hovered, preview);

            ui.add_space(6.0);
        }
        if !*is_root && parent_display_mode == DisplayMode::Flex {
            egui::CollapsingHeader::new("Flex Item")
                .default_open(true)
                .show(ui, |ui| {
                    ui.add_space(4.0);
                    egui::Grid::new("ig")
                        .num_columns(2)
                        .spacing([10.0, 6.0])
                        .show(ui, |ui| {
                            {
                                let Some(n) = cfg.root.get_mut(sel_path) else {
                                    return;
                                };
                                label_with_help(
                                    ui,
                                    "flex-grow",
                                    "How much this node grows relative to siblings (0 = don't grow)",
                                );
                                *changed |= ui
                                    .add(
                                        egui::Slider::new(&mut n.flex_grow, 0.0..=5.0)
                                            .max_decimals(2),
                                    )
                                    .changed();
                                ui.end_row();
                                label_with_help(
                                    ui,
                                    "flex-shrink",
                                    "How much this node shrinks relative to siblings (0 = don't shrink)",
                                );
                                *changed |= ui
                                    .add(
                                        egui::Slider::new(&mut n.flex_shrink, 0.0..=5.0)
                                            .max_decimals(2),
                                    )
                                    .changed();
                                ui.end_row();
                                label_with_help(
                                    ui,
                                    "flex-basis",
                                    "Initial size along main axis before grow/shrink",
                                );
                                *hover_basis =
                                    val_row(ui, "ib", &mut n.flex_basis, changed, any_hovered);
                                ui.end_row();
                                label_with_help(
                                    ui,
                                    "align-self",
                                    "Override parent's align-items for this child",
                                );
                                *hover_align_self = combo(
                                    ui,
                                    "ias",
                                    &mut n.align_self,
                                    &[
                                        ("Auto", AlignSelf::Auto),
                                        ("FlexStart", AlignSelf::FlexStart),
                                        ("FlexEnd", AlignSelf::FlexEnd),
                                        ("Center", AlignSelf::Center),
                                        ("Baseline", AlignSelf::Baseline),
                                        ("Stretch", AlignSelf::Stretch),
                                        ("Start", AlignSelf::Start),
                                        ("End", AlignSelf::End),
                                    ],
                                    changed,
                                    any_hovered,
                                );
                                ui.end_row();
                                label_with_help(
                                    ui,
                                    "order",
                                    "Controls visual ordering of flex items (lower first)",
                                );
                                *changed |=
                                    ui.add(egui::Slider::new(&mut n.order, -10..=10)).changed();
                                ui.end_row();
                            }
                        });
                    ui.add_space(2.0);

                    let has_item_hover = hover_basis.is_some() || hover_align_self.is_some();
                    if has_item_hover {
                        *any_hovered = true;
                        let p = &mut **preview;
                        let sp = sel_path.as_slice();
                        let needs_rebuild = apply_hover(
                            *hover_basis, cfg, p, sp,
                            |n| n.flex_basis, |n, v| n.flex_basis = v,
                        ) | apply_hover(
                            *hover_align_self, cfg, p, sp,
                            |n| n.align_self, |n, v| n.align_self = v,
                        );
                        if needs_rebuild {
                            cfg.request_rebuild();
                        }
                    }
                });

            ui.add_space(6.0);
        }
    } // end if path_valid

    // ── Templates ─────────────────────────────────────────────────────
    egui::CollapsingHeader::new("Templates")
        .default_open(false)
        .show(ui, |ui| {
            let templates: &[(&str, TemplateFn)] = &[
                ("Holy Grail", flexplore::templates::holy_grail),
                ("Sidebar + Content", flexplore::templates::sidebar_content),
                ("Card Grid", flexplore::templates::card_grid),
                ("Nav Bar", flexplore::templates::nav_bar),
                ("Grid Dashboard", flexplore::templates::grid_dashboard),
                ("Grid Gallery", flexplore::templates::grid_gallery),
            ];
            ui.horizontal_wrapped(|ui| {
                for (name, builder) in templates {
                    if ui.button(*name).clicked() {
                        cfg.root = builder();
                        cfg.select(vec![]);
                        **preview = None;
                        *changed = true;
                    }
                }
            });
        });

    ui.add_space(6.0);

    // ── Background ────────────────────────────────────────────────────
    egui::CollapsingHeader::new("Background")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let prev = cfg.bg_mode;
                ui.radio_value(&mut cfg.bg_mode, BackgroundMode::Pastel, "Pastel")
                    .on_hover_text("Fill leaf nodes with solid pastel colors");
                ui.radio_value(
                    &mut cfg.bg_mode,
                    BackgroundMode::RandomArt,
                    "Generative Art",
                )
                .on_hover_text("Fill leaf nodes with procedurally generated art textures");
                if cfg.bg_mode != prev {
                    *changed = true;
                }
            });
            let mut hover_palette: Option<ColorPalette> = None;
            ui.horizontal(|ui| {
                ui.label("palette");
                let pal_resp = egui::ComboBox::from_id_salt("palette_sel")
                    .selected_text(cfg.palette.to_string())
                    .width(110.0)
                    .show_ui(ui, |ui| {
                        for p in ColorPalette::iter() {
                            let r = ui.selectable_label(cfg.palette == p, p.to_string());
                            if r.clicked() {
                                cfg.palette = p;
                                *changed = true;
                            } else if r.hovered() {
                                hover_palette = Some(p);
                            }
                        }
                    });
                if pal_resp.inner.is_some() {
                    *any_hovered = true;
                }
            });
            if let Some(p) = hover_palette {
                *any_hovered = true;
                if cfg.palette != p {
                    if preview.is_none() {
                        **preview = Some((**cfg).clone());
                    }
                    cfg.palette = p;
                    cfg.request_rebuild();
                }
            }
            if cfg.bg_mode == BackgroundMode::RandomArt {
                let cur = cfg.art_style.to_string();
                let mut hover_art: Option<ArtStyle> = None;
                let art_resp = egui::ComboBox::from_label("style")
                    .selected_text(&cur)
                    .show_ui(ui, |ui| {
                        for style in ArtStyle::iter() {
                            let name = style.to_string();
                            let r = ui.selectable_label(cfg.art_style == style, &name);
                            if r.clicked() {
                                cfg.art_style = style;
                                *changed = true;
                            } else if r.hovered() {
                                hover_art = Some(style);
                            }
                        }
                    });
                if art_resp.inner.is_some() {
                    *any_hovered = true;
                }
                if let Some(v) = hover_art {
                    *any_hovered = true;
                    if cfg.art_style != v {
                        if preview.is_none() {
                            **preview = Some((**cfg).clone());
                        }
                        cfg.art_style = v;
                        cfg.request_rebuild();
                    }
                }
                let pd = cfg.art_depth;
                ui.add(
                    egui::Slider::new(&mut cfg.art_depth, 1..=9)
                        .text("depth"),
                )
                .on_hover_text("Expression tree depth — higher = more complex");
                if cfg.art_depth != pd {
                    *changed = true;
                }
                ui.add(
                    egui::Slider::new(&mut cfg.art_anim, 0.0..=2.0)
                        .text("anim speed")
                        .step_by(0.05),
                )
                .on_hover_text("How fast the generative art animates (0 = static)");
                ui.horizontal(|ui| {
                    if ui
                        .button("New seed")
                        .on_hover_text("Randomize the seed")
                        .clicked()
                    {
                        cfg.art_seed = rand::random::<u64>();
                        *changed = true;
                    }
                    if ui
                        .button("Regenerate")
                        .on_hover_text("Re-render art with current settings")
                        .clicked()
                    {
                        *changed = true;
                    }
                });
            }
        });

    ui.add_space(6.0);
    if ui
        .button("Reset to defaults")
        .on_hover_text("Restore all settings and the node tree to the initial state")
        .clicked()
    {
        **cfg = FlexConfig::default();
        **preview = None;
        *changed = true;
    }

    ui.add_space(6.0);

    // ── Import / Export ───────────────────────────────────────────────
    egui::CollapsingHeader::new("Import / Export")
        .default_open(false)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .button("Save file (Ctrl+S)")
                    .on_hover_text("Save layout to a JSON file")
                    .clicked()
                {
                    save_file(cfg, toast, ui.ctx());
                }
                if ui
                    .button("Open file (Ctrl+O)")
                    .on_hover_text("Load layout from a JSON file")
                    .clicked()
                {
                    if let Some(loaded) = open_file() {
                        **cfg = loaded;
                        cfg.request_rebuild();
                        **preview = None;
                        history.push((**cfg).clone());
                        cg.dirty = true;
                        *net_dirty = true;
                    }
                }
            });
            ui.add_space(4.0);
            ui.label("Paste JSON to import:");
            ui.add(
                egui::TextEdit::multiline(&mut **import_buf)
                    .desired_rows(3)
                    .desired_width(f32::INFINITY),
            );
            if ui.button("Load from JSON").clicked()
                && !import_buf.is_empty()
                && let Some(loaded) = crate::persist::import_json(import_buf)
            {
                **cfg = loaded;
                cfg.request_rebuild();
                **preview = None;
                history.push((**cfg).clone());
                import_buf.clear();
                cg.dirty = true;
                *net_dirty = true;
            }

            #[cfg(target_arch = "wasm32")]
            {
                ui.add_space(4.0);
                if ui.button("Share URL").on_hover_text("Copy a shareable URL to clipboard").clicked() {
                    if let Some(url) = crate::persist::make_share_url(cfg) {
                        ui.ctx().copy_text(url);
                        let now = ui.ctx().input(|i| i.time);
                        **toast = Some(("Share URL copied!".into(), now + 2.0));
                    }
                }
            }
        });

    ui.add_space(4.0);

    // ── Code export ───────────────────────────────────────────────────
    ui.label("Copy code:");
    ui.horizontal_wrapped(|ui| {
        let pal = cfg.palette;
        for (name, emitter) in CODEGEN_TARGETS {
            if ui
                .button(*name)
                .on_hover_text(format!("Copy {name} code to clipboard"))
                .clicked()
            {
                match emitter(&cfg.root, pal) {
                    Ok(code) => {
                        ui.ctx().copy_text(code);
                        let now = ui.ctx().input(|i| i.time);
                        **toast = Some((format!("Copied {name}!"), now + 2.0));
                    }
                    Err(e) => {
                        let now = ui.ctx().input(|i| i.time);
                        **toast = Some((format!("Error: {e}"), now + 3.0));
                    }
                }
            }
        }
    });

    ui.add_space(4.0);
    if ui.button("Preview code").on_hover_text("Open the code preview panel on the right").clicked() {
        cg.preview_open = true;
        cg.dirty = true;
    }
}

// ─── Theme ───────────────────────────────────────────────────────────────────

fn apply_theme(ctx: &egui::Context, theme: Theme) {
    let flavor = match theme {
        Theme::Latte => catppuccin::PALETTE.latte,
        Theme::Frappe => catppuccin::PALETTE.frappe,
        Theme::Macchiato => catppuccin::PALETTE.macchiato,
        Theme::Mocha => catppuccin::PALETTE.mocha,
    };
    let c = &flavor.colors;
    let cc =
        |color: &catppuccin::Color| egui::Color32::from_rgb(color.rgb.r, color.rgb.g, color.rgb.b);

    let no_rounding = egui::CornerRadius::ZERO;
    let mut v = if theme.is_light() {
        egui::Visuals::light()
    } else {
        egui::Visuals::dark()
    };

    let bg = cc(&c.base);
    let fg = cc(&c.text);
    let s0 = cc(&c.surface0);
    let s1 = cc(&c.surface1);
    let s2 = cc(&c.surface2);
    let o0 = cc(&c.overlay0);
    let crust = cc(&c.crust);

    v.panel_fill = bg;
    v.window_fill = bg;
    v.extreme_bg_color = crust;
    v.widgets.inactive.bg_fill = s0;
    v.widgets.inactive.weak_bg_fill = s0;
    v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, s1);
    v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, fg);
    v.widgets.hovered.bg_fill = s1;
    v.widgets.hovered.weak_bg_fill = s1;
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, o0);
    v.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, fg);
    v.widgets.active.bg_fill = fg;
    v.widgets.active.weak_bg_fill = fg;
    v.widgets.active.fg_stroke = egui::Stroke::new(1.5, bg);
    v.widgets.open.bg_fill = s0;
    v.widgets.open.fg_stroke = egui::Stroke::new(1.0, fg);
    v.widgets.noninteractive.bg_fill = bg;
    v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, o0);
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, s1);
    v.override_text_color = Some(fg);
    v.window_stroke = egui::Stroke::new(1.0, s1);
    v.selection.bg_fill = s2;
    v.window_corner_radius = no_rounding;
    v.menu_corner_radius = no_rounding;
    v.widgets.inactive.corner_radius = no_rounding;
    v.widgets.hovered.corner_radius = no_rounding;
    v.widgets.active.corner_radius = no_rounding;
    v.widgets.open.corner_radius = no_rounding;
    v.widgets.noninteractive.corner_radius = no_rounding;
    ctx.set_visuals(v);
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(6.0, 3.0);
    style.spacing.button_padding = egui::vec2(6.0, 2.0);
    style.spacing.slider_width = 110.0;
    ctx.set_style(style);
}

// ─── Grid UI sections ────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn draw_grid_container_section(
    ui: &mut egui::Ui,
    cfg: &mut bevy::prelude::ResMut<FlexConfig>,
    sel_path: &mut Vec<usize>,
    changed: &mut bool,
    any_hovered: &mut bool,
    hover_row_gap: &mut Option<ValueConfig>,
    hover_column_gap: &mut Option<ValueConfig>,
    hover_justify: &mut Option<JustifyContent>,
    hover_align_items: &mut Option<AlignItems>,
    hover_align_content: &mut Option<AlignContent>,
    preview: &mut bevy::prelude::Local<Option<FlexConfig>>,
) {
    egui::CollapsingHeader::new("Grid Container")
        .default_open(true)
        .show(ui, |ui| {
            ui.add_space(4.0);

            // ── Template Columns ─────────────────────────────────────
            {
                let Some(n) = cfg.root.get_mut(sel_path) else { return };
                track_list_editor(ui, "columns", "gtc", &mut n.grid_template_columns, changed);
            }
            ui.add_space(4.0);

            // ── Template Rows ────────────────────────────────────────
            {
                let Some(n) = cfg.root.get_mut(sel_path) else { return };
                track_list_editor(ui, "rows", "gtr", &mut n.grid_template_rows, changed);
            }
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // ── Auto Flow ────────────────────────────────────────────
            egui::Grid::new("gc_flow")
                .num_columns(2)
                .spacing([10.0, 6.0])
                .show(ui, |ui| {
                    let Some(n) = cfg.root.get_mut(sel_path) else { return };
                    label_with_help(ui, "auto-flow", "Direction that auto-placed items flow");
                    combo(
                        ui,
                        "gaf",
                        &mut n.grid_auto_flow,
                        &[
                            ("Row", GridAutoFlow::Row),
                            ("Column", GridAutoFlow::Column),
                            ("RowDense", GridAutoFlow::RowDense),
                            ("ColumnDense", GridAutoFlow::ColumnDense),
                        ],
                        changed,
                        any_hovered,
                    );
                    ui.end_row();
                });

            ui.add_space(4.0);

            // ── Auto Rows / Auto Columns ─────────────────────────────
            {
                let Some(n) = cfg.root.get_mut(sel_path) else { return };
                track_list_editor(ui, "auto-cols", "gac", &mut n.grid_auto_columns, changed);
            }
            ui.add_space(2.0);
            {
                let Some(n) = cfg.root.get_mut(sel_path) else { return };
                track_list_editor(ui, "auto-rows", "gar", &mut n.grid_auto_rows, changed);
            }

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // ── Gaps (shared with flex) ──────────────────────────────
            egui::Grid::new("gc_gaps")
                .num_columns(2)
                .spacing([10.0, 6.0])
                .show(ui, |ui| {
                    let Some(n) = cfg.root.get_mut(sel_path) else { return };
                    label_with_help(ui, "row-gap", "Spacing between rows");
                    *hover_row_gap = val_row(ui, "grg", &mut n.row_gap, changed, any_hovered);
                    ui.end_row();
                    label_with_help(ui, "column-gap", "Spacing between columns");
                    *hover_column_gap = val_row(ui, "gcg", &mut n.column_gap, changed, any_hovered);
                    ui.end_row();
                });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // ── Alignment (shared with flex) ─────────────────────────
            egui::Grid::new("gc_align")
                .num_columns(2)
                .spacing([10.0, 6.0])
                .show(ui, |ui| {
                    let Some(n) = cfg.root.get_mut(sel_path) else { return };
                    label_with_help(ui, "justify", "How items are distributed along the row axis");
                    *hover_justify = combo(
                        ui, "gjc", &mut n.justify_content,
                        &[
                            ("Default", JustifyContent::Default),
                            ("Start", JustifyContent::Start),
                            ("End", JustifyContent::End),
                            ("Center", JustifyContent::Center),
                            ("Stretch", JustifyContent::Stretch),
                            ("SpaceBetween", JustifyContent::SpaceBetween),
                            ("SpaceAround", JustifyContent::SpaceAround),
                            ("SpaceEvenly", JustifyContent::SpaceEvenly),
                        ],
                        changed, any_hovered,
                    );
                    ui.end_row();
                    label_with_help(ui, "align-items", "How items are aligned along the column axis");
                    *hover_align_items = combo(
                        ui, "gai", &mut n.align_items,
                        &[
                            ("Default", AlignItems::Default),
                            ("Start", AlignItems::Start),
                            ("End", AlignItems::End),
                            ("Center", AlignItems::Center),
                            ("Baseline", AlignItems::Baseline),
                            ("Stretch", AlignItems::Stretch),
                        ],
                        changed, any_hovered,
                    );
                    ui.end_row();
                    label_with_help(ui, "align-content", "How grid tracks are distributed along the column axis");
                    *hover_align_content = combo(
                        ui, "gac2", &mut n.align_content,
                        &[
                            ("Default", AlignContent::Default),
                            ("Start", AlignContent::Start),
                            ("End", AlignContent::End),
                            ("Center", AlignContent::Center),
                            ("Stretch", AlignContent::Stretch),
                            ("SpaceBetween", AlignContent::SpaceBetween),
                            ("SpaceAround", AlignContent::SpaceAround),
                            ("SpaceEvenly", AlignContent::SpaceEvenly),
                        ],
                        changed, any_hovered,
                    );
                    ui.end_row();
                });

            let has_hover = hover_row_gap.is_some()
                || hover_column_gap.is_some()
                || hover_justify.is_some()
                || hover_align_items.is_some()
                || hover_align_content.is_some();
            if has_hover {
                *any_hovered = true;
                let p = &mut **preview;
                let sp = sel_path.as_slice();
                let needs_rebuild =
                    apply_hover(*hover_row_gap, cfg, p, sp, |n| n.row_gap, |n, v| n.row_gap = v)
                    | apply_hover(*hover_column_gap, cfg, p, sp, |n| n.column_gap, |n, v| n.column_gap = v)
                    | apply_hover(*hover_justify, cfg, p, sp, |n| n.justify_content, |n, v| n.justify_content = v)
                    | apply_hover(*hover_align_items, cfg, p, sp, |n| n.align_items, |n, v| n.align_items = v)
                    | apply_hover(*hover_align_content, cfg, p, sp, |n| n.align_content, |n, v| n.align_content = v);
                if needs_rebuild {
                    cfg.request_rebuild();
                }
            }
        });
}

fn draw_grid_item_section(
    ui: &mut egui::Ui,
    cfg: &mut bevy::prelude::ResMut<FlexConfig>,
    sel_path: &mut Vec<usize>,
    changed: &mut bool,
    hover_align_self: &mut Option<AlignSelf>,
    any_hovered: &mut bool,
    preview: &mut bevy::prelude::Local<Option<FlexConfig>>,
) {
    egui::CollapsingHeader::new("Grid Item")
        .default_open(true)
        .show(ui, |ui| {
            ui.add_space(4.0);
            egui::Grid::new("gi")
                .num_columns(2)
                .spacing([10.0, 6.0])
                .show(ui, |ui| {
                    let Some(n) = cfg.root.get_mut(sel_path) else { return };

                    // ── grid-column ──
                    label_with_help(ui, "grid-column", "Column placement (start line / span)");
                    grid_placement_editor(ui, "gic", &mut n.grid_column, changed);
                    ui.end_row();

                    // ── grid-row ──
                    label_with_help(ui, "grid-row", "Row placement (start line / span)");
                    grid_placement_editor(ui, "gir", &mut n.grid_row, changed);
                    ui.end_row();

                    // ── align-self ──
                    label_with_help(ui, "align-self", "Override parent's align-items for this child");
                    *hover_align_self = combo(
                        ui, "gias", &mut n.align_self,
                        &[
                            ("Auto", AlignSelf::Auto),
                            ("Start", AlignSelf::Start),
                            ("End", AlignSelf::End),
                            ("Center", AlignSelf::Center),
                            ("Baseline", AlignSelf::Baseline),
                            ("Stretch", AlignSelf::Stretch),
                        ],
                        changed, any_hovered,
                    );
                    ui.end_row();

                    // ── order ──
                    label_with_help(ui, "order", "Controls visual ordering (lower first)");
                    *changed |= ui.add(egui::Slider::new(&mut n.order, -10..=10)).changed();
                    ui.end_row();
                });

            if hover_align_self.is_some() {
                *any_hovered = true;
                let p = &mut **preview;
                let sp = sel_path.as_slice();
                let needs_rebuild = apply_hover(
                    *hover_align_self, cfg, p, sp,
                    |n| n.align_self, |n, v| n.align_self = v,
                );
                if needs_rebuild {
                    cfg.request_rebuild();
                }
            }
        });
}

/// Editor for a list of grid track sizes (e.g. grid-template-columns).
fn track_list_editor(
    ui: &mut egui::Ui,
    label: &str,
    id_prefix: &str,
    tracks: &mut Vec<GridTrackSize>,
    changed: &mut bool,
) {
    ui.horizontal(|ui| {
        label_with_help(
            ui,
            label,
            &format!("Grid track definitions for {label}"),
        );
        if ui.small_button("+").on_hover_text("Add track").clicked() {
            tracks.push(GridTrackSize::Fr(1.0));
            *changed = true;
        }
    });
    let mut remove_idx = None;
    for (i, track) in tracks.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            let tid = format!("{id_prefix}_{i}");
            let cur_kind = track.kind();
            egui::ComboBox::from_id_salt(&tid)
                .width(72.0)
                .selected_text(cur_kind.to_string())
                .show_ui(ui, |ui| {
                    for kind in GridTrackKind::iter() {
                        if ui.selectable_label(cur_kind == kind, kind.to_string()).clicked() {
                            let n = track.num().unwrap_or(1.0);
                            *track = GridTrackSize::cast(kind, n);
                            *changed = true;
                        }
                    }
                });
            if let Some(n) = track.num() {
                let mut n = n;
                let (lo, hi) = match track.kind() {
                    GridTrackKind::Px => (0.0_f32, 800.0_f32),
                    GridTrackKind::Fr => (0.1_f32, 10.0_f32),
                    _ => (0.0_f32, 100.0_f32),
                };
                let decimals = if matches!(track.kind(), GridTrackKind::Fr) { 1 } else { 0 };
                if ui.add(egui::Slider::new(&mut n, lo..=hi).max_decimals(decimals)).changed() {
                    track.set_num(n);
                    *changed = true;
                }
            }
            if ui.small_button("x").on_hover_text("Remove track").clicked() {
                remove_idx = Some(i);
            }
        });
    }
    if let Some(i) = remove_idx {
        tracks.remove(i);
        *changed = true;
    }
}

/// Editor for a GridPlacement value.
fn grid_placement_editor(
    ui: &mut egui::Ui,
    id: &str,
    placement: &mut GridPlacement,
    changed: &mut bool,
) {
    ui.horizontal(|ui| {
        let modes = ["Auto", "Start", "Span", "Start+Span"];
        let cur_idx = match placement {
            GridPlacement::Auto => 0,
            GridPlacement::Start(_) => 1,
            GridPlacement::Span(_) => 2,
            GridPlacement::StartSpan(_, _) => 3,
        };
        egui::ComboBox::from_id_salt(id)
            .width(90.0)
            .selected_text(modes[cur_idx])
            .show_ui(ui, |ui| {
                for (i, mode) in modes.iter().enumerate() {
                    if ui.selectable_label(cur_idx == i, *mode).clicked() && cur_idx != i {
                        *placement = match i {
                            0 => GridPlacement::Auto,
                            1 => GridPlacement::Start(1),
                            2 => GridPlacement::Span(1),
                            3 => GridPlacement::StartSpan(1, 1),
                            _ => GridPlacement::Auto,
                        };
                        *changed = true;
                    }
                }
            });
        match placement {
            GridPlacement::Start(s) => {
                let mut v = *s as i32;
                if ui.add(egui::Slider::new(&mut v, -10..=20).prefix("line ")).changed() {
                    *s = v as i16;
                    *changed = true;
                }
            }
            GridPlacement::Span(n) => {
                let mut v = *n as i32;
                if ui.add(egui::Slider::new(&mut v, 1..=12).prefix("span ")).changed() {
                    *n = v as u16;
                    *changed = true;
                }
            }
            GridPlacement::StartSpan(s, n) => {
                let mut vs = *s as i32;
                let mut vn = *n as i32;
                if ui.add(egui::Slider::new(&mut vs, -10..=20).prefix("line ")).changed() {
                    *s = vs as i16;
                    *changed = true;
                }
                if ui.add(egui::Slider::new(&mut vn, 1..=12).prefix("span ")).changed() {
                    *n = vn as u16;
                    *changed = true;
                }
            }
            GridPlacement::Auto => {}
        }
    });
}

// ─── egui helpers ─────────────────────────────────────────────────────────────

fn label_with_help(ui: &mut egui::Ui, text: &str, help: &str) {
    ui.horizontal(|ui| {
        ui.label(text);
        ui.weak("?").on_hover_text(help);
    });
}

fn combo<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    label: &str,
    val: &mut T,
    options: &[(&str, T)],
    changed: &mut bool,
    any_open: &mut bool,
) -> Option<T> {
    let sel = options
        .iter()
        .find(|(_, v)| *v == *val)
        .map(|(s, _)| *s)
        .unwrap_or("?");
    let mut hover = None;
    let resp = egui::ComboBox::from_id_salt(label)
        .selected_text(sel)
        .width(130.0)
        .show_ui(ui, |ui| {
            for (name, opt) in options {
                let r = ui.selectable_label(*val == *opt, *name);
                if r.clicked() {
                    *val = *opt;
                    *changed = true;
                } else if r.hovered() {
                    hover = Some(*opt);
                }
            }
        });
    if resp.inner.is_some() {
        *any_open = true;
    }
    hover
}

fn val_row(
    ui: &mut egui::Ui,
    id: &str,
    val: &mut ValueConfig,
    changed: &mut bool,
    any_open: &mut bool,
) -> Option<ValueConfig> {
    let mut hover = None;
    let mut is_open = false;
    ui.horizontal(|ui| {
        let cur = val.kind();
        let resp = egui::ComboBox::from_id_salt(id)
            .width(72.0)
            .selected_text(cur.to_string())
            .show_ui(ui, |ui| {
                for kind in ValueKind::iter() {
                    let r = ui.selectable_label(cur == kind, kind.to_string());
                    if r.clicked() {
                        *val = val.cast(kind);
                        *changed = true;
                    } else if r.hovered() {
                        hover = Some(val.cast(kind));
                    }
                }
            });
        if resp.inner.is_some() {
            is_open = true;
        }
        if let Some(n) = val.num() {
            let mut n = n;
            let (lo, hi) = if val.kind() == ValueKind::Px {
                (0.0_f32, 800.0_f32)
            } else {
                (0.0_f32, 100.0_f32)
            };
            if ui
                .add(egui::Slider::new(&mut n, lo..=hi).max_decimals(0))
                .changed()
            {
                val.set_num(n);
                *changed = true;
            }
        }
    });
    if is_open {
        *any_open = true;
    }
    hover
}

/// Editor for a per-side `Sides` value. Shows a single slider when uniform,
/// or four sliders (top/right/bottom/left) when sides differ.
fn sides_editor(
    ui: &mut egui::Ui,
    label: &str,
    id_prefix: &str,
    help: &str,
    sides: &mut Sides,
    changed: &mut bool,
) {
    let uniform = sides.is_uniform();
    ui.horizontal(|ui| {
        label_with_help(ui, label, help);
        let mut link = uniform;
        if ui
            .toggle_value(&mut link, if uniform { "🔗" } else { "⛓" })
            .on_hover_text(if uniform {
                "Click to set each side independently"
            } else {
                "Click to link all sides to one value"
            })
            .changed()
        {
            if link {
                // Unify to top value
                *sides = Sides::uniform(sides.top);
                *changed = true;
            }
        }
    });

    if uniform {
        // Single slider controlling all sides
        let mut v = sides.top;
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            let mut dummy = false;
            single_val_editor(ui, &format!("{id_prefix}_u"), &mut v, changed, &mut dummy);
        });
        if v != sides.top {
            *sides = Sides::uniform(v);
        }
    } else {
        for (side_label, side_id, getter, setter) in [
            ("top", format!("{id_prefix}_t"), (|s: &Sides| s.top) as fn(&Sides) -> ValueConfig, (|s: &mut Sides, v: ValueConfig| s.top = v) as fn(&mut Sides, ValueConfig)),
            ("right", format!("{id_prefix}_r"), (|s: &Sides| s.right) as fn(&Sides) -> ValueConfig, (|s: &mut Sides, v: ValueConfig| s.right = v) as fn(&mut Sides, ValueConfig)),
            ("bottom", format!("{id_prefix}_b"), (|s: &Sides| s.bottom) as fn(&Sides) -> ValueConfig, (|s: &mut Sides, v: ValueConfig| s.bottom = v) as fn(&mut Sides, ValueConfig)),
            ("left", format!("{id_prefix}_l"), (|s: &Sides| s.left) as fn(&Sides) -> ValueConfig, (|s: &mut Sides, v: ValueConfig| s.left = v) as fn(&mut Sides, ValueConfig)),
        ] {
            ui.horizontal(|ui| {
                ui.add_space(14.0);
                ui.label(side_label);
                let mut v = getter(sides);
                let mut dummy = false;
                single_val_editor(ui, &side_id, &mut v, changed, &mut dummy);
                setter(sides, v);
            });
        }
    }
}

/// Editor for a per-corner `Corners` value.
fn corners_editor(
    ui: &mut egui::Ui,
    label: &str,
    _id_prefix: &str,
    help: &str,
    corners: &mut Corners,
    changed: &mut bool,
) {
    let uniform = corners.is_uniform();
    ui.horizontal(|ui| {
        label_with_help(ui, label, help);
        let mut link = uniform;
        if ui
            .toggle_value(&mut link, if uniform { "🔗" } else { "⛓" })
            .on_hover_text(if uniform {
                "Click to set each corner independently"
            } else {
                "Click to link all corners"
            })
            .changed()
        {
            if link {
                *corners = Corners::uniform(corners.top_left);
                *changed = true;
            }
        }
    });

    if uniform {
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            if ui
                .add(
                    egui::Slider::new(&mut corners.top_left, 0.0..=100.0)
                        .max_decimals(0)
                        .suffix("px"),
                )
                .changed()
            {
                let v = corners.top_left;
                *corners = Corners::uniform(v);
                *changed = true;
            }
        });
    } else {
        for (corner_label, val) in [
            ("top-left", &mut corners.top_left),
            ("top-right", &mut corners.top_right),
            ("bottom-right", &mut corners.bottom_right),
            ("bottom-left", &mut corners.bottom_left),
        ] {
            ui.horizontal(|ui| {
                ui.add_space(14.0);
                ui.label(corner_label);
                if ui
                    .add(
                        egui::Slider::new(val, 0.0..=100.0)
                            .max_decimals(0)
                            .suffix("px"),
                    )
                    .changed()
                {
                    *changed = true;
                }
            });
        }
    }
}

/// Inline value-kind combo + slider for a single `ValueConfig`.
fn single_val_editor(
    ui: &mut egui::Ui,
    id: &str,
    val: &mut ValueConfig,
    changed: &mut bool,
    _any_open: &mut bool,
) {
    let cur = val.kind();
    egui::ComboBox::from_id_salt(id)
        .width(60.0)
        .selected_text(cur.to_string())
        .show_ui(ui, |ui| {
            for kind in ValueKind::iter() {
                if ui.selectable_label(cur == kind, kind.to_string()).clicked() {
                    *val = val.cast(kind);
                    *changed = true;
                }
            }
        });
    if let Some(n) = val.num() {
        let mut n = n;
        let (lo, hi) = if val.kind() == ValueKind::Px {
            (0.0_f32, 800.0_f32)
        } else {
            (0.0_f32, 100.0_f32)
        };
        if ui
            .add(egui::Slider::new(&mut n, lo..=hi).max_decimals(0))
            .changed()
        {
            val.set_num(n);
            *changed = true;
        }
    }
}

// ─── File picker helpers ─────────────────────────────────────────────────────

#[allow(unused_variables)]
fn save_file(
    cfg: &FlexConfig,
    toast: &mut Local<Option<(String, f64)>>,
    ctx: &egui::Context,
) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(json) = crate::persist::export_json(cfg) {
            crate::persist::trigger_download(&json);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(json) = crate::persist::export_json(cfg) {
            let path = rfd::FileDialog::new()
                .set_file_name("flexplore-layout.json")
                .add_filter("JSON", &["json"])
                .save_file();
            if let Some(path) = path {
                match std::fs::write(&path, &json) {
                    Ok(()) => {
                        let now = ctx.input(|i| i.time);
                        **toast = Some((format!("Saved to {}", path.display()), now + 2.0));
                    }
                    Err(e) => {
                        let now = ctx.input(|i| i.time);
                        **toast = Some((format!("Save error: {e}"), now + 3.0));
                    }
                }
            }
        }
    }
}

fn open_file() -> Option<FlexConfig> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .pick_file()?;
        let data = std::fs::read_to_string(path).ok()?;
        crate::persist::import_json(&data)
    }
    #[cfg(target_arch = "wasm32")]
    {
        // WASM file open is handled via the JSON paste import
        None
    }
}
