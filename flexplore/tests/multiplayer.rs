//! Live multiplayer: two and three real instances on one room.
//!
//! Each instance is its own headless Bevy app running the real networking
//! systems (`elect_host`, `pump`), each with its own real `MatchboxSocket`.
//! The apps are introduced by an in-process full-mesh signaling server and
//! then talk peer-to-peer over an actual WebRTC data channel, so the whole
//! shared-document contract crosses a real wire: host election, greetings,
//! the document handed to a late joiner, and edits sequenced by the host.
//!
//! There is no window and no renderer: `MinimalPlugins` only. An instance is
//! driven the way the editor drives the network: the document is mutated
//! locally and the edit dropped into the outbox.
//!
//! These wait for real peer handshakes, so they are `#[ignore]`d and opt in:
//!
//! ```sh
//! cargo test -p flexplore --test multiplayer -- --ignored --nocapture
//! ```

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use bevy::prelude::*;
use flexplore::config::{FlexConfig, HoverPreview, NodeConfig};
use flexplore::history::UndoHistory;
use flexplore::net::{NetPlugin, PendingEdits};
use flexplore_net::{LayoutEdit, NetState, RoomId, Signaling};

/// A local full-mesh signaling server, on a free loopback port. Runs on its
/// own tokio runtime in a background thread, exactly like `matchbox_server`;
/// the port is discovered by probing before the thread takes it over.
fn start_signaling_server() -> u16 {
    let probe = std::net::TcpListener::bind("127.0.0.1:0").expect("a free loopback port");
    let port = probe.local_addr().expect("probe address").port();
    drop(probe);

    std::thread::Builder::new()
        .name("signaling-server".into())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("a tokio runtime for the signaling server");
            let server =
                matchbox_signaling::SignalingServer::full_mesh_builder(([127, 0, 0, 1], port))
                    .build();
            runtime
                .block_on(server.serve())
                .expect("the signaling server ran");
        })
        .expect("spawn the signaling server thread");

    port
}

/// How long to wait for peers to find each other, and for an edit to settle.
const CONNECT: Duration = Duration::from_secs(45);
const SETTLE: Duration = Duration::from_secs(30);

static ROOM_SEQ: AtomicU64 = AtomicU64::new(0);

/// A room unique to this run, so a leftover peer from a previous scenario can
/// never leak into the next.
fn fresh_room(label: &str) -> RoomId {
    let n = ROOM_SEQ.fetch_add(1, Ordering::Relaxed);
    RoomId::parse(&format!("mp-{label}-{n}-{:x}", std::process::id()))
        .expect("a generated room name is valid")
}

/// One headless instance: a real app, a real socket, the app's own systems.
fn instance(name: &str, room: &RoomId, port: u16) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<FlexConfig>()
        .init_resource::<HoverPreview>()
        .insert_resource(UndoHistory::new(FlexConfig::default()))
        .insert_resource(room.clone())
        .insert_resource(Signaling(format!("ws://127.0.0.1:{port}")))
        .add_plugins(NetPlugin);
    app.world_mut().resource_mut::<NetState>().name = name.into();
    app
}

fn net(app: &App) -> &NetState {
    app.world().resource::<NetState>()
}

fn root(app: &App) -> &NodeConfig {
    &app.world().resource::<FlexConfig>().root
}

