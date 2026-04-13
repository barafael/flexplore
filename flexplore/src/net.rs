//! Multiplayer networking — connects to a flexplore-server instance and
//! synchronises layout state via Lightyear.

use std::net::SocketAddr;
use std::time::Duration;

use bevy::prelude::*;
use lightyear::netcode::NetcodeClient;
use lightyear::prelude::client::NetcodeConfig;
use lightyear::prelude::client::*;
use lightyear::prelude::input::native::{ActionState, InputMarker};
use lightyear::prelude::*;
use lightyear::webtransport::prelude::client::WebTransportClientIo;

use flexplore_protocol::*;

use crate::history::UndoHistory;
use flexplore::config::FlexConfig;

// --- Resources ---

/// Connection parameters, set before the `NetPlugin` systems run.
#[derive(Resource, Clone)]
pub struct NetConfig {
    pub server_addr: SocketAddr,
    pub client_id: u64,
}

/// Tracks the last revision we applied so we don't re-apply unchanged state.
#[derive(Resource, Default)]
pub struct LastAppliedRevision(pub u64);

/// Accumulates edits made locally this frame; flushed to the network each tick.
#[derive(Resource, Default)]
pub struct PendingEdits(pub Vec<LayoutEdit>);

/// Other users' cursors, rebuilt each frame from replicated `PeerCursor` components.
#[derive(Resource, Default)]
pub struct RemoteCursors(pub Vec<RemoteCursorInfo>);

pub struct RemoteCursorInfo {
    pub name: String,
    pub selected: Vec<usize>,
    pub color_index: u8,
}

// --- Plugin ---

pub struct NetPlugin;

impl Plugin for NetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProtocolPlugin);
        app.add_plugins(ClientPlugins {
            tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
        });

        app.init_resource::<LastAppliedRevision>();
        app.init_resource::<PendingEdits>();
        app.init_resource::<RemoteCursors>();

        app.add_systems(Startup, connect_to_server);
        app.add_systems(
            FixedPreUpdate,
            flush_pending_edits
                .in_set(lightyear::prelude::client::input::InputSystems::WriteClientInputs),
        );
        app.add_systems(Update, (apply_server_state, collect_remote_cursors));
    }
}

/// Initiate connection to the server using the `NetConfig` resource.
fn connect_to_server(mut commands: Commands, net_cfg: Res<NetConfig>) {
    info!(
        "Connecting to flexplore server at {} as client {}",
        net_cfg.server_addr, net_cfg.client_id
    );

    let auth = Authentication::Manual {
        server_addr: net_cfg.server_addr,
        client_id: net_cfg.client_id,
        private_key: PRIVATE_KEY,
        protocol_id: PROTOCOL_ID,
    };

    let netcode = NetcodeClient::new(
        auth,
        NetcodeConfig {
            client_timeout_secs: 5,
            token_expire_secs: -1,
            ..default()
        },
    )
    .expect("Failed to create NetcodeClient");

    let entity = commands
        .spawn((
            Client::default(),
            netcode,
            PeerAddr(net_cfg.server_addr),
            WebTransportClientIo {
                certificate_digest: String::new(),
            },
            ReplicationReceiver::default(),
        ))
        .id();

    commands.trigger(Connect { entity });
}

/// Each fixed tick, bundle accumulated edits into a `LayoutInput` and send via ActionState.
fn flush_pending_edits(
    mut pending: ResMut<PendingEdits>,
    mut query: Query<&mut ActionState<LayoutInput>, With<InputMarker<LayoutInput>>>,
) {
    if pending.0.is_empty() {
        return;
    }

    let input = LayoutInput::from_edits(&pending.0);
    pending.0.clear();

    for mut action_state in query.iter_mut() {
        action_state.0 = input.clone();
    }
}

/// When the server's `SharedLayout` changes, apply it to the local `FlexConfig`.
fn apply_server_state(
    layouts: Query<&SharedLayout, Changed<SharedLayout>>,
    mut cfg: ResMut<FlexConfig>,
    mut history: ResMut<UndoHistory>,
    mut last_rev: ResMut<LastAppliedRevision>,
) {
    for shared in layouts.iter() {
        if shared.revision <= last_rev.0 {
            continue;
        }
        last_rev.0 = shared.revision;

        cfg.root = shared.root.clone();
        cfg.bg_mode = shared.bg_mode;
        cfg.art_style = shared.art_style;
        cfg.art_seed = shared.art_seed;
        cfg.art_depth = shared.art_depth;
        cfg.theme = shared.theme;
        cfg.palette = shared.palette;
        cfg.request_rebuild();

        history.push(cfg.clone());
    }
}

/// Collect all remote peer cursors into a resource for the UI to render.
fn collect_remote_cursors(cursors: Query<&PeerCursor>, mut remote: ResMut<RemoteCursors>) {
    remote.0.clear();
    for cursor in cursors.iter() {
        remote.0.push(RemoteCursorInfo {
            name: cursor.name.clone(),
            selected: cursor.selected.clone(),
            color_index: cursor.color_index,
        });
    }
}
