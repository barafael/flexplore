//! In-session networking: submit local edits, apply sequenced ones.
//!
//! The one rule: **an edit is applied only when it arrives sequenced**. Local
//! edits go into the outbox and come back through the same path as remote
//! ones, so every peer sees one ordering. Solo editing takes the same path —
//! the lone peer is its own sequencer — so the networked code is always
//! exercised.
//!
//! The one liberty taken with that rule: the editor's widgets mutate the
//! document in place as the user drags a slider or types, so a local edit is
//! visible before its sequenced echo returns. Edits replace whole state, so
//! the echo finds nothing left to do; and host order still decides when two
//! peers edit at once, because the later sequenced edit overwrites.

use bevy::prelude::*;
use flexplore_net::{
    CH_RELIABLE, Document, LayoutEdit, MatchboxSocket, NetMsg, NetState, PeerId, PeerState, RoomId,
    Signaling, broadcast, decode, new_player_name, room_id, send_to,
};

use crate::config::{FlexConfig, HoverPreview};
use crate::history::UndoHistory;
use crate::presence::RemotePeers;

/// Foreign edits arriving faster than this (a peer dragging a slider) share
/// one undo entry.
const FOREIGN_HISTORY_COALESCE_SECS: f64 = 0.5;
/// A peer alone in its room for this long owns the document it started with
/// (its autosave, say). Shorter than that, an empty room may just be a slow
/// handshake to peers who hold the real document.
const ALONE_OWNS_DOCUMENT_SECS: f64 = 3.0;

/// The outbox: layout edits the UI made this frame (already applied to the
/// local document), waiting to be sequenced.
#[derive(Resource, Default)]
pub struct PendingEdits(pub Vec<LayoutEdit>);

/// Joins the room named by the [`RoomId`] resource (or a fresh one) through
/// the [`Signaling`] server, and runs host election and the message pump
/// every frame. The presence overlay is the app's; this plugin only keeps
/// [`RemotePeers`] up to date.
pub struct NetPlugin;

impl Plugin for NetPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<RoomId>() {
            app.insert_resource(room_id());
        }
        app.init_resource::<Signaling>()
            .init_resource::<NetState>()
            .init_resource::<PendingEdits>()
            .init_resource::<RemotePeers>()
            .add_systems(Startup, (name_player, flexplore_net::open_socket).chain())
            .add_systems(Update, (elect_host, pump).chain());
    }
}

/// Pick a display name once, unless the app already set one.
fn name_player(mut net: ResMut<NetState>) {
    if net.name.is_empty() {
        net.name = new_player_name();
    }
}

/// Fold the socket's peer changes into [`NetState`]: connected peers are
/// added, disconnected ones dropped. Shared by the pump and the host election
/// so the two can never disagree on who is present.
///
/// Takes the resource rather than a plain `&mut NetState`, and writes only
/// when a peer actually came or went: this runs every frame, and marking the
/// state changed every frame would leave change-driven systems nothing to
/// skip.
pub fn sync_peers(socket: &mut MatchboxSocket, net: &mut ResMut<NetState>) {
    for (peer, state) in socket.update_peers() {
        match state {
            PeerState::Connected => {
                if !net.peers.contains(&peer) {
                    info!(%peer, "peer connected");
                    net.peers.push(peer);
                }
            }
            PeerState::Disconnected => {
                if net.peers.contains(&peer) {
                    info!(%peer, "peer disconnected");
                    net.peers.retain(|p| *p != peer);
                }
            }
        }
    }
}

/// The host is the peer with the lexicographically smallest `PeerId`,
/// recomputed every frame so host loss self-heals. A freshly promoted host
/// resumes numbering after the last edit it applied, so the sequence numbers
/// its guests have already seen are never re-issued.
pub fn elect_host(socket: Option<ResMut<MatchboxSocket>>, mut net: ResMut<NetState>) {
    let Some(mut socket) = socket else {
        return;
    };

    sync_peers(&mut socket, &mut net);

    // Read before writing, here and below: this runs every frame, and a write
    // marks the state changed for every system that redraws on a change.
    if net.my_id.is_none()
        && let Some(id) = socket.id()
    {
        net.my_id = Some(id);
    }
    let Some(me) = net.my_id else {
        return;
    };

    let is_host = net.peers.iter().all(|p| me.to_string() < p.to_string());
    if is_host == net.is_host {
        return;
    }
    net.is_host = is_host;
    if is_host {
        net.next_seq = net.last_applied_seq.map_or(0, |s| s + 1);
        info!(name = %net.name, "became host");
    } else {
        info!("relinquished the host");
    }
}

/// Everything the pump reads and mutates besides the socket: the document and
/// its undo history, the live hover preview, the outbox and the room's
/// presence. One parameter so the pump stays under the argument limit.
#[derive(bevy::ecs::system::SystemParam)]
pub struct Session<'w> {
    pub net: ResMut<'w, NetState>,
    pub cfg: ResMut<'w, FlexConfig>,
    pub history: ResMut<'w, UndoHistory>,
    pub preview: ResMut<'w, HoverPreview>,
    pub outbox: ResMut<'w, PendingEdits>,
    pub remote: ResMut<'w, RemotePeers>,
    pub time: Res<'w, Time>,
}

