use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use clap::Parser;
use regex::Regex;

/// Regenerate golden files, render every backend, and build the HTML overview.
#[derive(Parser)]
#[command(name = "build-overview")]
struct Cli {
    /// Only render these test cases (default: all).
    cases: Vec<String>,

    /// Only run these backends (e.g. --backend swift --backend html).
    /// Default: all backends.
    #[arg(long = "backend")]
    backends: Vec<String>,
}

const VIEWPORT_W: f64 = 400.0;
const VIEWPORT_H: f64 = 300.0;
const INDENT: &str = "  ";

const TAILWIND_HEADER: &str = r#"<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<script src="https://cdn.tailwindcss.com"></script>
<style>html,body{margin:0;height:100%}body{display:flex;flex-direction:column;align-items:flex-start}</style>
</head>
<body>
"#;

const TAILWIND_FOOTER: &str = r#"<div id="tw-ready" style="position:fixed;bottom:0;right:0;width:1px;height:1px;pointer-events:none"></div>
</body>
</html>
"#;

const OVERVIEW_IMAGES: &[(&str, &str)] = &[
    ("Bevy", "rendered_bevy.png"),
    ("HTML/CSS", "rendered_html.png"),
    ("Tailwind", "rendered_tailwind.png"),
    ("Flutter", "rendered_flutter.png"),
    ("SwiftUI", "rendered_swift.png"),
    ("Iced", "rendered_iced.png"),
];

