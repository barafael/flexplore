//! P2P WebRTC networking for flexplore, ported from the omdurman/gnils approach.
//!
//! Architecture: *host-sequenced, whole-document edits with client-side
//! prediction*; see the `flexplore-proto` crate docs for the model. This crate
//! owns the matchbox socket, the per-peer [`NetState`], and the host key used
//! to agree on who sequences edits. The app-side systems live in the
//! `flexplore` crate's `net` module.
//!
//! The socket layer uses `matchbox_socket` directly (the bevy-agnostic core of
//! the matchbox stack) rather than `bevy_matchbox`, which keeps the bevy
//! version coupling in this workspace only. The native message-loop future is spawned on a dedicated tokio runtime (webrtc-rs
//! needs a real runtime, see [`spawn_message_loop`]).

use bevy::prelude::*;
use matchbox_socket::{MessageLoopFuture, RtcIceServerConfig, WebRtcSocket, WebRtcSocketBuilder};
use std::ops::{Deref, DerefMut};

// Re-export the wire protocol so consumers can depend on flexplore-net alone.
pub use flexplore_proto::{
    Control, Ephemeral, FlexSnapshot, LayoutEdit, NetMsg, PROTOCOL_VERSION, PeerBytes, decode,
    enc_msg,
};

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

// ── Net state ────────────────────────────────────────────────────────────────

/// How long a peer with no host waits before claiming hosting itself. Long
/// enough for the WebRTC handshake to an existing room to complete, so a
/// joiner does not mistake a slow connection for an empty room. Being alone
/// has no cost: edits are local-only until a peer connects.
pub const CLAIM_GRACE_SECS: f64 = 5.0;
/// How long peers wait after the host leaves before the most senior eligible
/// peer promotes itself (covers a brief reconnect blip of the host).
pub const PROMOTE_GRACE_SECS: f64 = 1.0;

/// The key peers compare to agree on a host: higher epoch wins, then the
/// earlier claim, then the lower peer id. Every peer evaluates the same
/// announced values, so the comparison is consistent everywhere.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct HostKey {
    pub epoch: u32,
    pub claimed_at: std::cmp::Reverse<u64>,
    pub id: std::cmp::Reverse<PeerId>,
}

impl HostKey {
    pub fn new(epoch: u32, claimed_at_ms: u64, id: PeerId) -> Self {
        Self {
            epoch,
            claimed_at: std::cmp::Reverse(claimed_at_ms),
            id: std::cmp::Reverse(id),
        }
    }

    pub fn host(&self) -> PeerId {
        self.id.0
    }

    pub fn claimed_at_ms(&self) -> u64 {
        self.claimed_at.0
    }
}

/// What we know about a connected peer from its `Hello`.
#[derive(Clone, Copy, Debug, Default)]
pub struct PeerInfo {
    pub client: u64,
    pub has_document: bool,
    pub protocol_ok: bool,
}

#[derive(Resource)]
pub struct NetState {
    /// Connected peers (excluding `my_id`).
    pub peers: Vec<PeerId>,
    pub my_id: Option<PeerId>,
    /// Random tag identifying this process's edits on the wire.
    pub client: u64,
    /// The host we currently follow, if any. `None` while joining, and for
    /// the grace period after the host leaves.
    pub host: Option<HostKey>,
    /// True once we hold the room's document (installed a snapshot, or we
    /// are/were host). Only such peers are eligible for promotion.
    pub has_document: bool,
    /// Per-peer `Hello` data.
    pub info: Vec<(PeerId, PeerInfo)>,
    /// Guest: a snapshot request is outstanding; retried until one lands.
    pub needs_snapshot: bool,
    pub snapshot_retry_timer: f64,
    /// Host-only: the next sequence number to assign in the current epoch.
    pub next_seq: u32,
    /// Highest sequence number applied locally in the current epoch.
    pub last_applied_seq: Option<u32>,
    /// Next `local_id` for our own edits.
    pub next_local_id: u32,
    /// Our own edits applied locally but not yet echoed back by the host, in
    /// submission order. Re-applied on top of every foreign edit (rebase) and
    /// resent when the host changes.
    pub pending_own: Vec<(u32, LayoutEdit)>,
    /// `Time::elapsed_secs_f64` at which we last had no host; drives the
    /// claim/promotion grace periods.
    pub unhosted_since: Option<f64>,
    /// Highest epoch seen from anyone, so a promotion always starts a newer one.
    pub max_epoch_seen: u32,
}

