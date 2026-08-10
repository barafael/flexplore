//! P2P WebRTC multiplayer via matchbox, ported from the omdurman/gnils approach.
//!
//! Architecture: *host-sequenced event sourcing*. Every peer holds the full
//! layout state. A non-host submits its edit as [`NetMsg::Game`] to the host
//! only; the host assigns the next canonical sequence number and rebroadcasts
//! it as [`NetMsg::Sequenced`] to every peer (including looping it back to
//! itself). Every peer — originator included — applies an edit only when it
//! arrives as `Sequenced`, so all peers observe one canonical, ordered stream.
//!
//! The host is the lowest-sorted `PeerId`, re-derived on every peer change, so
//! when the host disconnects the next peer is promoted automatically and
//! resumes sequencing at `last_applied_seq + 1`.

use bevy::prelude::*;

use flexplore_net::{
    CH_RELIABLE, CH_UNRELIABLE, Control, Ephemeral, FlexSnapshot, LayoutEdit, MatchboxSocket,
    NetMsg, NetState, PeerId, PeerState, RoomId, build_socket, decode, enc_msg, new_player_name,
    room_id,
};

use flexplore::config::FlexConfig;
use crate::cursors::{RemotePeers, color_for_peer};
use crate::history::UndoHistory;

// ── Resources ────────────────────────────────────────────────────────────────

/// Layout edits staged by the UI this frame; routed onto the wire by
/// [`flush_pending`] (guest→host) or the host loopback (host→self sequence).
#[derive(Resource, Default)]
pub struct PendingEdits(pub Vec<LayoutEdit>);

/// Wire-level outgoing staging: reliable broadcast + reliable targeted. Filled
/// by [`handle_socket`] (sequenced echoes, control replies) and drained by
/// [`flush_pending`].
#[derive(Resource, Default)]
pub struct WirePending {
    pub broadcast: Vec<NetMsg>,
    pub targeted: Vec<(NetMsg, PeerId)>,
}

/// Frame-scoped incoming buffers: ephemeral messages for
/// [`apply_ephemeral`], and the host's self-loopback queue.
#[derive(Resource, Default)]
pub struct PendingIncoming {
    pub ephemeral: Vec<(Ephemeral, PeerId)>,
    pub loopback: Vec<NetMsg>,
}

/// The local player's display identity (announced to peers on connect).
#[derive(Resource)]
pub struct LocalPlayer {
    pub name: String,
}

impl Default for LocalPlayer {
    fn default() -> Self {
        Self {
            name: new_player_name(),
        }
    }
}

// ── Plugin ───────────────────────────────────────────────────────────────────

pub struct NetPlugin;

impl Plugin for NetPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NetState::default())
            .insert_resource(PendingEdits::default())
            .insert_resource(WirePending::default())
            .insert_resource(PendingIncoming::default())
            .insert_resource(LocalPlayer::default())
            .insert_resource(crate::cursors::RemotePeers::default())
            .insert_resource(crate::cursors::CursorBroadcastTimer::default())
            .add_systems(Startup, open_socket_for_room)
            .add_systems(
                Update,
                (
                    handle_socket,
                    retry_snapshot_request,
                    send_player_info_on_connect,
                    apply_ephemeral,
                    prune_remote_peers,
                    flush_pending,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    crate::cursors::broadcast_cursor,
                    crate::cursors::broadcast_selection,
                    crate::cursors::cursor_overlay_ui,
                    crate::cursors::remote_selection_highlight,
                ),
            );
    }
}

/// Mint a room id, register it, and open the matchbox socket.
fn open_socket_for_room(mut commands: Commands) {
    let room = room_id();
    info!(%room, "joining room");
    commands.insert_resource(RoomId(room.clone()));
    commands.insert_resource(build_socket(&room));
}