fn main() -> Result<()> {
    let cli = Cli::parse();
    let filter = cli.cases;
    let backends = cli.backends;

    let run_backend =
        |name: &str| backends.is_empty() || backends.iter().any(|b| b.eq_ignore_ascii_case(name));

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
    let testdata = root.join("testdata");
    let tools = root.join("tools");

    check_wsl()?;

    run_cmd(
        "Regenerating golden files",
        Command::new("cargo")
            .args(["run", "-p", "update-snapshots"])
            .current_dir(&root),
    )?;

    if run_backend("bevy") {
        run_cmd(
            "Rendering Bevy screenshots",
            Command::new("cargo")
                .args(["run", "-p", "bevy-golden", "--"])
                .args(&filter)
                .current_dir(&root),
        )?;
    }

    if run_backend("html") {
        render_html(&testdata, &filter)?;
    }

    if run_backend("tailwind") {
        render_tailwind(&testdata, &filter)?;
    }

    if run_backend("flutter") {
        if tool_available("flutter") {
            render_flutter(&testdata, &tools, &filter)?;
        } else {
            eprintln!("  SKIP: flutter not found");
        }
    }

    if run_backend("swift") {
        if tool_available("swift") {
            render_swift(&testdata, &tools, &filter)?;
        } else {
            eprintln!("  SKIP: swift not found (requires macOS)");
        }
    }

    if run_backend("iced") {
        render_iced(&testdata, &root, &filter)?;
    }

    build_overview(&testdata)?;

    eprintln!();
    eprintln!("Done! Open testdata/overview.html to view the comparison.");
    Ok(())
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// On Windows, script-based CLI tools (npx, flutter) need their `.cmd`/`.bat`
/// wrapper to be invoked directly. This returns the platform-appropriate name.
fn script_cmd(base: &str) -> String {
    if !cfg!(windows) {
        return base.to_string();
    }
    for ext in ["cmd", "bat"] {
        let candidate = format!("{base}.{ext}");
        if which(&candidate) {
            return candidate;
        }
    }
    base.to_string()
}

fn which(name: &str) -> bool {
    Command::new(if cfg!(windows) { "where" } else { "which" })
        .arg(name)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn run_cmd(label: &str, cmd: &mut Command) -> Result<()> {
    eprintln!(">>> {label}");
    let status = cmd.status().with_context(|| format!("failed to launch: {label}"))?;
    if !status.success() {
        bail!("{label} failed with {status}");
    }
    Ok(())
}

fn tool_available(name: &str) -> bool {
    Command::new(script_cmd(name))
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn check_wsl() -> Result<()> {
    if let Ok(version) = fs::read_to_string("/proc/version") {
        if version.to_lowercase().contains("microsoft") {
            bail!(
                "This tool must run from Git Bash, not WSL.\n\
                 Bevy rendering requires native GPU access (DX12)."
            );
        }
    }
    Ok(())
}

fn list_cases(
    testdata: &Path,
    filter: &[String],
    require_file: Option<&str>,
) -> Result<Vec<String>> {
    let mut cases: Vec<String> = fs::read_dir(testdata)
        .context("cannot read testdata directory")?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .filter(|e| {
            require_file.map_or(true, |f| e.path().join(f).exists())
        })
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|name| filter.is_empty() || filter.contains(name))
        .collect();
    cases.sort();
    Ok(cases)
}

fn path_to_file_url(path: &Path) -> Result<String> {
    let canonical = path.canonicalize()?;
    let s = canonical.to_string_lossy().replace('\\', "/");
    // Strip the \\?\ prefix that Windows canonicalize adds
    let s = s.strip_prefix("//?/").unwrap_or(&s);
    Ok(format!("file:///{s}"))
}

use headless_chrome::browser::tab::Tab;
use headless_chrome::protocol::cdp::Emulation::SetDeviceMetricsOverride;
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;

/// Launch a headless Chromium browser for screenshot capture.
fn launch_browser() -> Result<headless_chrome::Browser> {
    let options = headless_chrome::LaunchOptions {
        window_size: Some((VIEWPORT_W as u32, VIEWPORT_H as u32)),
        headless: true,
        ..Default::default()
    };
    headless_chrome::Browser::new(options).context("failed to launch Chromium")
}

/// Set the exact viewport size on a tab (no scrollbars, no window chrome).
fn set_viewport(tab: &Tab) -> Result<()> {
    tab.call_method(SetDeviceMetricsOverride {
        width: VIEWPORT_W as u32,
        height: VIEWPORT_H as u32,
        device_scale_factor: 1.0,
        mobile: false,
        screen_orientation: None,
        scale: None,
        screen_height: None,
        screen_width: None,
        position_x: None,
        position_y: None,
        dont_set_visible_size: None,
        viewport: None,
        display_feature: None,
        device_posture: None,
    })?;
    Ok(())
}

/// Navigate a tab to a URL, optionally wait for a selector, and save a screenshot.
fn screenshot_tab(tab: &Tab, url: &str, out: &Path, wait_for: Option<&str>) -> Result<()> {
    tab.navigate_to(url)?;
    tab.wait_until_navigated()?;
    if let Some(selector) = wait_for {
        tab.wait_for_element_with_custom_timeout(selector, std::time::Duration::from_secs(15))
            .with_context(|| format!("timed out waiting for {selector}"))?;
    }
    let png = tab.capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, true)?;
    fs::write(out, png).with_context(|| format!("failed to write {}", out.display()))?;
    Ok(())
}

// ─── Renderers ────────────────────────────────────────────────────────────────

fn render_html(testdata: &Path, filter: &[String]) -> Result<()> {
    let cases = list_cases(testdata, filter, None)?;
    eprintln!(">>> Rendering HTML screenshots ({} cases)", cases.len());
    if cases.is_empty() {
        return Ok(());
    }

    let browser = launch_browser()?;
    let tab = browser.new_tab()?;
    set_viewport(&tab)?;

    for name in &cases {
        let html_file = testdata.join(name).join("expected.html");
        if !html_file.exists() {
            eprintln!("  SKIP: {name} (no expected.html)");
            continue;
        }
        let out = testdata.join(name).join("rendered_html.png");
        let url = path_to_file_url(&html_file)?;
        screenshot_tab(&tab, &url, &out, None)?;
        eprintln!("  Saved: {name}/rendered_html.png");
    }
    Ok(())
}

fn render_tailwind(testdata: &Path, filter: &[String]) -> Result<()> {
    let cases = list_cases(testdata, filter, None)?;
    eprintln!(">>> Rendering Tailwind screenshots ({} cases)", cases.len());
    if cases.is_empty() {
        return Ok(());
    }

    let browser = launch_browser()?;
    let tab = browser.new_tab()?;
    set_viewport(&tab)?;
    let mut tmp_files = Vec::new();

    for name in &cases {
        let tw_file = testdata.join(name).join("expected.tailwind.html");
        if !tw_file.exists() {
            eprintln!("  SKIP: {name} (no expected.tailwind.html)");
            continue;
        }
        let tmp_file = testdata.join(name).join("_tmp_tailwind.html");

        let fragment = fs::read_to_string(&tw_file)
            .with_context(|| format!("failed to read {}", tw_file.display()))?;
        fs::write(&tmp_file, format!("{TAILWIND_HEADER}{fragment}{TAILWIND_FOOTER}"))
            .with_context(|| format!("failed to write {}", tmp_file.display()))?;

        let url = path_to_file_url(&tmp_file)?;
        let out = testdata.join(name).join("rendered_tailwind.png");
        let result = screenshot_tab(&tab, &url, &out, Some("#tw-ready"));
        tmp_files.push(tmp_file);

        match result {
            Ok(()) => eprintln!("  Saved: {name}/rendered_tailwind.png"),
            Err(e) => eprintln!("  ERROR: {name}: {e:#}"),
        }
    }

    for f in &tmp_files {
        let _ = fs::remove_file(f);
    }
    Ok(())
}

fn render_flutter(testdata: &Path, tools: &Path, filter: &[String]) -> Result<()> {
    let flutter_dir = tools.join("flutter-golden");

    // Copy Roboto font from Flutter SDK if needed
    if let Ok(output) = Command::new(script_cmd("flutter")).args(["--no-color", "sdk-path"]).output() {
        let sdk = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
        let font_src = sdk.join("bin/cache/artifacts/material_fonts/roboto-regular.ttf");
        let font_dst = flutter_dir.join("fonts/Roboto-Regular.ttf");
        if font_src.exists() && !font_dst.exists() {
            fs::create_dir_all(font_dst.parent().unwrap())?;
            fs::copy(&font_src, &font_dst)?;
            eprintln!("  Copied Roboto font from Flutter SDK");
        }
    }

    eprintln!(">>> Regenerating Flutter widget files");
    generate_flutter_cases(testdata, &flutter_dir)?;

    run_cmd(
        "Running Flutter golden tests",
        Command::new(script_cmd("flutter"))
            .args(["test", "--update-goldens", "test/golden_test.dart"])
            .current_dir(&flutter_dir),
    )?;

    // Copy goldens to testdata
    eprintln!(">>> Copying Flutter goldens to testdata");
    let goldens_dir = flutter_dir.join("test/goldens");
    let cases = list_cases(testdata, filter, None)?;
    for name in &cases {
        let src = goldens_dir.join(format!("{name}.png"));
        let dst = testdata.join(name).join("rendered_flutter.png");
        if src.exists() {
            fs::copy(&src, &dst)?;
            eprintln!("  Copied: {name}/rendered_flutter.png");
        }
    }
    Ok(())
}

fn render_swift(testdata: &Path, tools: &Path, filter: &[String]) -> Result<()> {
    let swift_dir = tools.join("swift-golden");

    eprintln!(">>> Regenerating Swift view files");
    generate_swift_cases(testdata, &swift_dir)?;

    // Record mode always "fails" tests by design — ignore the exit code.
    eprintln!(">>> Running Swift snapshot tests");
    Command::new("swift")
        .arg("test")
        .current_dir(&swift_dir)
        .env("SWIFT_SNAPSHOT_RECORD", "1")
        .status()
        .context("failed to launch swift test")?;

    // Copy snapshots to testdata
    eprintln!(">>> Copying Swift snapshots to testdata");
    let snapshots_dir = swift_dir.join("Tests/SwiftGoldenTests/__Snapshots__/GoldenTests");
    let cases = list_cases(testdata, filter, None)?;
    for name in &cases {
        let src = snapshots_dir.join(format!("test_{name}.1.png"));
        let dst = testdata.join(name).join("rendered_swift.png");
        if src.exists() {
            fs::copy(&src, &dst)?;
            eprintln!("  Copied: {name}/rendered_swift.png");
        }
    }
    Ok(())
}

fn render_iced(testdata: &Path, root: &Path, filter: &[String]) -> Result<()> {
    let iced_dir = root.join("tools/iced-golden");

    run_cmd(
        "Building Iced golden renderer",
        Command::new("cargo")
            .args(["build", "--release", "--manifest-path"])
            .arg(iced_dir.join("Cargo.toml")),
    )?;

    let mut cmd = Command::new(iced_dir.join("target/release/iced-golden"));
    cmd.arg(testdata);
    cmd.args(filter);
    run_cmd("Rendering Iced screenshots", &mut cmd)?;
    Ok(())
}

// ─── Overview HTML ────────────────────────────────────────────────────────────

fn build_overview(testdata: &Path) -> Result<()> {
    eprintln!(">>> Building overview page");

    let all_cases = list_cases(testdata, &[], None)?;
    let cases: Vec<String> = all_cases
        .into_iter()
        .filter(|name| {
            OVERVIEW_IMAGES
                .iter()
                .any(|(_, f)| testdata.join(name).join(f).exists())
        })
        .collect();

    let mut html = String::from(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Flexplore Render Overview</title>
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body { font-family: system-ui, sans-serif; background: #111; color: #ddd; padding: 24px; }
    h1 { text-align: center; margin-bottom: 24px; }
    .case { margin-bottom: 32px; border: 1px solid #333; border-radius: 8px; overflow: hidden; }
    .case h2 { background: #1a1a2e; padding: 10px 16px; font-size: 16px; }
    .images { display: flex; gap: 2px; background: #222; }
    .panel { flex: 1; min-width: 0; background: #1a1a1a; }
    .label { text-align: center; padding: 4px; font-size: 12px; font-weight: 600; background: #0f3460; }
    .panel img { width: 100%; height: auto; display: block; }
    .missing { text-align: center; padding: 40px; color: #666; font-style: italic; }
  </style>
</head>
<body>
  <h1>Flexplore Render Overview</h1>
"#,
    );

    for name in &cases {
        html.push_str(&format!("  <section class=\"case\">\n    <h2>{name}</h2>\n    <div class=\"images\">\n"));
        for (label, filename) in OVERVIEW_IMAGES {
            html.push_str(&format!("      <div class=\"panel\">\n        <div class=\"label\">{label}</div>\n"));
            if testdata.join(name).join(filename).exists() {
                html.push_str(&format!("        <img src=\"{name}/{filename}\" loading=\"lazy\">\n"));
            } else {
                html.push_str("        <div class=\"missing\">not rendered</div>\n");
            }
            html.push_str("      </div>\n");
        }
        html.push_str("    </div>\n  </section>\n");
    }

    html.push_str("</body>\n</html>\n");

    let output = testdata.join("overview.html");
    fs::write(&output, &html)?;
    eprintln!("  Written to {}", output.display());
    Ok(())
}

// ─── Codegen ──────────────────────────────────────────────────────────────────

fn snake_to_camel(s: &str) -> String {
    s.split('_')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().to_string() + c.as_str(),
            }
        })
        .collect()
}

/// Generate Dart widget files and golden test from testdata/*/expected.dart.
fn generate_flutter_cases(testdata: &Path, flutter_dir: &Path) -> Result<()> {
    let lib_dir = flutter_dir.join("lib/cases");
    let test_dir = flutter_dir.join("test");
    fs::create_dir_all(&lib_dir)?;

    let cases = list_cases(testdata, &[], Some("expected.dart"))?;

    // Generate a widget file per case
    for name in &cases {
        let dart_src = fs::read_to_string(testdata.join(name).join("expected.dart"))?;
        let class_name = snake_to_camel(name);
        let widget_code = format!(
            "// AUTO-GENERATED — do not edit. Run `cargo run -p build-overview` to regenerate.\n\
             import 'package:flutter/material.dart';\n\
             \n\
             class {class_name} extends StatelessWidget {{\n\
             {INDENT}const {class_name}({{super.key}});\n\
             \n\
             {INDENT}@override\n\
             {INDENT}{dart_src}}}\n"
        );
        fs::write(lib_dir.join(format!("{name}.dart")), widget_code)?;
    }

    // Generate barrel export
    let mut barrel = String::from("// AUTO-GENERATED\n");
    for name in &cases {
        barrel.push_str(&format!("export 'cases/{name}.dart';\n"));
    }
    fs::write(lib_dir.parent().unwrap().join("flutter_golden.dart"), barrel)?;

    // Generate golden test file
    let test_imports: String = cases
        .iter()
        .map(|name| format!("import 'package:flutter_golden/cases/{name}.dart';"))
        .collect::<Vec<_>>()
        .join("\n");

    let test_cases: String = cases
        .iter()
        .map(|name| {
            let class_name = snake_to_camel(name);
            format!(
                "  testWidgets('{name}', (tester) async {{\n\
                 {INDENT}{INDENT}tester.view.devicePixelRatio = 1.0;\n\
                 {INDENT}{INDENT}addTearDown(() => tester.view.resetDevicePixelRatio());\n\
                 {INDENT}{INDENT}await tester.binding.setSurfaceSize(const Size(400, 300));\n\
                 {INDENT}{INDENT}await tester.pumpWidget(\n\
                 {INDENT}{INDENT}{INDENT}MaterialApp(\n\
                 {INDENT}{INDENT}{INDENT}{INDENT}debugShowCheckedModeBanner: false,\n\
                 {INDENT}{INDENT}{INDENT}{INDENT}home: Scaffold(\n\
                 {INDENT}{INDENT}{INDENT}{INDENT}{INDENT}backgroundColor: const Color(0xFF1C1C2E),\n\
                 {INDENT}{INDENT}{INDENT}{INDENT}{INDENT}body: {class_name}(),\n\
                 {INDENT}{INDENT}{INDENT}{INDENT}),\n\
                 {INDENT}{INDENT}{INDENT}),\n\
                 {INDENT}{INDENT});\n\
                 {INDENT}{INDENT}// Consume any overflow errors so the golden is still captured.\n\
                 {INDENT}{INDENT}tester.takeException();\n\
                 {INDENT}{INDENT}await expectLater(\n\
                 {INDENT}{INDENT}{INDENT}find.byType(MaterialApp),\n\
                 {INDENT}{INDENT}{INDENT}matchesGoldenFile('goldens/{name}.png'),\n\
                 {INDENT}{INDENT});\n\
                 {INDENT}}});"
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let test_code = format!(
        "// AUTO-GENERATED — do not edit. Run `cargo run -p build-overview` to regenerate.\n\
         import 'dart:io';\n\
         import 'dart:ui' as ui;\n\
         \n\
         import 'package:flutter/material.dart';\n\
         import 'package:flutter_test/flutter_test.dart';\n\
         \n\
         {test_imports}\n\
         \n\
         void main() {{\n\
         {INDENT}setUpAll(() async {{\n\
         {INDENT}{INDENT}// Load Roboto so golden tests render real text instead of placeholder blocks.\n\
         {INDENT}{INDENT}final fontFile = File('fonts/Roboto-Regular.ttf');\n\
         {INDENT}{INDENT}if (fontFile.existsSync()) {{\n\
         {INDENT}{INDENT}{INDENT}await ui.loadFontFromList(fontFile.readAsBytesSync(), fontFamily: 'Roboto');\n\
         {INDENT}{INDENT}}}\n\
         {INDENT}}});\n\
         \n\
         {test_cases}\n\
         }}\n"
    );
    fs::write(test_dir.join("golden_test.dart"), test_code)?;

    eprintln!(
        "  Generated {} widget files in {}",
        cases.len(),
        lib_dir.display()
    );
    eprintln!(
        "  Generated golden test in {}",
        test_dir.join("golden_test.dart").display()
    );
    Ok(())
}

/// Generate Swift view files and snapshot test from testdata/*/expected.swift.
fn generate_swift_cases(testdata: &Path, swift_dir: &Path) -> Result<()> {
    let cases_dir = swift_dir.join("Sources/SwiftGolden/Cases");
    let test_dir = swift_dir.join("Tests/SwiftGoldenTests");
    fs::create_dir_all(&cases_dir)?;

    let cases = list_cases(testdata, &[], Some("expected.swift"))?;

    let re_width = Regex::new(r"UIScreen\.main\.bounds\.width\s*\*\s*([\d.]+)")?;
    let re_height = Regex::new(r"UIScreen\.main\.bounds\.height\s*\*\s*([\d.]+)")?;

    // Generate a view file per case
    for name in &cases {
        let swift_src = fs::read_to_string(testdata.join(name).join("expected.swift"))?;
        let class_name = snake_to_camel(name);

        // Adapt: rename ContentView, replace UIScreen references with constants
        let adapted = swift_src.replace(
            "struct ContentView:",
            &format!("public struct {class_name}View:"),
        );
        let adapted = re_width
            .replace_all(&adapted, |caps: &regex::Captures| {
                let factor: f64 = caps[1].parse().unwrap_or(1.0);
                format!("{:.1}", VIEWPORT_W * factor)
            })
            .into_owned();
        let adapted = re_height
            .replace_all(&adapted, |caps: &regex::Captures| {
                let factor: f64 = caps[1].parse().unwrap_or(1.0);
                format!("{:.1}", VIEWPORT_H * factor)
            })
            .into_owned();

        let view_code = format!(
            "// AUTO-GENERATED — do not edit. Run `cargo run -p build-overview` to regenerate.\n\
             import SwiftUI\n\n\
             {adapted}"
        );
        fs::write(cases_dir.join(format!("{name}.swift")), view_code)?;
    }

    // Generate snapshot test file
    let test_funcs: String = cases
        .iter()
        .map(|name| {
            let class_name = snake_to_camel(name);
            format!(
                "    func test_{name}() {{\n\
                 {INDENT}{INDENT}{INDENT}let view = NSHostingController(rootView:\n\
                 {INDENT}{INDENT}{INDENT}{INDENT}{class_name}View()\n\
                 {INDENT}{INDENT}{INDENT}{INDENT}{INDENT}.frame(width: {VIEWPORT_W:.0}, height: {VIEWPORT_H:.0}, alignment: .topLeading)\n\
                 {INDENT}{INDENT}{INDENT}{INDENT}{INDENT}.background(Color(red: 0.11, green: 0.11, blue: 0.18))\n\
                 {INDENT}{INDENT}{INDENT})\n\
                 {INDENT}{INDENT}{INDENT}assertSnapshot(of: view, as: .image(size: CGSize(width: {VIEWPORT_W:.0}, height: {VIEWPORT_H:.0})))\n\
                 {INDENT}{INDENT}}}"
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let test_code = format!(
        "// AUTO-GENERATED — do not edit. Run `cargo run -p build-overview` to regenerate.\n\
         import XCTest\n\
         import SwiftUI\n\
         import AppKit\n\
         import SnapshotTesting\n\
         \n\
         @testable import SwiftGolden\n\
         \n\
         final class GoldenTests: XCTestCase {{\n\
         {INDENT}override func invokeTest() {{\n\
         {INDENT}{INDENT}// When SWIFT_SNAPSHOT_RECORD=1, always (re-)generate golden PNGs.\n\
         {INDENT}{INDENT}if ProcessInfo.processInfo.environment[\"SWIFT_SNAPSHOT_RECORD\"] == \"1\" {{\n\
         {INDENT}{INDENT}{INDENT}withSnapshotTesting(record: .all) {{\n\
         {INDENT}{INDENT}{INDENT}{INDENT}super.invokeTest()\n\
         {INDENT}{INDENT}{INDENT}}}\n\
         {INDENT}{INDENT}}} else {{\n\
         {INDENT}{INDENT}{INDENT}super.invokeTest()\n\
         {INDENT}{INDENT}}}\n\
         {INDENT}}}\n\
         \n\
         {test_funcs}\n\
         }}\n"
    );
    fs::write(test_dir.join("GoldenTests.swift"), test_code)?;

    eprintln!(
        "  Generated {} view files in {}",
        cases.len(),
        cases_dir.display()
    );
    eprintln!(
        "  Generated snapshot test in {}",
        test_dir.join("GoldenTests.swift").display()
    );
    Ok(())
}
