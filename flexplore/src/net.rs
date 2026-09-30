//! Multiplayer systems: socket handling, host election, edit sequencing with
//! client-side prediction, snapshots and presence routing.
//!
//! See the `flexplore-proto` crate docs for the model. In short: every edit is
//! applied locally the moment the user makes it, then submitted to the host.
//! The host orders edits and echoes them to everyone. A peer skips the echo of
//! its own edits and re-applies its still-pending edits on top of foreign
//! ones, so every peer converges on the host's order.

use bevy::prelude::*;

use flexplore_net::{
    CH_RELIABLE, CH_UNRELIABLE, CLAIM_GRACE_SECS, Control, Ephemeral, FlexSnapshot, HostKey,
    LayoutEdit, MatchboxSocket, NetMsg, NetState, PROMOTE_GRACE_SECS, PROTOCOL_VERSION, PeerId,
    PeerInfo, PeerState, build_socket, decode, enc_msg, new_player_name, peer_from_bytes,
    peer_to_bytes, room_id, unix_ms,
};

use crate::cursors::{RemotePeers, color_for_peer};
use crate::history::UndoHistory;
use crate::panel::HoverPreview;
use flexplore::config::FlexConfig;

/// Foreign edits arriving faster than this (a peer dragging a slider) share
/// one undo entry.
const FOREIGN_HISTORY_COALESCE_SECS: f64 = 0.5;
/// Upper bound on unacknowledged own edits we remember. Older ones are
/// superseded anyway (every edit replaces whole state), so dropping them only
/// affects echo matching, which treats an unknown old echo as superseded.
const MAX_PENDING_OWN: usize = 256;

// ── Resources ────────────────────────────────────────────────────────────────

/// Layout edits the UI made this frame (already applied locally). Routed onto
/// the wire by [`flush_pending`].
#[derive(Resource, Default)]
pub struct PendingEdits(pub Vec<LayoutEdit>);

/// Outgoing reliable messages staged by the systems, drained by
/// [`flush_pending`].
#[derive(Resource, Default)]
pub struct WirePending {
    pub broadcast: Vec<NetMsg>,
    pub targeted: Vec<(NetMsg, PeerId)>,
}

/// Ephemeral messages received this frame, consumed by [`apply_ephemeral`].
#[derive(Resource, Default)]
pub struct PendingIncoming {
    pub ephemeral: Vec<(Ephemeral, PeerId)>,
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
        app.init_resource::<NetState>()
            .init_resource::<PendingEdits>()
            .init_resource::<WirePending>()
            .init_resource::<PendingIncoming>()
            .init_resource::<LocalPlayer>()
            .init_resource::<crate::cursors::RemotePeers>()
            .init_resource::<crate::cursors::CursorBroadcastTimer>()
            .add_systems(Startup, open_socket_for_room)
            .add_systems(
                Update,
                (
                    handle_socket,
                    elect_host,
                    retry_snapshot_request,
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
                    crate::cursors::remote_selection_highlight,
                ),
            )
            // Painting must happen inside egui's pass, or the next pass clears it.
            .add_systems(
                bevy_egui::EguiPrimaryContextPass,
                crate::cursors::cursor_overlay_ui.after(crate::panel::panel_system),
            );
    }
}

/// Mint a room id and open the matchbox socket.
fn open_socket_for_room(mut commands: Commands) {
    let room = room_id();
    info!(%room, "joining room");
    commands.insert_resource(build_socket(&room));
}