// ── Socket receive + dispatch ────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn handle_socket(
    socket: Option<ResMut<MatchboxSocket>>,
    mut net: ResMut<NetState>,
    mut pending: ResMut<PendingEdits>,
    mut wire: ResMut<WirePending>,
    mut incoming: ResMut<PendingIncoming>,
    mut cfg: ResMut<FlexConfig>,
    mut history: ResMut<UndoHistory>,
) {
    let Some(mut socket) = socket else {
        return;
    };
    let Ok(peer_updates) = socket.try_update_peers() else {
        return;
    };

    // ── Peer tracking + host election ──────────────────────────────────────
    let mut peers_changed = false;
    let mut newly_connected: Vec<PeerId> = Vec::new();
    for (peer, peer_state) in peer_updates {
        match peer_state {
            PeerState::Connected if !net.peers.contains(&peer) => {
                net.peers.push(peer);
                newly_connected.push(peer);
                peers_changed = true;
                info!(?peer, "peer connected");
            }
            PeerState::Disconnected => {
                let before = net.peers.len();
                net.peers.retain(|&p| p != peer);
                peers_changed |= net.peers.len() != before;
                info!(?peer, "peer disconnected");
            }
            _ => {}
        }
    }

    // Reconcile my_id with the socket's assigned id (None until the signalling
    // server hands one out; differs after a reconnect that swaps the socket).
    let socket_id = socket.id();
    let my_id_changed = socket_id.is_some_and(|id| Some(id) != net.my_id);
    if my_id_changed {
        net.my_id = socket_id;
    }
    if peers_changed || my_id_changed {
        net.refresh_sorted();
    }
    if let Some(my_id) = net.my_id
        && (peers_changed || my_id_changed)
    {
        let new_host_is_me = net.sorted_all().first() == Some(&my_id);
        let promoted = new_host_is_me && !net.is_host;
        if promoted {
            // A freshly promoted host must resume sequencing where the previous
            // host left off; otherwise it would re-issue seqs that the dedup
            // (`last_applied_seq`) would silently drop — a permanent desync.
            net.next_seq = net.last_applied_seq.map_or(0, |s| s + 1);
            info!(
                next_seq = net.next_seq,
                "promoted to host after previous host disconnect; resumed sequence numbering"
            );
        }
        net.is_host = new_host_is_me;
    }

    // ── Snapshot handshake on (re)connect ──────────────────────────────────
    // Host: proactively push the current document to any peer that just
    // connected (catches up both late joiners and reconnecting peers that
    // missed `Sequenced` echoes during a WebRTC blip).
    if net.is_host && !newly_connected.is_empty() {
        let snapshot = build_snapshot(&cfg, net.last_applied_seq.unwrap_or(0));
        for peer in &newly_connected {
            info!(?peer, "host: pushing snapshot to (re)connected peer");
            wire.targeted.push((
                NetMsg::Control(Control::Snapshot(Box::new(snapshot.clone()))),
                *peer,
            ));
        }
    }
    // Non-host: ask the host for the document (fallback in case the proactive
    // push is lost). Retried by `retry_snapshot_request` until applied.
    if !net.is_host && !newly_connected.is_empty() && !net.snapshot_applied {
        net.needs_snapshot = true;
        net.snapshot_retry_timer = 0.0;
        if let Some(host) = net.host_id() {
            wire.targeted
                .push((NetMsg::Control(Control::RequestSnapshot), host));
        }
    }

    // ── Receive + decode ───────────────────────────────────────────────────
    let reliable: Vec<(PeerId, Box<[u8]>)> = socket.channel_mut(CH_RELIABLE).receive();
    let unreliable: Vec<(PeerId, Box<[u8]>)> = socket.channel_mut(CH_UNRELIABLE).receive();
    let is_host = net.is_host;

    // Host loopback: events the host sequenced for itself (below). They flow
    // through the identical apply path as remote `Sequenced` events so every
    // peer — host included — observes the same ordered stream.
    let loopback: Vec<(PeerId, NetMsg)> = if let Some(my_id) = net.my_id {
        incoming
            .loopback
            .drain(..)
            .map(|msg| (my_id, msg))
            .collect()
    } else {
        incoming.loopback.clear();
        Vec::new()
    };

    let decoded = reliable
        .into_iter()
        .chain(unreliable)
        .filter_map(|(peer, raw)| match decode(&raw) {
            Some(msg) => Some((peer, msg)),
            None => {
                warn!("unknown message, ignoring");
                None
            }
        })
        .chain(loopback);

    for (peer, msg) in decoded {
        match msg {
            NetMsg::Game(edit) => {
                if !is_host {
                    // Not the host — re-forward to whoever we currently consider
                    // host (transient election disagreement right after a peer
                    // change). If no host is known yet, bounce it back onto our
                    // own pending buffer for retry.
                    match net.host_id() {
                        Some(host) => {
                            wire.targeted.push((NetMsg::Game(edit), host));
                        }
                        None => {
                            pending.0.push(edit);
                        }
                    }
                    continue;
                }
                let seq = net.next_seq;
                net.next_seq += 1;
                let sequenced = NetMsg::Sequenced { seq, edit };
                wire.broadcast.push(sequenced.clone());
                // Echo to our own loopback so the host applies its own sequenced
                // events through the same path as everyone else (next frame).
                incoming.loopback.push(sequenced);
            }
            NetMsg::Sequenced { seq, edit } => {
                // Apply each seq exactly once. The reliable channel is ordered
                // and `seq` is monotonic, so any seq at or below the highest
                // applied is a duplicate — drop it.
                if net.last_applied_seq.is_some_and(|last| seq <= last) {
                    continue;
                }
                net.last_applied_seq = Some(seq);
                apply_edit(&mut cfg, &edit);
                cfg.request_rebuild();
                history.push(cfg.clone());
            }
            NetMsg::Ephemeral(eph) => {
                incoming.ephemeral.push((eph, peer));
            }
            NetMsg::Control(Control::RequestSnapshot) => {
                if !is_host {
                    continue;
                }
                info!(?peer, "host: peer requested snapshot");
                let snapshot = build_snapshot(&cfg, net.last_applied_seq.unwrap_or(0));
                wire.targeted
                    .push((NetMsg::Control(Control::Snapshot(Box::new(snapshot))), peer));
            }
            NetMsg::Control(Control::SnapshotReceived) => {
                net.snapshot_pending.retain(|&p| p != peer);
            }
            NetMsg::Control(Control::Snapshot(snapshot)) => {
                // Accept only if it carries state ahead of what we have.
                let ahead = match net.last_applied_seq {
                    Some(applied) => snapshot.last_seq > applied,
                    None => true,
                };
                if !ahead {
                    info!("ignoring snapshot that is not ahead of local state");
                    continue;
                }
                net.snapshot_applied = true;
                net.needs_snapshot = false;
                net.snapshot_retry_timer = 0.0;
                info!(
                    last_seq = snapshot.last_seq,
                    "received snapshot, installing"
                );
                install_snapshot(&mut cfg, &snapshot);
                cfg.request_rebuild();
                history.push(cfg.clone());
                net.last_applied_seq = Some(snapshot.last_seq);
                if let Some(host) = net.host_id() {
                    wire.targeted
                        .push((NetMsg::Control(Control::SnapshotReceived), host));
                }
            }
        }
    }
}

