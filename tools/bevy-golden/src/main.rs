use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use flexplore::render::{RenderJob, render_to_images};

/// Render flexplore golden screenshots with Bevy.
#[derive(clap::Parser)]
#[command(name = "bevy-golden")]
struct Arguments {
    /// Testdata directory (default: the repository's testdata/).
    #[arg(long, default_value_os_t = golden_common::default_testdata_dir())]
    testdata: PathBuf,

    /// Only render these test cases (default: all).
    cases: Vec<String>,
}

fn main() -> Result<()> {
    let cli = Arguments::parse();
    let jobs: Vec<RenderJob> = golden_common::load_jobs(&cli.testdata, &cli.cases)?
        .into_iter()
        .map(|job| RenderJob {
            name: job.name,
            node: job.node,
            palette: job.palette,
        })
        .collect();
    render_to_images(jobs, cli.testdata);
    Ok(())
}
