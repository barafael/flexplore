use std::path::PathBuf;

use bevy::prelude::*;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
use bevy::render::view::window::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::render::RenderPlugin;
use bevy::window::{PrimaryWindow, WindowResolution};

use crate::config::{ColorPalette, NodeConfig};

/// Frames to let Bevy's UI layout settle after spawning a new tree.
const SETTLE_FRAMES: u32 = 4;

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
    eprintln!("Will render {} test case(s) to {}", jobs.len(), output_dir.display());

    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "flexplain render".into(),
                    resolution: WindowResolution::new(400, 300)
                        .with_scale_factor_override(1.0),
                    ..default()
                }),
                ..default()
            })
            .set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    backends: if cfg!(windows) {
                        Some(Backends::DX12)
                    } else {
                        None // auto-detect on other platforms
                    },
                    ..default()
                }),
                ..default()
            }))
        .insert_resource(ClearColor(Color::srgba(0.11, 0.11, 0.17, 1.0)))
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
                // Pipeline is warm and UI shaders are compiled.
                // Start counting settle frames for layout convergence.
                queue.phase = Phase::Settling;
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
        Color::srgba(0.11, 0.11, 0.17, 1.0)
    };

    // Force root to fill the viewport, matching HTML `body { height: 100% }`.
    let height = if is_root {
        Val::Percent(100.0)
    } else {
        to_val(&node.height)
    };

    let style = Node {
        flex_direction: node.flex_direction,
        flex_wrap: node.flex_wrap,
        justify_content: node.justify_content,
        align_items: node.align_items,
        align_content: node.align_content,
        align_self: node.align_self,
        flex_grow: node.flex_grow,
        flex_shrink: node.flex_shrink,
        flex_basis: to_val(&node.flex_basis),
        row_gap: to_val(&node.row_gap),
        column_gap: to_val(&node.column_gap),
        width: to_val(&node.width),
        height,
        min_width: to_val(&node.min_width),
        min_height: to_val(&node.min_height),
        max_width: to_val(&node.max_width),
        max_height: to_val(&node.max_height),
        padding: UiRect::all(to_val(&node.padding)),
        margin: UiRect::all(to_val(&node.margin)),
        ..default()
    };

    let mut ec = commands.spawn((style, BackgroundColor(bg)));

    if !node.visible {
        ec.insert(Visibility::Hidden);
    }

    let entity = ec.id();

    if is_leaf {
        ec.with_children(|parent| {
            parent
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_child((
                    Text::new(&node.label),
                    TextFont {
                        font_size: 26.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.05, 0.05, 0.1, 0.85)),
                ));
        });
    } else {
        let mut sorted: Vec<&NodeConfig> = node.children.iter().collect();
        sorted.sort_by_key(|c| c.order);
        let child_entities: Vec<Entity> = sorted
            .iter()
            .map(|child| spawn_node_entity(commands, child, leaf_idx, palette, false))
            .collect();
        commands.entity(entity).add_children(&child_entities);
    }

    entity
}

fn to_val(v: &crate::config::ValueConfig) -> Val {
    match v {
        crate::config::ValueConfig::Auto => Val::Auto,
        crate::config::ValueConfig::Px(n) => Val::Px(*n),
        crate::config::ValueConfig::Percent(n) => Val::Percent(*n),
        crate::config::ValueConfig::Vw(n) => Val::Vw(*n),
        crate::config::ValueConfig::Vh(n) => Val::Vh(*n),
    }
}
