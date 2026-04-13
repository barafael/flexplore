//! Flexplore multiplayer protocol — shared types between client and server.

use bevy::ecs::entity::MapEntities;
use bevy::prelude::*;
use lightyear::prelude::input::native::InputPlugin;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

// Re-export core config types used by both sides.
pub use flexplore_core::config::{ArtStyle, BackgroundMode, ColorPalette, NodeConfig, Theme};

// --- Constants ---

pub const FIXED_TIMESTEP_HZ: f64 = 10.0;
pub const REPLICATION_INTERVAL: Duration = Duration::from_millis(200);
pub const PROTOCOL_ID: u64 = 0xF1_E300;
pub const PRIVATE_KEY: [u8; 32] = [0u8; 32];
pub const SERVER_PORT: u16 = 5870;

// --- Replicated components ---

/// The authoritative layout state, replicated from server to all clients.
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SharedLayout {
    pub root: NodeConfig,
    pub bg_mode: BackgroundMode,
    pub art_style: ArtStyle,
    pub art_seed: u64,
    pub art_depth: u32,
    pub theme: Theme,
    pub palette: ColorPalette,
    /// Monotonically increasing revision counter so clients know when to rebuild.
    pub revision: u64,
}

/// Identifies which peer owns a cursor entity.
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PeerId(pub lightyear::prelude::PeerId);

/// A remote user's cursor / selection state, replicated to all clients.
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PeerCursor {
    /// The node path this peer has selected (empty = root).
    pub selected: Vec<usize>,
    /// Display name for this peer.
    pub name: String,
    /// Assigned color index (0..N) for visual distinction.
    pub color_index: u8,
}

// --- Edit operations (sent as client input) ---

/// A single edit operation sent from a client to the server.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum LayoutEdit {
    /// Replace the entire layout tree (e.g. loading a template or import).
    ReplaceRoot(NodeConfig),
    /// Replace a single node at the given path.
    UpdateNode { path: Vec<usize>, node: NodeConfig },
    /// Add a child node at the given parent path.
    AddChild {
        parent_path: Vec<usize>,
        child: NodeConfig,
    },
    /// Remove the node at the given path.
    RemoveNode { path: Vec<usize> },
    /// Move a node from `src` to `dst_parent` at `dst_index`.
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
    /// Update this client's selection path (cursor broadcast).
    UpdateSelection { selected: Vec<usize> },
}

/// Wrapper input type for Lightyear's input system.
/// Edits are serialized as JSON bytes because `LayoutEdit` contains types
/// that don't implement `Reflect` (e.g. `NodeConfig`).
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, Reflect)]
pub struct LayoutInput {
    /// JSON-serialized `Vec<LayoutEdit>`.
    pub edits_json: Vec<u8>,
}

impl LayoutInput {
    pub fn from_edits(edits: &[LayoutEdit]) -> Self {
        Self {
            edits_json: serde_json::to_vec(edits).unwrap_or_default(),
        }
    }

    pub fn decode_edits(&self) -> Vec<LayoutEdit> {
        if self.edits_json.is_empty() {
            return vec![];
        }
        serde_json::from_slice(&self.edits_json).unwrap_or_default()
    }
}

impl MapEntities for LayoutInput {
    fn map_entities<M: EntityMapper>(&mut self, _entity_mapper: &mut M) {}
}

// --- Protocol Plugin ---

/// Registers protocol types (inputs and replicated components).
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputPlugin::<LayoutInput>::default());

        app.register_component::<SharedLayout>();
        app.register_component::<PeerId>();
        app.register_component::<PeerCursor>();
    }
}
