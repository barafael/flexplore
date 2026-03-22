use std::path::PathBuf;

use anyhow::Result;
use flexplore_core::codegen::{
    emit_bevy_code, emit_dioxus, emit_egui, emit_flutter, emit_html_css, emit_iced, emit_react,
    emit_react_native, emit_swiftui, emit_tailwind,
};
use flexplore_core::config::LayoutInput;
use flexplore_core::fixtures::all_fixtures;

fn testdata_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("testdata")
}

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

        let targets: Vec<(&str, String)> = vec![
            ("expected.html", emit_html_css(&f.node, f.palette)?),
            ("expected.rs", emit_bevy_code(&f.node, f.palette)?),
            ("expected.jsx", emit_react(&f.node, f.palette)?),
            ("expected.tailwind.html", emit_tailwind(&f.node, f.palette)?),
            ("expected.swift", emit_swiftui(&f.node, f.palette)?),
            ("expected.dart", emit_flutter(&f.node, f.palette)?),
            ("expected.iced.rs", emit_iced(&f.node, f.palette)?),
            ("expected.rn.jsx", emit_react_native(&f.node, f.palette)?),
            ("expected.dioxus.rs", emit_dioxus(&f.node, f.palette)?),
            ("expected.egui.rs", emit_egui(&f.node, f.palette)?),
        ];

        for (filename, content) in &targets {
            std::fs::write(case_dir.join(filename), content)?;
        }

        eprintln!("  updated: {}", f.name);
    }

    eprintln!("done — {} fixtures updated", fixtures.len());
    Ok(())
}
