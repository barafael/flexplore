//! Flexplore — flexbox layout code generator library.

pub use flexplore_core::codegen;
pub use flexplore_core::fixtures;
pub use flexplore_core::templates;

pub mod art;
pub mod bevy_node;
pub mod config;
#[cfg(not(target_arch = "wasm32"))]
pub mod render;
