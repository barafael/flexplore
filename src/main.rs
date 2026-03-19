//! Flexplore — interactive Bevy 0.18 flexbox explorer.

mod art;
mod codegen;
mod config;
mod history;
mod panel;
mod viz;

use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

use art::ArtState;
use config::FlexConfig;
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
        .insert_resource(UndoHistory::new(FlexConfig::default()))
        .add_systems(Startup, setup)
        .add_systems(EguiPrimaryContextPass, panel::panel_system)
        .add_systems(
            Update,
            (
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
