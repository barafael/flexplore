//! Live check of Bevy 0.20's interaction mechanisms on plain UI nodes.
//!
//! The viz spawns nodes *without* any interaction component: picking
//! auto-maintains `PickingInteraction` on them (Hovered/Pressed/None), while
//! `picking::hover::Hovered` and `ui::Pressed` are opt-in/widget components
//! that nothing inserts on plain nodes. This example drives the picking input
//! with synthetic `WindowEvent`s and prints, per frame, which components
//! actually appear on:
//!
//! - `PlainNode`/`PlainChild` — the viz bundle (no interaction component),
//! - `HoverOptIn` — the same node plus a spawned `Hovered::default()`.
//!
//! Run with: `cargo run -p flexplore --example interaction_probe`

use bevy::input::ButtonState;
use bevy::input::mouse::{MouseButton, MouseButtonInput};
use bevy::picking::hover::{Hovered, PickingInteraction};
use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy::window::{CursorMoved, PrimaryWindow, WindowEvent};

/// Frames per phase: idle, hover, press, press, release, settle, settle.
const PHASE: u32 = 7;
/// Cursor position over each probe node (logical pixels), phase order.
const PROBES: [(&str, Vec2); 2] = [
    ("plain", Vec2::new(150.0, 100.0)),
    ("hover-opt-in", Vec2::new(450.0, 100.0)),
];

#[derive(Resource, Default)]
struct Frame(u32);

fn probe_node(left: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(left),
        top: Val::Px(10.0),
        width: Val::Px(200.0),
        height: Val::Px(200.0),
        ..default()
    }
}

fn spawn_probes(mut commands: Commands) {
    commands.spawn(Camera2d);

    // The viz bundle: no interaction component at all, with a nested child
    // so the "topmost picked node" logic is exercised.
    let plain = commands
        .spawn((
            probe_node(50.0),
            BackgroundColor(Color::srgb(0.4, 0.4, 0.4)),
        ))
        .id();
    let child = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(50.0),
                top: Val::Px(50.0),
                width: Val::Px(100.0),
                height: Val::Px(100.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.6, 0.6, 0.6)),
        ))
        .id();
    commands.entity(plain).add_child(child);

    // Opt-in `Hovered`: picking keeps it up to date, but only if spawned.
    commands.spawn((
        probe_node(350.0),
        BackgroundColor(Color::srgb(0.4, 0.4, 0.5)),
        Hovered::default(),
    ));
}

/// Move the cursor over the current phase's probe; press/release mid-phase.
fn drive(
    mut frame: ResMut<Frame>,
    window: Single<Entity, With<PrimaryWindow>>,
    mut window_events: MessageWriter<WindowEvent>,
    mut exit: MessageWriter<AppExit>,
) {
    let f = frame.0;
    let phase = (f / PHASE) as usize;
    let Some((_, pos)) = PROBES.get(phase).map(|(name, pos)| (*name, *pos)) else {
        exit.write(AppExit::Success);
        return;
    };
    window_events.write(WindowEvent::CursorMoved(CursorMoved {
        window: *window,
        position: pos,
        delta: None,
    }));
    let button = |state| {
        WindowEvent::MouseButtonInput(MouseButtonInput {
            window: *window,
            button: MouseButton::Left,
            state,
        })
    };
    match f % PHASE {
        2 | 3 => {
            window_events.write(button(ButtonState::Pressed));
        }
        4 => {
            window_events.write(button(ButtonState::Released));
        }
        _ => {}
    }
    frame.0 += 1;
}

fn report(
    frame: Res<Frame>,
    probes: Query<(&PickingInteraction, Option<&Hovered>, Option<&Pressed>)>,
) {
    let f = frame.0;
    let Some((name, _)) = PROBES.get((f / PHASE) as usize) else {
        return;
    };
    if f % PHASE > 4 {
        return; // only report hover / press / release frames
    }
    let mut line = String::new();
    for (picking, hovered, pressed) in &probes {
        let h: String = hovered.map(|h| h.0.to_string()).unwrap_or("—".into());
        let p: String = pressed.map(|_| "yes".to_string()).unwrap_or("—".into());
        line.push_str(&format!(" [picking={picking:?} Hovered={h} Pressed={p}]"));
    }
    println!("frame {f:>2} phase={name:<12} t={} {line}", f % PHASE);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "flexplore interaction probe".into(),
                resolution:
                    bevy::window::WindowResolution::new(700, 300).with_scale_factor_override(1.0),
                ..default()
            }),
            ..default()
        }))
        .init_resource::<Frame>()
        .add_systems(Startup, spawn_probes)
        .add_systems(Update, drive)
        .add_systems(Last, report)
        .run();

    // Verdict (read the printed table): on the plain stack, `PickingInteraction`
    // goes Hovered -> Pressed -> None and only the topmost node is picked,
    // while the `Hovered`/`Pressed` columns stay "—" — nothing maintains those
    // components on plain nodes. The opt-in node's `Hovered` flips true/false.
}