/// Re-request the snapshot every 2s until it arrives (covers a lost first push).
fn retry_snapshot_request(time: Res<Time>, mut net: ResMut<NetState>, mut wire: ResMut<WirePending>) {
    if !net.needs_snapshot || net.snapshot_applied {
        return;
    }
    net.snapshot_retry_timer += time.delta_secs_f64();
    if net.snapshot_retry_timer > 2.0 {
        net.snapshot_retry_timer = 0.0;
        if let Some(host) = net.host_id() {
            info!("retrying snapshot request");
            wire.targeted
                .push((NetMsg::Control(Control::RequestSnapshot), host));
        }
    }
}

/// Announce our identity to each peer exactly once (targeted, reliable).
fn send_player_info_on_connect(
    net: Res<NetState>,
    local: Res<LocalPlayer>,
    mut wire: ResMut<WirePending>,
    mut notified: Local<Vec<PeerId>>,
) {
    let color = net.my_id.map_or([0xFFu8, 0xFF, 0xFF], color_for_peer);
    for &peer in &net.peers {
        if !notified.contains(&peer) {
            notified.push(peer);
            wire.targeted.push((
                NetMsg::Ephemeral(Ephemeral::PlayerInfo {
                    name: local.name.clone(),
                    color,
                }),
                peer,
            ));
        }
    }
}

/// Move ephemeral messages (cursors, selection, identity) into [`RemotePeers`].
fn apply_ephemeral(
    time: Res<Time>,
    mut incoming: ResMut<PendingIncoming>,
    mut remote: ResMut<RemotePeers>,
) {
    let now = time.elapsed_secs_f64();
    for (eph, peer) in incoming.ephemeral.drain(..) {
        let entry = remote.entry(peer);
        match eph {
            Ephemeral::CursorPos { nx, ny } => {
                let pos = Vec2::new(nx, ny);
                let prev = entry.cursor.unwrap_or(pos);
                entry.cursor_prev = Some(prev);
                entry.cursor = Some(pos);
                entry.last_update = now;
            }
            Ephemeral::Selection { path } => {
                entry.selection = path;
            }
            Ephemeral::PlayerInfo { name, color } => {
                entry.name = name;
                entry.color = color;
            }
        }
    }
}