impl Default for NetState {
    fn default() -> Self {
        Self {
            peers: Vec::new(),
            my_id: None,
            client: rand::random::<u64>(),
            host: None,
            has_document: false,
            info: Vec::new(),
            needs_snapshot: false,
            snapshot_retry_timer: 0.0,
            next_seq: 0,
            last_applied_seq: None,
            next_local_id: 0,
            pending_own: Vec::new(),
            unhosted_since: None,
            max_epoch_seen: 0,
        }
    }
}

impl NetState {
    pub fn host_id(&self) -> Option<PeerId> {
        self.host.map(|k| k.host())
    }

    pub fn is_host(&self) -> bool {
        self.my_id.is_some() && self.host_id() == self.my_id
    }

    pub fn peer_info(&self, id: PeerId) -> Option<&PeerInfo> {
        self.info.iter().find(|(p, _)| *p == id).map(|(_, i)| i)
    }

    pub fn set_peer_info(&mut self, id: PeerId, info: PeerInfo) {
        match self.info.iter_mut().find(|(p, _)| *p == id) {
            Some(slot) => slot.1 = info,
            None => self.info.push((id, info)),
        }
    }

    /// The peer that should claim hosting while there is no host: the lowest
    /// id among peers holding the document, or among everyone if nobody does.
    pub fn election_winner(&self) -> Option<PeerId> {
        let my_id = self.my_id?;
        let eligible: Vec<PeerId> = self
            .peers
            .iter()
            .copied()
            .filter(|p| self.peer_info(*p).is_some_and(|i| i.has_document))
            .chain(self.has_document.then_some(my_id))
            .collect();
        if eligible.is_empty() {
            self.peers.iter().copied().chain(Some(my_id)).min()
        } else {
            eligible.into_iter().min()
        }
    }

    /// Take the next `local_id` for an own edit.
    pub fn next_local_id(&mut self) -> u32 {
        let id = self.next_local_id;
        self.next_local_id = self.next_local_id.wrapping_add(1);
        id
    }
}

/// Convert between matchbox peer ids and the transport-neutral bytes on the wire.
pub fn peer_to_bytes(id: PeerId) -> flexplore_proto::PeerBytes {
    *id.0.as_bytes()
}

pub fn peer_from_bytes(bytes: flexplore_proto::PeerBytes) -> PeerId {
    PeerId(uuid::Uuid::from_bytes(bytes))
}

/// Milliseconds since the UNIX epoch, used to order host claims across peers.
pub fn unix_ms() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now() as u64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

// ── Socket resource ──────────────────────────────────────────────────────────

/// A [`WebRtcSocket`] as a Bevy resource, built on `matchbox_socket` directly
/// (see the crate docs).
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
#[rustfmt::skip]
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
#[rustfmt::skip]
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
    use rand::seq::IndexedRandom;
    *slice
        .choose(&mut rand::rng())
        .expect("petname word lists are non-empty")
}

/// Generate a short hyphenated room id like `swift-otter`.
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
/// back a fresh petname if absent); on native, take `argv[1]`, minting a fresh
/// petname if absent so two unrelated users never share a room by default.
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
            .filter(|arg| !arg.is_empty())
            .unwrap_or_else(new_room_petname);
        info!(%room, "using room (pass a room name as the first argument to join one)");
        room
    }
}
