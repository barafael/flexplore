use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use strum::IntoEnumIterator;

use crate::codegen::{emit_bevy_code, emit_html_css, emit_swiftui, emit_tailwind};
use crate::config::*;
use crate::history::UndoHistory;

// ─── Tree UI helper ───────────────────────────────────────────────────────────

/// Return type: (clicked path, remove requested, drag-drop move request (from, to_parent, to_idx))
fn draw_tree_ui(
    ui: &mut egui::Ui,
    node: &mut NodeConfig,
    path: &mut Vec<usize>,
    selected: &[usize],
    changed: &mut bool,
) -> (Option<Vec<usize>>, bool, Option<(Vec<usize>, Vec<usize>, usize)>) {
    let mut clicked = None;
    let mut remove = false;
    let mut dnd_move = None;
    let is_selected = path.as_slice() == selected;
    let is_root = path.is_empty();
    let row_id = egui::Id::new("tree_dnd").with(&*path);

    // Drop target: this row accepts drops *above* it (insert at this index).
    let (_inner, drop_payload) = ui.dnd_drop_zone::<Vec<usize>, ()>(egui::Frame::NONE, |ui| {
        // Drag source: this row can be dragged (except root).
        let draw_row = |ui: &mut egui::Ui| {
            ui.add_space(path.len() as f32 * 14.0);
            let icon = if node.children.is_empty() { "□" } else { "▣" };
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
                .selectable_label(false, format!("{} {}", icon, node.label))
                .clicked()
            {
                clicked = Some(path.clone());
            }
        };

        if is_root {
            ui.horizontal(draw_row);
        } else {
            ui.dnd_drag_source(row_id, path.clone(), |ui| {
                ui.horizontal(draw_row);
            });
        }
    });

    // Check if something was dropped on this row.
    if let Some(dragged) = drop_payload {
        if !is_root {
            let parent_path = path[..path.len() - 1].to_vec();
            let idx = path[path.len() - 1];
            dnd_move = Some(((*dragged).clone(), parent_path, idx));
        } else {
            // Dropped on root => append as child at position 0.
            dnd_move = Some(((*dragged).clone(), vec![], 0));
        }
    }

    for i in 0..node.children.len() {
        path.push(i);
        let (r, rem, mv) = draw_tree_ui(ui, &mut node.children[i], path, selected, changed);
        path.pop();
        if r.is_some() {
            clicked = r;
        }
        if rem {
            remove = true;
        }
        if mv.is_some() {
            dnd_move = mv;
        }
    }
    (clicked, remove, dnd_move)
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
    let Some(node) = cfg.root.get(path) else { return false };
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

pub fn panel_system(
    mut contexts: EguiContexts,
    mut cfg: ResMut<FlexConfig>,
    mut history: ResMut<UndoHistory>,
    mut preview: Local<Option<FlexConfig>>,
    mut applied_theme: Local<Option<Theme>>,
    mut import_buf: Local<String>,
    mut toast: Local<Option<(String, f64)>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    // ── Undo / Redo shortcuts ────────────────────────────────────────────────
    let undo_pressed = ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Z));
    let redo_pressed = ctx.input_mut(|i| {
        i.consume_key(egui::Modifiers::COMMAND, egui::Key::Y)
            || i.consume_key(
                egui::Modifiers::COMMAND.plus(egui::Modifiers::SHIFT),
                egui::Key::Z,
            )
    });
    if undo_pressed {
        if let Some(snapshot) = history.undo() {
            *cfg = snapshot.clone();
            cfg.request_rebuild();
            *preview = None;
        }
    }
    if redo_pressed {
        if let Some(snapshot) = history.redo() {
            *cfg = snapshot.clone();
            cfg.request_rebuild();
            *preview = None;
        }
    }

    // ── Tree navigation shortcuts ────────────────────────────────────────────
    let key_add_child = ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Enter));
    let key_add_sibling = ctx.input_mut(|i| i.consume_key(egui::Modifiers::SHIFT, egui::Key::Enter));
    let key_delete = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Delete));
    let key_parent = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
    let key_next = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown));
    let key_prev = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp));

    if *applied_theme != Some(cfg.theme) {
        apply_theme(ctx, cfg.theme);
        *applied_theme = Some(cfg.theme);
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
    let mut hover_padding: Option<ValueConfig> = None;
    let mut hover_basis: Option<ValueConfig> = None;
    let mut hover_align_self: Option<AlignSelf> = None;
    let mut hover_margin: Option<ValueConfig> = None;

    let mut sel_path = cfg.selected().to_vec();
    let mut is_root = sel_path.is_empty();

    // ── Apply tree navigation shortcuts ──────────────────────────────────────
    // Only handle when no text edit is focused.
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
            let n = cfg.root.count_leaves();
            let lbl = format!("node{}", n + 1);
            if let Some(parent) = cfg.root.get_mut(&sel_path[..pidx]) {
                parent.children.push(NodeConfig::new_leaf(&lbl, 80.0, 80.0));
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
        if key_next && !is_root {
            let pidx = sel_path.len() - 1;
            let idx = sel_path[pidx];
            let sibling_count = cfg.root.get(&sel_path[..pidx]).map_or(0, |p| p.children.len());
            if idx + 1 < sibling_count {
                sel_path[pidx] = idx + 1;
                cfg.select(sel_path.clone());
            }
        }
        if key_prev && !is_root {
            let pidx = sel_path.len() - 1;
            let idx = sel_path[pidx];
            if idx > 0 {
                sel_path[pidx] = idx - 1;
                cfg.select(sel_path.clone());
            }
        }
    }

    egui::SidePanel::left("flex_panel")
        .exact_width(PANEL_WIDTH)
        .resizable(false)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(4.0);

                // ── Toolbar ───────────────────────────────────────────────────────
                ui.horizontal(|ui| {
                    if ui.button(match cfg.theme { Theme::Dark => "Light mode", Theme::Light => "Dark mode" }).clicked() {
                        cfg.theme = match cfg.theme { Theme::Dark => Theme::Light, Theme::Light => Theme::Dark };
                        changed = true;
                    }
                    ui.separator();
                    if ui.add_enabled(history.can_undo(), egui::Button::new("⟲ Undo")).clicked() {
                        if let Some(snapshot) = history.undo() {
                            *cfg = snapshot.clone();
                            cfg.request_rebuild();
                            *preview = None;
                        }
                    }
                    if ui.add_enabled(history.can_redo(), egui::Button::new("⟳ Redo")).clicked() {
                        if let Some(snapshot) = history.redo() {
                            *cfg = snapshot.clone();
                            cfg.request_rebuild();
                            *preview = None;
                        }
                    }
                });
                ui.add_space(4.0);

                // ── Tree ─────────────────────────────────────────────────────────
                egui::CollapsingHeader::new("Tree")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("+ Child").on_hover_text("Add a new child node inside the selected node").clicked() {
                                let n = cfg.root.count_leaves();
                                let lbl = format!("node{}", n + 1);
                                if let Some(node) = cfg.root.get_mut(&sel_path) {
                                    node.children.push(NodeConfig::new_leaf(&lbl, 80.0, 80.0));
                                    changed = true;
                                }
                            }
                            if !is_root && ui.button("+ Sibling").on_hover_text("Add a new node next to the selected node (same parent)").clicked() {
                                let pidx = sel_path.len() - 1;
                                let n = cfg.root.count_leaves();
                                let lbl = format!("node{}", n + 1);
                                if let Some(parent) = cfg.root.get_mut(&sel_path[..pidx]) {
                                    parent.children.push(NodeConfig::new_leaf(&lbl, 80.0, 80.0));
                                    changed = true;
                                }
                            }
                        });
                        ui.add_space(2.0);
                        let sel_snapshot = cfg.selected().to_vec();
                        let (clicked, remove_req, dnd_req) = draw_tree_ui(ui, &mut cfg.root, &mut vec![], &sel_snapshot, &mut changed);
                        if let Some((from, to_parent, to_idx)) = dnd_req {
                            if crate::config::move_node(&mut cfg.root, &from, &to_parent, to_idx) {
                                cfg.sanitize_selection();
                                changed = true;
                            }
                        }
                        if remove_req && !sel_path.is_empty() {
                            let pidx = sel_path.len() - 1;
                            let idx = sel_path[pidx];
                            if let Some(parent) = cfg.root.get_mut(&sel_path[..pidx]) {
                                parent.children.remove(idx);
                            }
                            let new_path = sel_path[..pidx].to_vec();
                            sel_path = new_path.clone();
                            is_root = sel_path.is_empty();
                            cfg.select(new_path);
                            changed = true;
                        }
                        if let Some(p) = clicked
                            && p != cfg.selected() {
                                sel_path = p.clone();
                                is_root = sel_path.is_empty();
                                cfg.select(p);
                                *preview = None;
                            }
                    });

                ui.add_space(6.0);

                if cfg.root.get(&sel_path).is_some() {

                egui::CollapsingHeader::new("Flex Container")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.add_space(4.0);
                        egui::Grid::new("cg1").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                            {
                                let Some(n) = cfg.root.get_mut(&sel_path) else { return };
                                label_with_help(ui, "direction", "The main axis along which children are laid out (Row = horizontal, Column = vertical)");
                                hover_direction = combo(ui, "fd", &mut n.flex_direction, &[
                                    ("Row", FlexDirection::Row), ("Column", FlexDirection::Column),
                                    ("RowReverse", FlexDirection::RowReverse),
                                    ("ColumnReverse", FlexDirection::ColumnReverse),
                                ], &mut changed, &mut any_hovered); ui.end_row();

                                label_with_help(ui, "wrap", "Whether children wrap to new lines when they overflow the container");
                                hover_wrap = combo(ui, "fw", &mut n.flex_wrap, &[
                                    ("NoWrap", FlexWrap::NoWrap), ("Wrap", FlexWrap::Wrap),
                                    ("WrapReverse", FlexWrap::WrapReverse),
                                ], &mut changed, &mut any_hovered); ui.end_row();

                                label_with_help(ui, "justify", "How children are distributed along the main axis (e.g. centered, spaced evenly)");
                                hover_justify = combo(ui, "jc", &mut n.justify_content, &[
                                    ("Default", JustifyContent::Default),
                                    ("FlexStart", JustifyContent::FlexStart),
                                    ("FlexEnd", JustifyContent::FlexEnd),
                                    ("Center", JustifyContent::Center),
                                    ("SpaceBetween", JustifyContent::SpaceBetween),
                                    ("SpaceAround", JustifyContent::SpaceAround),
                                    ("SpaceEvenly", JustifyContent::SpaceEvenly),
                                    ("Stretch", JustifyContent::Stretch),
                                    ("Start", JustifyContent::Start), ("End", JustifyContent::End),
                                ], &mut changed, &mut any_hovered); ui.end_row();

                                label_with_help(ui, "align-items", "How children are aligned along the cross axis (perpendicular to direction)");
                                hover_align_items = combo(ui, "ai", &mut n.align_items, &[
                                    ("Default", AlignItems::Default),
                                    ("FlexStart", AlignItems::FlexStart),
                                    ("FlexEnd", AlignItems::FlexEnd),
                                    ("Center", AlignItems::Center),
                                    ("Baseline", AlignItems::Baseline),
                                    ("Stretch", AlignItems::Stretch),
                                    ("Start", AlignItems::Start), ("End", AlignItems::End),
                                ], &mut changed, &mut any_hovered); ui.end_row();

                                label_with_help(ui, "align-content", "How wrapped lines are distributed along the cross axis (only applies when wrapping)");
                                hover_align_content = combo(ui, "ac", &mut n.align_content, &[
                                    ("Default", AlignContent::Default),
                                    ("FlexStart", AlignContent::FlexStart),
                                    ("FlexEnd", AlignContent::FlexEnd),
                                    ("Center", AlignContent::Center),
                                    ("SpaceBetween", AlignContent::SpaceBetween),
                                    ("SpaceAround", AlignContent::SpaceAround),
                                    ("SpaceEvenly", AlignContent::SpaceEvenly),
                                    ("Stretch", AlignContent::Stretch),
                                    ("Start", AlignContent::Start), ("End", AlignContent::End),
                                ], &mut changed, &mut any_hovered); ui.end_row();
                            }
                        });
                        ui.add_space(4.0); ui.separator(); ui.add_space(4.0);
                        egui::Grid::new("cg2").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                            {
                                let Some(n) = cfg.root.get_mut(&sel_path) else { return };
                                label_with_help(ui, "row-gap", "Spacing between rows of children");
                                hover_row_gap = val_row(ui, "rg", &mut n.row_gap, &mut changed, &mut any_hovered);
                                ui.end_row();
                                label_with_help(ui, "column-gap", "Spacing between columns of children");
                                hover_column_gap = val_row(ui, "cgap", &mut n.column_gap, &mut changed, &mut any_hovered);
                                ui.end_row();
                            }
                        });
                        ui.add_space(2.0);

                        let has_container_hover = hover_direction.is_some() || hover_wrap.is_some() || hover_justify.is_some()
                            || hover_align_items.is_some() || hover_align_content.is_some()
                            || hover_row_gap.is_some() || hover_column_gap.is_some();
                        if has_container_hover {
                            any_hovered = true;
                            let p = &mut *preview; let sp = &sel_path;
                            let needs_rebuild =
                                apply_hover(hover_direction,     &mut cfg, p, sp, |n| n.flex_direction,        |n, v| n.flex_direction  = v) |
                                apply_hover(hover_wrap,    &mut cfg, p, sp, |n| n.flex_wrap,              |n, v| n.flex_wrap        = v) |
                                apply_hover(hover_justify, &mut cfg, p, sp, |n| n.justify_content,        |n, v| n.justify_content  = v) |
                                apply_hover(hover_align_items,      &mut cfg, p, sp, |n| n.align_items,            |n, v| n.align_items      = v) |
                                apply_hover(hover_align_content,      &mut cfg, p, sp, |n| n.align_content,          |n, v| n.align_content    = v) |
                                apply_hover(hover_row_gap,      &mut cfg, p, sp, |n| n.row_gap,        |n, v| n.row_gap          = v) |
                                apply_hover(hover_column_gap,    &mut cfg, p, sp, |n| n.column_gap,     |n, v| n.column_gap       = v);
                            if needs_rebuild { cfg.request_rebuild(); }
                        }
                    });

                ui.add_space(6.0);

                egui::CollapsingHeader::new("Sizing")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.add_space(4.0);
                        egui::Grid::new("sg").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                            {
                                let Some(n) = cfg.root.get_mut(&sel_path) else { return };
                                label_with_help(ui, "width", "The preferred width of this node");    hover_width    = val_row(ui, "sw",    &mut n.width,      &mut changed, &mut any_hovered); ui.end_row();
                                label_with_help(ui, "height", "The preferred height of this node");   hover_height    = val_row(ui, "sh",    &mut n.height,     &mut changed, &mut any_hovered); ui.end_row();
                                label_with_help(ui, "min-width", "The minimum width this node can shrink to");  hover_min_width = val_row(ui, "sminw", &mut n.min_width,  &mut changed, &mut any_hovered); ui.end_row();
                                label_with_help(ui, "min-height", "The minimum height this node can shrink to"); hover_min_height = val_row(ui, "sminh", &mut n.min_height, &mut changed, &mut any_hovered); ui.end_row();
                                label_with_help(ui, "max-width", "The maximum width this node can grow to");  hover_max_width = val_row(ui, "smaxw", &mut n.max_width,  &mut changed, &mut any_hovered); ui.end_row();
                                label_with_help(ui, "max-height", "The maximum height this node can grow to"); hover_max_height = val_row(ui, "smaxh", &mut n.max_height, &mut changed, &mut any_hovered); ui.end_row();
                                label_with_help(ui, "padding", "Space between this node's border and its children");    hover_padding  = val_row(ui, "spad",  &mut n.padding,    &mut changed, &mut any_hovered); ui.end_row();
                            }
                        });
                        ui.add_space(2.0);

                        let has_sizing_hover = hover_width.is_some() || hover_height.is_some() || hover_min_width.is_some()
                            || hover_min_height.is_some() || hover_max_width.is_some() || hover_max_height.is_some()
                            || hover_padding.is_some();
                        if has_sizing_hover {
                            any_hovered = true;
                            let p = &mut *preview; let sp = &sel_path;
                            let needs_rebuild =
                                apply_hover(hover_width,    &mut cfg, p, sp, |n| n.width,      |n, v| n.width      = v) |
                                apply_hover(hover_height,    &mut cfg, p, sp, |n| n.height,     |n, v| n.height     = v) |
                                apply_hover(hover_min_width, &mut cfg, p, sp, |n| n.min_width,  |n, v| n.min_width  = v) |
                                apply_hover(hover_min_height, &mut cfg, p, sp, |n| n.min_height, |n, v| n.min_height = v) |
                                apply_hover(hover_max_width, &mut cfg, p, sp, |n| n.max_width,  |n, v| n.max_width  = v) |
                                apply_hover(hover_max_height, &mut cfg, p, sp, |n| n.max_height, |n, v| n.max_height = v) |
                                apply_hover(hover_padding,  &mut cfg, p, sp, |n| n.padding,    |n, v| n.padding    = v);
                            if needs_rebuild { cfg.request_rebuild(); }
                        }
                    });

                ui.add_space(6.0);

                if !is_root {
                    egui::CollapsingHeader::new("Flex Item")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.add_space(4.0);
                            egui::Grid::new("ig").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                                {
                                    let Some(n) = cfg.root.get_mut(&sel_path) else { return };
                                    label_with_help(ui, "flex-grow", "How much this node grows relative to siblings when there is extra space (0 = don't grow)");
                                    changed |= ui.add(egui::Slider::new(&mut n.flex_grow, 0.0..=5.0).max_decimals(2)).changed();
                                    ui.end_row();
                                    label_with_help(ui, "flex-shrink", "How much this node shrinks relative to siblings when space is tight (0 = don't shrink)");
                                    changed |= ui.add(egui::Slider::new(&mut n.flex_shrink, 0.0..=5.0).max_decimals(2)).changed();
                                    ui.end_row();
                                    label_with_help(ui, "flex-basis", "The initial size along the main axis before grow/shrink is applied");
                                    hover_basis = val_row(ui, "ib", &mut n.flex_basis, &mut changed, &mut any_hovered);
                                    ui.end_row();
                                    label_with_help(ui, "align-self", "Override the parent's align-items for this specific child");
                                    hover_align_self = combo(ui, "ias", &mut n.align_self, &[
                                        ("Auto", AlignSelf::Auto), ("FlexStart", AlignSelf::FlexStart),
                                        ("FlexEnd", AlignSelf::FlexEnd), ("Center", AlignSelf::Center),
                                        ("Baseline", AlignSelf::Baseline), ("Stretch", AlignSelf::Stretch),
                                        ("Start", AlignSelf::Start), ("End", AlignSelf::End),
                                    ], &mut changed, &mut any_hovered);
                                    ui.end_row();
                                    label_with_help(ui, "margin", "Space outside this node's border, pushing it away from siblings");
                                    hover_margin = val_row(ui, "im", &mut n.margin, &mut changed, &mut any_hovered);
                                    ui.end_row();
                                }
                            });
                            ui.add_space(2.0);

                            let has_item_hover = hover_basis.is_some() || hover_align_self.is_some() || hover_margin.is_some();
                            if has_item_hover {
                                any_hovered = true;
                                let p = &mut *preview; let sp = &sel_path;
                                let needs_rebuild =
                                    apply_hover(hover_basis,  &mut cfg, p, sp, |n| n.flex_basis, |n, v| n.flex_basis = v) |
                                    apply_hover(hover_align_self,     &mut cfg, p, sp, |n| n.align_self,         |n, v| n.align_self = v) |
                                    apply_hover(hover_margin, &mut cfg, p, sp, |n| n.margin,     |n, v| n.margin     = v);
                                if needs_rebuild { cfg.request_rebuild(); }
                            }
                        });

                    ui.add_space(6.0);
                }

                } // end if path_valid

                egui::CollapsingHeader::new("Background")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let prev = cfg.bg_mode;
                            ui.radio_value(&mut cfg.bg_mode, BackgroundMode::Pastel, "Pastel").on_hover_text("Fill leaf nodes with solid pastel colors");
                            ui.radio_value(&mut cfg.bg_mode, BackgroundMode::RandomArt, "Generative Art").on_hover_text("Fill leaf nodes with procedurally generated art textures");
                            if cfg.bg_mode != prev { changed = true; }
                        });
                        if cfg.bg_mode == BackgroundMode::RandomArt {
                            let cur = cfg.art_style.to_string();
                            let mut hover_art: Option<ArtStyle> = None;
                            let art_resp = egui::ComboBox::from_label("style")
                                .selected_text(&cur)
                                .show_ui(ui, |ui| {
                                    for style in ArtStyle::iter() {
                                        let name = style.to_string();
                                        let r = ui.selectable_label(cfg.art_style == style, &name);
                                        if r.clicked() { cfg.art_style = style; changed = true; }
                                        else if r.hovered() { hover_art = Some(style); }
                                    }
                                });
                            if art_resp.inner.is_some() { any_hovered = true; }
                            if let Some(v) = hover_art {
                                any_hovered = true;
                                if cfg.art_style != v {
                                    if preview.is_none() { *preview = Some(cfg.clone()); }
                                    cfg.art_style = v; cfg.request_rebuild();
                                }
                            }
                            let pd = cfg.art_depth;
                            ui.add(egui::Slider::new(&mut cfg.art_depth, 1..=9).text("depth")).on_hover_text("Expression tree depth — higher values produce more complex patterns");
                            if cfg.art_depth != pd { changed = true; }
                            ui.add(egui::Slider::new(&mut cfg.art_anim, 0.0..=2.0).text("anim speed").step_by(0.05)).on_hover_text("How fast the generative art animates (0 = static)");
                            ui.horizontal(|ui| {
                                if ui.button("New seed").on_hover_text("Randomize the seed for a completely different pattern").clicked() { cfg.art_seed = rand::random::<u64>(); changed = true; }
                                if ui.button("Regenerate").on_hover_text("Re-render art with the current settings").clicked() { changed = true; }
                            });
                        }
                    });

                ui.add_space(6.0);
                if ui.button("Reset to defaults").on_hover_text("Restore all settings and the node tree to the initial state").clicked() {
                    *cfg = FlexConfig::default(); *preview = None;
                    changed = true;
                }

                ui.add_space(6.0);
                egui::CollapsingHeader::new("Import / Export")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("Export JSON").on_hover_text("Download layout as JSON").clicked() {
                                if let Some(json) = crate::persist::export_json(&cfg) {
                                    #[cfg(target_arch = "wasm32")]
                                    crate::persist::trigger_download(&json);
                                    #[cfg(not(target_arch = "wasm32"))]
                                    { ui.ctx().copy_text(json); }
                                }
                            }
                        });
                        ui.label("Paste JSON to import:");
                        ui.add(egui::TextEdit::multiline(&mut *import_buf).desired_rows(3).desired_width(f32::INFINITY));
                        if ui.button("Load from JSON").clicked() && !import_buf.is_empty() {
                            if let Some(loaded) = crate::persist::import_json(&import_buf) {
                                *cfg = loaded;
                                cfg.request_rebuild();
                                *preview = None;
                                history.push(cfg.clone());
                                import_buf.clear();
                            }
                        }
                    });

                ui.add_space(4.0);
                ui.label("Copy code:");
                ui.horizontal(|ui| {
                    let copy_targets: &[(&str, fn(&NodeConfig) -> anyhow::Result<String>)] = &[
                        ("Bevy", |r| emit_bevy_code(r)),
                        ("HTML/CSS", |r| emit_html_css(r)),
                        ("Tailwind", |r| emit_tailwind(r)),
                        ("SwiftUI", |r| emit_swiftui(r)),
                    ];
                    for (name, emitter) in copy_targets {
                        if ui.button(*name).on_hover_text(format!("Copy {name} code to clipboard")).clicked() {
                            match emitter(&cfg.root) {
                                Ok(code) => {
                                    ui.ctx().copy_text(code);
                                    let now = ui.ctx().input(|i| i.time);
                                    *toast = Some((format!("Copied {name}!"), now + 2.0));
                                }
                                Err(e) => {
                                    let now = ui.ctx().input(|i| i.time);
                                    *toast = Some((format!("Error: {e}"), now + 3.0));
                                }
                            }
                        }
                    }
                });
            });
        });

    if changed {
        *preview = None;
        cfg.request_rebuild();
        history.push(cfg.clone());
    } else if !any_hovered && let Some(saved) = preview.take() {
        *cfg = saved;
        cfg.sanitize_selection();
        cfg.request_rebuild();
    }

    // ── Toast overlay ────────────────────────────────────────────────────────
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

    Ok(())
}

