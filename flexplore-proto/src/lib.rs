//! Wire protocol for flexplore multiplayer.
//!
//! This crate defines the *contract* — the message types peers exchange and
//! their postcard encoding — so the transport (`flexplore-net`) and any future
//! non-bevy consumer (relay server, headless peer, fuzzer) share one wire
//! format. It depends on `flexplore-core` for the document types only; there
//! is no bevy, matchbox or tokio here.
//!
//! # Model
//!
//! *Host-sequenced, whole-document edits with client-side prediction.*
//!
//! Every peer holds the full layout. One peer is the **host** for the current
//! **epoch**; it assigns a sequence number to every edit and rebroadcasts it as
//! [`NetMsg::Sequenced`]. Edits carry the originating `client` tag and a
//! per-client `local_id`, so the originator recognises its own echo (it has
//! already applied the edit locally) and every other peer applies the edit in
//! host order. Because the only edits are whole-document replacements, a peer
//! re-applies its own unacknowledged edits on top of foreign ones, which makes
//! the result last-writer-wins in host order on every peer.
//!
//! The host changes when the previous host leaves. A promotion starts a new
//! epoch; higher epochs always win, and ties (two peers claiming the same
//! epoch) are broken by the earlier claim time, then the lower peer id.

use flexplore_core::config::{ArtStyle, BackgroundMode, ColorPalette, NodeConfig, Theme};
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

/// Bump whenever the encoding of any message changes. Peers announce it in
/// [`Control::Hello`]; a mismatch is logged and the peer's messages ignored.
pub const PROTOCOL_VERSION: u32 = 2;

/// A transport peer id (matchbox's `PeerId` is a UUID) as raw bytes, so this
/// crate stays independent of the transport.
pub type PeerBytes = [u8; 16];

// ── Layout edits (the only things that mutate the document) ──────────────────

/// A single document mutation. Both variants replace whole state, which is
/// what makes prediction + rebase trivial: re-applying an edit is idempotent.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum LayoutEdit {
    /// Replace the entire layout tree. Boxed so the wire enums stay small.
    ReplaceRoot(Box<NodeConfig>),
    /// Update visual settings (theme, palette, art, background).
    UpdateSettings {
        bg_mode: BackgroundMode,
        art_style: ArtStyle,
        art_seed: u64,
        art_depth: u32,
        theme: Theme,
        palette: ColorPalette,
    },
}

/// Full document snapshot, used to catch up joiners and to reset everyone
/// when a new host takes over.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FlexSnapshot {
    pub root: NodeConfig,
    pub bg_mode: BackgroundMode,
    pub art_style: ArtStyle,
    pub art_seed: u64,
    pub art_depth: u32,
    pub theme: Theme,
    pub palette: ColorPalette,
    /// The sending host's epoch and when it claimed hosting (UNIX ms). Together
    /// with the sender's peer id this is the host key receivers compare.
    pub epoch: u32,
    pub claimed_at_ms: u64,
    /// Highest `Sequenced` seq of this epoch folded into the snapshot; `None`
    /// when the host has not sequenced anything yet in this epoch.
    pub last_seq: Option<u32>,
}

/// Display-only state shared between peers but never recorded. Sent on the
/// unreliable channel (except `PlayerInfo`, one-shot reliable on connect).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Ephemeral {
    /// Cursor position normalised to the layout viewport, `[0,1]` on each axis.
    CursorPos { nx: f32, ny: f32 },
    /// The node path this peer has selected (empty = root).
    Selection { path: Vec<usize> },
    /// Identity announcement (once per new peer).
    PlayerInfo { name: String, color: [u8; 3] },
}

/// Membership and snapshot handshake. Always reliable.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Control {
    /// Sent to every peer on connect, and again whenever `has_document` or
    /// the host view changes. Lets peers agree on the host and know who is
    /// eligible to take over.
    Hello {
        protocol: u32,
        /// Random tag identifying this peer's edits (see [`NetMsg::Game`]).
        client: u64,
        /// The sender's view of the current host key, if it has one.
        epoch: u32,
        claimed_at_ms: u64,
        epoch_host: Option<PeerBytes>,
        /// True once the sender holds the room's document (it installed a
        /// snapshot, or it is/was the host).
        has_document: bool,
    },
    /// Joiner → host: ask for the current document.
    RequestSnapshot,
    /// Host → peer: the full document + host key + watermark.
    Snapshot(Box<FlexSnapshot>),
}

// ── Wire protocol ────────────────────────────────────────────────────────────

/// Top-level wire envelope.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum NetMsg {
    /// Unsequenced edit submission, sent guest→host. The guest has already
    /// applied it locally (prediction); it is never applied directly by the
    /// host, only sequenced.
    Game {
        client: u64,
        local_id: u32,
        edit: LayoutEdit,
    },
    /// Canonical, host-sequenced edit, sent host→all. `client`/`local_id`
    /// identify the originator so it can skip its own echo.
    Sequenced {
        epoch: u32,
        seq: u32,
        client: u64,
        local_id: u32,
        edit: LayoutEdit,
    },
    Ephemeral(Ephemeral),
    Control(Control),
}

