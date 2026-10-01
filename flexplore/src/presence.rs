//! Remote-peer presence: who is in the room, where their pointer is, and
//! what they have selected. Filled by [`crate::net::pump`] from `Hello`,
//! `Cursor` and `Selection` messages; drawn by the app's cursor overlay.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use bevy::prelude::*;
use flexplore_net::PeerId;

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

/// Pick a colour deterministically from a peer id, so the same peer is the
/// same colour on every client.
pub fn color_for_peer(id: PeerId) -> [u8; 3] {
    let mut h = DefaultHasher::new();
    id.hash(&mut h);
    PEER_COLORS[(h.finish() as usize) % PEER_COLORS.len()]
}

/// All remote peers' presence state.
#[derive(Resource, Default)]
pub struct RemotePeers(pub Vec<RemotePeer>);

#[derive(Clone, Debug)]
pub struct RemotePeer {
    pub id: PeerId,
    /// The name from the peer's `Hello`; empty until it arrives.
    pub name: String,
    pub color: [u8; 3],
    /// Latest pointer position as fractions of the layout viewport.
    pub cursor: Option<Vec2>,
    pub cursor_prev: Option<Vec2>,
    /// Smoothed display position (fractions), persisted across frames.
    pub cursor_display: Option<Vec2>,
    /// `Time::elapsed_secs_f64` of the last cursor sample.
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
            self.0.last_mut().expect("just pushed")
        }
    }

    pub fn get(&self, id: PeerId) -> Option<&RemotePeer> {
        self.0.iter().find(|p| p.id == id)
    }

    /// Remove entries for peers no longer connected.
    pub fn prune(&mut self, connected: &[PeerId]) {
        self.0.retain(|p| connected.contains(&p.id));
    }
}
