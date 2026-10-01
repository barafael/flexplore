//! Host-relay sequencing and the matchbox socket for networked flexplore.
//!
//! Peer-to-peer over WebRTC via `bevy_matchbox`; the signaling server is used
//! only for introductions. One peer acts as **host**, the single sequencing
//! authority (see the `flexplore-proto` crate docs for the message flow). The
//! host is the peer with the lexicographically smallest `PeerId`, recomputed
//! every frame so host loss self-heals. The wire types live in
//! `flexplore-proto` and are re-exported here.

use bevy::prelude::*;
use bevy_matchbox::prelude::*;

pub use bevy_matchbox::prelude::{MatchboxSocket, PeerId, PeerState};
pub use flexplore_proto::{Document, LayoutEdit, NetMsg, decode, encode};

/// Signaling server used to introduce peers, set at compile time with
/// `MATCHBOX_SERVER`.
///
/// The default is omdurman's shared deployment — its room namespace is not
/// ours, so point `MATCHBOX_SERVER` at your own `matchbox_server` for anything
/// beyond local testing. Only peer-introduction traffic crosses it; edits are
/// peer-to-peer.
pub const SIGNALING_SERVER: &str = match option_env!("MATCHBOX_SERVER") {
    Some(s) => s,
    None => "wss://omdurman-matchbox.fly.dev",
};

/// Reliable, ordered channel. Everything here mutates the document, and the
/// streams that do not — cursors and selections — are a few dozen bytes at
/// ~10 Hz, well within what ordering costs. One channel keeps the negotiation
/// simple.
pub const CH_RELIABLE: usize = 0;

/// Everything the front-end needs to know about the connection.
#[derive(Resource, Default)]
pub struct NetState {
    pub peers: Vec<PeerId>,
    pub my_id: Option<PeerId>,
    pub is_host: bool,
    /// Host-only: next sequence number to assign.
    pub next_seq: u32,
    /// Highest applied sequence number, for dropping duplicate deliveries.
    pub last_applied_seq: Option<u32>,
    /// This peer's display name, announced in its `Hello`.
    pub name: String,
    /// Peers we have already sent our [`NetMsg::Hello`] to. Without this a
    /// per-frame greet loop would flood the channel; disconnected peers may
    /// stay listed here — a reconnect arrives with a fresh id anyway.
    pub greeted: Vec<PeerId>,
    /// Whether this peer holds the room's document: it edited, applied or
    /// received one, or it was alone in the room long enough to own what it
    /// started with. A peer that just arrived holds nothing yet, so it takes
    /// the document a greeted peer answers with — even if its id made it the
    /// host — and never hands out the blank it started from.
    pub has_document: bool,
}

impl NetState {
    /// Drop everything the current room's socket established — peers, id,
    /// host flag, sequence counters — but keep the player's own name, which
    /// identifies the player rather than the session. Joining a new room must
    /// not arrive already believing this peer hosts it, or drop the new
    /// room's first edits as duplicates.
    pub fn leave_room(&mut self) {
        let name = std::mem::take(&mut self.name);
        *self = Self {
            name,
            ..Self::default()
        };
    }

    /// Am I the sequencing authority? True for the host, and for a solo peer
    /// so that a single user keeps editing before anyone else joins.
    pub fn sequences(&self) -> bool {
        self.is_host || self.peers.is_empty()
    }

    /// Has this sequence number already been applied? The reliable channel
    /// is ordered and `seq` is monotonic, so anything at or below the
    /// high-water mark is a duplicate.
    pub fn is_duplicate(&self, seq: u32) -> bool {
        self.last_applied_seq.is_some_and(|last| seq <= last)
    }

    /// The peer to submit edits to: the lowest id in the room, unless this
    /// peer sequences itself.
    pub fn host(&self) -> Option<PeerId> {
        if self.sequences() {
            return None;
        }
        self.peers.iter().copied().min_by_key(|p| p.to_string())
    }
}

/// Why a room name was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomIdError {
    Empty,
    TooLong {
        len: usize,
    },
    /// The offending character, so the message can name it rather than
    /// saying "invalid".
    BadChar(char),
}

impl core::fmt::Display for RoomIdError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RoomIdError::Empty => write!(f, "a room name cannot be empty"),
            RoomIdError::TooLong { len } => write!(
                f,
                "a room name may be at most {} characters, got {len}",
                RoomId::MAX_LEN
            ),
            RoomIdError::BadChar(c) => write!(
                f,
                "'{c}' is not allowed in a room name; use letters, digits, '-' or '_'"
            ),
        }
    }
}

impl core::error::Error for RoomIdError {}

