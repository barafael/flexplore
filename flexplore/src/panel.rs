use std::sync::Arc;

use bevy::prelude::{Local, ResMut, Result, Vec2};
use bevy_egui::{EguiContexts, egui};
use egui::text::LayoutJob;
use strum::IntoEnumIterator;

use crate::highlight::Lang;
use crate::viz::ArrowNav;
use flexplore::codegen::{
    emit_bevy_code, emit_dioxus, emit_egui, emit_flutter, emit_html_css, emit_iced, emit_react,
    emit_react_native, emit_swiftui, emit_tailwind,
};
use flexplore::config::*;
use flexplore::history::UndoHistory;

type TemplateFn = fn() -> NodeConfig;
type CodegenFn = fn(&NodeConfig, ColorPalette) -> anyhow::Result<String>;

/// (display name, emitter, syntax-highlighting language)
const CODEGEN_TARGETS: &[(&str, CodegenFn, Lang)] = &[
    ("Bevy", emit_bevy_code, Lang::Rust),
    ("HTML/CSS", emit_html_css, Lang::Html),
    ("Tailwind", emit_tailwind, Lang::Html),
    ("React", emit_react, Lang::Jsx),
    ("SwiftUI", emit_swiftui, Lang::Swift),
    ("Flutter", emit_flutter, Lang::Dart),
    ("Iced", emit_iced, Lang::Rust),
    ("egui", emit_egui, Lang::Rust),
    ("React Native", emit_react_native, Lang::Jsx),
    ("Dioxus", emit_dioxus, Lang::Rust),
];

/// A transient message in the bottom-right corner: `(text, egui time at which it expires)`.
type Toast = Option<(String, f64)>;

// ─── Left-panel tab ──────────────────────────────────────────────────────────

/// Mirrors the selected node's display mode; clicking the other tab switches it.
#[derive(Clone, Copy, PartialEq)]
enum LeftTab {
    Flexbox,
    CssGrid,
}

// ─── Codegen right panel state ───────────────────────────────────────────────

pub(crate) struct CodegenState {
    framework_idx: usize,
    preview_open: bool,
    cached_code: String,
    dirty: bool,
    /// `cached_code` tokenised for display, tagged with the framework and
    /// theme brightness it was built for. Rebuilt only when one of those changes.
    highlighted: Option<(usize, bool, Arc<LayoutJob>)>,
}

impl Default for CodegenState {
    fn default() -> Self {
        Self {
            framework_idx: 0,
            preview_open: false,
            cached_code: String::new(),
            dirty: true,
            highlighted: None,
        }
    }
}

impl CodegenState {
    /// Regenerate `cached_code` if the document changed.
    fn refresh(&mut self, cfg: &FlexConfig) {
        if !self.dirty {
            return;
        }
        let (_, emitter, _) = CODEGEN_TARGETS[self.framework_idx];
        self.cached_code = match emitter(&cfg.root, cfg.palette) {
            Ok(code) => code,
            Err(e) => format!("Error: {e}"),
        };
        self.highlighted = None;
        self.dirty = false;
    }

    /// The highlighted listing, re-tokenised only when code, language or theme changed.
    fn highlighted_job(&mut self, light: bool) -> Arc<LayoutJob> {
        match &self.highlighted {
            Some((idx, l, job)) if *idx == self.framework_idx && *l == light => job.clone(),
            _ => {
                let lang = CODEGEN_TARGETS[self.framework_idx].2;
                let font = egui::FontId::monospace(12.0);
                let job = Arc::new(crate::highlight::highlight(
                    &self.cached_code,
                    lang,
                    font,
                    light,
                ));
                self.highlighted = Some((self.framework_idx, light, job.clone()));
                job
            }
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

/// Pointer must move this far from the press origin before a row drag starts.
const DRAG_THRESHOLD: f32 = 4.0;

/// Move the node at `src` so it becomes child `dst_idx` of `dst_parent`.
/// Returns the moved node's new path, or `None` when the move is a no-op or
/// invalid (moving a node into its own subtree).
fn move_node(
    root: &mut NodeConfig,
    src: &[usize],
    dst_parent: &[usize],
    dst_idx: usize,
) -> Option<Vec<usize>> {
    let (&src_idx, src_parent) = src.split_last()?;
    if dst_parent.starts_with(src) {
        return None;
    }
    if src_parent == dst_parent {
        // Dropping right before or after itself changes nothing.
        if dst_idx == src_idx || dst_idx == src_idx + 1 {
            return None;
        }
        let parent = root.get_mut(src_parent)?;
        if src_idx >= parent.children.len() {
            return None;
        }
        let node = parent.children.remove(src_idx);
        let at = if dst_idx > src_idx {
            dst_idx - 1
        } else {
            dst_idx
        }
        .min(parent.children.len());
        parent.children.insert(at, node);
        let mut path = src_parent.to_vec();
        path.push(at);
        return Some(path);
    }
    // Cross-parent: validate both ends before touching the tree.
    root.get(dst_parent)?;
    let src_len = src_parent.len();
    let mut dst_parent = dst_parent.to_vec();
    // Removing the source shifts its later siblings — and any destination
    // path that runs through one of them — down by one.
    if dst_parent.len() > src_len
        && dst_parent[..src_len] == *src_parent
        && dst_parent[src_len] > src_idx
    {
        dst_parent[src_len] -= 1;
    }
    let parent = root.get_mut(src_parent)?;
    if src_idx >= parent.children.len() {
        return None;
    }
    let node = parent.children.remove(src_idx);
    let dest = root.get_mut(&dst_parent)?;
    let at = dst_idx.min(dest.children.len());
    dest.children.insert(at, node);
    dst_parent.push(at);
    Some(dst_parent)
}

// ─── Tree UI helper ───────────────────────────────────────────────────────────

fn draw_drop_indicator(ui: &mut egui::Ui, indent: usize) {
    ui.horizontal(|ui| {
        ui.add_space(indent as f32 * 14.0);
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 2.0), egui::Sense::hover());
        ui.painter()
            .rect_filled(rect, 0.0, egui::Color32::from_rgb(0x60, 0xA0, 0xFF));
    });
}

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
        path.split_last()
            .is_some_and(|(last, parent)| parent == dp.as_slice() && *di == *last)
    });
    if is_drop_here {
        draw_drop_indicator(ui, path.len());
    }

    // Rect of the label editor on the selected row: dragging inside it selects
    // text and must not start a row drag.
    let mut edit_rect: Option<egui::Rect> = None;
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
            edit_rect = Some(r.rect);
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
    if !is_root {
        // Start a drag once the primary button, pressed on this row, has
        // moved a few pixels.
        if drag.dragging.is_none() {
            let (down, origin, pos) = ui.ctx().input(|i| {
                (
                    i.pointer.primary_down(),
                    i.pointer.press_origin(),
                    i.pointer.latest_pos(),
                )
            });
            if down
                && let (Some(origin), Some(pos)) = (origin, pos)
                && row_rect.contains(origin)
                && !edit_rect.is_some_and(|r| r.contains(origin))
                && (pos - origin).length() > DRAG_THRESHOLD
            {
                drag.dragging = Some(path.clone());
            }
        }

        // This row is a potential drop target, unless it is the dragged node
        // itself or one of its descendants.
        if let Some(src) = &drag.dragging
            && !path.starts_with(src)
            && let Some(pos) = ui.ctx().input(|i| i.pointer.latest_pos())
            && row_rect.contains(pos)
        {
            let parent_path = path[..path.len() - 1].to_vec();
            let idx = *path.last().unwrap();
            // Drop above or below based on pointer position relative to row center
            let insert_idx = if pos.y < row_rect.center().y {
                idx
            } else {
                idx + 1
            };
            drag.drop_target = Some((parent_path, insert_idx));
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
        let after_last = drag
            .drop_target
            .as_ref()
            .is_some_and(|(dp, di)| dp.as_slice() == path.as_slice() && *di == node.children.len());
        if after_last {
            draw_drop_indicator(ui, path.len() + 1);
        }
    }

    (clicked, remove)
}

// ─── Hover preview ────────────────────────────────────────────────────────────

pub use flexplore::config::HoverPreview;

