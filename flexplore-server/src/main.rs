//! Flexplore multiplayer server — authoritative layout state + edit relay.

use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;

use bevy::prelude::*;
use clap::Parser;
use lightyear::netcode::NetcodeServer;
use lightyear::prelude::input::native::ActionState;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use lightyear::webtransport::prelude::{Identity, server::WebTransportServerIo};

use flexplore_core::config::NodeConfig;
use flexplore_protocol::{
    self as proto, FIXED_TIMESTEP_HZ, LayoutEdit, LayoutInput, PRIVATE_KEY, PROTOCOL_ID,
    PeerCursor, ProtocolPlugin, REPLICATION_INTERVAL, SERVER_PORT, SharedLayout,
};

// --- CLI ---

#[derive(Parser, Debug)]
#[command(
    name = "flexplore-server",
    about = "Flexplore collaborative layout server"
)]
struct Args {
    /// Port to listen on.
    #[arg(short, long, default_value_t = SERVER_PORT)]
    port: u16,
}

// --- Resources ---

/// Server-side authoritative layout. Mirrors the replicated `SharedLayout` component.
#[derive(Resource)]
struct AuthoritativeLayout {
    layout: SharedLayout,
    /// Entity that holds the replicated `SharedLayout` component.
    entity: Option<Entity>,
}

/// Tracks connected peers and their assigned color indices.
#[derive(Resource, Default)]
struct PeerRegistry {
    /// (PeerId, cursor entity, session entity, color_index).
    peers: Vec<(lightyear::prelude::PeerId, Entity, Entity, u8)>,
    next_color: u8,
}

impl PeerRegistry {
    fn allocate_color(&mut self) -> u8 {
        let c = self.next_color;
        self.next_color = self.next_color.wrapping_add(1);
        c
    }

    fn remove_peer(&mut self, peer_id: lightyear::prelude::PeerId) {
        self.peers.retain(|(id, _, _, _)| *id != peer_id);
    }
}

// --- Plugin ---

fn main() {
    let args = Args::parse();

    let default_root = {
        let mut root = NodeConfig::new_container("root");
        root.min_height = flexplore_core::config::ValueConfig::Px(200.0);
        root.children = vec![
            NodeConfig::new_leaf("A", 80.0, 80.0),
            NodeConfig::new_leaf("B", 120.0, 100.0),
            NodeConfig::new_leaf("C", 60.0, 60.0),
            NodeConfig::new_leaf("D", 100.0, 80.0),
        ];
        root
    };

    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(bevy::log::LogPlugin::default())
        .add_plugins(ProtocolPlugin)
        .add_plugins(ServerPlugins {
            tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
        })
        .insert_resource(AuthoritativeLayout {
            layout: SharedLayout {
                root: default_root,
                bg_mode: flexplore_core::config::BackgroundMode::Pastel,
                art_style: flexplore_core::config::ArtStyle::ExprTree,
                art_seed: 137,
                art_depth: 5,
                theme: flexplore_core::config::Theme::Mocha,
                palette: flexplore_core::config::ColorPalette::Pastel1,
                revision: 0,
            },
            entity: None,
        })
        .init_resource::<PeerRegistry>()
        .add_systems(Startup, start_server(args.port))
        .add_systems(Startup, spawn_layout_entity)
        .add_systems(FixedUpdate, apply_client_edits)
        .add_observer(handle_new_client_link)
        .add_observer(handle_client_connected)
        .add_observer(handle_client_disconnected)
        .run();
}

