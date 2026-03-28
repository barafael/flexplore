//! Flexplore — interactive Bevy 0.18 flexbox explorer.

mod highlight;
mod history;
#[cfg(feature = "multiplayer")]
mod net;
mod panel;
mod persist;
mod viz;

use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

use flexplore::art::ArtState;
use flexplore::config::FlexConfig;
use history::UndoHistory;

/// Parsed CLI args for the flexplore app.
#[cfg(not(target_arch = "wasm32"))]
struct CliArgs {
    /// If set, connect to this server address for multiplayer.
    server: Option<std::net::SocketAddr>,
    /// Client id (defaults to random).
    client_id: u64,
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_cli() -> CliArgs {
    let mut server = None;
    let mut client_id: u64 = rand::random();
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--server" | "-s" if i + 1 < args.len() => {
                i += 1;
                server = args[i].parse().ok();
            }
            "--id" if i + 1 < args.len() => {
                i += 1;
                client_id = args[i].parse().unwrap_or(client_id);
            }
            _ => {}
        }
        i += 1;
    }
    CliArgs { server, client_id }
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    let cli = parse_cli();

    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Flexplore".into(),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }),
        EguiPlugin::default(),
    ))
    .init_resource::<FlexConfig>()
    .init_resource::<ArtState>()
    .init_resource::<viz::ArrowNav>()
    .init_resource::<flexplore::config::RightPanelOpen>()
    .insert_resource(UndoHistory::new(FlexConfig::default()))
    .add_systems(Startup, (setup, load_autosave))
    .add_systems(EguiPrimaryContextPass, panel::panel_system)
    .add_systems(Update, auto_save_system)
    .add_systems(
        Update,
        (
            viz::viz_arrow_nav,
            viz::viz_click,
            viz::viz_tooltip,
            viz::rebuild_viz,
            viz::animate_art,
        )
            .chain(),
    );

    // ── Multiplayer (opt-in via --server) ────────────────────────────────────
    #[cfg(all(feature = "multiplayer", not(target_arch = "wasm32")))]
    if let Some(server_addr) = cli.server {
        app.insert_resource(net::NetConfig {
            server_addr,
            client_id: cli.client_id,
        });
        app.add_plugins(net::NetPlugin);
    }

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn load_autosave(mut cfg: ResMut<FlexConfig>, mut history: ResMut<UndoHistory>) {
    if let Some(loaded) = persist::auto_load() {
        *cfg = loaded;
        cfg.request_rebuild();
        history.push(cfg.clone());
    }
}

fn auto_save_system(cfg: Res<FlexConfig>, mut timer: Local<Option<Timer>>, time: Res<Time>) {
    let t = timer.get_or_insert_with(|| Timer::from_seconds(2.0, TimerMode::Repeating));
    t.tick(time.delta());
    if t.just_finished() && cfg.is_changed() {
        persist::auto_save(&cfg);
    }
}