// ── Socket receive + dispatch ────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn handle_socket(
    mut commands: Commands,
    socket: Option<ResMut<MatchboxSocket>>,
    mut net: ResMut<NetState>,
    mut wire: ResMut<WirePending>,
    mut incoming: ResMut<PendingIncoming>,
    mut cfg: ResMut<FlexConfig>,
    mut history: ResMut<UndoHistory>,
    mut preview: ResMut<HoverPreview>,
    local: Res<LocalPlayer>,
    time: Res<Time>,
    mut last_foreign_push: Local<f64>,
) {
    let Some(mut socket) = socket else {
        return;
    };
    let now = time.elapsed_secs_f64();

    let peer_updates = match socket.try_update_peers() {
        Ok(updates) => updates,
        Err(e) => {
            // The socket is gone for good; carry on as a local-only session.
            warn!(error = %e, "socket closed; continuing offline");
            commands.remove_resource::<MatchboxSocket>();
            net.peers.clear();
            net.info.clear();
            net.host = None;
            net.pending_own.clear();
            wire.broadcast.clear();
            wire.targeted.clear();
            return;
        }
    };

    // ── Peer tracking ──────────────────────────────────────────────────────
    let mut newly_connected: Vec<PeerId> = Vec::new();
    for (peer, peer_state) in peer_updates {
        match peer_state {
            PeerState::Connected if !net.peers.contains(&peer) => {
                net.peers.push(peer);
                newly_connected.push(peer);
                info!(?peer, "peer connected");
            }
            PeerState::Disconnected => {
                net.peers.retain(|&p| p != peer);
                net.info.retain(|(p, _)| *p != peer);
                info!(?peer, "peer disconnected");
                if net.host_id() == Some(peer) {
                    info!("host left; waiting for a successor");
                    net.host = None;
                    net.unhosted_since = Some(now);
                    net.needs_snapshot = false;
                }
            }
            _ => {}
        }
    }

    // Reconcile my_id with the socket's assigned id (None until the signalling
    // server hands one out; differs after a reconnect that swaps the socket).
    let socket_id = socket.id();
    if socket_id.is_some_and(|id| Some(id) != net.my_id) {
        if net.my_id.is_some() {
            // New identity: peers see the old id leave. Start over as a joiner
            // (we keep the document, so we stay eligible for promotion).
            net.host = None;
            net.unhosted_since = Some(now);
        }
        net.my_id = socket_id;
    }
    let Some(my_id) = net.my_id else {
        // Nothing can be addressed until we have an id; keep messages queued.
        return;
    };

    // ── Greet new peers ────────────────────────────────────────────────────
    for &peer in &newly_connected {
        wire.targeted.push((hello(&net), peer));
        wire.targeted.push((
            NetMsg::Ephemeral(Ephemeral::PlayerInfo {
                name: local.name.clone(),
                color: color_for_peer(my_id),
            }),
            peer,
        ));
        if net.is_host() {
            info!(?peer, "host: pushing snapshot to new peer");
            let snapshot = build_snapshot(&cfg, &net);
            wire.targeted
                .push((NetMsg::Control(Control::Snapshot(Box::new(snapshot))), peer));
        }
    }
    // A joiner asks its host for the document as soon as the host is
    // reachable (the host's proactive push covers the usual case).
    if !net.is_host()
        && let Some(host) = net.host_id()
        && newly_connected.contains(&host)
        && !net.has_document
    {
        net.needs_snapshot = true;
        wire.targeted
            .push((NetMsg::Control(Control::RequestSnapshot), host));
    }

    // ── Receive + dispatch ─────────────────────────────────────────────────
    let reliable: Vec<(PeerId, Box<[u8]>)> = socket.channel_mut(CH_RELIABLE).receive();
    let unreliable: Vec<(PeerId, Box<[u8]>)> = socket.channel_mut(CH_UNRELIABLE).receive();
    let decoded = reliable
        .into_iter()
        .chain(unreliable)
        .filter_map(|(peer, raw)| decode(&raw).map(|msg| (peer, msg)));

    for (peer, msg) in decoded {
        // Ignore peers speaking another protocol version (logged on Hello).
        if net.peer_info(peer).is_some_and(|i| !i.protocol_ok) {
            continue;
        }
        match msg {
            NetMsg::Game {
                client,
                local_id,
                edit,
            } => {
                if !net.is_host() {
                    // Sent to us by a peer that still thinks we host. Forward
                    // to the host we know, if any; otherwise the sender's
                    // pending list resends it once a host is known.
                    if let Some(host) = net.host_id() {
                        wire.targeted.push((
                            NetMsg::Game {
                                client,
                                local_id,
                                edit,
                            },
                            host,
                        ));
                    }
                    continue;
                }
                let Some(key) = net.host else { continue };
                let seq = net.next_seq;
                net.next_seq = net.next_seq.wrapping_add(1);
                net.last_applied_seq = Some(seq);
                wire.broadcast.push(NetMsg::Sequenced {
                    epoch: key.epoch,
                    seq,
                    client,
                    local_id,
                    edit: edit.clone(),
                });
                // The host applies foreign edits at sequencing time.
                if client != net.client {
                    apply_foreign(
                        &mut cfg,
                        &mut preview,
                        &mut history,
                        &net,
                        &edit,
                        now,
                        &mut last_foreign_push,
                    );
                }
            }

            NetMsg::Sequenced {
                epoch,
                seq,
                client,
                local_id,
                edit,
            } => {
                // Our own broadcasts never come back; anything sequenced by a
                // higher epoch than ours means we were superseded as host.
                let from_current_host = net
                    .host
                    .is_some_and(|k| k.host() == peer && k.epoch == epoch);
                if !from_current_host {
                    if net.host.is_none_or(|k| epoch > k.epoch) {
                        // A host we have not heard about yet (its snapshot or
                        // Hello is still in flight). Follow it provisionally
                        // with the weakest possible key so its real key wins
                        // ties, and fetch its document.
                        adopt_host(&mut net, HostKey::new(epoch, u64::MAX, peer), &mut wire);
                        net.needs_snapshot = true;
                        wire.targeted
                            .push((NetMsg::Control(Control::RequestSnapshot), peer));
                    } else {
                        debug!(?peer, epoch, "dropping Sequenced from a stale host");
                        continue;
                    }
                }
                // Each seq is applied at most once; the channel is ordered, so
                // anything at or below the watermark is a duplicate.
                if net.last_applied_seq.is_some_and(|last| seq <= last) {
                    continue;
                }
                if net
                    .last_applied_seq
                    .is_some_and(|last| seq > last.wrapping_add(1))
                {
                    warn!(seq, "gap in sequenced edits; requesting a snapshot");
                    net.needs_snapshot = true;
                }
                net.last_applied_seq = Some(seq);

                if client == net.client {
                    // Our own edit coming back. If it is still pending we have
                    // already applied it (and everything older is now acked).
                    // If it is older than everything pending, a newer edit of
                    // ours supersedes it. Only a duplicate sequencing of an
                    // already-acked edit (a resend after a host change) needs
                    // applying, to match what every other peer just did.
                    if let Some(pos) = net.pending_own.iter().position(|(id, _)| *id == local_id) {
                        net.pending_own.drain(..=pos);
                        continue;
                    }
                    if net
                        .pending_own
                        .first()
                        .is_some_and(|(first, _)| *first > local_id)
                    {
                        continue;
                    }
                }
                apply_foreign(
                    &mut cfg,
                    &mut preview,
                    &mut history,
                    &net,
                    &edit,
                    now,
                    &mut last_foreign_push,
                );
            }

            NetMsg::Ephemeral(eph) => {
                incoming.ephemeral.push((eph, peer));
            }

            NetMsg::Control(Control::Hello {
                protocol,
                client,
                epoch,
                claimed_at_ms,
                epoch_host,
                has_document,
            }) => {
                let protocol_ok = protocol == PROTOCOL_VERSION;
                if !protocol_ok {
                    warn!(
                        ?peer,
                        theirs = protocol,
                        ours = PROTOCOL_VERSION,
                        "peer speaks another protocol version; ignoring it"
                    );
                }
                net.set_peer_info(
                    peer,
                    PeerInfo {
                        client,
                        has_document,
                        protocol_ok,
                    },
                );
                if !protocol_ok {
                    continue;
                }
                net.max_epoch_seen = net.max_epoch_seen.max(epoch);
                if let Some(host) = epoch_host {
                    let key = HostKey::new(epoch, claimed_at_ms, peer_from_bytes(host));
                    if net.host.is_none_or(|cur| key > cur) {
                        adopt_host(&mut net, key, &mut wire);
                        if !net.is_host() && !net.has_document {
                            net.needs_snapshot = true;
                            if net.peers.contains(&key.host()) {
                                wire.targeted
                                    .push((NetMsg::Control(Control::RequestSnapshot), key.host()));
                            }
                        }
                    }
                }
            }

            NetMsg::Control(Control::RequestSnapshot) => {
                if !net.is_host() {
                    continue;
                }
                info!(?peer, "host: peer requested snapshot");
                let snapshot = build_snapshot(&cfg, &net);
                wire.targeted
                    .push((NetMsg::Control(Control::Snapshot(Box::new(snapshot))), peer));
            }

            NetMsg::Control(Control::Snapshot(snapshot)) => {
                let key = HostKey::new(snapshot.epoch, snapshot.claimed_at_ms, peer);
                // Accept from the host we follow (a refresh) or from a host
                // with a stronger claim (a promotion, or a real key replacing
                // a provisional one).
                let accept = net
                    .host
                    .is_none_or(|cur| key >= cur || (cur.host() == peer && cur.epoch == key.epoch));
                if !accept {
                    info!(?peer, "ignoring snapshot from a stale host");
                    continue;
                }
                let was_host = net.is_host();
                adopt_host(&mut net, key, &mut wire);
                net.last_applied_seq = snapshot.last_seq;
                net.needs_snapshot = false;
                net.snapshot_retry_timer = 0.0;
                let first_document = !net.has_document;
                net.has_document = true;
                info!(
                    ?peer,
                    epoch = snapshot.epoch,
                    last_seq = ?snapshot.last_seq,
                    was_host,
                    "installing snapshot"
                );
                install_snapshot(&mut cfg, &snapshot);
                // Whatever we were previewing is gone with the old tree.
                preview.0 = None;
                // Keep our unacknowledged edits on top and hand them to this
                // host; duplicates are harmless (whole-state edits).
                for (local_id, edit) in &net.pending_own {
                    apply_edit(&mut cfg, edit);
                    wire.targeted.push((
                        NetMsg::Game {
                            client: net.client,
                            local_id: *local_id,
                            edit: edit.clone(),
                        },
                        peer,
                    ));
                }
                cfg.sanitize_selection();
                cfg.request_rebuild();
                history.push(cfg.clone());
                if first_document {
                    // We are now eligible to take over: tell everyone.
                    let msg = hello(&net);
                    for &p in &net.peers {
                        wire.targeted.push((msg.clone(), p));
                    }
                }
            }
        }
    }
}