fn start_server(port: u16) -> impl Fn(Commands) {
    move |mut commands: Commands| {
        let server_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), port);

        let certificate = Identity::self_signed(["localhost", "127.0.0.1", "::1"])
            .expect("Failed to generate self-signed certificate");

        let cert_hash = certificate.certificate_chain().as_slice()[0].hash();
        let hash_hex: String = cert_hash
            .as_ref()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        info!("Certificate hash: {hash_hex}");

        // Serve cert hash over HTTP for browser clients.
        let http_port = port + 1;
        let hash_for_http = hash_hex.clone();
        std::thread::spawn(move || {
            use std::io::{Read, Write};
            let Ok(listener) = std::net::TcpListener::bind(SocketAddr::new(
                Ipv4Addr::UNSPECIFIED.into(),
                http_port,
            )) else {
                error!("Failed to start cert hash HTTP server on port {http_port}");
                return;
            };
            info!("Cert hash available at http://0.0.0.0:{http_port}/");
            for mut stream in listener.incoming().flatten() {
                let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
                let mut buf = [0u8; 1024];
                let _ = stream.read(&mut buf);
                let body = &hash_for_http;
                let response = format!(
                    "HTTP/1.1 200 OK\r\n\
                     Access-Control-Allow-Origin: *\r\n\
                     Content-Type: text/plain\r\n\
                     Content-Length: {}\r\n\
                     \r\n\
                     {body}",
                    body.len(),
                );
                let _ = stream.write_all(response.as_bytes());
            }
        });

        let netcode = NetcodeServer::new(NetcodeConfig {
            protocol_id: PROTOCOL_ID,
            private_key: PRIVATE_KEY,
            ..Default::default()
        });

        let entity = commands
            .spawn((
                netcode,
                LocalAddr(server_addr),
                WebTransportServerIo { certificate },
            ))
            .id();

        commands.trigger(Start { entity });
        info!("Flexplore server starting on {server_addr} (WebTransport/QUIC)");
    }
}

/// Spawn the single replicated entity that carries the shared layout.
fn spawn_layout_entity(mut commands: Commands, mut auth: ResMut<AuthoritativeLayout>) {
    let entity = commands
        .spawn((
            auth.layout.clone(),
            Replicate::to_clients(NetworkTarget::All),
        ))
        .id();
    auth.entity = Some(entity);
    info!("Spawned replicated layout entity {entity:?}");
}

/// Attach a ReplicationSender when a new client link is created.
fn handle_new_client_link(trigger: On<Add, LinkOf>, mut commands: Commands) {
    info!("New client link {:?}", trigger.entity);
    commands.entity(trigger.entity).insert((
        ReplicationSender::new(REPLICATION_INTERVAL, SendUpdatesMode::SinceLastAck, false),
        Name::from("Client"),
    ));
}

/// Spawn a cursor entity and a session entity for the newly connected client.
/// The session entity has `ControlledBy` so Lightyear routes input to it.
fn handle_client_connected(
    trigger: On<Add, Connected>,
    query: Query<&RemoteId, With<ClientOf>>,
    mut registry: ResMut<PeerRegistry>,
    mut commands: Commands,
) {
    let Ok(client_id) = query.get(trigger.entity) else {
        return;
    };
    let peer_id = client_id.0;
    let color = registry.allocate_color();

    // Cursor entity: replicated to all clients for presence display.
    let cursor_entity = commands
        .spawn((
            proto::PeerId(peer_id),
            PeerCursor {
                selected: vec![],
                name: format!("User {}", peer_id.to_bits() % 1000),
                color_index: color,
            },
            Replicate::to_clients(NetworkTarget::All),
        ))
        .id();

    // Session entity: receives input from this client via ControlledBy.
    let session_entity = commands
        .spawn((
            proto::PeerId(peer_id),
            ControlledBy {
                owner: trigger.entity,
                lifetime: Default::default(),
            },
        ))
        .id();

    registry
        .peers
        .push((peer_id, cursor_entity, session_entity, color));
    info!("Client {peer_id:?} connected (color {color})");
}

