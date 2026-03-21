use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use flexplore::{config::LayoutInput, render::{RenderJob, render_to_images}};

fn load_jobs(testdata_dir: &Path, filter: &[String]) -> Result<Vec<RenderJob>> {
    let mut jobs = Vec::new();

    let mut entries: Vec<_> = std::fs::read_dir(testdata_dir)
        .context("cannot read testdata directory")?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();

        if !filter.is_empty() && !filter.iter().any(|f| f == &name) {
            continue;
        }

        let input_path = entry.path().join("input.json");
        let json = match std::fs::read_to_string(&input_path) {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e).with_context(|| format!("failed to read {}", input_path.display())),
        };
        let input: LayoutInput = serde_json::from_str(&json)
            .with_context(|| format!("failed to parse {}", input_path.display()))?;

        jobs.push(RenderJob { name, node: input.node, palette: input.palette });
    }

    Ok(jobs)
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let filter: Vec<String> = args[1..].to_vec();
    let testdata_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../testdata")
        .canonicalize()
        .context("cannot find testdata directory")?;

    let jobs = load_jobs(&testdata_dir, &filter)?;
    render_to_images(jobs, testdata_dir);
    Ok(())
}