/// While the user drags a slider or types, the live document is still sent to
/// peers this often; the undo entry is only written when the interaction ends.
const LIVE_SEND_INTERVAL: f64 = 0.15;

/// Dropdown options the pointer rests on this frame; each is applied to the
/// selected node as a live preview and reverted when the pointer leaves.
#[derive(Clone, Copy, Default)]
struct HoverState {
    /// Some popup is open or an option is hovered: keep the preview alive.
    any: bool,
    direction: Option<FlexDirection>,
    wrap: Option<FlexWrap>,
    justify: Option<JustifyContent>,
    align_items: Option<AlignItems>,
    align_content: Option<AlignContent>,
    row_gap: Option<ValueConfig>,
    column_gap: Option<ValueConfig>,
    width: Option<ValueConfig>,
    height: Option<ValueConfig>,
    min_width: Option<ValueConfig>,
    min_height: Option<ValueConfig>,
    max_width: Option<ValueConfig>,
    max_height: Option<ValueConfig>,
    basis: Option<ValueConfig>,
    align_self: Option<AlignSelf>,
}

// ─── Panel context ───────────────────────────────────────────────────────────

/// Everything the panel sections read and mutate, bundled so each section is
/// a plain function of `(ui, &mut PanelCtx)`.
struct PanelCtx<'a> {
    cfg: &'a mut FlexConfig,
    history: &'a mut UndoHistory,
    preview: &'a mut Option<FlexConfig>,
    import_buf: &'a mut String,
    toast: &'a mut Toast,
    cg: &'a mut CodegenState,
    drag: &'a mut DragState,
    /// Theme the egui visuals were last built for; `None` forces a re-apply.
    applied_theme: &'a mut Option<Theme>,
    show_help: &'a mut bool,
    hover: HoverState,
    /// The document was edited this frame: rebuild, regenerate code, and
    /// (once the interaction ends) write an undo entry.
    changed: bool,
    /// The document must be sent to peers this frame.
    net_dirty: bool,
    /// Path of the selected node, kept in sync with `cfg.selected()`.
    sel_path: Vec<usize>,
}

impl PanelCtx<'_> {
    fn is_root(&self) -> bool {
        self.sel_path.is_empty()
    }

    fn select(&mut self, path: Vec<usize>) {
        self.sel_path = path.clone();
        self.cfg.select(path);
    }

    fn selected_node_mut(&mut self) -> Option<&mut NodeConfig> {
        self.cfg.root.get_mut(&self.sel_path)
    }

    fn toast(&mut self, ctx: &egui::Context, msg: impl Into<String>, secs: f64) {
        let now = ctx.input(|i| i.time);
        *self.toast = Some((msg.into(), now + secs));
    }

    /// Replace the whole document (undo/redo/open/import/net).
    fn replace_document(&mut self, doc: FlexConfig, push_history: bool) {
        *self.cfg = doc;
        self.cfg.request_rebuild();
        *self.preview = None;
        if push_history {
            self.history.push(self.cfg.clone());
        }
        self.cg.dirty = true;
        self.net_dirty = true;
        self.sel_path = self.cfg.selected().to_vec();
    }

    fn undo(&mut self) {
        if let Some(snapshot) = self.history.undo() {
            let doc = snapshot.clone();
            self.replace_document(doc, false);
        }
    }

    fn redo(&mut self) {
        if let Some(snapshot) = self.history.redo() {
            let doc = snapshot.clone();
            self.replace_document(doc, false);
        }
    }

    fn new_leaf_label(&self) -> String {
        format!("node{}", self.cfg.root.count_leaves() + 1)
    }

    fn add_child(&mut self) {
        let lbl = self.new_leaf_label();
        if let Some(node) = self.selected_node_mut() {
            node.children.push(NodeConfig::new_leaf(&lbl, 80.0, 80.0));
            self.changed = true;
        }
    }

    fn add_sibling(&mut self) {
        if self.is_root() {
            return;
        }
        let pidx = self.sel_path.len() - 1;
        let sibling_idx = self.sel_path[pidx];
        let lbl = self.new_leaf_label();
        if let Some(parent) = self.cfg.root.get_mut(&self.sel_path[..pidx]) {
            let insert_at = (sibling_idx + 1).min(parent.children.len());
            parent
                .children
                .insert(insert_at, NodeConfig::new_leaf(&lbl, 80.0, 80.0));
            self.changed = true;
        }
    }

    fn duplicate(&mut self) {
        if self.is_root() {
            return;
        }
        let pidx = self.sel_path.len() - 1;
        let sibling_idx = self.sel_path[pidx];
        if let Some(parent) = self.cfg.root.get_mut(&self.sel_path[..pidx])
            && let Some(clone) = parent.children.get(sibling_idx).cloned()
        {
            let insert_at = (sibling_idx + 1).min(parent.children.len());
            parent.children.insert(insert_at, clone);
            self.changed = true;
        }
    }

    fn delete_selected(&mut self) {
        if self.is_root() {
            return;
        }
        let pidx = self.sel_path.len() - 1;
        let idx = self.sel_path[pidx];
        if let Some(parent) = self.cfg.root.get_mut(&self.sel_path[..pidx])
            && idx < parent.children.len()
        {
            parent.children.remove(idx);
        }
        let parent_path = self.sel_path[..pidx].to_vec();
        self.select(parent_path);
        self.changed = true;
    }

    fn select_parent(&mut self) {
        if self.is_root() {
            return;
        }
        let mut path = self.sel_path.clone();
        path.pop();
        self.select(path);
    }

    /// Apply one hovered value to the selected node as a preview, taking a
    /// snapshot of the document first. Returns true if the value changed.
    fn apply_hover<T: PartialEq + Clone>(
        &mut self,
        opt: Option<T>,
        get: impl Fn(&NodeConfig) -> T,
        set: impl FnOnce(&mut NodeConfig, T),
    ) -> bool {
        let Some(v) = opt else { return false };
        let Some(node) = self.cfg.root.get(&self.sel_path) else {
            return false;
        };
        if get(node) == v {
            return false;
        }
        if self.preview.is_none() {
            *self.preview = Some(self.cfg.clone());
        }
        if let Some(node) = self.cfg.root.get_mut(&self.sel_path) {
            set(node, v);
        }
        true
    }

    /// Preview a document-level value (theme, palette, art style).
    fn preview_setting<T: PartialEq + Copy>(
        &mut self,
        hovered: Option<T>,
        get: impl Fn(&FlexConfig) -> T,
        set: impl FnOnce(&mut FlexConfig, T),
    ) -> bool {
        let Some(v) = hovered else { return false };
        self.hover.any = true;
        if get(self.cfg) == v {
            return false;
        }
        if self.preview.is_none() {
            *self.preview = Some(self.cfg.clone());
        }
        set(self.cfg, v);
        true
    }
}

// ─── Keyboard shortcuts ──────────────────────────────────────────────────────

struct Shortcuts {
    undo: bool,
    redo: bool,
    add_child: bool,
    add_sibling: bool,
    delete: bool,
    parent: bool,
    duplicate: bool,
    save: bool,
    open: bool,
    help: bool,
    arrow: Option<Vec2>,
}

