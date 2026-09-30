use std::path::PathBuf;

use bevy::{
    prelude::*,
    render::{
        RenderPlugin,
        settings::{Backends, RenderCreation, WgpuSettings},
        view::window::screenshot::{Screenshot, ScreenshotCaptured},
    },
    window::{PrimaryWindow, WindowResolution},
};

use crate::bevy_node;
use crate::config::{ColorPalette, NodeConfig};

/// Frames to let Bevy's UI layout settle after spawning a new tree.
const SETTLE_FRAMES: u32 = 4;

/// Max frames to wait for the GPU pipeline before giving up.
/// At 60 fps this is ~5 seconds — enough for shader compilation, short enough to fail fast.
const PIPELINE_TIMEOUT_FRAMES: u32 = 300;

/// A single render job: name, layout tree, and palette.
pub struct RenderJob {
    pub name: String,
    pub node: NodeConfig,
    pub palette: ColorPalette,
}

/// Render each job to `{output_dir}/{name}/rendered_bevy.png`.
/// Opens a Bevy window, captures one screenshot per job, then exits.
pub fn render_to_images(jobs: Vec<RenderJob>, output_dir: PathBuf) {
    if jobs.is_empty() {
        eprintln!("No render jobs.");
        return;
    }
    eprintln!(
        "Will render {} test case(s) to {}",
        jobs.len(),
        output_dir.display()
    );

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "flexplore render".into(),
                        resolution: WindowResolution::new(400, 300).with_scale_factor_override(1.0),
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(Box::new(WgpuSettings {
                        // `None` would disable rendering entirely; keep the
                        // default (all backends, honouring WGPU_BACKEND) except
                        // on Windows, where DX12 is the reliable choice.
                        backends: if cfg!(windows) {
                            Some(Backends::DX12)
                        } else {
                            WgpuSettings::default().backends
                        },
                        ..default()
                    })),
                    ..default()
                }),
        )
        .insert_resource(ClearColor(bevy_node::CONTAINER_BG))
        .insert_resource(RenderQueue {
            jobs,
            current: 0,
            output_dir,
            phase: Phase::WarmingUp,
            frames_waited: 0,
        })
        .insert_resource(PipelineReady(false))
        .insert_resource(ScreenshotSaved(false))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, drive_rendering)
        .run();
}

/// Rendering proceeds in phases for each job.
enum Phase {
    /// Fire a throwaway screenshot to wait for the GPU pipeline to be ready.
    /// The callback proves the pipeline can produce frames.
    WarmingUp,
    /// Probe requested; waiting for its callback before spawning UI.
    WaitingForPipeline,
    /// UI tree has been spawned; waiting SETTLE_FRAMES for layout to converge.
    Settling,
    /// Real screenshot requested; waiting for the observer callback.
    Capturing,
}

#[derive(Resource)]
struct RenderQueue {
    jobs: Vec<RenderJob>,
    current: usize,
    output_dir: PathBuf,
    phase: Phase,
    frames_waited: u32,
}

#[derive(Resource)]
struct PipelineReady(bool);