/// Follow `key` as the current host. Resets the per-epoch watermark when the
/// epoch or host actually changes.
fn adopt_host(net: &mut NetState, key: HostKey, wire: &mut WirePending) {
    let changed = net
        .host
        .is_none_or(|cur| cur.host() != key.host() || cur.epoch != key.epoch);
    let was_host = net.is_host();
    net.host = Some(key);
    net.max_epoch_seen = net.max_epoch_seen.max(key.epoch);
    net.unhosted_since = None;
    if changed {
        net.last_applied_seq = None;
        if was_host && !net.is_host() {
            info!(host = ?key.host(), epoch = key.epoch, "demoted; following the new host");
            // Our sequencing role ended; from now on our edits are predictions.
            let msg = hello(net);
            for &p in &net.peers {
                wire.targeted.push((msg.clone(), p));
            }
        } else if !net.is_host() {
            info!(host = ?key.host(), epoch = key.epoch, "following host");
        }
    }
}

/// Apply an edit from another peer, then re-apply our own unacknowledged
/// edits on top (prediction rebase), mirroring both into the hover preview's
/// saved copy so ending a hover cannot undo a remote change.
fn apply_foreign(
    cfg: &mut FlexConfig,
    preview: &mut HoverPreview,
    history: &mut UndoHistory,
    net: &NetState,
    edit: &LayoutEdit,
    now: f64,
    last_history_push: &mut f64,
) {
    apply_edit(cfg, edit);
    for (_, own) in &net.pending_own {
        apply_edit(cfg, own);
    }
    if let Some(saved) = preview.0.as_mut() {
        apply_edit(saved, edit);
        for (_, own) in &net.pending_own {
            apply_edit(saved, own);
        }
    }
    cfg.sanitize_selection();
    cfg.request_rebuild();
    if now - *last_history_push >= FOREIGN_HISTORY_COALESCE_SECS {
        history.push(cfg.clone());
    } else {
        history.replace_top(cfg.clone());
    }
    *last_history_push = now;
}

