//! Wire protocol for flexplore multiplayer.
//!
//! This crate is deliberately dependency-light (no bevy, no matchbox, no
//! tokio): it defines the *contract* — the message types peers exchange and
//! their postcard encoding — so the transport and any future non-bevy consumer
//! (relay server, headless peer, fuzzer) share one wire format.
//!
//! The networking stack lives in `flexplore-net`, which re-exports everything
//! here.

use flexplore_core::config::{
    ArtStyle, BackgroundMode, ColorPalette, NodeConfig, Theme,
};
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

// ── Layout edits (the only things that mutate the document) ──────────────────

/// A single document mutation. Submitted guest→host as [`NetMsg::Game`],
/// sequenced by the host into [`NetMsg::Sequenced`], then applied on echo.
/// `seq` doubles as the revision counter.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum LayoutEdit {
    /// Replace the entire layout tree (template load, import, snapshot install).
    ReplaceRoot(NodeConfig),
    /// Replace a single node at the given path.
    UpdateNode { path: Vec<usize>, node: NodeConfig },
    /// Add a child node at the given parent path.
    AddChild { parent_path: Vec<usize>, child: NodeConfig },
    /// Remove the node at the given path (empty path = no-op, can't remove root).
    RemoveNode { path: Vec<usize> },
    /// Move a node from `src_path` into `dst_parent` at `dst_index`.
    MoveNode {
        src_path: Vec<usize>,
        dst_parent: Vec<usize>,
        dst_index: usize,
    },
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

/// Full document snapshot, used to catch up late joiners. `last_seq` is the
/// highest sequence number already baked into the tree so the joiner can set
/// its `last_applied_seq` watermark and ignore re-delivered live events.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FlexSnapshot {
    pub root: NodeConfig,
    pub bg_mode: BackgroundMode,
    pub art_style: ArtStyle,
    pub art_seed: u64,
    pub art_depth: u32,
    pub theme: Theme,
    pub palette: ColorPalette,
    pub last_seq: u32,
}

/// Display-only state shared between peers but never recorded. Sent on the
/// unreliable channel (except `PlayerInfo`, one-shot reliable on connect).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Ephemeral {
    /// Normalized screen-space cursor position `[nx, ny]` in `[0,1]`.
    CursorPos { nx: f32, ny: f32 },
    /// The node path this peer has selected (empty = root).
    Selection { path: Vec<usize> },
    /// Identity announcement (once per new peer).
    PlayerInfo { name: String, color: [u8; 3] },
}

/// Snapshot-handshake messages. Always reliable. `Snapshot` is boxed so the
/// `NetMsg`/`Control` enums stay pointer-sized even though a snapshot carries a
/// whole node tree.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Control {
    /// Late joiner → host: ask for the current document.
    RequestSnapshot,
    /// Joiner → host: ack the snapshot so the host can stop tracking it.
    SnapshotReceived,
    /// Host → joiner: the full document + watermark.
    Snapshot(Box<FlexSnapshot>),
}

// ── Wire protocol ────────────────────────────────────────────────────────────

/// Top-level wire envelope. The sub-enums encode the *intent* of a message —
/// document-mutating vs ephemeral vs control — so receivers can route each
/// category without an exhaustive top-level match.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum NetMsg {
    /// Unsequenced edit submission, sent guest→host. Never applied directly.
    Game(LayoutEdit),
    /// Canonical, host-sequenced edit, sent host→all. The only form applied.
    Sequenced { seq: u32, edit: LayoutEdit },
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
/// payload.
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
            LayoutEdit::ReplaceRoot(leaf.clone()),
            LayoutEdit::UpdateNode {
                path: vec![1, 0],
                node: leaf.clone(),
            },
            LayoutEdit::AddChild {
                parent_path: vec![0],
                child: leaf.clone(),
            },
            LayoutEdit::RemoveNode { path: vec![2] },
            LayoutEdit::MoveNode {
                src_path: vec![1],
                dst_parent: vec![0],
                dst_index: 3,
            },
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

    fn sample_messages() -> Vec<NetMsg> {
        let mut msgs = vec![
            NetMsg::Game(sample_edits()[0].clone()),
            NetMsg::Sequenced {
                seq: 7,
                edit: sample_edits()[1].clone(),
            },
            NetMsg::Ephemeral(Ephemeral::CursorPos { nx: 0.25, ny: 0.75 }),
            NetMsg::Ephemeral(Ephemeral::Selection { path: vec![0, 1] }),
            NetMsg::Ephemeral(Ephemeral::PlayerInfo {
                name: "Brave Otter".into(),
                color: [200, 100, 50],
            }),
        ];
        let snapshot = FlexSnapshot {
            root: NodeConfig::new_leaf("root", 800.0, 600.0),
            bg_mode: BackgroundMode::Pastel,
            art_style: ArtStyle::ExprTree,
            art_seed: 1,
            art_depth: 2,
            theme: Theme::Latte,
            palette: ColorPalette::Pastel1,
            last_seq: 42,
        };
        msgs.push(NetMsg::Control(Control::RequestSnapshot));
        msgs.push(NetMsg::Control(Control::SnapshotReceived));
        msgs.push(NetMsg::Control(Control::Snapshot(Box::new(snapshot))));
        msgs
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
    fn edits_round_trip() {
        for edit in sample_edits() {
            let msg = NetMsg::Game(edit.clone());
            let raw = enc_msg(&msg).unwrap();
            assert_eq!(decode(&raw).unwrap(), msg);
        }
    }

    #[test]
    fn snapshot_survives_boxing() {
        let snapshot = FlexSnapshot {
            root: NodeConfig::new_leaf("root", 800.0, 600.0),
            bg_mode: BackgroundMode::Pastel,
            art_style: ArtStyle::ExprTree,
            art_seed: 1,
            art_depth: 2,
            theme: Theme::Latte,
            palette: ColorPalette::Pastel1,
            last_seq: 42,
        };
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