/// Clean up when a client disconnects.
fn handle_client_disconnected(
    trigger: On<Remove, Connected>,
    query: Query<&RemoteId, With<ClientOf>>,
    mut registry: ResMut<PeerRegistry>,
    mut commands: Commands,
) {
    let Ok(client_id) = query.get(trigger.entity) else {
        return;
    };
    let peer_id = client_id.0;

    // Despawn cursor and session entities.
    if let Some((_, cursor_entity, session_entity, _)) =
        registry.peers.iter().find(|(id, _, _, _)| *id == peer_id)
    {
        commands.entity(*cursor_entity).despawn();
        commands.entity(*session_entity).despawn();
    }
    registry.remove_peer(peer_id);
    info!("Client {peer_id:?} disconnected");
}

/// Process edit inputs from all connected clients each server tick.
fn apply_client_edits(
    mut auth: ResMut<AuthoritativeLayout>,
    mut layout_q: Query<&mut SharedLayout>,
    mut cursor_q: Query<(&proto::PeerId, &mut PeerCursor)>,
    session_q: Query<(&proto::PeerId, &ActionState<LayoutInput>)>,
) {
    let mut dirty = false;

    for (peer, action_state) in session_q.iter() {
        let edits = action_state.0.decode_edits();
        if edits.is_empty() {
            continue;
        }
        let peer_id = peer.0;

        for edit in &edits {
            match edit {
                LayoutEdit::ReplaceRoot(new_root) => {
                    auth.layout.root = new_root.clone();
                    dirty = true;
                }
                LayoutEdit::UpdateNode { path, node } => {
                    if let Some(target) = auth.layout.root.get_mut(path) {
                        *target = node.clone();
                        dirty = true;
                    }
                }
                LayoutEdit::AddChild { parent_path, child } => {
                    if let Some(parent) = auth.layout.root.get_mut(parent_path) {
                        parent.children.push(child.clone());
                        dirty = true;
                    }
                }
                LayoutEdit::RemoveNode { path } => {
                    if path.is_empty() {
                        continue; // Can't remove root.
                    }
                    let parent_path = &path[..path.len() - 1];
                    let child_idx = path[path.len() - 1];
                    if let Some(parent) = auth.layout.root.get_mut(parent_path) {
                        if child_idx < parent.children.len() {
                            parent.children.remove(child_idx);
                            dirty = true;
                        }
                    }
                }
                LayoutEdit::MoveNode {
                    src_path,
                    dst_parent,
                    dst_index,
                } => {
                    if src_path.is_empty() {
                        continue;
                    }
                    let src_parent_path = &src_path[..src_path.len() - 1];
                    let src_idx = src_path[src_path.len() - 1];
                    let node = {
                        let Some(src_parent) = auth.layout.root.get_mut(src_parent_path) else {
                            continue;
                        };
                        if src_idx >= src_parent.children.len() {
                            continue;
                        }
                        src_parent.children.remove(src_idx)
                    };
                    if let Some(dst) = auth.layout.root.get_mut(dst_parent) {
                        let idx = (*dst_index).min(dst.children.len());
                        dst.children.insert(idx, node);
                        dirty = true;
                    }
                }
                LayoutEdit::UpdateSettings {
                    bg_mode,
                    art_style,
                    art_seed,
                    art_depth,
                    theme,
                    palette,
                } => {
                    auth.layout.bg_mode = *bg_mode;
                    auth.layout.art_style = *art_style;
                    auth.layout.art_seed = *art_seed;
                    auth.layout.art_depth = *art_depth;
                    auth.layout.theme = *theme;
                    auth.layout.palette = *palette;
                    dirty = true;
                }
                LayoutEdit::UpdateSelection { selected } => {
                    for (pid, mut cursor) in cursor_q.iter_mut() {
                        if pid.0 == peer_id {
                            cursor.selected = selected.clone();
                        }
                    }
                }
            }
        }
    }

    if dirty {
        auth.layout.revision += 1;
        if let Some(entity) = auth.entity {
            if let Ok(mut shared) = layout_q.get_mut(entity) {
                *shared = auth.layout.clone();
            }
        }
    }
}