/// Drain the outbox, then apply whatever arrived.
pub fn pump(
    socket: Option<ResMut<MatchboxSocket>>,
    mut session: Session,
    mut last_history_push: Local<f64>,
    mut alone_since: Local<Option<f64>>,
) {
    let Some(mut socket) = socket else {
        // No socket at all: the edits are already on screen, and there is
        // nobody to tell. Only reachable if the socket was closed.
        session.outbox.0.clear();
        return;
    };

    sync_peers(&mut socket, &mut session.net);
    let peers = session.net.peers.clone();
    session.remote.prune(&peers);

    // Announce ourselves to peers we have not greeted yet: once when we first
    // have a name, and afterwards only to peers that join later. The host
    // answers a greeting with the document, which is how a late joiner
    // catches up.
    let unacquainted: Vec<PeerId> = peers
        .iter()
        .filter(|p| !session.net.greeted.contains(p))
        .copied()
        .collect();
    if !session.net.name.is_empty() && !unacquainted.is_empty() {
        let hello = NetMsg::Hello {
            name: session.net.name.clone(),
        };
        broadcast(&mut socket, &unacquainted, &hello);
        info!(name = %session.net.name, greeted = unacquainted.len(), "greeted the room");
        session.net.greeted.extend(unacquainted);
    }

    let host = session.net.host();
    let now = session.time.elapsed_secs_f64();

    // A peer nobody joined for a while owns what it started with.
    if peers.is_empty() && session.net.my_id.is_some() {
        let since = *alone_since.get_or_insert(now);
        if now - since >= ALONE_OWNS_DOCUMENT_SECS && !session.net.has_document {
            session.net.has_document = true;
        }
    } else {
        *alone_since = None;
    }

    // 1. Submit local edits. Editing makes this peer's document the real one. The sequencer handles its own immediately,
    //    through the same arm that handles guests' — one serialization point.
    for edit in std::mem::take(&mut session.outbox.0) {
        session.net.has_document = true;
        if session.net.sequences() {
            sequence_and_broadcast(
                &mut socket,
                &mut session,
                &peers,
                edit,
                now,
                &mut last_history_push,
            );
        } else if let Some(host) = host {
            send_to(&mut socket, host, &NetMsg::Edit(edit));
        } else {
            warn!("no host to submit to; dropping the edit");
        }
    }

    // 2. Apply what arrived.
    let inbox: Vec<(PeerId, Box<[u8]>)> = socket.channel_mut(CH_RELIABLE).receive();
    for (from, raw) in inbox {
        let Some(msg) = decode(&raw) else {
            continue;
        };
        match msg {
            // A guest's submission: order it and rebroadcast.
            NetMsg::Edit(edit) if session.net.sequences() => {
                sequence_and_broadcast(
                    &mut socket,
                    &mut session,
                    &peers,
                    edit,
                    now,
                    &mut last_history_push,
                );
            }
            // A guest cannot sequence.
            NetMsg::Edit(_) => {}
            NetMsg::Sequenced { seq, edit } => {
                apply(&mut session, seq, edit, now, &mut last_history_push);
            }
            NetMsg::Hello { name } => {
                info!(%from, %name, "peer greeted the room");
                session.remote.entry(from).name = name;
                // Every peer holding the document answers, not only the
                // host: the greeter may have drawn the lowest id and become
                // host while still holding nothing.
                if session.net.has_document {
                    let doc = document(&session.cfg, &session.net);
                    send_to(&mut socket, from, &NetMsg::Document(Box::new(doc)));
                }
            }
            // A peer holding nothing yet takes the first document it is
            // handed; one holding a document takes only the host's, which is
            // the only authority.
            NetMsg::Document(doc) => {
                let from_host = host == Some(from);
                if !session.net.has_document || from_host {
                    install(&mut session, *doc);
                }
            }
            NetMsg::Cursor { pos } => {
                let entry = session.remote.entry(from);
                let pos = Vec2::new(pos[0], pos[1]);
                entry.cursor_prev = Some(entry.cursor.unwrap_or(pos));
                entry.cursor = Some(pos);
                entry.last_update = now;
            }
            NetMsg::Selection { path } => {
                session.remote.entry(from).selection = path;
            }
        }
    }
}

/// Assign the next sequence number, tell everyone, and apply locally.
fn sequence_and_broadcast(
    socket: &mut MatchboxSocket,
    session: &mut Session,
    peers: &[PeerId],
    edit: LayoutEdit,
    now: f64,
    last_history_push: &mut f64,
) {
    let seq = session.net.next_seq;
    session.net.next_seq += 1;
    broadcast(
        socket,
        peers,
        &NetMsg::Sequenced {
            seq,
            edit: edit.clone(),
        },
    );
    apply(session, seq, edit, now, last_history_push);
}

