//! Remote-peer presence: live mouse cursors + selection highlights.
//!
//! Each remote peer's state (identity, cursor, selected node path) lives in
//! the [`RemotePeers`] resource, populated by [`crate::net::apply_ephemeral`]
//! from unreliable `Ephemeral` messages.
//!
//! Cursor positions are normalised to the *layout viewport* (the window minus
//! the side panels), so peers with different window sizes or panel states see
//! each other pointing at the same part of the layout. Broadcasting is
//! throttled to ~10 Hz on the unreliable channel (latest value wins; packet
//! loss is harmless). Rendering interpolates between the last two samples and
//! smooths toward the result.

use std::collections::{HashMap, hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use flexplore_net::{Ephemeral, MatchboxSocket, NetMsg, NetState, PeerId, broadcast_unreliable};

use crate::viz::VizNodePath;
use flexplore::config::{FlexConfig, PANEL_WIDTH, RIGHT_PANEL_WIDTH, RightPanelOpen};

/// Distinct peer cursor colours.
pub const PEER_COLORS: [[u8; 3]; 8] = [
    [0xF3, 0x8B, 0x6B], // coral
    [0x8B, 0xC3, 0x4A], // green
    [0x4F, 0xC3, 0xF7], // sky
    [0xBA, 0x68, 0xC8], // purple
    [0xFF, 0xD5, 0x4F], // amber
    [0x4D, 0xDB, 0xD6], // teal
    [0xFF, 0x8A, 0x65], // orange
    [0xAE, 0xE6, 0x5A], // lime
];

/// A cursor that has not been updated for this long is hidden (the peer's
/// pointer left the layout, or the peer is idle).
const CURSOR_STALE_SECS: f64 = 2.0;
/// Selection is resent at this interval so late joiners and lost packets
/// still converge.
const SELECTION_RESEND_SECS: f32 = 1.0;

/// Pick a colour deterministically from a peer id (so the same peer is always
/// the same colour across all clients).
pub fn color_for_peer(id: PeerId) -> [u8; 3] {
    let mut h = DefaultHasher::new();
    id.hash(&mut h);
    PEER_COLORS[(h.finish() as usize) % PEER_COLORS.len()]
}

// ── Remote peer state ────────────────────────────────────────────────────────

/// All remote peers' presence state, indexed by [`PeerId`].
#[derive(Resource, Default)]
pub struct RemotePeers(pub Vec<RemotePeer>);

#[derive(Clone)]
pub struct RemotePeer {
    pub id: PeerId,
    pub name: String,
    pub color: [u8; 3],
    /// Latest normalised `[x,y]` in `[0,1]` within the layout viewport.
    pub cursor: Option<Vec2>,
    pub cursor_prev: Option<Vec2>,
    /// Smoothed display position (normalised), persisted across frames.
    pub cursor_display: Option<Vec2>,
    pub last_update: f64,
    /// Selected node path (empty = root).
    pub selection: Vec<usize>,
}

impl RemotePeers {
    /// Borrow or insert the entry for `id`.
    pub fn entry(&mut self, id: PeerId) -> &mut RemotePeer {
        if let Some(i) = self.0.iter().position(|p| p.id == id) {
            &mut self.0[i]
        } else {
            self.0.push(RemotePeer {
                id,
                name: String::new(),
                color: color_for_peer(id),
                cursor: None,
                cursor_prev: None,
                cursor_display: None,
                last_update: 0.0,
                selection: Vec::new(),
            });
            self.0.last_mut().unwrap()
        }
    }

    /// Remove entries for peers no longer connected.
    pub fn prune(&mut self, connected: &[PeerId]) {
        self.0.retain(|p| connected.contains(&p.id));
    }
}

/// Throttle cursor broadcasts to ~10 Hz.
#[derive(Resource)]
pub struct CursorBroadcastTimer(Timer);

impl Default for CursorBroadcastTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.1, TimerMode::Repeating))
    }
}

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

// ── Broadcast systems ────────────────────────────────────────────────────────

/// Broadcast the local cursor position (~10 Hz), normalised to the layout
/// viewport, on the unreliable channel. Nothing is sent while the pointer is
/// over a panel or outside the window, so the remote cursor fades out.
pub fn broadcast_cursor(
    mut timer: ResMut<CursorBroadcastTimer>,
    time: Res<Time>,
    mut contexts: EguiContexts,
    right_open: Res<RightPanelOpen>,
    net: Res<NetState>,
    socket: Option<ResMut<MatchboxSocket>>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }
    let Some(mut socket) = socket else { return };
    if net.peers.is_empty() {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let Some(pos) = ctx.input(|i| i.pointer.latest_pos()) else {
        return;
    };
    let viewport = layout_viewport(ctx, right_open.0);
    if !viewport.contains(pos) || viewport.width() <= 0.0 || viewport.height() <= 0.0 {
        return;
    }
    broadcast_unreliable(
        &mut socket,
        &net.peers,
        &NetMsg::Ephemeral(Ephemeral::CursorPos {
            nx: (pos.x - viewport.left()) / viewport.width(),
            ny: (pos.y - viewport.top()) / viewport.height(),
        }),
    );
}

/// Broadcast the local selection whenever it changes, and once a second
/// regardless (unreliable channel; late joiners need it too).
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
    broadcast_unreliable(
        &mut socket,
        &net.peers,
        &NetMsg::Ephemeral(Ephemeral::Selection {
            path: current.to_vec(),
        }),
    );
}

// ── Rendering ────────────────────────────────────────────────────────────────

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
        let t = (((now - peer.last_update) / 0.1).clamp(0.0, 1.0)) as f32;
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
    // Desired set: (path, color) per remote peer that has a non-empty selection.
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

    // Despawn old highlights.
    for e in &highlights {
        commands.entity(e).despawn();
    }

    // Index viz nodes by path for O(1) lookup.
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
