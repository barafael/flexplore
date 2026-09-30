//! Shared plumbing for the golden-screenshot tools under `tools/`.
//!
//! Every tool reads `testdata/<case>/input.json` (a serialized
//! [`LayoutInput`]) and writes `testdata/<case>/rendered_<backend>.png`.
//! This crate holds what they all need: the job loader, the viewport size,
//! the colour palette, a few layout helpers, and (behind features) the
//! headless-Chrome and PNG helpers.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

pub use flexplore_core::art::palette_color;
pub use flexplore_core::config;
use flexplore_core::config::{ColorPalette, JustifyContent, LayoutInput, NodeConfig, ValueConfig};

#[cfg(feature = "chrome")]
pub mod chrome;
pub mod html;
#[cfg(feature = "png")]
pub mod png;

/// Golden viewport width in CSS pixels; every backend renders at this size.
pub const VIEWPORT_W: f32 = 400.0;
/// Golden viewport height in CSS pixels.
pub const VIEWPORT_H: f32 = 300.0;

/// One fixture to render: its name, layout tree, palette and where the
/// `rendered_*.png` goes (`output_dir/<name>/`).
pub struct RenderJob {
    pub name: String,
    pub node: NodeConfig,
    pub palette: ColorPalette,
    pub output_dir: PathBuf,
}

impl RenderJob {
    /// `output_dir/<name>/<file>` — the path a backend writes its output to.
    pub fn output_path(&self, file: &str) -> PathBuf {
        self.output_dir.join(&self.name).join(file)
    }
}

/// The repository's `testdata/` directory, resolved relative to this crate's
/// manifest (so the tools work from any working directory).
pub fn default_testdata_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../testdata");
    dir.canonicalize().unwrap_or(dir)
}

/// Walk `testdata_dir`, parse every `<case>/input.json`, and keep the cases
/// named in `filter` (all of them when it is empty). Cases are returned in
/// name order. Naming a case that does not exist is an error, so a typo in
/// CI fails loudly instead of silently rendering nothing.
pub fn load_jobs(testdata_dir: &Path, filter: &[String]) -> Result<Vec<RenderJob>> {
    let mut entries: Vec<_> = std::fs::read_dir(testdata_dir)
        .with_context(|| format!("cannot read testdata directory {}", testdata_dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let mut jobs = Vec::new();
    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !filter.is_empty() && !filter.contains(&name) {
            continue;
        }

        let input_path = entry.path().join("input.json");
        let json = match std::fs::read_to_string(&input_path) {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => {
                return Err(e).with_context(|| format!("failed to read {}", input_path.display()));
            }
        };
        let input: LayoutInput = serde_json::from_str(&json)
            .with_context(|| format!("failed to parse {}", input_path.display()))?;

        jobs.push(RenderJob {
            name,
            node: input.node,
            palette: input.palette,
            output_dir: testdata_dir.to_path_buf(),
        });
    }

    let missing: Vec<&String> = filter
        .iter()
        .filter(|f| !jobs.iter().any(|j| &j.name == *f))
        .collect();
    if !missing.is_empty() {
        bail!(
            "no such test case(s) in {}: {}",
            testdata_dir.display(),
            missing
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    Ok(jobs)
}

/// Palette colour for the `idx`-th leaf as 8-bit RGB.
pub fn palette_rgb8(palette: ColorPalette, idx: usize) -> (u8, u8, u8) {
    let (r, g, b) = palette_color(palette, idx);
    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

/// Resolve a length to pixels. Percentages are relative to `parent_px`,
/// viewport units to the golden viewport, and `auto` resolves to 0 so callers
/// can treat "0" as "unspecified".
pub fn resolve_to_px(v: &ValueConfig, parent_px: f32) -> f32 {
    match v {
        ValueConfig::Auto => 0.0,
        ValueConfig::Px(n) => *n,
        ValueConfig::Percent(n) => n / 100.0 * parent_px,
        ValueConfig::Vw(n) => n / 100.0 * VIEWPORT_W,
        ValueConfig::Vh(n) => n / 100.0 * VIEWPORT_H,
    }
}

/// `justify-content` as seen along the *visual* axis: a reversed flex
/// direction flips the main axis, so flex-start/end (and start/end) swap.
pub fn effective_justify(jc: JustifyContent, is_reversed: bool) -> JustifyContent {
    if !is_reversed {
        return jc;
    }
    match jc {
        JustifyContent::FlexStart => JustifyContent::FlexEnd,
        JustifyContent::FlexEnd => JustifyContent::FlexStart,
        JustifyContent::Start => JustifyContent::End,
        JustifyContent::End => JustifyContent::Start,
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_testdata_dir_exists() {
        assert!(default_testdata_dir().is_dir());
    }

    #[test]
    fn loads_all_fixtures_and_rejects_unknown_case() {
        let dir = default_testdata_dir();
        let jobs = load_jobs(&dir, &[]).unwrap();
        assert!(jobs.len() >= 30, "expected the committed fixtures");
        assert!(load_jobs(&dir, &["no_such_case".to_string()]).is_err());
    }

    #[test]
    fn reversed_justify_swaps_ends() {
        assert_eq!(
            effective_justify(JustifyContent::FlexStart, true),
            JustifyContent::FlexEnd
        );
        assert_eq!(
            effective_justify(JustifyContent::Center, true),
            JustifyContent::Center
        );
    }
}
