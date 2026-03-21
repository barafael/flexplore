use std::path::PathBuf;

use flexplore::codegen::{
    emit_bevy_code, emit_flutter, emit_html_css, emit_iced, emit_react, emit_swiftui,
    emit_tailwind,
};
use flexplore::fixtures::all_fixtures;

fn testdata_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("testdata")
}

fn main() {
    let fixtures = all_fixtures();
    let dir = testdata_dir();

    for f in &fixtures {
        let case_dir = dir.join(&f.name);
        std::fs::create_dir_all(&case_dir).unwrap();

        let input_json = serde_json::to_string_pretty(&f.node).unwrap();
        std::fs::write(case_dir.join("input.json"), &input_json).unwrap();

        let targets: Vec<(&str, String)> = vec![
            ("expected.html", emit_html_css(&f.node, f.palette).unwrap()),
            ("expected.rs", emit_bevy_code(&f.node, f.palette).unwrap()),
            ("expected.jsx", emit_react(&f.node, f.palette).unwrap()),
            (
                "expected.tailwind.html",
                emit_tailwind(&f.node, f.palette).unwrap(),
            ),
            (
                "expected.swift",
                emit_swiftui(&f.node, f.palette).unwrap(),
            ),
            ("expected.dart", emit_flutter(&f.node, f.palette).unwrap()),
            (
                "expected.iced.rs",
                emit_iced(&f.node, f.palette).unwrap(),
            ),
        ];

        for (filename, content) in &targets {
            std::fs::write(case_dir.join(filename), content).unwrap();
        }

        eprintln!("  updated: {}", f.name);
    }

    eprintln!("done — {} fixtures updated", fixtures.len());
}