/// Consume this frame's shortcut keys.
fn read_shortcuts(ctx: &egui::Context) -> Shortcuts {
    // While a text field has focus, leave every key to it (arrows, Delete,
    // Escape, Ctrl+Z, `?` ...). `egui_wants_keyboard_input` reflects the
    // previous frame, which is what we want here.
    let typing = ctx.egui_wants_keyboard_input();
    let key =
        |mods: egui::Modifiers, k: egui::Key| !typing && ctx.input_mut(|i| i.consume_key(mods, k));
    // Redo before undo: `consume_key` ignores an extra Shift, so checking
    // Ctrl+Z first would swallow Ctrl+Shift+Z as an undo.
    let redo = key(egui::Modifiers::COMMAND, egui::Key::Y)
        || key(
            egui::Modifiers::COMMAND.plus(egui::Modifiers::SHIFT),
            egui::Key::Z,
        );
    let undo = key(egui::Modifiers::COMMAND, egui::Key::Z);
    let add_child = key(egui::Modifiers::COMMAND, egui::Key::Enter);
    let add_sibling = key(egui::Modifiers::SHIFT, egui::Key::Enter);
    let delete = key(egui::Modifiers::NONE, egui::Key::Delete);
    let parent = key(egui::Modifiers::NONE, egui::Key::Escape);
    let duplicate = key(egui::Modifiers::COMMAND, egui::Key::D);
    let save = ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::S));
    let open = ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::O));
    let help = key(egui::Modifiers::SHIFT, egui::Key::Slash);

    // Arrow keys → spatial navigation
    let arrow = if key(egui::Modifiers::NONE, egui::Key::ArrowUp) {
        Some(Vec2::new(0.0, -1.0))
    } else if key(egui::Modifiers::NONE, egui::Key::ArrowDown) {
        Some(Vec2::new(0.0, 1.0))
    } else if key(egui::Modifiers::NONE, egui::Key::ArrowLeft) {
        Some(Vec2::new(-1.0, 0.0))
    } else if key(egui::Modifiers::NONE, egui::Key::ArrowRight) {
        Some(Vec2::new(1.0, 0.0))
    } else {
        None
    };

    Shortcuts {
        undo,
        redo,
        add_child,
        add_sibling,
        delete,
        parent,
        duplicate,
        save,
        open,
        help,
        arrow,
    }
}

/// Undo/redo, theme, file and help shortcuts.
fn apply_global_shortcuts(
    ctx: &egui::Context,
    p: &mut PanelCtx,
    keys: &Shortcuts,
    arrow_nav: &mut ArrowNav,
) {
    if keys.undo {
        p.undo();
    }
    if keys.redo {
        p.redo();
    }
    arrow_nav.0 = keys.arrow;

    if *p.applied_theme != Some(p.cfg.theme) {
        apply_theme(ctx, p.cfg.theme);
        *p.applied_theme = Some(p.cfg.theme);
    }

    // ── File picker (Ctrl+S / Ctrl+O) ─────────────────────────────────────────
    if keys.save {
        save_file(p, ctx);
    }
    if keys.open {
        open_file(p, ctx);
    }

    if keys.help {
        *p.show_help = !*p.show_help;
    }
}

/// Tree editing shortcuts (add / duplicate / delete / select parent).
fn apply_tree_shortcuts(p: &mut PanelCtx, keys: &Shortcuts) {
    if keys.add_child {
        p.add_child();
    }
    if keys.add_sibling {
        p.add_sibling();
    }
    if keys.duplicate {
        p.duplicate();
    }
    if keys.delete {
        p.delete_selected();
    }
    if keys.parent {
        p.select_parent();
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
    mut hover_preview: ResMut<HoverPreview>,
    mut applied_theme: Local<Option<Theme>>,
    mut commit_pending: Local<bool>,
    mut last_live_send: Local<f64>,
    mut import_buf: Local<String>,
    mut toast: Local<Toast>,
    mut codegen: Local<CodegenState>,
    mut drag: Local<DragState>,
    mut show_help: Local<bool>,
    #[cfg(feature = "multiplayer")] mut pending_edits: Option<ResMut<flexplore::net::PendingEdits>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    // egui 0.34+ shows top-level panels inside a root `Ui` rather than on the
    // `Context` directly.
    let mut root_ui = egui::Ui::new(
        ctx.clone(),
        egui::Id::new("flexplore_root"),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );

    let sel_path = cfg.selected().to_vec();
    let mut p = PanelCtx {
        cfg: &mut cfg,
        history: &mut history,
        preview: &mut hover_preview.0,
        import_buf: &mut import_buf,
        toast: &mut toast,
        cg: &mut codegen,
        drag: &mut drag,
        applied_theme: &mut applied_theme,
        show_help: &mut show_help,
        hover: HoverState::default(),
        changed: false,
        net_dirty: false,
        sel_path,
    };

    let keys = read_shortcuts(ctx);
    apply_global_shortcuts(ctx, &mut p, &keys, &mut arrow_nav);
    apply_tree_shortcuts(&mut p, &keys);

    draw_left_panel(&mut root_ui, &mut p);

    right_panel_open.0 = p.cg.preview_open;
    if p.cg.preview_open {
        draw_codegen_panel(&mut root_ui, &mut p);
    }

    commit_or_revert(ctx, &mut p, &mut commit_pending, &mut last_live_send);
    commit_drag(ctx, &mut p);
    draw_toast(ctx, p.toast);
    draw_help_window(ctx, p.show_help);

    // ── Send accumulated edits to the network ─────────────────────────────────
    #[cfg(feature = "multiplayer")]
    if p.net_dirty
        && let Some(ref mut edits) = pending_edits
    {
        edits
            .0
            .push(flexplore_net::LayoutEdit::ReplaceRoot(Box::new(
                p.cfg.root.clone(),
            )));
        edits.0.push(flexplore_net::LayoutEdit::UpdateSettings {
            bg_mode: p.cfg.bg_mode,
            art_style: p.cfg.art_style,
            art_seed: p.cfg.art_seed,
            art_depth: p.cfg.art_depth,
            theme: p.cfg.theme,
            palette: p.cfg.palette,
        });
    }

    Ok(())
}

/// Commit this frame's edit (rebuild, undo entry, live send) or revert an
/// expired hover preview.
fn commit_or_revert(
    ctx: &egui::Context,
    p: &mut PanelCtx,
    commit_pending: &mut bool,
    last_live_send: &mut f64,
) {
    if p.changed {
        *p.preview = None;
        p.cfg.request_rebuild();
        p.cg.dirty = true;
        *commit_pending = true;
    } else if !p.hover.any
        && let Some(saved) = p.preview.take()
    {
        *p.cfg = saved;
        p.cfg.sanitize_selection();
        p.cfg.request_rebuild();
        *p.applied_theme = None;
        p.cg.dirty = true;
    }
    // A slider drag or a typing burst becomes one undo step, written when the
    // interaction ends; peers still see the live document at a steady rate.
    if *commit_pending {
        let now = ctx.input(|i| i.time);
        let interacting = ctx.input(|i| i.pointer.any_down()) || ctx.egui_wants_keyboard_input();
        if !interacting {
            p.history.push(p.cfg.clone());
            p.net_dirty = true;
            *commit_pending = false;
            *last_live_send = now;
        } else if now - *last_live_send > LIVE_SEND_INTERVAL {
            p.net_dirty = true;
            *last_live_send = now;
        }
    }
}

/// On pointer release, move the dragged tree node to the drop target.
fn commit_drag(ctx: &egui::Context, p: &mut PanelCtx) {
    if ctx.input(|i| i.pointer.any_down()) {
        return;
    }
    let src = p.drag.dragging.take();
    let dst = p.drag.drop_target.take();
    if let (Some(src), Some((dst_parent, dst_idx))) = (src, dst)
        && let Some(new_path) = move_node(&mut p.cfg.root, &src, &dst_parent, dst_idx)
    {
        p.select(new_path);
        p.cfg.request_rebuild();
        p.history.push(p.cfg.clone());
        p.cg.dirty = true;
        p.net_dirty = true;
    }
}

fn draw_toast(ctx: &egui::Context, toast: &mut Toast) {
    let Some((msg, expiry)) = toast else { return };
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
                        ui.colored_label(egui::Color32::WHITE, msg.as_str());
                    });
            });
        ctx.request_repaint();
    } else {
        *toast = None;
    }
}

fn draw_help_window(ctx: &egui::Context, show_help: &mut bool) {
    if !*show_help {
        return;
    }
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

// ─── Left panel ──────────────────────────────────────────────────────────────

fn draw_left_panel(root_ui: &mut egui::Ui, p: &mut PanelCtx) {
    egui::Panel::left("flex_panel")
        .exact_size(PANEL_WIDTH)
        .resizable(false)
        .show_separator_line(false)
        .show(root_ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(4.0);
                draw_toolbar(ui, p);
                ui.add_space(4.0);
                draw_mode_tabs(ui, p);
                ui.separator();
                draw_layout_panel(ui, p);
            });
        });
}