/// Encode a `NetMsg` for the wire. Returns `None` if encoding fails or would
/// produce a zero-length payload. WebRTC data channels may silently drop a
/// zero-byte payload, so we refuse to emit one entirely.
pub fn enc_msg(msg: &NetMsg) -> Option<Box<[u8]>> {
    match postcard::to_allocvec(msg) {
        Ok(v) if !v.is_empty() => Some(v.into_boxed_slice()),
        Ok(_) => {
            error!("postcard produced an empty NetMsg encoding; dropping");
            None
        }
        Err(e) => {
            error!("postcard encode failed: {e}");
            None
        }
    }
}

/// Decode a `NetMsg` from the wire. Returns `None` on a corrupt or truncated
/// payload. The caller decides how loudly to complain, so one misbehaving
/// peer cannot flood the log through this function.
pub fn decode(raw: &[u8]) -> Option<NetMsg> {
    postcard::from_bytes(raw)
        .inspect_err(|e| warn!("matchbox decode error: {e}"))
        .ok()
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_edits() -> Vec<LayoutEdit> {
        let leaf = NodeConfig::new_leaf("child", 100.0, 50.0);
        vec![
            LayoutEdit::ReplaceRoot(Box::new(leaf)),
            LayoutEdit::UpdateSettings {
                bg_mode: BackgroundMode::RandomArt,
                art_style: ArtStyle::Voronoi,
                art_seed: 42,
                art_depth: 3,
                theme: Theme::Mocha,
                palette: ColorPalette::Dark2,
            },
        ]
    }

    fn sample_snapshot() -> FlexSnapshot {
        FlexSnapshot {
            root: NodeConfig::new_leaf("root", 800.0, 600.0),
            bg_mode: BackgroundMode::Pastel,
            art_style: ArtStyle::ExprTree,
            art_seed: 1,
            art_depth: 2,
            theme: Theme::Latte,
            palette: ColorPalette::Pastel1,
            epoch: 3,
            claimed_at_ms: 1_700_000_000_000,
            last_seq: Some(42),
        }
    }

    fn sample_messages() -> Vec<NetMsg> {
        vec![
            NetMsg::Game {
                client: 0xDEAD_BEEF,
                local_id: 5,
                edit: sample_edits()[0].clone(),
            },
            NetMsg::Sequenced {
                epoch: 2,
                seq: 7,
                client: 0xDEAD_BEEF,
                local_id: 5,
                edit: sample_edits()[1].clone(),
            },
            NetMsg::Ephemeral(Ephemeral::CursorPos { nx: 0.25, ny: 0.75 }),
            NetMsg::Ephemeral(Ephemeral::Selection { path: vec![0, 1] }),
            NetMsg::Ephemeral(Ephemeral::PlayerInfo {
                name: "Brave Otter".into(),
                color: [200, 100, 50],
            }),
            NetMsg::Control(Control::Hello {
                protocol: PROTOCOL_VERSION,
                client: 1,
                epoch: 2,
                claimed_at_ms: 12345,
                epoch_host: Some([7; 16]),
                has_document: true,
            }),
            NetMsg::Control(Control::Hello {
                protocol: PROTOCOL_VERSION,
                client: 1,
                epoch: 0,
                claimed_at_ms: 0,
                epoch_host: None,
                has_document: false,
            }),
            NetMsg::Control(Control::RequestSnapshot),
            NetMsg::Control(Control::Snapshot(Box::new(sample_snapshot()))),
        ]
    }

    #[test]
    fn all_variants_round_trip() {
        for msg in sample_messages() {
            let raw = enc_msg(&msg).expect("encode should succeed");
            assert!(!raw.is_empty(), "no zero-length payloads");
            let back = decode(&raw).expect("decode should succeed");
            assert_eq!(back, msg);
        }
    }

    #[test]
    fn snapshot_survives_boxing() {
        let snapshot = sample_snapshot();
        let msg = NetMsg::Control(Control::Snapshot(Box::new(snapshot.clone())));
        let raw = enc_msg(&msg).unwrap();
        match decode(&raw).unwrap() {
            NetMsg::Control(Control::Snapshot(b)) => {
                assert_eq!(&*b, &snapshot);
            }
            other => panic!("unexpected decode: {other:?}"),
        }
    }

    #[test]
    fn corrupt_payload_is_rejected() {
        assert!(decode(&[0xff, 0x00, 0x01]).is_none());
        assert!(decode(&[]).is_none());
    }

    #[test]
    fn partial_eq_is_exact() {
        let a = NetMsg::Ephemeral(Ephemeral::CursorPos { nx: 0.1, ny: 0.2 });
        let b = NetMsg::Ephemeral(Ephemeral::CursorPos { nx: 0.1, ny: 0.3 });
        assert_ne!(a, b);
    }
}
