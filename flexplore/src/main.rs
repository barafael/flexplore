//! Flexplore — interactive Bevy 0.18 flexbox explorer.

mod history;
mod panel;
mod persist;
mod viz;

use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

use flexplore::art::ArtState;
use flexplore::config::FlexConfig;
use history::UndoHistory;

fn main() {
    App::new()
        .add_plugins((
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
        )
        .run();
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