/// Tick every app until `done` holds, or fail with `what` after `timeout`.
fn wait_for(apps: &mut [App], timeout: Duration, what: &str, done: impl Fn(&[App]) -> bool) {
    let start = Instant::now();
    loop {
        for app in apps.iter_mut() {
            app.update();
        }
        if done(apps) {
            return;
        }
        assert!(start.elapsed() < timeout, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Start a signaling server and put one instance per name into a fresh room,
/// waiting until each sees all the others and exactly one is the host.
fn connect(label: &str, names: &[&str]) -> (Vec<App>, RoomId, u16) {
    let port = start_signaling_server();
    let room = fresh_room(label);
    println!(
        "[server] full-mesh signaling on ws://127.0.0.1:{port}/{}",
        room.0
    );
    let mut apps: Vec<App> = names.iter().map(|n| instance(n, &room, port)).collect();
    wait_connected(&mut apps);
    (apps, room, port)
}

fn wait_connected(apps: &mut [App]) {
    let others = apps.len() - 1;
    wait_for(
        apps,
        CONNECT,
        "the instances never saw each other",
        |apps| {
            apps.iter().all(|a| net(a).peers.len() == others)
                && apps.iter().filter(|a| net(a).is_host).count() == 1
        },
    );
}

fn host_index(apps: &[App]) -> usize {
    apps.iter().position(|a| net(a).is_host).expect("a host")
}

/// Edit the document the way the editor does: mutate locally, then drop the
/// whole-tree edit into the outbox.
fn edit(app: &mut App, label: &str) -> NodeConfig {
    let mut tree = NodeConfig::new_container("root");
    tree.children.push(NodeConfig::new_leaf(label, 50.0, 50.0));
    app.world_mut().resource_mut::<FlexConfig>().root = tree.clone();
    app.world_mut()
        .resource_mut::<PendingEdits>()
        .0
        .push(LayoutEdit::ReplaceRoot(Box::new(tree.clone())));
    tree
}

/// Wait until every instance shows `expected` *and* has applied sequence
/// number `seq`: the editing peer's own tree matches before its sequenced
/// echo returns, so the watermark is what proves the edit went round.
fn wait_root(apps: &mut [App], expected: &NodeConfig, seq: u32, what: &str) {
    wait_for(apps, SETTLE, what, |apps| {
        apps.iter()
            .all(|a| root(a) == expected && net(a).last_applied_seq == Some(seq))
    });
    for (i, app) in apps.iter().enumerate() {
        let n = net(app);
        println!(
            "[state] instance {i}: host={} has_document={} next_seq={} last_applied={:?} peers={}",
            n.is_host,
            n.has_document,
            n.next_seq,
            n.last_applied_seq,
            n.peers.len()
        );
    }
}

#[test]
#[ignore = "needs a real peer handshake; run with --ignored"]
fn two_instances_elect_one_host_and_share_edits() {
    let (mut apps, _room, _port) = connect("pair", &["ada", "bob"]);
    let host = host_index(&apps);
    let guest = 1 - host;
    println!("[pair] host is instance {host}");

    // The guest's edit goes to the host, is sequenced, and lands on both.
    let tree = edit(&mut apps[guest], "from-guest");
    wait_root(
        &mut apps,
        &tree,
        0,
        "the guest's edit never reached the host",
    );

    // The host's own edit is sequenced on the spot and reaches the guest.
    let tree = edit(&mut apps[host], "from-host");
    wait_root(
        &mut apps,
        &tree,
        1,
        "the host's edit never reached the guest",
    );
}

#[test]
#[ignore = "needs a real peer handshake; run with --ignored"]
fn a_late_joiner_receives_the_document() {
    let (mut apps, room, port) = connect("late", &["ada", "bob"]);
    let host = host_index(&apps);
    let tree = edit(&mut apps[host], "before-cara");
    wait_root(
        &mut apps,
        &tree,
        0,
        "the pair never agreed before the third joined",
    );

    apps.push(instance("cara", &room, port));
    wait_for(&mut apps, CONNECT, "cara never saw the room", |apps| {
        net(&apps[2]).peers.len() == 2
    });
    // Cara greets, the host answers with the document, cara takes it verbatim.
    wait_root(&mut apps, &tree, 0, "cara never received the document");

    // And she is a full participant afterwards.
    let tree = edit(&mut apps[2], "from-cara");
    wait_root(&mut apps, &tree, 1, "cara's edit never reached the others");
}
