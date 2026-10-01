//! Live peer cursors and selection highlights.
//!
//! Positions travel as fractions of the *layout viewport* (the window minus
//! the side panels), so peers with different window sizes or panel states see
//! each other pointing at the same part of the layout. Broadcast at a lazy
//! 10 Hz on the one reliable channel; rendering interpolates between the last
//! two samples and smooths toward the result. Incoming samples land in
//! [`RemotePeers`] through the network pump.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use flexplore_net::{MatchboxSocket, NetMsg, NetState, broadcast};

use crate::viz::VizNodePath;
use flexplore::config::{FlexConfig, PANEL_WIDTH, RIGHT_PANEL_WIDTH, RightPanelOpen};
use flexplore::presence::RemotePeers;

/// Seconds between cursor samples.
const CURSOR_INTERVAL: f32 = 0.1;
/// A cursor that has not been updated for this long is hidden (the peer's
/// pointer left the layout, or the peer is idle).
const CURSOR_STALE_SECS: f64 = 2.0;
/// Selection is resent at this interval so late joiners still converge.
const SELECTION_RESEND_SECS: f32 = 1.0;

/// Marker for a spawned selection-highlight border so we can despawn + rebuild.
#[derive(Component)]
pub struct RemoteSelHighlight;

/// The layout viewport in egui points: the window minus the side panels. Both
/// the sender and the receiver normalise against their own viewport.
fn layout_viewport(ctx: &egui::Context, right_open: bool) -> egui::Rect {
    let mut rect = ctx.viewport_rect();
    rect.min.x += PANEL_WIDTH;
    if right_open {
        rect.max.x -= RIGHT_PANEL_WIDTH;
    }
    rect
}

/// Broadcast this pointer at a lazy 10 Hz as fractions of the layout
/// viewport. Nothing is sent while the pointer is over a panel or outside the
/// window, so the remote cursor fades out.
pub fn broadcast_cursor(
    time: Res<Time>,
    mut contexts: EguiContexts,
    right_open: Res<RightPanelOpen>,
    net: Res<NetState>,
    socket: Option<ResMut<MatchboxSocket>>,
    mut next_at: Local<f32>,
) {
    let Some(mut socket) = socket else {
        return;
    };
    if net.peers.is_empty() || time.elapsed_secs() < *next_at {
        return;
    }
    *next_at = time.elapsed_secs() + CURSOR_INTERVAL;
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let Some(pos) = ctx.input(|i| i.pointer.latest_pos()) else {
        return;
    };
    let viewport = layout_viewport(ctx, right_open.0);
    if !viewport.contains(pos) || viewport.width() <= 0.0 || viewport.height() <= 0.0 {
        return;
    }
    let fractions = [
        (pos.x - viewport.left()) / viewport.width(),
        (pos.y - viewport.top()) / viewport.height(),
    ];
    broadcast(&mut socket, &net.peers, &NetMsg::Cursor { pos: fractions });
}

/// Broadcast the local selection whenever it changes, and once a second
/// regardless, so a late joiner sees it too.
pub fn broadcast_selection(
    cfg: Res<FlexConfig>,
    net: Res<NetState>,
    time: Res<Time>,
    socket: Option<ResMut<MatchboxSocket>>,
    mut last: Local<Vec<usize>>,
    mut since_send: Local<f32>,
) {
    *since_send += time.delta_secs();
    let current = cfg.selected();
    if current == last.as_slice() && *since_send < SELECTION_RESEND_SECS {
        return;
    }
    last.clear();
    last.extend_from_slice(current);
    *since_send = 0.0;
    let Some(mut socket) = socket else { return };
    if net.peers.is_empty() {
        return;
    }
    broadcast(
        &mut socket,
        &net.peers,
        &NetMsg::Selection {
            path: current.to_vec(),
        },
    );
}

