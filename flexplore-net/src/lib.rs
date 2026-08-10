//! P2P WebRTC networking for flexplore, ported from the omdurman/gnils approach.
//!
//! Architecture: *host-sequenced event sourcing*. Every peer holds the full
//! layout state. A non-host submits its edit as [`NetMsg::Game`] to the host
//! only; the host assigns the next canonical sequence number and rebroadcasts
//! it as [`NetMsg::Sequenced`] to every peer (including looping it back to
//! itself). Every peer — originator included — applies an edit only when it
//! arrives as `Sequenced`, so all peers observe one canonical, ordered stream.
//!
//! The host is the lowest-sorted `PeerId` in the room, re-derived on every
//! peer change, so when the host disconnects the next-lowest `PeerId` is
//! promoted automatically and resumes sequencing at `last_applied_seq + 1`.
//!
//! The socket layer uses `matchbox_socket` directly (the bevy-agnostic core of
//! the matchbox stack), so this crate works with bevy 0.18 even though the
//! `bevy_matchbox` integration in the fork targets bevy 0.19. The native
//! message-loop future is spawned on a dedicated tokio runtime (webrtc-rs
//! needs a real runtime, see [`spawn_message_loop`]).

use bevy::prelude::*;
use flexplore_core::config::{
    ArtStyle, BackgroundMode, ColorPalette, NodeConfig, Theme,
};
use matchbox_socket::{MessageLoopFuture, RtcIceServerConfig, WebRtcSocket, WebRtcSocketBuilder};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

/// Signaling server URL. Overridable at build time via `MATCHBOX_SERVER`.
pub const SIGNALING_SERVER: &str = if let Some(s) = option_env!("MATCHBOX_SERVER") {
    s
} else {
    "wss://omdurman-matchbox.fly.dev"
};

/// Reliable, ordered channel: layout edits, `Sequenced` echoes, snapshots.
pub const CH_RELIABLE: usize = 0;
/// Unreliable channel: ephemeral display state (cursors, selection, identity).
pub const CH_UNRELIABLE: usize = 1;

pub use matchbox_socket::{ChannelConfig, PeerId, PeerState};

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
#[derive(Serialize, Deserialize, Clone, Debug)]
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
#[derive(Serialize, Deserialize, Clone, Debug)]
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
#[derive(Serialize, Deserialize, Clone, Debug)]
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
#[derive(Serialize, Deserialize, Clone, Debug)]
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

pub fn decode(raw: &[u8]) -> Option<NetMsg> {
    postcard::from_bytes(raw)
        .inspect_err(|e| warn!("matchbox decode error: {e}"))
        .ok()
}

// ── Net state ────────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct NetState {
    /// Connected peers (excluding `my_id`).
    pub peers: Vec<PeerId>,
    pub my_id: Option<PeerId>,
    pub is_host: bool,
    /// Peers we've sent a snapshot to and are waiting for an ack from.
    pub snapshot_pending: Vec<PeerId>,
    /// True until the first snapshot arrives (guest) / always false for host.
    pub needs_snapshot: bool,
    pub snapshot_retry_timer: f64,
    /// Set after the first snapshot is installed; prevents double-install.
    pub snapshot_applied: bool,
    /// Host-only: the next canonical sequence number to assign.
    pub next_seq: u32,
    /// Highest sequence number applied locally; drops duplicate delivery.
    pub last_applied_seq: Option<u32>,
    /// All peers (including `my_id`) in canonical sorted order.
    sorted_all: Vec<PeerId>,
}

impl NetState {
    /// Rebuild `sorted_all` from `peers` + `my_id`. Call after any mutation.
    pub fn refresh_sorted(&mut self) {
        self.sorted_all.clear();
        self.sorted_all.extend(self.peers.iter().copied());
        if let Some(me) = self.my_id {
            self.sorted_all.push(me);
        }
        self.sorted_all.sort();
    }

    /// Canonical sorted list of all peers including the local player.
    pub fn sorted_all(&self) -> &[PeerId] {
        &self.sorted_all
    }

    /// The canonical host: the lowest-sorted peer id across everyone.
    /// Re-derived on every peer change, so a guest is promoted automatically
    /// when the previous host disconnects.
    pub fn host_id(&self) -> Option<PeerId> {
        self.sorted_all.first().copied()
    }
}