/// Apply a sequenced edit, if it is new.
pub fn apply(
    session: &mut Session,
    seq: u32,
    edit: LayoutEdit,
    now: f64,
    last_history_push: &mut f64,
) {
    if session.net.is_duplicate(seq) {
        return;
    }
    session.net.last_applied_seq = Some(seq);
    session.net.has_document = true;

    if !apply_edit(&mut session.cfg, &edit) {
        // Our own edit coming back, or a repeat: nothing left to do.
        return;
    }
    // A hover preview's saved copy must learn the change too, or ending the
    // hover would undo it.
    if let Some(saved) = session.preview.0.as_mut() {
        apply_edit(saved, &edit);
    }
    session.cfg.sanitize_selection();
    session.cfg.request_rebuild();
    // A burst from a peer dragging a slider becomes one undo step.
    if now - *last_history_push >= FOREIGN_HISTORY_COALESCE_SECS {
        session.history.push(session.cfg.clone());
    } else {
        session.history.replace_top(session.cfg.clone());
    }
    *last_history_push = now;
    debug!(seq, "edit applied");
}

/// Apply a [`LayoutEdit`] to the document in place; `true` if anything
/// changed. Both variants replace whole state, so applying is idempotent.
pub fn apply_edit(cfg: &mut FlexConfig, edit: &LayoutEdit) -> bool {
    match edit {
        LayoutEdit::ReplaceRoot(node) => {
            if cfg.root == **node {
                return false;
            }
            cfg.root = (**node).clone();
            true
        }
        LayoutEdit::UpdateSettings {
            bg_mode,
            art_style,
            art_seed,
            art_depth,
            theme,
            palette,
        } => {
            let changed = cfg.bg_mode != *bg_mode
                || cfg.art_style != *art_style
                || cfg.art_seed != *art_seed
                || cfg.art_depth != *art_depth
                || cfg.theme != *theme
                || cfg.palette != *palette;
            cfg.bg_mode = *bg_mode;
            cfg.art_style = *art_style;
            cfg.art_seed = *art_seed;
            cfg.art_depth = *art_depth;
            cfg.theme = *theme;
            cfg.palette = *palette;
            changed
        }
    }
}

/// The document as the host hands it to a peer that just greeted.
pub fn document(cfg: &FlexConfig, net: &NetState) -> Document {
    Document {
        root: cfg.root.clone(),
        bg_mode: cfg.bg_mode,
        art_style: cfg.art_style,
        art_seed: cfg.art_seed,
        art_depth: cfg.art_depth,
        theme: cfg.theme,
        palette: cfg.palette,
        last_seq: net.last_applied_seq,
    }
}

/// Take the host's document verbatim and start applying live edits after it.
fn install(session: &mut Session, doc: Document) {
    info!(last_seq = ?doc.last_seq, "received the document");
    let cfg = &mut session.cfg;
    cfg.root = doc.root;
    cfg.bg_mode = doc.bg_mode;
    cfg.art_style = doc.art_style;
    cfg.art_seed = doc.art_seed;
    cfg.art_depth = doc.art_depth;
    cfg.theme = doc.theme;
    cfg.palette = doc.palette;
    cfg.sanitize_selection();
    cfg.request_rebuild();
    // Whatever was being previewed belonged to the old tree.
    session.preview.0 = None;
    session.history.push(cfg.clone());
    session.net.last_applied_seq = doc.last_seq;
    session.net.has_document = true;
    if session.net.is_host {
        // Elected before the document arrived: number on from where the
        // peer that held it left off.
        session.net.next_seq = doc.last_seq.map_or(0, |s| s + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flexplore_core::config::NodeConfig;

    #[test]
    fn re_applying_an_edit_changes_nothing() {
        let mut cfg = FlexConfig::default();
        let edit = LayoutEdit::ReplaceRoot(Box::new(NodeConfig::new_leaf("solo", 10.0, 10.0)));
        assert!(
            apply_edit(&mut cfg, &edit),
            "the first application changes the tree"
        );
        assert!(
            !apply_edit(&mut cfg, &edit),
            "an echo of it finds nothing to do"
        );
    }

    #[test]
    fn settings_edits_report_whether_anything_changed() {
        let mut cfg = FlexConfig::default();
        let settings = |cfg: &FlexConfig| LayoutEdit::UpdateSettings {
            bg_mode: cfg.bg_mode,
            art_style: cfg.art_style,
            art_seed: cfg.art_seed,
            art_depth: cfg.art_depth,
            theme: cfg.theme,
            palette: cfg.palette,
        };
        let same = settings(&cfg);
        assert!(!apply_edit(&mut cfg, &same), "same settings: no change");

        let mut bumped = FlexConfig::default();
        bumped.art_seed += 1;
        assert!(apply_edit(&mut cfg, &settings(&bumped)));
        assert_eq!(cfg.art_seed, bumped.art_seed);
    }
}
