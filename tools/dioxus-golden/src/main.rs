//! Headless Dioxus renderer for flexplore golden tests.
//!
//! Reads `testdata/{case}/input.json`, generates the HTML a Dioxus webview
//! would lay out (divs with inline CSS, see `golden_common::html`), captures
//! a headless Chromium screenshot, and saves `rendered_dioxus.png`.

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use golden_common::{chrome, html};

/// Render flexplore golden screenshots with Dioxus.
#[derive(Parser)]
#[command(name = "dioxus-golden")]
struct Arguments {
    /// Testdata directory (default: the repository's testdata/).
    #[arg(long, default_value_os_t = golden_common::default_testdata_dir())]
    testdata: PathBuf,

    /// Only render these test cases (default: all).
    cases: Vec<String>,
}

fn main() -> Result<()> {
    let cli = Arguments::parse();
    let jobs = golden_common::load_jobs(&cli.testdata, &cli.cases)?;
    if jobs.is_empty() {
        eprintln!("No render jobs found in {}.", cli.testdata.display());
        return Ok(());
    }
    eprintln!(
        "Will render {} case(s) from {}",
        jobs.len(),
        cli.testdata.display()
    );

    let browser = chrome::launch_browser()?;
    let tab = browser.new_tab()?;
    chrome::set_viewport(&tab)?;

    for job in &jobs {
        let page = html::generate_html(&job.node, job.palette, &html::DIOXUS);
        chrome::screenshot_html(
            &tab,
            &page,
            &job.output_path("_tmp_dioxus.html"),
            &job.output_path("rendered_dioxus.png"),
        )?;
        eprintln!("  Saved: {}/rendered_dioxus.png", job.name);
    }

    eprintln!("All done!");
    Ok(())
}