/// Theme picker, undo/redo and help buttons.
fn draw_toolbar(ui: &mut egui::Ui, p: &mut PanelCtx) {
    ui.horizontal(|ui| {
        let mut hover_theme: Option<Theme> = None;
        let theme_resp = egui::ComboBox::from_id_salt("theme_sel")
            .selected_text(p.cfg.theme.to_string())
            .width(90.0)
            .show_ui(ui, |ui| {
                for t in Theme::iter() {
                    let r = ui.selectable_label(p.cfg.theme == t, t.to_string());
                    if r.clicked() {
                        p.cfg.theme = t;
                        p.changed = true;
                    } else if r.hovered() {
                        hover_theme = Some(t);
                    }
                }
            });
        if theme_resp.inner.is_some() {
            p.hover.any = true;
        }
        if p.preview_setting(hover_theme, |c| c.theme, |c, t| c.theme = t) {
            *p.applied_theme = None;
        }
        ui.separator();
        if ui
            .add_enabled(p.history.can_undo(), egui::Button::new("⟲ Undo"))
            .clicked()
        {
            p.undo();
        }
        if ui
            .add_enabled(p.history.can_redo(), egui::Button::new("⟳ Redo"))
            .clicked()
        {
            p.redo();
        }
        if ui
            .button("?")
            .on_hover_text("Keyboard shortcuts (Shift+/)")
            .clicked()
        {
            *p.show_help = !*p.show_help;
        }
    });
}

/// Flexbox / CSS Grid tabs: the tab reflects the selected node's display
/// mode, and clicking the other one switches the mode.
fn draw_mode_tabs(ui: &mut egui::Ui, p: &mut PanelCtx) {
    let mode = p
        .cfg
        .root
        .get(&p.sel_path)
        .map(|n| n.display_mode)
        .unwrap_or(DisplayMode::Flex);
    let mut tab = match mode {
        DisplayMode::Flex => LeftTab::Flexbox,
        DisplayMode::Grid => LeftTab::CssGrid,
    };
    ui.horizontal(|ui| {
        ui.selectable_value(&mut tab, LeftTab::Flexbox, "Flexbox");
        ui.selectable_value(&mut tab, LeftTab::CssGrid, "CSS Grid");
    });
    let target_mode = match tab {
        LeftTab::CssGrid => DisplayMode::Grid,
        LeftTab::Flexbox => DisplayMode::Flex,
    };
    if let Some(node) = p.selected_node_mut()
        && node.display_mode != target_mode
    {
        node.display_mode = target_mode;
        p.changed = true;
    }
}

// ─── Layout panel contents (shared by Flexbox and CSS Grid tabs) ─────────────

fn draw_layout_panel(ui: &mut egui::Ui, p: &mut PanelCtx) {
    draw_tree_section(ui, p);

    ui.add_space(6.0);

    if p.cfg.root.get(&p.sel_path).is_some() {
        draw_node_sections(ui, p);
    }

    draw_templates_section(ui, p);
    ui.add_space(6.0);
    draw_background_section(ui, p);

    ui.add_space(6.0);
    if ui
        .button("Reset to defaults")
        .on_hover_text("Restore all settings and the node tree to the initial state")
        .clicked()
    {
        *p.cfg = FlexConfig::default();
        *p.preview = None;
        p.changed = true;
    }

    ui.add_space(6.0);
    draw_import_export_section(ui, p);

    ui.add_space(4.0);
    draw_code_export(ui, p);
}

fn draw_tree_section(ui: &mut egui::Ui, p: &mut PanelCtx) {
    egui::CollapsingHeader::new("Tree")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .button("+ Child")
                    .on_hover_text("Add a new child node inside the selected node")
                    .clicked()
                {
                    p.add_child();
                }
                if !p.is_root()
                    && ui
                        .button("+ Sibling")
                        .on_hover_text("Add a new node next to the selected node (same parent)")
                        .clicked()
                {
                    p.add_sibling();
                }
                if !p.is_root()
                    && ui
                        .button("Dup")
                        .on_hover_text("Duplicate selected node (Ctrl+D)")
                        .clicked()
                {
                    p.duplicate();
                }
            });
            ui.add_space(2.0);
            // The target is recomputed from the rows drawn this frame.
            p.drag.drop_target = None;
            let selected = p.sel_path.clone();
            let (clicked, remove_req) = draw_tree_ui(
                ui,
                &mut p.cfg.root,
                &mut vec![],
                &selected,
                &mut p.changed,
                p.drag,
            );
            if remove_req {
                p.delete_selected();
            }
            if let Some(path) = clicked
                && path != p.sel_path
            {
                p.select(path);
                *p.preview = None;
            }
        });
}

/// Sections editing the selected node. Every section re-fetches the node,
/// so an edit in one section cannot invalidate a borrow in the next.
fn draw_node_sections(ui: &mut egui::Ui, p: &mut PanelCtx) {
    {
        let Some(n) = p.cfg.root.get_mut(&p.sel_path) else {
            return;
        };
        ui.horizontal(|ui| {
            if ui
                .checkbox(&mut n.visible, "Visible")
                .on_hover_text("Whether this node is displayed in the layout")
                .changed()
            {
                p.changed = true;
            }
        });
    }
    ui.add_space(4.0);

    // ── Text content ──────────────────────────────────────────────
    {
        let Some(n) = p.cfg.root.get_mut(&p.sel_path) else {
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
                p.changed = true;
            }
        });
    }
    ui.add_space(4.0);

    let sel_display_mode = p
        .cfg
        .root
        .get(&p.sel_path)
        .map(|n| n.display_mode)
        .unwrap_or(DisplayMode::Flex);

    if sel_display_mode == DisplayMode::Grid {
        draw_grid_container_section(ui, p);
    } else {
        draw_flex_container_section(ui, p);
    }

    ui.add_space(6.0);
    draw_sizing_section(ui, p);
    ui.add_space(6.0);
    draw_spacing_section(ui, p);
    ui.add_space(6.0);
    draw_border_section(ui, p);
    ui.add_space(6.0);

    // ── Item properties (non-root) ──────────────────────────────
    // Whether to show grid-item or flex-item controls depends on the
    // *parent's* display mode, not the node's own.
    if p.is_root() {
        return;
    }
    let parent_display_mode = p
        .cfg
        .root
        .get(&p.sel_path[..p.sel_path.len() - 1])
        .map(|n| n.display_mode)
        .unwrap_or(DisplayMode::Flex);
    match parent_display_mode {
        DisplayMode::Grid => draw_grid_item_section(ui, p),
        DisplayMode::Flex => draw_flex_item_section(ui, p),
    }
    ui.add_space(6.0);
}

fn draw_flex_container_section(ui: &mut egui::Ui, p: &mut PanelCtx) {
    egui::CollapsingHeader::new("Flex Container")
        .default_open(true)
        .show(ui, |ui| {
            ui.add_space(4.0);
            egui::Grid::new("cg1")
                .num_columns(2)
                .spacing([10.0, 6.0])
                .show(ui, |ui| {
                    let Some(n) = p.cfg.root.get_mut(&p.sel_path) else {
                        return;
                    };
                    label_with_help(
                        ui,
                        "direction",
                        "The main axis along which children are laid out",
                    );
                    p.hover.direction = combo(
                        ui,
                        "fd",
                        &mut n.flex_direction,
                        &[
                            ("Row", FlexDirection::Row),
                            ("Column", FlexDirection::Column),
                            ("RowReverse", FlexDirection::RowReverse),
                            ("ColumnReverse", FlexDirection::ColumnReverse),
                        ],
                        &mut p.changed,
                        &mut p.hover.any,
                    );
                    ui.end_row();

                    label_with_help(
                        ui,
                        "wrap",
                        "Whether children wrap to new lines when they overflow",
                    );
                    p.hover.wrap = combo(
                        ui,
                        "fw",
                        &mut n.flex_wrap,
                        &[
                            ("NoWrap", FlexWrap::NoWrap),
                            ("Wrap", FlexWrap::Wrap),
                            ("WrapReverse", FlexWrap::WrapReverse),
                        ],
                        &mut p.changed,
                        &mut p.hover.any,
                    );
                    ui.end_row();

                    label_with_help(
                        ui,
                        "justify",
                        "How children are distributed along the main axis",
                    );
                    p.hover.justify = combo(
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
                        &mut p.changed,
                        &mut p.hover.any,
                    );
                    ui.end_row();

                    label_with_help(
                        ui,
                        "align-items",
                        "How children are aligned along the cross axis",
                    );
                    p.hover.align_items = combo(
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
                        &mut p.changed,
                        &mut p.hover.any,
                    );
                    ui.end_row();

                    label_with_help(
                        ui,
                        "align-content",
                        "How wrapped lines are distributed along the cross axis",
                    );
                    p.hover.align_content = combo(
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
                        &mut p.changed,
                        &mut p.hover.any,
                    );
                    ui.end_row();
                });
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);
            egui::Grid::new("cg2")
                .num_columns(2)
                .spacing([10.0, 6.0])
                .show(ui, |ui| {
                    let Some(n) = p.cfg.root.get_mut(&p.sel_path) else {
                        return;
                    };
                    label_with_help(ui, "row-gap", "Spacing between rows of children");
                    p.hover.row_gap =
                        val_row(ui, "rg", &mut n.row_gap, &mut p.changed, &mut p.hover.any);
                    ui.end_row();
                    label_with_help(ui, "column-gap", "Spacing between columns of children");
                    p.hover.column_gap = val_row(
                        ui,
                        "cgap",
                        &mut n.column_gap,
                        &mut p.changed,
                        &mut p.hover.any,
                    );
                    ui.end_row();
                });
            ui.add_space(2.0);

            apply_container_hover(p);
        });
}