#[derive(Resource)]
struct ScreenshotSaved(bool);

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn drive_rendering(
    mut commands: Commands,
    mut queue: ResMut<RenderQueue>,
    mut exit: MessageWriter<AppExit>,
    mut ready: ResMut<PipelineReady>,
    mut saved: ResMut<ScreenshotSaved>,
    ui_roots: Query<Entity, (With<Node>, Without<ChildOf>)>,
    window: Single<Entity, With<PrimaryWindow>>,
) {
    if queue.current >= queue.jobs.len() {
        exit.write(AppExit::Success);
        return;
    }

    match queue.phase {
        Phase::WarmingUp => {
            // Spawn the first job's UI immediately so the render pipeline
            // starts compiling UI shaders during warmup.
            let job = &queue.jobs[queue.current];
            eprintln!("Spawning UI: {}", job.name);
            spawn_node_tree(&mut commands, &job.node, job.palette);
            // Request a throwaway screenshot whose callback proves the
            // GPU render pipeline has produced at least one frame.
            commands
                .spawn(Screenshot::window(*window))
                .observe(signal_pipeline_ready);
            queue.phase = Phase::WaitingForPipeline;
        }
        Phase::WaitingForPipeline => {
            if ready.0 {
                ready.0 = false;
                queue.frames_waited = 0;
                // Pipeline is warm and UI shaders are compiled.
                // Start counting settle frames for layout convergence.
                queue.phase = Phase::Settling;
            } else {
                queue.frames_waited += 1;
                if queue.frames_waited >= PIPELINE_TIMEOUT_FRAMES {
                    eprintln!(
                        "ERROR: GPU render pipeline did not become ready after {} frames. \
                         The RenderApp likely failed to initialize (no GPU available?).",
                        PIPELINE_TIMEOUT_FRAMES
                    );
                    exit.write(AppExit::error());
                }
            }
        }
        Phase::Settling => {
            queue.frames_waited += 1;
            if queue.frames_waited >= SETTLE_FRAMES {
                let job = &queue.jobs[queue.current];
                let path = queue.output_dir.join(&job.name).join("rendered_bevy.png");
                eprintln!("Capturing screenshot: {}", path.display());
                commands
                    .spawn(Screenshot::window(*window))
                    .observe(save_and_signal(path));
                queue.phase = Phase::Capturing;
            }
        }
        Phase::Capturing => {
            if saved.0 {
                saved.0 = false;
                queue.current += 1;
                queue.frames_waited = 0;

                for entity in ui_roots.iter() {
                    commands.entity(entity).despawn();
                }

                if let Some(job) = queue.jobs.get(queue.current) {
                    eprintln!("Spawning UI: {}", job.name);
                    spawn_node_tree(&mut commands, &job.node, job.palette);
                    // Pipeline is already warm; go straight to settling.
                    queue.phase = Phase::Settling;
                } else {
                    eprintln!("All done!");
                    exit.write(AppExit::Success);
                }
            }
        }
    }
}

/// Observer for the warmup probe — signals pipeline readiness, discards the image.
fn signal_pipeline_ready(
    _screenshot_captured: On<ScreenshotCaptured>,
    mut ready: ResMut<PipelineReady>,
) {
    ready.0 = true;
}

/// Observer that saves the screenshot to disk and sets the ScreenshotSaved flag.
fn save_and_signal(path: PathBuf) -> impl FnMut(On<ScreenshotCaptured>, ResMut<ScreenshotSaved>) {
    move |screenshot_captured, mut saved| {
        let img = screenshot_captured.image.clone();
        match img.try_into_dynamic() {
            Ok(dyn_img) => {
                let img = dyn_img.to_rgb8();
                match img.save_with_format(&path, image::ImageFormat::Png) {
                    Ok(()) => eprintln!("  Saved: {}", path.display()),
                    Err(e) => eprintln!("  ERROR saving {}: {e}", path.display()),
                }
            }
            Err(e) => eprintln!("  ERROR converting screenshot: {e}"),
        }
        saved.0 = true;
    }
}

// ─── Build the Bevy UI tree directly from NodeConfig ─────────────────────────

fn spawn_node_tree(commands: &mut Commands, root: &NodeConfig, palette: ColorPalette) {
    let mut leaf_idx = 0;
    spawn_node_entity(commands, root, &mut leaf_idx, palette, true);
}

/// Spawn `node` exactly as the app's live preview does (see `viz.rs`), minus
/// the selection outline and generative-art textures.
fn spawn_node_entity(
    commands: &mut Commands,
    node: &NodeConfig,
    leaf_idx: &mut usize,
    palette: ColorPalette,
    is_root: bool,
) -> Entity {
    let is_leaf = node.children.is_empty();

    let bg = if is_leaf {
        let (r, g, b) = crate::art::palette_color(palette, *leaf_idx);
        *leaf_idx += 1;
        Color::srgb(r, g, b)
    } else {
        bevy_node::CONTAINER_BG
    };

    let mut style = bevy_node::node_to_bevy(node);
    if is_root {
        // Force root to fill the viewport, matching HTML `body { height: 100% }`.
        style.height = Val::Percent(100.0);
    }

    let entity = commands
        .spawn((
            style,
            BackgroundColor(bg),
            BorderColor::all(bevy_node::BORDER_COLOR),
            bevy_node::node_visibility(node),
        ))
        .id();

    if is_leaf {
        bevy_node::spawn_leaf_text(commands, entity, node);
    } else {
        bevy_node::spawn_container_label(commands, entity, node);
        let child_entities: Vec<Entity> = bevy_node::sorted_child_indices(node)
            .into_iter()
            .map(|i| spawn_node_entity(commands, &node.children[i], leaf_idx, palette, false))
            .collect();
        commands.entity(entity).add_children(&child_entities);
    }

    entity
}