#[derive(Resource)]
pub struct RoomId(pub String);

impl RoomId {
    pub fn new(s: String) -> Self {
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// ── Socket resource ──────────────────────────────────────────────────────────

/// A [`WebRtcSocket`] as a Bevy resource. Implemented against
/// `matchbox_socket` directly so we don't depend on the fork's bevy 0.19
/// integration (flexplore is on bevy 0.18).
#[derive(Resource)]
pub struct MatchboxSocket(WebRtcSocket);

impl Deref for MatchboxSocket {
    type Target = WebRtcSocket;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MatchboxSocket {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<WebRtcSocketBuilder> for MatchboxSocket {
    fn from(builder: WebRtcSocketBuilder) -> Self {
        Self::from(builder.build())
    }
}

impl From<(WebRtcSocket, MessageLoopFuture)> for MatchboxSocket {
    fn from((socket, message_loop_fut): (WebRtcSocket, MessageLoopFuture)) -> Self {
        spawn_message_loop(message_loop_fut);
        MatchboxSocket(socket)
    }
}

/// Spawn the matchbox message-loop future so it keeps running for the lifetime
/// of the socket.
///
/// On native, `webrtc-rs` (used by `matchbox_socket`) needs a live multi-
/// threaded tokio runtime; polling from Bevy's `IoTaskPool` lets ICE connect
/// but the DTLS/SCTP handshake never completes, so data channels never open.
/// Spawning on a real tokio runtime fixes this.
///
/// On WASM there is no tokio (the browser provides WebRTC), so we fall back to
/// `IoTaskPool::spawn(..).detach()`.
#[cfg(not(target_arch = "wasm32"))]
fn spawn_message_loop(fut: MessageLoopFuture) {
    use std::sync::OnceLock;
    use tokio::runtime::Runtime;
    use tokio::task::JoinHandle;

    static MATCHBOX_RUNTIME: OnceLock<Runtime> = OnceLock::new();

    let runtime = MATCHBOX_RUNTIME.get_or_init(|| {
        // webrtc-rs uses rustls for DTLS; rustls 0.23 needs a process-level
        // CryptoProvider installed before any config is built.
        let _ = rustls::crypto::ring::default_provider().install_default();
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("matchbox")
            .build()
            .expect("failed to build matchbox tokio runtime")
    });

    let _handle: JoinHandle<()> = runtime.spawn(async move {
        let _ = fut.await;
    });
}

#[cfg(target_arch = "wasm32")]
fn spawn_message_loop(fut: MessageLoopFuture) {
    use bevy::tasks::IoTaskPool;
    IoTaskPool::get().spawn(fut).detach();
}

/// Build a [`MatchboxSocket`] for the given room. Keeps ICE config and channel
/// layout in one place. `?next=20` lets up to 20 peers join the full-mesh room.
pub fn build_socket(room: &str) -> MatchboxSocket {
    let url = format!("{SIGNALING_SERVER}/{room}?next=20");
    info!(%room, %url, "opening matchbox socket");

    let ice_config = RtcIceServerConfig {
        urls: vec![
            "stun:stun.l.google.com:19302".to_string(),
            "stun:stun1.l.google.com:19302".to_string(),
        ],
        username: None,
        credential: None,
    };

    let builder = WebRtcSocketBuilder::new(&url)
        .ice_server(ice_config)
        .reconnect_attempts(None) // unlimited reconnection attempts
        .add_reliable_channel() // channel 0: edits, sequenced echoes, snapshots
        .add_unreliable_channel(); // channel 1: cursors, selection, identity

    MatchboxSocket::from(builder)
}

/// Open the socket + register the `RoomId` resource. Runs in `Startup`.
pub fn open_socket(mut commands: Commands, room: Res<RoomId>) {
    commands.insert_resource(build_socket(&room.0));
}

/// Broadcast an ephemeral message to every peer on the unreliable channel.
/// Send failures are silently dropped — the next sample supersedes.
pub fn broadcast_unreliable(socket: &mut MatchboxSocket, peers: &[PeerId], msg: &NetMsg) {
    if peers.is_empty() {
        return;
    }
    let Some(encoded) = enc_msg(msg) else {
        return;
    };
    let channel = socket.channel_mut(CH_UNRELIABLE);
    for &peer in peers {
        let _ = channel.try_send(encoded.clone(), peer);
    }
}

// ── Room-id minting ──────────────────────────────────────────────────────────

/// Adjectives for friendly room names.
const PET_ADJECTIVES: &[&str] = &[
    "ancient", "amber", "azure", "bold", "brave", "bright", "brisk", "bronze", "calm", "clever",
    "copper", "coral", "crimson", "crystal", "daring", "dawn", "dusty", "eager", "ember", "fierce",
    "frosty", "gentle", "gilded", "golden", "grand", "happy", "hidden", "ivory", "jade", "jolly",
    "keen", "lively", "lucky", "merry", "misty", "noble", "nimble", "onyx", "pearl", "proud",
    "quiet", "quick", "radiant", "rapid", "rosy", "royal", "ruby", "rustic", "shy", "silent",
    "silver", "sleepy", "smoky", "solemn", "sparkling", "stormy", "sunny", "swift", "tame", "tawny",
    "tender", "tiny", "twilight", "valiant", "velvet", "violet", "vivid", "wandering", "wild",
    "windy", "winter", "wise", "woven", "young", "zealous",
];

/// Nouns for friendly room names.
const PET_NOUNS: &[&str] = &[
    "albatross", "badger", "bear", "bison", "boar", "buffalo", "camel", "caribou", "cheetah",
    "cobra", "condor", "cougar", "coyote", "crane", "crow", "deer", "dingo", "dolphin", "dove",
    "eagle", "elk", "falcon", "ferret", "finch", "flamingo", "fox", "gazelle", "gecko", "goose",
    "hare", "hawk", "hedgehog", "heron", "horse", "hyena", "ibex", "jackal", "jaguar", "kestrel",
    "koala", "lemur", "leopard", "lion", "lizard", "llama", "lynx", "magpie", "marten", "meerkat",
    "mongoose", "moose", "narwhal", "newt", "ocelot", "orca", "osprey", "otter", "owl", "panda",
    "panther", "pelican", "penguin", "pony", "puffin", "puma", "quail", "rabbit", "raccoon",
    "raven", "reindeer", "salmon", "seal", "serval", "shark", "sloth", "sparrow", "stoat", "stork",
    "swan", "tapir", "tiger", "toucan", "turtle", "vulture", "walrus", "weasel", "wolf",
    "wolverine", "wombat", "yak", "zebra",
];

fn pick<T: Copy>(slice: &[T]) -> T {
    slice[rand::random::<usize>() % slice.len()]
}

/// Generate a short hyphenated room id like `swift-otter`.
#[cfg(target_arch = "wasm32")]
fn new_room_petname() -> String {
    format!("{}-{}", pick(PET_ADJECTIVES), pick(PET_NOUNS))
}

/// Generate a friendly two-word player display name like `Brave Otter`.
pub fn new_player_name() -> String {
    let adj: &str = pick(PET_ADJECTIVES);
    let noun: &str = pick(PET_NOUNS);
    let cap = |w: &str| -> String {
        let mut c = w.chars();
        match c.next() {
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            None => String::new(),
        }
    };
    format!("{} {}", cap(adj), cap(noun))
}

/// Resolve the room id: on WASM, read `?room=` from the URL (minting + writing
/// back a fresh petname if absent); on native, take `argv[1]` (default
/// `dev-room`).
pub fn room_id() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsValue;
        let win = web_sys::window().expect("window always available on wasm");
        let href = win.location().href().ok().unwrap_or_default();

        if let Ok(url) = web_sys::Url::new(&href) {
            if let Some(id) = url.search_params().get("room") {
                if !id.is_empty() {
                    return id;
                }
            }
        }

        let new_id = new_room_petname();

        if let Ok(url) = web_sys::Url::new(&href) {
            url.search_params().set("room", &new_id);
            if let Ok(history) = win.history() {
                let _ = history.replace_state_with_url(&JsValue::NULL, "", Some(&url.href()));
            }
        }

        new_id
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let room = std::env::args()
            .nth(1)
            .unwrap_or_else(|| "dev-room".to_string());
        info!(%room, "using room");
        room
    }
}