/// The room to join. Peers sharing a room id find each other. It comes from
/// the URL (a share link, or a fresh generated room) or the command line.
#[derive(Resource, Clone, Debug, PartialEq, Eq)]
pub struct RoomId(pub String);

impl RoomId {
    /// Long enough for a descriptive name, short enough to type and to read
    /// back to someone over the phone.
    pub const MAX_LEN: usize = 40;

    /// Validate a room name handed over by a share link or the command line.
    ///
    /// The name is interpolated into the signaling URL's path, so it is
    /// restricted to characters that need no escaping: letters, digits, `-`
    /// and `_`. That rules out `/`, which would otherwise let a typo silently
    /// redirect the socket to a different path, and whitespace, which is
    /// invisible in a name two people are trying to match.
    ///
    /// ASCII-only, deliberately: the point of a room name is that two people
    /// can agree on it out of band and type it identically, and non-ASCII
    /// invites homoglyph and normalisation mismatches that look like the
    /// network being broken.
    pub fn parse(name: &str) -> Result<Self, RoomIdError> {
        if name.is_empty() {
            return Err(RoomIdError::Empty);
        }
        let len = name.chars().count();
        if len > Self::MAX_LEN {
            return Err(RoomIdError::TooLong { len });
        }
        if let Some(c) = name
            .chars()
            .find(|c| !(c.is_ascii_alphanumeric() || *c == '-' || *c == '_'))
        {
            return Err(RoomIdError::BadChar(c));
        }
        Ok(Self(name.to_string()))
    }

    /// A fresh room nobody else is in, named like `swift-otter`.
    pub fn fresh() -> Self {
        Self(format!("{}-{}", pick(PET_ADJECTIVES), pick(PET_NOUNS)))
    }
}

/// Where peers find each other: [`SIGNALING_SERVER`] unless something else is
/// inserted. A resource, so the multiplayer tests run the app's own
/// [`open_socket`] against an in-process server.
#[derive(Resource, Clone, Debug)]
pub struct Signaling(pub String);

impl Default for Signaling {
    fn default() -> Self {
        Self(SIGNALING_SERVER.to_string())
    }
}

/// Open the room's socket, unless one is already open.
///
/// The socket lives as long as the room. Only [`close_socket`] — a room
/// change — makes the next call open a fresh one, so peer ids stay what they
/// were and peers go on addressing the right one.
pub fn open_socket(
    mut commands: Commands,
    room: Res<RoomId>,
    signaling: Res<Signaling>,
    socket: Option<Res<MatchboxSocket>>,
) {
    if socket.is_some() {
        return;
    }
    let url = format!("{}/{}", signaling.0, room.0);
    info!(%url, "opening matchbox socket");
    commands.insert_resource(MatchboxSocket::from(
        WebRtcSocketBuilder::new(url)
            .reconnect_attempts(None)
            .add_reliable_channel(),
    ));
}

/// Leave the room: drop its socket and everything the socket established.
/// One step, so a new socket can never inherit the old room's peers, id or
/// host flag — the next [`open_socket`] starts from a clean [`NetState`].
pub fn close_socket(commands: &mut Commands, net: &mut NetState) {
    commands.remove_resource::<MatchboxSocket>();
    net.leave_room();
}

/// Send to one peer on the reliable channel.
pub fn send_to(socket: &mut MatchboxSocket, peer: PeerId, msg: &NetMsg) {
    broadcast(socket, &[peer], msg);
}

/// Send to every connected peer.
pub fn broadcast(socket: &mut MatchboxSocket, peers: &[PeerId], msg: &NetMsg) {
    let Some(bytes) = encode(msg) else {
        return;
    };
    for &peer in peers {
        if let Err(error) = socket
            .channel_mut(CH_RELIABLE)
            .try_send(bytes.clone(), peer)
        {
            warn!(%error, %peer, "send failed");
        }
    }
}

// ── Names and rooms ──────────────────────────────────────────────────────────

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

/// Generate a friendly two-word player display name like `Brave Otter`.
pub fn new_player_name() -> String {
    let cap = |w: &str| -> String {
        let mut c = w.chars();
        match c.next() {
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            None => String::new(),
        }
    };
    format!("{} {}", cap(pick(PET_ADJECTIVES)), cap(pick(PET_NOUNS)))
}