/// Preview whichever container option is hovered (flex and grid containers
/// share these fields).
fn apply_container_hover(p: &mut PanelCtx) {
    let h = p.hover;
    let has_hover = h.direction.is_some()
        || h.wrap.is_some()
        || h.justify.is_some()
        || h.align_items.is_some()
        || h.align_content.is_some()
        || h.row_gap.is_some()
        || h.column_gap.is_some();
    if !has_hover {
        return;
    }
    p.hover.any = true;
    let needs_rebuild = p.apply_hover(
        h.direction,
        |n| n.flex_direction,
        |n, v| n.flex_direction = v,
    ) | p.apply_hover(h.wrap, |n| n.flex_wrap, |n, v| n.flex_wrap = v)
        | p.apply_hover(
            h.justify,
            |n| n.justify_content,
            |n, v| n.justify_content = v,
        )
        | p.apply_hover(h.align_items, |n| n.align_items, |n, v| n.align_items = v)
        | p.apply_hover(
            h.align_content,
            |n| n.align_content,
            |n, v| n.align_content = v,
        )
        | p.apply_hover(h.row_gap, |n| n.row_gap, |n, v| n.row_gap = v)
        | p.apply_hover(h.column_gap, |n| n.column_gap, |n, v| n.column_gap = v);
    if needs_rebuild {
        p.cfg.request_rebuild();
    }
}

fn draw_sizing_section(ui: &mut egui::Ui, p: &mut PanelCtx) {
    egui::CollapsingHeader::new("Sizing")
        .default_open(true)
        .show(ui, |ui| {
            ui.add_space(4.0);
            egui::Grid::new("sg")
                .num_columns(2)
                .spacing([10.0, 6.0])
                .show(ui, |ui| {
                    let Some(n) = p.cfg.root.get_mut(&p.sel_path) else {
                        return;
                    };
                    let (changed, any) = (&mut p.changed, &mut p.hover.any);
                    label_with_help(ui, "width", "The preferred width of this node");
                    p.hover.width = val_row(ui, "sw", &mut n.width, changed, any);
                    ui.end_row();
                    label_with_help(ui, "height", "The preferred height of this node");
                    p.hover.height = val_row(ui, "sh", &mut n.height, changed, any);
                    ui.end_row();
                    label_with_help(ui, "min-width", "The minimum width this node can shrink to");
                    p.hover.min_width = val_row(ui, "sminw", &mut n.min_width, changed, any);
                    ui.end_row();
                    label_with_help(
                        ui,
                        "min-height",
                        "The minimum height this node can shrink to",
                    );
                    p.hover.min_height = val_row(ui, "sminh", &mut n.min_height, changed, any);
                    ui.end_row();
                    label_with_help(ui, "max-width", "The maximum width this node can grow to");
                    p.hover.max_width = val_row(ui, "smaxw", &mut n.max_width, changed, any);
                    ui.end_row();
                    label_with_help(ui, "max-height", "The maximum height this node can grow to");
                    p.hover.max_height = val_row(ui, "smaxh", &mut n.max_height, changed, any);
                    ui.end_row();
                });
            ui.add_space(2.0);

            let h = p.hover;
            let has_hover = h.width.is_some()
                || h.height.is_some()
                || h.min_width.is_some()
                || h.min_height.is_some()
                || h.max_width.is_some()
                || h.max_height.is_some();
            if has_hover {
                p.hover.any = true;
                let needs_rebuild = p.apply_hover(h.width, |n| n.width, |n, v| n.width = v)
                    | p.apply_hover(h.height, |n| n.height, |n, v| n.height = v)
                    | p.apply_hover(h.min_width, |n| n.min_width, |n, v| n.min_width = v)
                    | p.apply_hover(h.min_height, |n| n.min_height, |n, v| n.min_height = v)
                    | p.apply_hover(h.max_width, |n| n.max_width, |n, v| n.max_width = v)
                    | p.apply_hover(h.max_height, |n| n.max_height, |n, v| n.max_height = v);
                if needs_rebuild {
                    p.cfg.request_rebuild();
                }
            }
        });
}

fn draw_spacing_section(ui: &mut egui::Ui, p: &mut PanelCtx) {
    egui::CollapsingHeader::new("Spacing")
        .default_open(true)
        .show(ui, |ui| {
            ui.add_space(4.0);
            let Some(n) = p.cfg.root.get_mut(&p.sel_path) else {
                return;
            };
            sides_editor(
                ui,
                "padding",
                "pad",
                "Space between border and children",
                &mut n.padding,
                &mut p.changed,
            );
            ui.add_space(4.0);
            sides_editor(
                ui,
                "margin",
                "mar",
                "Space outside the border",
                &mut n.margin,
                &mut p.changed,
            );
        });
}

fn draw_border_section(ui: &mut egui::Ui, p: &mut PanelCtx) {
    egui::CollapsingHeader::new("Border")
        .default_open(false)
        .show(ui, |ui| {
            ui.add_space(4.0);
            let Some(n) = p.cfg.root.get_mut(&p.sel_path) else {
                return;
            };
            sides_editor(
                ui,
                "border-width",
                "bw",
                "Border thickness per side",
                &mut n.border_width,
                &mut p.changed,
            );
            ui.add_space(4.0);
            corners_editor(
                ui,
                "border-radius",
                "br",
                "Corner rounding",
                &mut n.border_radius,
                &mut p.changed,
            );
        });
}

fn draw_flex_item_section(ui: &mut egui::Ui, p: &mut PanelCtx) {
    egui::CollapsingHeader::new("Flex Item")
        .default_open(true)
        .show(ui, |ui| {
            ui.add_space(4.0);
            egui::Grid::new("ig")
                .num_columns(2)
                .spacing([10.0, 6.0])
                .show(ui, |ui| {
                    let Some(n) = p.cfg.root.get_mut(&p.sel_path) else {
                        return;
                    };
                    label_with_help(
                        ui,
                        "flex-grow",
                        "How much this node grows relative to siblings (0 = don't grow)",
                    );
                    p.changed |= ui
                        .add(egui::Slider::new(&mut n.flex_grow, 0.0..=5.0).max_decimals(2))
                        .changed();
                    ui.end_row();
                    label_with_help(
                        ui,
                        "flex-shrink",
                        "How much this node shrinks relative to siblings (0 = don't shrink)",
                    );
                    p.changed |= ui
                        .add(egui::Slider::new(&mut n.flex_shrink, 0.0..=5.0).max_decimals(2))
                        .changed();
                    ui.end_row();
                    label_with_help(
                        ui,
                        "flex-basis",
                        "Initial size along main axis before grow/shrink",
                    );
                    p.hover.basis = val_row(
                        ui,
                        "ib",
                        &mut n.flex_basis,
                        &mut p.changed,
                        &mut p.hover.any,
                    );
                    ui.end_row();
                    label_with_help(
                        ui,
                        "align-self",
                        "Override parent's align-items for this child",
                    );
                    p.hover.align_self = combo(
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
                        &mut p.changed,
                        &mut p.hover.any,
                    );
                    ui.end_row();
                    label_with_help(
                        ui,
                        "order",
                        "Controls visual ordering of flex items (lower first)",
                    );
                    p.changed |= ui.add(egui::Slider::new(&mut n.order, -10..=10)).changed();
                    ui.end_row();
                });
            ui.add_space(2.0);

            let h = p.hover;
            if h.basis.is_some() || h.align_self.is_some() {
                p.hover.any = true;
                let needs_rebuild =
                    p.apply_hover(h.basis, |n| n.flex_basis, |n, v| n.flex_basis = v)
                        | p.apply_hover(h.align_self, |n| n.align_self, |n, v| n.align_self = v);
                if needs_rebuild {
                    p.cfg.request_rebuild();
                }
            }
        });
}

