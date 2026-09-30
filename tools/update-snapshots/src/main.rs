use std::path::PathBuf;

use anyhow::Result;
use flexplore_core::codegen::emit_all;
use flexplore_core::config::LayoutInput;
use flexplore_core::fixtures::all_fixtures;

fn testdata_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("testdata")
}

/// Write `input.json` and one `expected.*` file per code generator for every
/// fixture. The target list lives in `flexplore_core::codegen::TARGETS`, so a
/// new generator only has to be added there.
fn main() -> Result<()> {
    let fixtures = all_fixtures();
    let dir = testdata_dir();

    for f in &fixtures {
        let case_dir = dir.join(&f.name);
        std::fs::create_dir_all(&case_dir)?;

        let input = LayoutInput {
            node: f.node.clone(),
            palette: f.palette,
        };
        let input_json = serde_json::to_string_pretty(&input)?;
        std::fs::write(case_dir.join("input.json"), &input_json)?;

        for (filename, content) in emit_all(&f.node, f.palette)? {
            std::fs::write(case_dir.join(filename), content)?;
        }

        eprintln!("  updated: {}", f.name);
    }

    eprintln!("done — {} fixtures updated", fixtures.len());
    Ok(())
}
