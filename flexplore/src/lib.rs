//! Flexplore — flexbox layout code generator library.

pub use flexplore_core::codegen;
pub use flexplore_core::fixtures;
pub use flexplore_core::templates;

pub mod art;
pub mod bevy_node;
pub mod config;
pub mod history;
#[cfg(feature = "multiplayer")]
pub mod net;
#[cfg(feature = "multiplayer")]
pub mod presence;
#[cfg(not(target_arch = "wasm32"))]
pub mod render;
