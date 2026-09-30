mod bevy;
mod common;
mod css;
mod dioxus;
mod egui;
mod flutter;
mod iced;
mod react;
mod react_native;
mod swiftui;
mod tailwind;

#[cfg(test)]
mod gen_tests;
#[cfg(test)]
mod snapshot_tests;

use anyhow::Result;

use crate::config::{ColorPalette, NodeConfig};

pub use bevy::emit_bevy_code;
pub use css::emit_html_css;
pub use dioxus::emit_dioxus;
pub use egui::emit_egui;
pub use flutter::emit_flutter;
pub use iced::emit_iced;
pub use react::emit_react;
pub use react_native::emit_react_native;
pub use swiftui::emit_swiftui;
pub use tailwind::emit_tailwind;

/// Signature shared by every code generator.
pub type Emitter = fn(&NodeConfig, ColorPalette) -> Result<String>;

/// Every code generator, paired with the snapshot file name its output is
/// stored under in `testdata/<case>/`. This is the single source of truth for
/// "all targets"; snapshot tests and `update-snapshots` iterate it.
pub const TARGETS: &[(&str, Emitter)] = &[
    ("expected.html", emit_html_css),
    ("expected.rs", emit_bevy_code),
    ("expected.jsx", emit_react),
    ("expected.tailwind.html", emit_tailwind),
    ("expected.swift", emit_swiftui),
    ("expected.dart", emit_flutter),
    ("expected.iced.rs", emit_iced),
    ("expected.rn.jsx", emit_react_native),
    ("expected.dioxus.rs", emit_dioxus),
    ("expected.egui.rs", emit_egui),
];

/// Run every generator in [`TARGETS`] on `node`, returning
/// `(snapshot file name, generated source)` pairs in `TARGETS` order.
pub fn emit_all(node: &NodeConfig, palette: ColorPalette) -> Result<Vec<(&'static str, String)>> {
    TARGETS
        .iter()
        .map(|(name, emit)| Ok((*name, emit(node, palette)?)))
        .collect()
}
