pub use flexplore_core::config::*;

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

pub const PANEL_WIDTH: f32 = 390.0;
pub const ART_TEXTURE_SIZE: u32 = 128;

// ─── Main resource (Bevy ECS) ────────────────────────────────────────────────

fn deserialize_theme<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Theme, D::Error> {
    use serde::Deserialize;
    let s = String::deserialize(d)?;
    Ok(match s.as_str() {
        "Dark" | "Mocha" => Theme::Mocha,
        "Light" | "Latte" => Theme::Latte,
        "Frappe" | "Frappé" => Theme::Frappe,
        "Macchiato" => Theme::Macchiato,
        _ => Theme::Mocha,
    })
}

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct FlexConfig {
    pub root: NodeConfig,
    #[serde(skip)]
    selected: Vec<usize>,
    pub bg_mode: BackgroundMode,
    pub art_style: ArtStyle,
    pub art_seed: u64,
    pub art_depth: u32,
    pub art_anim: f32,
    #[serde(deserialize_with = "deserialize_theme")]
    pub theme: Theme,
    #[serde(default = "default_palette")]
    pub palette: ColorPalette,
    #[serde(skip)]
    needs_rebuild: bool,
}

impl FlexConfig {
    pub fn selected(&self) -> &[usize] {
        &self.selected
    }

    /// Set the selected node path and mark for rebuild.
    pub fn select(&mut self, path: Vec<usize>) {
        self.selected = path;
        self.needs_rebuild = true;
    }

    /// Deselect towards root until the path is valid.
    pub fn sanitize_selection(&mut self) {
        while self.root.get(&self.selected).is_none() && !self.selected.is_empty() {
            self.selected.pop();
        }
    }

    pub fn request_rebuild(&mut self) {
        self.needs_rebuild = true;
    }

    /// Returns true (and resets the flag) if a rebuild was requested.
    pub fn take_rebuild(&mut self) -> bool {
        std::mem::replace(&mut self.needs_rebuild, false)
    }
}

impl Default for FlexConfig {
    fn default() -> Self {
        let mut root = NodeConfig::new_container("root");
        root.min_height = ValueConfig::Px(200.0);
        root.children = vec![
            NodeConfig::new_leaf("A", 80.0, 80.0),
            NodeConfig::new_leaf("B", 120.0, 100.0),
            NodeConfig::new_leaf("C", 60.0, 60.0),
            NodeConfig::new_leaf("D", 100.0, 80.0),
        ];
        Self {
            root,
            selected: vec![],
            bg_mode: BackgroundMode::Pastel,
            art_style: ArtStyle::ExprTree,
            art_seed: 137,
            art_depth: 5,
            art_anim: 0.0,
            theme: Theme::Mocha,
            palette: ColorPalette::Pastel1,
            needs_rebuild: true,
        }
    }
}
