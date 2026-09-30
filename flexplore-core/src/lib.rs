//! Flexplore core — layout config types, codegen, fixtures, and palettes.
//! Bevy is only an optional dependency (the `bevy` feature adds `Into` /
//! `to_bevy_*` conversions for the config types); everything else is plain Rust.

pub mod art;
pub mod codegen;
pub mod config;
pub mod fixtures;
pub mod templates;
