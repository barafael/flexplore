use std::path::PathBuf;

use bevy::prelude::*;
use bevy::render::view::window::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::window::PrimaryWindow;

use crate::config::{ColorPalette, NodeConfig};

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

    let spawn_fns: Vec<SpawnableJob> = jobs
        .into_iter()
        .map(|j| SpawnableJob {
            name: j.name,
            node: j.node,
            palette: j.palette,
        })
        .collect();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "flexplain render".into(),
                resolution: (800, 600).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(RenderQueue {
            jobs: spawn_fns,
            current: 0,
            output_dir,
            frames_waited: 0,
            screenshot_pending: false,
        })
        .insert_resource(ScreenshotSaved(false))
        .add_systems(Startup, setup_first)
        .add_systems(Update, drive_rendering)
        .run();
}

struct SpawnableJob {
    name: String,
    node: NodeConfig,
    palette: ColorPalette,
}

#[derive(Resource)]
struct RenderQueue {
    jobs: Vec<SpawnableJob>,
    current: usize,
    output_dir: PathBuf,
    frames_waited: u32,
    screenshot_pending: bool,
}

#[derive(Resource)]
struct ScreenshotSaved(bool);

fn setup_first(mut commands: Commands, queue: Res<RenderQueue>) {
    commands.spawn(Camera2d);
    if let Some(job) = queue.jobs.first() {
        eprintln!("Spawning UI: {}", job.name);
        spawn_node_tree(&mut commands, &job.node, job.palette);
    }
}

fn drive_rendering(
    mut commands: Commands,
    mut queue: ResMut<RenderQueue>,
    mut exit: MessageWriter<AppExit>,
    mut saved: ResMut<ScreenshotSaved>,
    ui_roots: Query<Entity, (With<Node>, Without<ChildOf>)>,
    window: Single<Entity, With<PrimaryWindow>>,
) {
    if queue.current >= queue.jobs.len() {
        exit.write(AppExit::Success);
        return;
    }

    queue.frames_waited += 1;

    // Wait a few frames for layout + render to settle, then request screenshot
    if queue.frames_waited == 4 && !queue.screenshot_pending {
        let job = &queue.jobs[queue.current];
        let path = queue.output_dir.join(&job.name).join("rendered_bevy.png");
        eprintln!("Capturing screenshot: {}", path.display());
        commands
            .spawn(Screenshot::window(*window))
            .observe(save_and_signal(path));
        queue.screenshot_pending = true;
    }

    // Wait for the observer to confirm the file was saved
    if queue.screenshot_pending && saved.0 {
        saved.0 = false;
        queue.screenshot_pending = false;
        queue.current += 1;
        queue.frames_waited = 0;

        for entity in ui_roots.iter() {
            commands.entity(entity).despawn();
        }

        if let Some(job) = queue.jobs.get(queue.current) {
            eprintln!("Spawning UI: {}", job.name);
            spawn_node_tree(&mut commands, &job.node, job.palette);
        } else {
            eprintln!("All done!");
            exit.write(AppExit::Success);
        }
    }
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
    spawn_node_entity(commands, root, &mut leaf_idx, palette);
}

fn spawn_node_entity(
    commands: &mut Commands,
    node: &NodeConfig,
    leaf_idx: &mut usize,
    palette: ColorPalette,
) -> Entity {
    let is_leaf = node.children.is_empty();

    let bg = if is_leaf {
        let (r, g, b) = crate::art::palette_color(palette, *leaf_idx);
        *leaf_idx += 1;
        Color::srgb(r, g, b)
    } else {
        Color::srgba(0.11, 0.11, 0.17, 1.0)
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
        height: to_val(&node.height),
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
            .map(|child| spawn_node_entity(commands, child, leaf_idx, palette))
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
