//! Saving raw RGBA framebuffers as PNG.

use std::path::Path;

use anyhow::{Context, Result, bail};

/// Save an RGBA8 buffer of `width`×`height` pixels to `path`, creating
/// parent directories as needed. When `target` is given and differs from the
/// buffer size (HiDPI screenshots), the image is resampled to it so goldens
/// are comparable across DPI settings.
pub fn save_rgba_png(
    path: &Path,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    target: Option<(u32, u32)>,
) -> Result<()> {
    let Some(img) = image::RgbaImage::from_raw(width, height, rgba) else {
        bail!("screenshot buffer does not match {width}x{height} RGBA");
    };

    let img = match target {
        Some((tw, th)) if tw != width || th != height => {
            image::imageops::resize(&img, tw, th, image::imageops::FilterType::Lanczos3)
        }
        _ => img,
    };

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    img.save(path)
        .with_context(|| format!("failed to save {}", path.display()))
}