/// Resolve the room to join. On the web it is `room=` in the URL fragment
/// (`#room=name`, a share link); a bare page gets a fresh room written back
/// into the address bar so the link stays shareable. Natively it is the first
/// command line argument. An invalid name degrades to a fresh room rather
/// than an error, since a bad share link should still land somewhere.
pub fn room_id() -> RoomId {
    #[cfg(target_arch = "wasm32")]
    {
        let fragment = read_fragment();
        if let Some(name) = fragment_param(&fragment, "room") {
            match RoomId::parse(&name) {
                Ok(room) => return room,
                Err(error) => warn!(%error, "ignoring the room in the URL"),
            }
        }

        let room = RoomId::fresh();
        write_fragment(&with_fragment_param(&fragment, "room", Some(&room.0)));
        room
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let room = match std::env::args().nth(1) {
            Some(name) => RoomId::parse(&name).unwrap_or_else(|error| {
                warn!(%error, "ignoring the room on the command line");
                RoomId::fresh()
            }),
            None => RoomId::fresh(),
        };
        info!(room = %room.0, "using room (pass a room name as the first argument to join one)");
        room
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_round_trips() {
        let msg = NetMsg::Sequenced {
            seq: 7,
            edit: LayoutEdit::UpdateSettings {
                bg_mode: flexplore_core::config::BackgroundMode::Pastel,
                art_style: flexplore_core::config::ArtStyle::Voronoi,
                art_seed: 9,
                art_depth: 2,
                theme: flexplore_core::config::Theme::Mocha,
                palette: flexplore_core::config::ColorPalette::Dark2,
            },
        };
        let bytes = encode(&msg).expect("encodes");
        assert_eq!(decode(&bytes).expect("decodes"), msg);
    }

    #[test]
    fn duplicate_sequence_numbers_are_recognised() {
        let mut net = NetState::default();
        assert!(!net.is_duplicate(0), "nothing applied yet");
        net.last_applied_seq = Some(3);
        assert!(net.is_duplicate(3), "the same seq is a duplicate");
        assert!(net.is_duplicate(2), "an older seq is a duplicate");
        assert!(!net.is_duplicate(4), "the next seq is new");
    }

    /// Only the no-peer arms; the `peers` branch needs a real `PeerId`. The
    /// guest-with-peers path is covered end-to-end by the multiplayer
    /// integration test in the `flexplore` crate.
    #[test]
    fn a_solo_peer_sequences_whether_or_not_it_is_host() {
        let mut net = NetState::default();
        assert!(net.sequences(), "a lone guest must keep editing");
        assert!(net.host().is_none(), "and has nobody to submit to");
        net.is_host = true;
        assert!(net.sequences(), "and so must a lone host");
    }

    /// Leaving the room must forget everything the old room's socket
    /// established but the player's own name, which belongs to the player.
    #[test]
    fn leaving_a_room_forgets_everything_but_the_name() {
        let mut net = NetState {
            is_host: true,
            next_seq: 12,
            last_applied_seq: Some(11),
            name: "ada".into(),
            has_document: true,
            ..NetState::default()
        };

        net.leave_room();

        assert_eq!(net.name, "ada", "the player's name is not per-room");
        assert!(!net.is_host, "host status belongs to the old room");
        assert!(net.peers.is_empty());
        assert_eq!(net.my_id, None, "the id came from the old socket");
        assert_eq!(net.next_seq, 0);
        assert_eq!(net.last_applied_seq, None, "or the first edit looks stale");
        assert!(net.greeted.is_empty());
        assert!(!net.has_document, "the document belonged to the old room");
    }

    #[test]
    fn generated_names_and_rooms_are_well_formed() {
        let name = new_player_name();
        assert_eq!(name.split(' ').count(), 2, "{name}");
        assert!(RoomId::parse(&RoomId::fresh().0).is_ok());
    }
}

#[cfg(test)]
mod room_tests {
    use super::*;

    #[test]
    fn ordinary_names_are_accepted() {
        for name in ["a", "game", "Room_7", "my-game-2", "ABC123"] {
            assert!(RoomId::parse(name).is_ok(), "{name} should be accepted");
        }
    }

    #[test]
    fn an_empty_name_is_refused() {
        assert_eq!(RoomId::parse(""), Err(RoomIdError::Empty));
    }

    #[test]
    fn an_overlong_name_is_refused() {
        let long = "a".repeat(RoomId::MAX_LEN + 1);
        assert_eq!(
            RoomId::parse(&long),
            Err(RoomIdError::TooLong {
                len: RoomId::MAX_LEN + 1
            })
        );
        assert!(RoomId::parse(&"a".repeat(RoomId::MAX_LEN)).is_ok());
    }

    /// A slash would redirect the socket to a different URL path, and
    /// whitespace is invisible in a name two people are trying to match.
    #[test]
    fn characters_that_would_change_the_url_are_refused() {
        for (name, bad) in [
            ("a/b", '/'),
            ("a b", ' '),
            ("a?b", '?'),
            ("a#b", '#'),
            ("a:b", ':'),
            ("a.b", '.'),
            ("../evil", '.'),
        ] {
            assert_eq!(
                RoomId::parse(name),
                Err(RoomIdError::BadChar(bad)),
                "{name} must be refused"
            );
        }
    }

    #[test]
    fn non_ascii_is_refused() {
        assert_eq!(RoomId::parse("café"), Err(RoomIdError::BadChar('é')));
        assert_eq!(RoomId::parse("рум"), Err(RoomIdError::BadChar('р')));
    }

    #[test]
    fn every_error_explains_itself() {
        assert!(RoomIdError::Empty.to_string().contains("empty"));
        let long = RoomIdError::TooLong { len: 99 }.to_string();
        assert!(long.contains("99"), "{long}");
        assert!(long.contains(&RoomId::MAX_LEN.to_string()), "{long}");
        let bad = RoomIdError::BadChar('/').to_string();
        assert!(bad.contains('/'), "must name the character: {bad}");
    }
}

// ── URL fragment helpers ─────────────────────────────────────────────────────
//
// The room travels in the URL *fragment* (`#room=name`), not a query
// parameter: the browser never sends the fragment to the server, so a room
// name cannot land in access logs or referrers, changing it does not reload
// the page, and caches see one URL for the app. The fragment is a `&`-separated
// list of `key=value` pairs so other state can share it.

/// The value of `key` in a fragment, given with or without its leading `#`.
#[cfg(any(target_arch = "wasm32", test))]
pub fn fragment_param(fragment: &str, key: &str) -> Option<String> {
    let fragment = fragment.strip_prefix('#').unwrap_or(fragment);
    fragment
        .split(['&', ';'])
        .find_map(|pair| pair.strip_prefix(key)?.strip_prefix('='))
        .map(str::to_string)
}

/// `fragment` with `key` set to `value` (or removed when `None`), every other
/// pair kept in place. Returned without the leading `#`.
#[cfg(any(target_arch = "wasm32", test))]
pub fn with_fragment_param(fragment: &str, key: &str, value: Option<&str>) -> String {
    let fragment = fragment.strip_prefix('#').unwrap_or(fragment);
    let mut pairs: Vec<String> = fragment
        .split(['&', ';'])
        .filter(|pair| !pair.is_empty() && pair.split('=').next() != Some(key))
        .map(str::to_string)
        .collect();
    if let Some(value) = value {
        pairs.push(format!("{key}={value}"));
    }
    pairs.join("&")
}

/// The page's current fragment (with its leading `#`, or empty).
#[cfg(target_arch = "wasm32")]
pub fn read_fragment() -> String {
    web_sys::window()
        .and_then(|w| w.location().hash().ok())
        .unwrap_or_default()
}

/// Replace the page's fragment in place: no navigation, no history entry,
/// so Back does not lead to a bare page that redirects forward again.
#[cfg(target_arch = "wasm32")]
pub fn write_fragment(fragment: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let url = format!("#{fragment}");
    match window.history() {
        Ok(history) => {
            let _ = history.replace_state_with_url(
                &web_sys::wasm_bindgen::JsValue::NULL,
                "",
                Some(&url),
            );
        }
        Err(_) => {
            let _ = window.location().set_hash(fragment);
        }
    }
}

#[cfg(test)]
mod fragment_tests {
    use super::*;

    #[test]
    fn parses_a_share_link_fragment_with_leading_hash() {
        assert_eq!(fragment_param("#room=abc", "room").as_deref(), Some("abc"));
    }

    #[test]
    fn tolerates_a_fragment_without_the_hash() {
        assert_eq!(fragment_param("room=abc", "room").as_deref(), Some("abc"));
    }

    #[test]
    fn finds_the_key_among_extra_params() {
        assert_eq!(
            fragment_param("#x=1&room=q-7&y=2", "room").as_deref(),
            Some("q-7")
        );
        assert_eq!(fragment_param("#rooms=1", "room"), None);
        assert_eq!(fragment_param("", "room"), None);
    }

    #[test]
    fn values_keep_their_own_equals_signs() {
        assert_eq!(
            fragment_param("#layout=YWJj==&room=r", "layout").as_deref(),
            Some("YWJj==")
        );
    }

    #[test]
    fn setting_a_param_keeps_the_others() {
        assert_eq!(
            with_fragment_param("#layout=abc", "room", Some("r1")),
            "layout=abc&room=r1"
        );
        assert_eq!(
            with_fragment_param("#room=old&layout=abc", "room", Some("new")),
            "layout=abc&room=new"
        );
        assert_eq!(
            with_fragment_param("#room=r&layout=abc", "layout", None),
            "room=r"
        );
        assert_eq!(with_fragment_param("", "room", Some("r")), "room=r");
        assert_eq!(with_fragment_param("#room=r", "room", None), "");
    }
}