/// Our membership announcement.
fn hello(net: &NetState) -> NetMsg {
    NetMsg::Control(Control::Hello {
        protocol: PROTOCOL_VERSION,
        client: net.client,
        epoch: net.host.map_or(0, |k| k.epoch),
        claimed_at_ms: net.host.map_or(0, |k| k.claimed_at_ms()),
        epoch_host: net.host.map(|k| peer_to_bytes(k.host())),
        has_document: net.has_document,
    })
}

// ── Host election ────────────────────────────────────────────────────────────

/// While nobody hosts, the most senior eligible peer claims hosting after a
/// grace period: 5 s when alone (a slow WebRTC handshake to an existing room
/// must not look like an empty room), 1 s after a host leaves.
fn elect_host(
    time: Res<Time>,
    mut net: ResMut<NetState>,
    mut wire: ResMut<WirePending>,
    cfg: Res<FlexConfig>,
) {
    let Some(my_id) = net.my_id else { return };
    if net.host.is_some() {
        net.unhosted_since = None;
        return;
    }
    let now = time.elapsed_secs_f64();
    let since = *net.unhosted_since.get_or_insert(now);
    let grace = if net.peers.is_empty() {
        CLAIM_GRACE_SECS
    } else {
        PROMOTE_GRACE_SECS
    };
    if now - since < grace || net.election_winner() != Some(my_id) {
        return;
    }

    let epoch = net.max_epoch_seen.wrapping_add(1);
    let key = HostKey::new(epoch, unix_ms(), my_id);
    net.host = Some(key);
    net.max_epoch_seen = epoch;
    net.has_document = true;
    net.next_seq = 0;
    net.last_applied_seq = None;
    net.needs_snapshot = false;
    // Everything we did so far is the document now.
    net.pending_own.clear();
    net.unhosted_since = None;
    if net.peers.is_empty() {
        info!(epoch, "hosting a new room");
    } else {
        info!(epoch, peers = net.peers.len(), "promoted to host");
    }
    let announce = hello(&net);
    let snapshot = NetMsg::Control(Control::Snapshot(Box::new(build_snapshot(&cfg, &net))));
    for &peer in &net.peers {
        wire.targeted.push((announce.clone(), peer));
        wire.targeted.push((snapshot.clone(), peer));
    }
}