fn draw_templates_section(ui: &mut egui::Ui, p: &mut PanelCtx) {
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
                        p.cfg.root = builder();
                        p.select(vec![]);
                        *p.preview = None;
                        p.changed = true;
                    }
                }
            });
        });
}

fn draw_background_section(ui: &mut egui::Ui, p: &mut PanelCtx) {
    egui::CollapsingHeader::new("Background")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let prev = p.cfg.bg_mode;
                ui.radio_value(&mut p.cfg.bg_mode, BackgroundMode::Pastel, "Pastel")
                    .on_hover_text("Fill leaf nodes with solid pastel colors");
                ui.radio_value(
                    &mut p.cfg.bg_mode,
                    BackgroundMode::RandomArt,
                    "Generative Art",
                )
                .on_hover_text("Fill leaf nodes with procedurally generated art textures");
                if p.cfg.bg_mode != prev {
                    p.changed = true;
                }
            });
            let mut hover_palette: Option<ColorPalette> = None;
            ui.horizontal(|ui| {
                ui.label("palette");
                let pal_resp = egui::ComboBox::from_id_salt("palette_sel")
                    .selected_text(p.cfg.palette.to_string())
                    .width(110.0)
                    .show_ui(ui, |ui| {
                        for pal in ColorPalette::iter() {
                            let r = ui.selectable_label(p.cfg.palette == pal, pal.to_string());
                            if r.clicked() {
                                p.cfg.palette = pal;
                                p.changed = true;
                            } else if r.hovered() {
                                hover_palette = Some(pal);
                            }
                        }
                    });
                if pal_resp.inner.is_some() {
                    p.hover.any = true;
                }
            });
            if p.preview_setting(hover_palette, |c| c.palette, |c, v| c.palette = v) {
                p.cfg.request_rebuild();
            }
            if p.cfg.bg_mode == BackgroundMode::RandomArt {
                let cur = p.cfg.art_style.to_string();
                let mut hover_art: Option<ArtStyle> = None;
                let art_resp = egui::ComboBox::from_label("style")
                    .selected_text(&cur)
                    .show_ui(ui, |ui| {
                        for style in ArtStyle::iter() {
                            let name = style.to_string();
                            let r = ui.selectable_label(p.cfg.art_style == style, &name);
                            if r.clicked() {
                                p.cfg.art_style = style;
                                p.changed = true;
                            } else if r.hovered() {
                                hover_art = Some(style);
                            }
                        }
                    });
                if art_resp.inner.is_some() {
                    p.hover.any = true;
                }
                if p.preview_setting(hover_art, |c| c.art_style, |c, v| c.art_style = v) {
                    p.cfg.request_rebuild();
                }
                let pd = p.cfg.art_depth;
                ui.add(egui::Slider::new(&mut p.cfg.art_depth, 1..=9).text("depth"))
                    .on_hover_text("Expression tree depth — higher = more complex");
                if p.cfg.art_depth != pd {
                    p.changed = true;
                }
                ui.add(
                    egui::Slider::new(&mut p.cfg.art_anim, 0.0..=2.0)
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
                        p.cfg.art_seed = rand::random::<u64>();
                        p.changed = true;
                    }
                    if ui
                        .button("Regenerate")
                        .on_hover_text("Re-render art with current settings")
                        .clicked()
                    {
                        // Only the textures change, not the document: no undo entry.
                        p.cfg.request_art_regen();
                    }
                });
            }
        });
}

fn draw_import_export_section(ui: &mut egui::Ui, p: &mut PanelCtx) {
    egui::CollapsingHeader::new("Import / Export")
        .default_open(false)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .button("Save file (Ctrl+S)")
                    .on_hover_text("Save layout to a JSON file")
                    .clicked()
                {
                    save_file(p, ui.ctx());
                }
                let open_btn = egui::Button::new("Open file (Ctrl+O)");
                if ui
                    .add_enabled(cfg!(not(target_arch = "wasm32")), open_btn)
                    .on_hover_text("Load layout from a JSON file")
                    .on_disabled_hover_text(
                        "Not available in the browser — paste the file into \"Import JSON\" below",
                    )
                    .clicked()
                {
                    open_file(p, ui.ctx());
                }
            });
            ui.add_space(4.0);
            ui.label("Paste JSON to import:");
            ui.add(
                egui::TextEdit::multiline(p.import_buf)
                    .desired_rows(3)
                    .desired_width(f32::INFINITY),
            );
            if ui.button("Load from JSON").clicked()
                && !p.import_buf.is_empty()
                && let Some(loaded) = crate::persist::import_json(p.import_buf)
            {
                p.replace_document(loaded, true);
                p.import_buf.clear();
            }

            #[cfg(target_arch = "wasm32")]
            {
                ui.add_space(4.0);
                if ui
                    .button("Share URL")
                    .on_hover_text("Copy a shareable URL to clipboard")
                    .clicked()
                    && let Some(url) = crate::persist::make_share_url(p.cfg)
                {
                    ui.ctx().copy_text(url);
                    p.toast(ui.ctx(), "Share URL copied!", 2.0);
                }
            }
        });
}

/// One "copy to clipboard" button per framework, plus the preview toggle.
fn draw_code_export(ui: &mut egui::Ui, p: &mut PanelCtx) {
    ui.label("Copy code:");
    ui.horizontal_wrapped(|ui| {
        let pal = p.cfg.palette;
        for (name, emitter, _) in CODEGEN_TARGETS {
            if ui
                .button(*name)
                .on_hover_text(format!("Copy {name} code to clipboard"))
                .clicked()
            {
                match emitter(&p.cfg.root, pal) {
                    Ok(code) => {
                        ui.ctx().copy_text(code);
                        p.toast(ui.ctx(), format!("Copied {name}!"), 2.0);
                    }
                    Err(e) => {
                        p.toast(ui.ctx(), format!("Error: {e}"), 3.0);
                    }
                }
            }
        }
    });

    ui.add_space(4.0);
    if ui
        .button("Preview code")
        .on_hover_text("Open the code preview panel on the right")
        .clicked()
    {
        p.cg.preview_open = true;
        p.cg.dirty = true;
        // The right panel takes space away from the viz.
        p.cfg.request_rebuild();
    }
}

// ─── Right panel (codegen preview) ───────────────────────────────────────────

fn draw_codegen_panel(root_ui: &mut egui::Ui, p: &mut PanelCtx) {
    egui::Panel::right("codegen_panel")
        .exact_size(RIGHT_PANEL_WIDTH)
        .resizable(false)
        .show_separator_line(false)
        .show(root_ui, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Framework:");
                egui::ComboBox::from_id_salt("cg_fw")
                    .selected_text(CODEGEN_TARGETS[p.cg.framework_idx].0)
                    .width(120.0)
                    .show_ui(ui, |ui| {
                        for (i, (name, _, _)) in CODEGEN_TARGETS.iter().enumerate() {
                            if ui
                                .selectable_label(p.cg.framework_idx == i, *name)
                                .clicked()
                            {
                                p.cg.framework_idx = i;
                                p.cg.dirty = true;
                            }
                        }
                    });
                if ui
                    .button("Copy")
                    .on_hover_text("Copy to clipboard")
                    .clicked()
                    && !p.cg.cached_code.is_empty()
                {
                    ui.ctx().copy_text(p.cg.cached_code.clone());
                    p.toast(ui.ctx(), "Copied!", 2.0);
                }
                if ui.button("x").on_hover_text("Close preview").clicked() {
                    p.cg.preview_open = false;
                    p.cfg.request_rebuild();
                }
            });
            ui.separator();

            p.cg.refresh(p.cfg);
            let job = p.cg.highlighted_job(p.cfg.theme.is_light());

            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let response = ui.label(job);
                    // Allow text selection via right-click context menu
                    response.context_menu(|ui| {
                        if ui.button("Copy all").clicked() {
                            ui.ctx().copy_text(p.cg.cached_code.clone());
                            ui.close();
                        }
                    });
                });
        });
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
    ctx.global_style_mut(|style| {
        style.spacing.item_spacing = egui::vec2(6.0, 3.0);
        style.spacing.button_padding = egui::vec2(6.0, 2.0);
        style.spacing.slider_width = 110.0;
    });
}