/// Render every remote peer's cursor as an arrow + name label. Runs inside
/// egui's pass (painting from `Update` would be cleared by the next pass).
pub fn cursor_overlay_ui(
    mut contexts: EguiContexts,
    time: Res<Time>,
    right_open: Res<RightPanelOpen>,
    mut remote: ResMut<RemotePeers>,
) {
    if remote.0.is_empty() {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let now = time.elapsed_secs_f64();
    let dt = time.delta_secs();
    const SMOOTH: f32 = 8.0;
    let alpha = 1.0 - (-SMOOTH * dt).exp();

    let viewport = layout_viewport(ctx, right_open.0);
    let mut visible: Vec<(Vec2, [u8; 3], String)> = Vec::new();
    for peer in remote.0.iter_mut() {
        let Some(pos) = peer.cursor else { continue };
        if now - peer.last_update > CURSOR_STALE_SECS {
            peer.cursor_display = None;
            continue;
        }
        // Interpolate from the previous sample toward the latest one over the
        // broadcast interval, then smooth toward that.
        let t = (((now - peer.last_update) / CURSOR_INTERVAL as f64).clamp(0.0, 1.0)) as f32;
        let prev = peer.cursor_prev.unwrap_or(pos);
        let target = prev.lerp(pos, t);
        let display = peer.cursor_display.get_or_insert(target);
        *display = display.lerp(target, alpha);
        let label = if peer.name.is_empty() {
            "…".to_string()
        } else {
            peer.name.clone()
        };
        visible.push((*display, peer.color, label));
    }

    if visible.is_empty() {
        return;
    }

    egui::Area::new(egui::Id::new("cursor_overlay"))
        .order(egui::Order::Foreground)
        .interactable(false)
        .show(ctx, |ui| {
            let painter = ui.painter();
            for (norm, color, label) in &visible {
                let px = viewport.left() + norm.x * viewport.width();
                let py = viewport.top() + norm.y * viewport.height();
                let tip = egui::pos2(px, py);
                let col = egui::Color32::from_rgb(color[0], color[1], color[2]);

                // A small arrow-cursor silhouette.
                let arrow = [
                    tip,
                    tip + egui::vec2(0.0, 14.0),
                    tip + egui::vec2(4.0, 10.0),
                    tip + egui::vec2(7.0, 16.0),
                    tip + egui::vec2(9.0, 15.0),
                    tip + egui::vec2(6.0, 9.0),
                    tip + egui::vec2(11.0, 9.0),
                ];
                painter.add(egui::Shape::convex_polygon(
                    arrow.into(),
                    col,
                    egui::Stroke::new(1.0_f32, egui::Color32::BLACK),
                ));
                painter.text(
                    tip + egui::vec2(13.0, 6.0),
                    egui::Align2::LEFT_CENTER,
                    label,
                    egui::FontId::proportional(11.0),
                    col,
                );
            }
        });
}

/// Render each remote peer's selected node with a coloured border. Rebuilds the
/// highlight children only when the selection set changes, or when a viz
/// rebuild has despawned them along with the tree.
pub fn remote_selection_highlight(
    mut commands: Commands,
    remote: Res<RemotePeers>,
    nodes: Query<(Entity, &VizNodePath)>,
    highlights: Query<Entity, With<RemoteSelHighlight>>,
    mut signature: Local<Vec<(Vec<usize>, [u8; 3])>>,
) {
    let mut desired: Vec<(Vec<usize>, [u8; 3])> = Vec::new();
    for peer in &remote.0 {
        if !peer.selection.is_empty() {
            desired.push((peer.selection.clone(), peer.color));
        }
    }
    desired.sort();
    let despawned_by_rebuild = highlights.is_empty() && !desired.is_empty();
    if desired == *signature && !despawned_by_rebuild {
        return;
    }
    *signature = desired.clone();

    for e in &highlights {
        commands.entity(e).despawn();
    }

    let by_path: HashMap<&[usize], Entity> =
        nodes.iter().map(|(e, p)| (p.0.as_slice(), e)).collect();

    for (path, color) in &desired {
        let Some(&entity) = by_path.get(path.as_slice()) else {
            continue;
        };
        let col = Color::srgb(
            color[0] as f32 / 255.0,
            color[1] as f32 / 255.0,
            color[2] as f32 / 255.0,
        );
        let highlight = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                GlobalZIndex(98),
                BorderColor::all(col),
                Pickable::IGNORE,
                RemoteSelHighlight,
            ))
            .id();
        commands.entity(entity).add_child(highlight);
    }
}