/// Drop presence entries for peers that have left.
fn prune_remote_peers(net: Res<NetState>, mut remote: ResMut<RemotePeers>) {
    remote.prune(&net.peers);
}

// ── Flush outbound ───────────────────────────────────────────────────────────

/// Route staged messages onto the wire. Local edits ([`PendingEdits`]) wrap as
/// [`NetMsg::Game`]: the host loops them back through its own sequencing arm,
/// a guest sends them targeted to the host. Wire-level broadcast/targeted
/// (sequenced echoes, control replies) are sent directly. Failed sends are
/// retained for retry next frame.
fn flush_pending(
    mut pending: ResMut<PendingEdits>,
    mut wire: ResMut<WirePending>,
    mut incoming: ResMut<PendingIncoming>,
    net: Res<NetState>,
    mut socket: Option<ResMut<MatchboxSocket>>,
) {
    let i_sequence = net.is_host || net.peers.is_empty();
    let host = net.host_id();

    // Local edits → NetMsg::Game, routed by role.
    let local_edits = std::mem::take(&mut pending.0);
    for edit in local_edits {
        if i_sequence {
            // The sequencer's own events loop back unsequenced so handle_socket
            // assigns their seq through the same arm as guest submissions — a
            // single serialization point.
            incoming.loopback.push(NetMsg::Game(edit));
        } else {
            let submission = NetMsg::Game(edit);
            let sent = match (host, enc_msg(&submission), socket.as_deref_mut()) {
                (Some(host), Some(encoded), Some(socket)) => socket
                    .channel_mut(CH_RELIABLE)
                    .try_send(encoded, host)
                    .inspect_err(|e| warn!(error = %e, "submit to host failed; will retry"))
                    .is_ok(),
                _ => false,
            };
            if !sent {
                pending.0.push(match submission {
                    NetMsg::Game(e) => e,
                    other => unreachable!("submission was Game, got {other:?}"),
                });
            }
        }
    }

    // Wire-level targeted (reliable).
    let targeted = std::mem::take(&mut wire.targeted);
    let mut retained_targeted: Vec<(NetMsg, PeerId)> = Vec::new();
    for (msg, peer) in targeted {
        let sent = match (enc_msg(&msg), socket.as_deref_mut()) {
            (Some(encoded), Some(socket)) => socket
                .channel_mut(CH_RELIABLE)
                .try_send(encoded, peer)
                .inspect_err(|e| warn!(error = %e, "reliable targeted send failed; will retry"))
                .is_ok(),
            _ => false,
        };
        if !sent {
            retained_targeted.push((msg, peer));
        }
    }
    wire.targeted = retained_targeted;

    // Wire-level broadcast (reliable).
    let broadcast = std::mem::take(&mut wire.broadcast);
    let mut retained_broadcast: Vec<NetMsg> = Vec::new();
    for msg in broadcast {
        if net.peers.is_empty() {
            // No peers yet — retain non-sequenced; drop already-applied echoes.
            if !matches!(msg, NetMsg::Sequenced { .. }) {
                retained_broadcast.push(msg);
            }
            continue;
        }
        let Some(socket) = socket.as_deref_mut() else {
            retained_broadcast.push(msg);
            continue;
        };
        let Some(encoded) = enc_msg(&msg) else {
            retained_broadcast.push(msg);
            continue;
        };
        let channel = socket.channel_mut(CH_RELIABLE);
        let mut all_ok = true;
        for &peer in &net.peers {
            if let Err(e) = channel.try_send(encoded.clone(), peer) {
                warn!(error = %e, "reliable broadcast send failed; will retry");
                all_ok = false;
            }
        }
        if !all_ok {
            retained_broadcast.push(msg);
        }
    }
    wire.broadcast = retained_broadcast;
}

// ── Edit application ─────────────────────────────────────────────────────────