// ─── Grid UI sections ────────────────────────────────────────────────────────

fn draw_grid_container_section(ui: &mut egui::Ui, p: &mut PanelCtx) {
    egui::CollapsingHeader::new("Grid Container")
        .default_open(true)
        .show(ui, |ui| {
            ui.add_space(4.0);

            // ── Template Columns ─────────────────────────────────────
            {
                let Some(n) = p.cfg.root.get_mut(&p.sel_path) else {
                    return;
                };
                track_list_editor(
                    ui,
                    "columns",
                    "gtc",
                    &mut n.grid_template_columns,
                    &mut p.changed,
                );
            }
            ui.add_space(4.0);

            // ── Template Rows ────────────────────────────────────────
            {
                let Some(n) = p.cfg.root.get_mut(&p.sel_path) else {
                    return;
                };
                track_list_editor(ui, "rows", "gtr", &mut n.grid_template_rows, &mut p.changed);
            }
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // ── Auto Flow ────────────────────────────────────────────
            egui::Grid::new("gc_flow")
                .num_columns(2)
                .spacing([10.0, 6.0])
                .show(ui, |ui| {
                    let Some(n) = p.cfg.root.get_mut(&p.sel_path) else {
                        return;
                    };
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
                        &mut p.changed,
                        &mut p.hover.any,
                    );
                    ui.end_row();
                });

            ui.add_space(4.0);

            // ── Auto Rows / Auto Columns ─────────────────────────────
            {
                let Some(n) = p.cfg.root.get_mut(&p.sel_path) else {
                    return;
                };
                track_list_editor(
                    ui,
                    "auto-cols",
                    "gac",
                    &mut n.grid_auto_columns,
                    &mut p.changed,
                );
            }
            ui.add_space(2.0);
            {
                let Some(n) = p.cfg.root.get_mut(&p.sel_path) else {
                    return;
                };
                track_list_editor(
                    ui,
                    "auto-rows",
                    "gar",
                    &mut n.grid_auto_rows,
                    &mut p.changed,
                );
            }

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // ── Gaps (shared with flex) ──────────────────────────────
            egui::Grid::new("gc_gaps")
                .num_columns(2)
                .spacing([10.0, 6.0])
                .show(ui, |ui| {
                    let Some(n) = p.cfg.root.get_mut(&p.sel_path) else {
                        return;
                    };
                    label_with_help(ui, "row-gap", "Spacing between rows");
                    p.hover.row_gap =
                        val_row(ui, "grg", &mut n.row_gap, &mut p.changed, &mut p.hover.any);
                    ui.end_row();
                    label_with_help(ui, "column-gap", "Spacing between columns");
                    p.hover.column_gap = val_row(
                        ui,
                        "gcg",
                        &mut n.column_gap,
                        &mut p.changed,
                        &mut p.hover.any,
                    );
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
                    let Some(n) = p.cfg.root.get_mut(&p.sel_path) else {
                        return;
                    };
                    label_with_help(
                        ui,
                        "justify",
                        "How items are distributed along the row axis",
                    );
                    p.hover.justify = combo(
                        ui,
                        "gjc",
                        &mut n.justify_content,
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
                        &mut p.changed,
                        &mut p.hover.any,
                    );
                    ui.end_row();
                    label_with_help(
                        ui,
                        "align-items",
                        "How items are aligned along the column axis",
                    );
                    p.hover.align_items = combo(
                        ui,
                        "gai",
                        &mut n.align_items,
                        &[
                            ("Default", AlignItems::Default),
                            ("Start", AlignItems::Start),
                            ("End", AlignItems::End),
                            ("Center", AlignItems::Center),
                            ("Baseline", AlignItems::Baseline),
                            ("Stretch", AlignItems::Stretch),
                        ],
                        &mut p.changed,
                        &mut p.hover.any,
                    );
                    ui.end_row();
                    label_with_help(
                        ui,
                        "align-content",
                        "How grid tracks are distributed along the column axis",
                    );
                    p.hover.align_content = combo(
                        ui,
                        "gac2",
                        &mut n.align_content,
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
                        &mut p.changed,
                        &mut p.hover.any,
                    );
                    ui.end_row();
                });

            apply_container_hover(p);
        });
}