/// Re-request the snapshot every 2 s until it arrives (covers a lost push).
fn retry_snapshot_request(
    time: Res<Time>,
    mut net: ResMut<NetState>,
    mut wire: ResMut<WirePending>,
) {
    if !net.needs_snapshot || net.is_host() {
        return;
    }
    let Some(host) = net.host_id() else { return };
    if !net.peers.contains(&host) {
        return;
    }
    net.snapshot_retry_timer += time.delta_secs_f64();
    if net.snapshot_retry_timer > 2.0 {
        net.snapshot_retry_timer = 0.0;
        info!("retrying snapshot request");
        wire.targeted
            .push((NetMsg::Control(Control::RequestSnapshot), host));
    }
}

// ── Presence ─────────────────────────────────────────────────────────────────

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

/// Route this frame's local edits and staged wire messages onto the socket.
/// Sends are best effort: the reliable channel only fails once a peer is gone,
/// and whole-state edits make any later edit a full repair.
fn flush_pending(
    mut pending: ResMut<PendingEdits>,
    mut wire: ResMut<WirePending>,
    mut net: ResMut<NetState>,
    mut socket: Option<ResMut<MatchboxSocket>>,
) {
    let local_edits = std::mem::take(&mut pending.0);
    if !net.peers.is_empty() {
        for edit in local_edits {
            let local_id = net.next_local_id();
            if let Some(key) = net.host.filter(|_| net.is_host()) {
                let seq = net.next_seq;
                net.next_seq = net.next_seq.wrapping_add(1);
                net.last_applied_seq = Some(seq);
                wire.broadcast.push(NetMsg::Sequenced {
                    epoch: key.epoch,
                    seq,
                    client: net.client,
                    local_id,
                    edit,
                });
            } else {
                if let Some(host) = net.host_id() {
                    wire.targeted.push((
                        NetMsg::Game {
                            client: net.client,
                            local_id,
                            edit: edit.clone(),
                        },
                        host,
                    ));
                }
                // Kept until the host echoes it (or a newer edit of ours is
                // acked); resent to whichever host sends us a snapshot.
                net.pending_own.push((local_id, edit));
                if net.pending_own.len() > MAX_PENDING_OWN {
                    let excess = net.pending_own.len() - MAX_PENDING_OWN;
                    net.pending_own.drain(..excess);
                }
            }
        }
    }
    // Alone: edits are local only, and nothing is worth queueing.

    let Some(socket) = socket.as_deref_mut() else {
        wire.targeted.clear();
        wire.broadcast.clear();
        return;
    };
    let channel = socket.channel_mut(CH_RELIABLE);
    for (msg, peer) in wire.targeted.drain(..) {
        if !net.peers.contains(&peer) {
            continue;
        }
        if let Some(encoded) = enc_msg(&msg)
            && let Err(e) = channel.try_send(encoded, peer)
        {
            warn!(?peer, error = %e, "reliable send failed; dropping message");
        }
    }
    for msg in wire.broadcast.drain(..) {
        let Some(encoded) = enc_msg(&msg) else {
            continue;
        };
        for &peer in &net.peers {
            if let Err(e) = channel.try_send(encoded.clone(), peer) {
                warn!(?peer, error = %e, "reliable broadcast failed; dropping message");
            }
        }
    }
}

// ── Edit application ─────────────────────────────────────────────────────────

/// Apply a [`LayoutEdit`] to the document in place. Both variants replace
/// whole state, so applying is idempotent and order alone decides the result.
pub fn apply_edit(cfg: &mut FlexConfig, edit: &LayoutEdit) {
    match edit {
        LayoutEdit::ReplaceRoot(node) => {
            cfg.root = (**node).clone();
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

// ── Snapshot helpers ─────────────────────────────────────────────────────────

fn build_snapshot(cfg: &FlexConfig, net: &NetState) -> FlexSnapshot {
    let key = net.host.expect("only the host builds snapshots");
    FlexSnapshot {
        root: cfg.root.clone(),
        bg_mode: cfg.bg_mode,
        art_style: cfg.art_style,
        art_seed: cfg.art_seed,
        art_depth: cfg.art_depth,
        theme: cfg.theme,
        palette: cfg.palette,
        epoch: key.epoch,
        claimed_at_ms: key.claimed_at_ms(),
        last_seq: net.last_applied_seq,
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
