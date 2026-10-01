//! Wire protocol for flexplore multiplayer.
//!
//! The message types peers exchange and their postcard encoding, so the
//! transport (`flexplore-net`) and any future non-bevy consumer share one wire
//! format. It depends on `flexplore-core` for the document types only.
//!
//! Peer-to-peer over WebRTC, with one peer acting as **host**, the single
//! sequencing authority:
//!
//! 1. A guest submits its edit as [`NetMsg::Edit`], to the host only.
//! 2. The host assigns the next sequence number and rebroadcasts
//!    [`NetMsg::Sequenced`] to everyone.
//! 3. Every peer applies an edit only on `Sequenced` — the host included,
//!    which applies it at the moment it sequences it.
//!
//! Edits replace whole state ([`LayoutEdit`]), so applying one twice is
//! harmless: the peer that made an edit sees it come back sequenced and finds
//! nothing to change.

use flexplore_core::config::{ArtStyle, BackgroundMode, ColorPalette, NodeConfig, Theme};
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

/// A single document mutation. Both variants replace whole state.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum LayoutEdit {
    /// Replace the entire layout tree. Boxed so the wire enum stays small.
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

/// The whole document, as the host hands it to a peer that just greeted the
/// room. `last_seq` is the highest sequence number already folded in, so the
/// receiver can drop re-delivered live edits.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Document {
    pub root: NodeConfig,
    pub bg_mode: BackgroundMode,
    pub art_style: ArtStyle,
    pub art_seed: u64,
    pub art_depth: u32,
    pub theme: Theme,
    pub palette: ColorPalette,
    pub last_seq: Option<u32>,
}

/// Top-level wire envelope.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum NetMsg {
    /// Guest -> host: an unsequenced submission. Never applied directly.
    Edit(LayoutEdit),
    /// Host -> all: the canonical ordered form. The only one that is applied.
    Sequenced { seq: u32, edit: LayoutEdit },
    /// Introduce yourself on join.
    Hello { name: String },
    /// Host -> a peer that greeted: the document as it stands. Guests take
    /// it verbatim; the host is the only authority.
    Document(Box<Document>),
    /// Any -> all: where my pointer is, as fractions of the layout viewport's
    /// width and height (0..1, origin top-left). Windows differ in size, so
    /// each receiver scales the fractions into its own pixels.
    Cursor { pos: [f32; 2] },
    /// Any -> all: the node path I have selected (empty = root).
    Selection { path: Vec<usize> },
}

/// Encode a message for the wire. `None` if encoding fails or would produce
/// a zero-length payload: a WebRTC data channel can silently drop a zero-byte
/// payload, so an invisible packet must never go out.
pub fn encode(msg: &NetMsg) -> Option<Box<[u8]>> {
    match postcard::to_allocvec(msg) {
        Ok(v) if !v.is_empty() => Some(v.into_boxed_slice()),
        Ok(_) => {
            error!("postcard produced an empty NetMsg encoding; dropping");
            None
        }
        Err(error) => {
            error!(%error, "postcard encode failed");
            None
        }
    }
}

/// Decode a message from the wire. `None` on a corrupt or truncated payload.
pub fn decode(raw: &[u8]) -> Option<NetMsg> {
    postcard::from_bytes(raw)
        .inspect_err(|error| warn!(%error, "NetMsg decode failed"))
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_edits() -> Vec<LayoutEdit> {
        vec![
            LayoutEdit::ReplaceRoot(Box::new(NodeConfig::new_leaf("child", 100.0, 50.0))),
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

    fn sample_document() -> Document {
        Document {
            root: NodeConfig::new_leaf("root", 800.0, 600.0),
            bg_mode: BackgroundMode::Pastel,
            art_style: ArtStyle::ExprTree,
            art_seed: 1,
            art_depth: 2,
            theme: Theme::Latte,
            palette: ColorPalette::Pastel1,
            last_seq: Some(42),
        }
    }

    fn sample_messages() -> Vec<NetMsg> {
        vec![
            NetMsg::Edit(sample_edits()[0].clone()),
            NetMsg::Sequenced {
                seq: 7,
                edit: sample_edits()[1].clone(),
            },
            NetMsg::Hello {
                name: "Brave Otter".into(),
            },
            NetMsg::Document(Box::new(sample_document())),
            NetMsg::Cursor { pos: [0.25, 0.75] },
            NetMsg::Selection { path: vec![0, 1] },
        ]
    }

    #[test]
    fn every_variant_round_trips() {
        for msg in sample_messages() {
            let raw = encode(&msg).expect("encodes");
            assert!(!raw.is_empty(), "no zero-length payloads");
            assert_eq!(decode(&raw).expect("decodes"), msg);
        }
    }

    #[test]
    fn a_document_with_no_edits_yet_round_trips() {
        let doc = Document {
            last_seq: None,
            ..sample_document()
        };
        let msg = NetMsg::Document(Box::new(doc.clone()));
        match decode(&encode(&msg).unwrap()).unwrap() {
            NetMsg::Document(back) => assert_eq!(*back, doc),
            other => panic!("decoded to the wrong variant: {other:?}"),
        }
    }

    #[test]
    fn decoding_garbage_yields_none() {
        assert!(decode(&[0xff, 0xff, 0xff, 0xff]).is_none());
        assert!(decode(&[]).is_none());
    }
}