fn draw_grid_item_section(ui: &mut egui::Ui, p: &mut PanelCtx) {
    egui::CollapsingHeader::new("Grid Item")
        .default_open(true)
        .show(ui, |ui| {
            ui.add_space(4.0);
            egui::Grid::new("gi")
                .num_columns(2)
                .spacing([10.0, 6.0])
                .show(ui, |ui| {
                    let Some(n) = p.cfg.root.get_mut(&p.sel_path) else {
                        return;
                    };

                    // ── grid-column ──
                    label_with_help(ui, "grid-column", "Column placement (start line / span)");
                    grid_placement_editor(ui, "gic", &mut n.grid_column, &mut p.changed);
                    ui.end_row();

                    // ── grid-row ──
                    label_with_help(ui, "grid-row", "Row placement (start line / span)");
                    grid_placement_editor(ui, "gir", &mut n.grid_row, &mut p.changed);
                    ui.end_row();

                    // ── align-self ──
                    label_with_help(
                        ui,
                        "align-self",
                        "Override parent's align-items for this child",
                    );
                    p.hover.align_self = combo(
                        ui,
                        "gias",
                        &mut n.align_self,
                        &[
                            ("Auto", AlignSelf::Auto),
                            ("Start", AlignSelf::Start),
                            ("End", AlignSelf::End),
                            ("Center", AlignSelf::Center),
                            ("Baseline", AlignSelf::Baseline),
                            ("Stretch", AlignSelf::Stretch),
                        ],
                        &mut p.changed,
                        &mut p.hover.any,
                    );
                    ui.end_row();

                    // ── order ──
                    label_with_help(ui, "order", "Controls visual ordering (lower first)");
                    p.changed |= ui.add(egui::Slider::new(&mut n.order, -10..=10)).changed();
                    ui.end_row();
                });

            let h = p.hover;
            if h.align_self.is_some() {
                p.hover.any = true;
                if p.apply_hover(h.align_self, |n| n.align_self, |n, v| n.align_self = v) {
                    p.cfg.request_rebuild();
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
        label_with_help(ui, label, &format!("Grid track definitions for {label}"));
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
                        if ui
                            .selectable_label(cur_kind == kind, kind.to_string())
                            .clicked()
                        {
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
                let decimals = if matches!(track.kind(), GridTrackKind::Fr) {
                    1
                } else {
                    0
                };
                if ui
                    .add(egui::Slider::new(&mut n, lo..=hi).max_decimals(decimals))
                    .changed()
                {
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
                if ui
                    .add(egui::Slider::new(&mut v, -10..=20).prefix("line "))
                    .changed()
                {
                    *s = v as i16;
                    *changed = true;
                }
            }
            GridPlacement::Span(n) => {
                let mut v = *n as i32;
                if ui
                    .add(egui::Slider::new(&mut v, 1..=12).prefix("span "))
                    .changed()
                {
                    *n = v as u16;
                    *changed = true;
                }
            }
            GridPlacement::StartSpan(s, n) => {
                let mut vs = *s as i32;
                let mut vn = *n as i32;
                if ui
                    .add(egui::Slider::new(&mut vs, -10..=20).prefix("line "))
                    .changed()
                {
                    *s = vs as i16;
                    *changed = true;
                }
                if ui
                    .add(egui::Slider::new(&mut vn, 1..=12).prefix("span "))
                    .changed()
                {
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

/// Dropdown over `options`. Returns the option under the pointer (for a live
/// preview) and sets `any_open` while the popup is open.
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

/// Value-kind dropdown plus slider for a `ValueConfig`. Returns the kind
/// under the pointer (cast from the current value) for a live preview.
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
        value_slider(ui, val, changed);
    });
    if is_open {
        *any_open = true;
    }
    hover
}

/// Slider for the numeric part of a `ValueConfig` (nothing for `Auto`).
fn value_slider(ui: &mut egui::Ui, val: &mut ValueConfig, changed: &mut bool) {
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

/// Link toggle shared by the per-side and per-corner editors. The user's
/// choice is remembered per editor in egui memory, so all sides can be
/// edited independently even while they happen to be equal. Returns whether
/// the editor is linked this frame and whether the user just linked it.
fn link_toggle(ui: &mut egui::Ui, id_prefix: &str, uniform: bool, what: &str) -> (bool, bool) {
    let link_id = ui.make_persistent_id((id_prefix, "linked"));
    let stored = ui.data_mut(|d| *d.get_temp_mut_or(link_id, true));
    let linked = stored && uniform;
    let mut link = linked;
    let toggled = ui
        .toggle_value(&mut link, if linked { "🔗" } else { "⛓" })
        .on_hover_text(if linked {
            format!("Click to set each {what} independently")
        } else {
            format!("Click to link all {what}s to one value")
        })
        .changed();
    if toggled {
        ui.data_mut(|d| d.insert_temp(link_id, link));
    }
    (linked, toggled && link)
}

/// Editor for a per-side `Sides` value. Shows a single slider when linked,
/// or four sliders (top/right/bottom/left) otherwise.
fn sides_editor(
    ui: &mut egui::Ui,
    label: &str,
    id_prefix: &str,
    help: &str,
    sides: &mut Sides,
    changed: &mut bool,
) {
    let mut linked = false;
    ui.horizontal(|ui| {
        label_with_help(ui, label, help);
        let (is_linked, just_linked) = link_toggle(ui, id_prefix, sides.is_uniform(), "side");
        linked = is_linked;
        if just_linked {
            // Unify to top value
            *sides = Sides::uniform(sides.top);
            *changed = true;
        }
    });

    if linked {
        // Single slider controlling all sides
        let mut v = sides.top;
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            single_val_editor(ui, &format!("{id_prefix}_u"), &mut v, changed);
        });
        if v != sides.top {
            *sides = Sides::uniform(v);
        }
    } else {
        let Sides {
            top,
            right,
            bottom,
            left,
        } = sides;
        for (side_label, suffix, v) in [
            ("top", "t", top),
            ("right", "r", right),
            ("bottom", "b", bottom),
            ("left", "l", left),
        ] {
            ui.horizontal(|ui| {
                ui.add_space(14.0);
                ui.label(side_label);
                single_val_editor(ui, &format!("{id_prefix}_{suffix}"), v, changed);
            });
        }
    }
}

/// Editor for a per-corner `Corners` value.
fn corners_editor(
    ui: &mut egui::Ui,
    label: &str,
    id_prefix: &str,
    help: &str,
    corners: &mut Corners,
    changed: &mut bool,
) {
    let mut linked = false;
    ui.horizontal(|ui| {
        label_with_help(ui, label, help);
        let (is_linked, just_linked) = link_toggle(ui, id_prefix, corners.is_uniform(), "corner");
        linked = is_linked;
        if just_linked {
            *corners = Corners::uniform(corners.top_left);
            *changed = true;
        }
    });

    if linked {
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
fn single_val_editor(ui: &mut egui::Ui, id: &str, val: &mut ValueConfig, changed: &mut bool) {
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
    value_slider(ui, val, changed);
}

// ─── File picker helpers ─────────────────────────────────────────────────────

/// Ctrl+S / "Save file": a native save dialog, or a browser download on wasm.
fn save_file(p: &mut PanelCtx, ctx: &egui::Context) {
    let Some(json) = crate::persist::export_json(p.cfg) else {
        return;
    };
    #[cfg(target_arch = "wasm32")]
    {
        crate::persist::trigger_download(&json);
        p.toast(ctx, "Downloading flexplore-layout.json", 2.0);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = rfd::FileDialog::new()
            .set_file_name("flexplore-layout.json")
            .add_filter("JSON", &["json"])
            .save_file();
        if let Some(path) = path {
            match std::fs::write(&path, &json) {
                Ok(()) => p.toast(ctx, format!("Saved to {}", path.display()), 2.0),
                Err(e) => p.toast(ctx, format!("Save error: {e}"), 3.0),
            }
        }
    }
}

/// Ctrl+O / "Open file": a native open dialog. The browser has no file
/// dialog here, so wasm points the user at the JSON import box instead.
fn open_file(p: &mut PanelCtx, ctx: &egui::Context) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = ctx;
        let loaded = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .pick_file()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|data| crate::persist::import_json(&data));
        if let Some(loaded) = loaded {
            p.replace_document(loaded, true);
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        p.toast(
            ctx,
            "Open file is not available in the browser — use Import / Export → paste JSON",
            3.0,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// root { a { a0, a1 }, b, c }
    fn tree() -> NodeConfig {
        let mut root = NodeConfig::new_container("root");
        let mut a = NodeConfig::new_container("a");
        a.children = vec![
            NodeConfig::new_leaf("a0", 1.0, 1.0),
            NodeConfig::new_leaf("a1", 1.0, 1.0),
        ];
        root.children = vec![
            a,
            NodeConfig::new_leaf("b", 1.0, 1.0),
            NodeConfig::new_leaf("c", 1.0, 1.0),
        ];
        root
    }

    fn labels(node: &NodeConfig) -> Vec<&str> {
        node.children.iter().map(|c| c.label.as_str()).collect()
    }

    #[test]
    fn same_parent_move_adjusts_for_removal() {
        let mut root = tree();
        // Drop "b" (index 1) after "c" (insert index 3).
        assert_eq!(move_node(&mut root, &[1], &[], 3), Some(vec![2]));
        assert_eq!(labels(&root), ["a", "c", "b"]);
        // Dropping right before or after itself is a no-op.
        assert_eq!(move_node(&mut root, &[1], &[], 1), None);
        assert_eq!(move_node(&mut root, &[1], &[], 2), None);
    }

    #[test]
    fn cross_parent_move_shifts_destination_path() {
        let mut root = tree();
        // Move "a" (root index 0) into ... no: move "b" into "a" at the end.
        assert_eq!(move_node(&mut root, &[1], &[0], 2), Some(vec![0, 2]));
        assert_eq!(labels(&root), ["a", "c"]);
        assert_eq!(labels(&root.children[0]), ["a0", "a1", "b"]);

        // Move "a0" out to root, before "c": the destination index is on the
        // root level, unaffected by the removal inside "a".
        assert_eq!(move_node(&mut root, &[0, 0], &[], 1), Some(vec![1]));
        assert_eq!(labels(&root), ["a", "a0", "c"]);

        // root { x, y { y0 } }: moving "x" into "y" shifts y's path from 1 to 0.
        let mut root = NodeConfig::new_container("root");
        let mut y = NodeConfig::new_container("y");
        y.children = vec![NodeConfig::new_leaf("y0", 1.0, 1.0)];
        root.children = vec![NodeConfig::new_leaf("x", 1.0, 1.0), y];
        assert_eq!(move_node(&mut root, &[0], &[1], 0), Some(vec![0, 0]));
        assert_eq!(labels(&root), ["y"]);
        assert_eq!(labels(&root.children[0]), ["x", "y0"]);
    }

    #[test]
    fn refuses_move_into_own_subtree() {
        let mut root = tree();
        assert_eq!(move_node(&mut root, &[0], &[0], 0), None);
        assert_eq!(move_node(&mut root, &[0], &[0, 1], 0), None);
        assert_eq!(labels(&root), ["a", "b", "c"]);
    }
}