/// Apply a [`LayoutEdit`] to the document in place. A stale-path op (e.g. a
/// concurrent delete invalidated it) is logged and skipped — consistency is
/// still guaranteed because every peer applies the same host-ordered stream, so
/// all peers skip the identical op and stay in sync.
fn apply_edit(cfg: &mut FlexConfig, edit: &LayoutEdit) {
    match edit {
        LayoutEdit::ReplaceRoot(node) => {
            cfg.root = node.clone();
        }
        LayoutEdit::UpdateNode { path, node } => {
            if let Some(n) = cfg.root.get_mut(path) {
                *n = node.clone();
            } else {
                warn!(?path, "UpdateNode: path not found, skipping");
            }
        }
        LayoutEdit::AddChild { parent_path, child } => {
            if let Some(p) = cfg.root.get_mut(parent_path) {
                p.children.push(child.clone());
            } else {
                warn!(?parent_path, "AddChild: parent not found, skipping");
            }
        }
        LayoutEdit::RemoveNode { path } => {
            remove_node(&mut cfg.root, path);
        }
        LayoutEdit::MoveNode {
            src_path,
            dst_parent,
            dst_index,
        } => {
            move_node(&mut cfg.root, src_path, dst_parent, *dst_index);
        }
        LayoutEdit::UpdateSettings {
            bg_mode,
            art_style,
            art_seed,
            art_depth,
            theme,
            palette,
        } => {
            cfg.bg_mode = *bg_mode;
            cfg.art_style = *art_style;
            cfg.art_seed = *art_seed;
            cfg.art_depth = *art_depth;
            cfg.theme = *theme;
            cfg.palette = *palette;
        }
    }
}

fn remove_node(root: &mut flexplore_core::config::NodeConfig, path: &[usize]) {
    let Some((last, parent_path)) = path.split_last() else {
        warn!("RemoveNode: cannot remove root");
        return;
    };
    if let Some(parent) = root.get_mut(parent_path) {
        if *last < parent.children.len() {
            parent.children.remove(*last);
        } else {
            warn!(?path, "RemoveNode: index out of range, skipping");
        }
    } else {
        warn!(?parent_path, "RemoveNode: parent not found, skipping");
    }
}

#[allow(clippy::ptr_arg)]
fn move_node(
    root: &mut flexplore_core::config::NodeConfig,
    src_path: &[usize],
    dst_parent: &[usize],
    dst_index: usize,
) {
    if src_path.is_empty() {
        warn!("MoveNode: cannot move root");
        return;
    }
    // Guard: refuse to move a node into its own descendant.
    if dst_parent.starts_with(src_path) {
        warn!("MoveNode: destination is inside source, skipping");
        return;
    }
    let Some((src_last, src_parent_path)) = src_path.split_last() else {
        return;
    };
    let node = {
        let Some(src_parent) = root.get_mut(src_parent_path) else {
            warn!(?src_parent_path, "MoveNode: source parent not found, skipping");
            return;
        };
        if *src_last >= src_parent.children.len() {
            warn!(?src_path, "MoveNode: source index out of range, skipping");
            return;
        }
        src_parent.children.remove(*src_last)
    };
    // Adjust the destination if it was within the same parent after the removal.
    let mut adjusted_dst = dst_parent.to_vec();
    if adjusted_dst.len() >= src_parent_path.len()
        && adjusted_dst[..src_parent_path.len()] == *src_parent_path
        && adjusted_dst.len() > src_parent_path.len()
        && adjusted_dst[src_parent_path.len()] > *src_last
    {
        adjusted_dst[src_parent_path.len()] -= 1;
    }
    let Some(dst_p) = root.get_mut(&adjusted_dst) else {
        warn!(?adjusted_dst, "MoveNode: destination parent not found, skipping");
        return;
    };
    let idx = dst_index.min(dst_p.children.len());
    dst_p.children.insert(idx, node);
}

// ── Snapshot helpers ─────────────────────────────────────────────────────────

fn build_snapshot(cfg: &FlexConfig, last_seq: u32) -> FlexSnapshot {
    FlexSnapshot {
        root: cfg.root.clone(),
        bg_mode: cfg.bg_mode,
        art_style: cfg.art_style,
        art_seed: cfg.art_seed,
        art_depth: cfg.art_depth,
        theme: cfg.theme,
        palette: cfg.palette,
        last_seq,
    }
}

fn install_snapshot(cfg: &mut FlexConfig, snapshot: &FlexSnapshot) {
    cfg.root = snapshot.root.clone();
    cfg.bg_mode = snapshot.bg_mode;
    cfg.art_style = snapshot.art_style;
    cfg.art_seed = snapshot.art_seed;
    cfg.art_depth = snapshot.art_depth;
    cfg.theme = snapshot.theme;
    cfg.palette = snapshot.palette;
}