// ─── Theme ───────────────────────────────────────────────────────────────────

fn apply_theme(ctx: &egui::Context, theme: Theme) {
    let no_rounding = egui::CornerRadius::ZERO;
    let mut v = match theme {
        Theme::Dark => {
            const BG: egui::Color32 = egui::Color32::from_rgb(0x10, 0x10, 0x14);
            const MID: egui::Color32 = egui::Color32::from_rgb(0x2a, 0x2a, 0x30);
            const FG: egui::Color32 = egui::Color32::from_rgb(0xe8, 0xe4, 0xd8);
            let mut v = egui::Visuals::dark();
            v.panel_fill = BG;
            v.window_fill = BG;
            v.extreme_bg_color = BG;
            v.widgets.inactive.bg_fill = MID;
            v.widgets.inactive.weak_bg_fill = MID;
            v.widgets.inactive.bg_stroke =
                egui::Stroke::new(1.0, egui::Color32::from_rgb(0x3a, 0x3a, 0x42));
            v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, FG);
            v.widgets.hovered.bg_fill = egui::Color32::from_rgb(0x38, 0x38, 0x42);
            v.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(0x38, 0x38, 0x42);
            v.widgets.hovered.bg_stroke =
                egui::Stroke::new(1.0, egui::Color32::from_rgb(0x88, 0x88, 0x98));
            v.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, FG);
            v.widgets.active.bg_fill = FG;
            v.widgets.active.weak_bg_fill = FG;
            v.widgets.active.fg_stroke = egui::Stroke::new(1.5, BG);
            v.widgets.open.bg_fill = MID;
            v.widgets.open.fg_stroke = egui::Stroke::new(1.0, FG);
            v.widgets.noninteractive.bg_fill = BG;
            v.widgets.noninteractive.fg_stroke =
                egui::Stroke::new(1.0, egui::Color32::from_rgb(0x70, 0x6e, 0x66));
            v.widgets.noninteractive.bg_stroke =
                egui::Stroke::new(1.0, egui::Color32::from_rgb(0x34, 0x34, 0x3a));
            v.override_text_color = Some(FG);
            v.window_stroke =
                egui::Stroke::new(1.0, egui::Color32::from_rgb(0x3a, 0x3a, 0x42));
            v.selection.bg_fill = egui::Color32::from_rgb(0x40, 0x40, 0x52);
            v
        }
        Theme::Light => {
            const BG: egui::Color32 = egui::Color32::from_rgb(0xf4, 0xf2, 0xee);
            const MID: egui::Color32 = egui::Color32::from_rgb(0xe0, 0xde, 0xd8);
            const FG: egui::Color32 = egui::Color32::from_rgb(0x20, 0x20, 0x24);
            let mut v = egui::Visuals::light();
            v.panel_fill = BG;
            v.window_fill = BG;
            v.extreme_bg_color = egui::Color32::WHITE;
            v.widgets.inactive.bg_fill = MID;
            v.widgets.inactive.weak_bg_fill = MID;
            v.widgets.inactive.bg_stroke =
                egui::Stroke::new(1.0, egui::Color32::from_rgb(0xc0, 0xbe, 0xb8));
            v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, FG);
            v.widgets.hovered.bg_fill = egui::Color32::from_rgb(0xd4, 0xd2, 0xcc);
            v.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(0xd4, 0xd2, 0xcc);
            v.widgets.hovered.bg_stroke =
                egui::Stroke::new(1.0, egui::Color32::from_rgb(0x88, 0x88, 0x90));
            v.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, FG);
            v.widgets.active.bg_fill = FG;
            v.widgets.active.weak_bg_fill = FG;
            v.widgets.active.fg_stroke = egui::Stroke::new(1.5, BG);
            v.widgets.open.bg_fill = MID;
            v.widgets.open.fg_stroke = egui::Stroke::new(1.0, FG);
            v.widgets.noninteractive.bg_fill = BG;
            v.widgets.noninteractive.fg_stroke =
                egui::Stroke::new(1.0, egui::Color32::from_rgb(0x60, 0x5e, 0x58));
            v.widgets.noninteractive.bg_stroke =
                egui::Stroke::new(1.0, egui::Color32::from_rgb(0xc4, 0xc2, 0xbc));
            v.override_text_color = Some(FG);
            v.window_stroke =
                egui::Stroke::new(1.0, egui::Color32::from_rgb(0xc0, 0xbe, 0xb8));
            v.selection.bg_fill = egui::Color32::from_rgb(0xc0, 0xd0, 0xe8);
            v
        }
    };
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
                (0.0_f32, 600.0_f32)
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
