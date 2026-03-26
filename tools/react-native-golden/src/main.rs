//! Headless React Native renderer for flexplore golden tests.
//!
//! Reads `testdata/{case}/input.json`, generates HTML that matches what
//! `react-native-web` renders (div elements with Yoga-style CSS defaults),
//! captures a headless Chrome screenshot, and saves `rendered_react_native.png`.
//!
//! React Native's layout engine (Yoga) uses different defaults from CSS:
//!   - flexDirection: column  (CSS default: row)
//!   - flexShrink: 0          (CSS default: 1)
//!   - alignContent: flex-start (CSS default: stretch)
//!
//! We emit explicit CSS to reproduce these semantics in a browser.

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Result};
use clap::Parser;

mod config;
use config::{ColorPalette, LayoutInput, NodeConfig, ValueConfig};

use headless_chrome::browser::tab::Tab;
use headless_chrome::protocol::cdp::Emulation::SetDeviceMetricsOverride;
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;

const VIEWPORT_W: u32 = 400;
const VIEWPORT_H: u32 = 300;

/// Render flexplore golden screenshots for React Native.
#[derive(Parser)]
#[command(name = "react-native-golden")]
struct Arguments {
    /// Path to the testdata directory.
    #[arg(default_value = "testdata")]
    testdata: PathBuf,

    /// Only render these test cases (default: all).
    cases: Vec<String>,
}

fn main() -> Result<()> {
    let cli = Arguments::parse();
    let filter: Vec<&str> = cli.cases.iter().map(|s| s.as_str()).collect();

    let jobs = load_jobs(&cli.testdata, &filter)?;
    if jobs.is_empty() {
        eprintln!("No render jobs found.");
        return Ok(());
    }
    eprintln!(
        "Will render {} case(s) from {}",
        jobs.len(),
        cli.testdata.display()
    );

    let browser = launch_browser()?;
    let tab = browser.new_tab()?;
    set_viewport(&tab)?;

    for job in &jobs {
        let html = generate_html(&job.node, job.palette);
        let out = job
            .output_dir
            .join(&job.name)
            .join("rendered_react_native.png");

        let tmp_html = job
            .output_dir
            .join(&job.name)
            .join("_tmp_react_native.html");
        std::fs::write(&tmp_html, &html)
            .with_context(|| format!("failed to write {}", tmp_html.display()))?;

        let url = path_to_file_url(&tmp_html)?;
        tab.navigate_to(&url)?;
        tab.wait_until_navigated()?;

        // Brief pause for layout
        std::thread::sleep(Duration::from_millis(100));

        let png = tab.capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, true)?;
        std::fs::write(&out, png).with_context(|| format!("failed to write {}", out.display()))?;

        let _ = std::fs::remove_file(&tmp_html);

        eprintln!("  Saved: {}/rendered_react_native.png", job.name);
    }

    eprintln!("All done!");
    Ok(())
}

// ─── Browser helpers ─────────────────────────────────────────────────────────

fn launch_browser() -> Result<headless_chrome::Browser> {
    let options = headless_chrome::LaunchOptions {
        window_size: Some((VIEWPORT_W, VIEWPORT_H)),
        headless: true,
        ..Default::default()
    };
    headless_chrome::Browser::new(options).context("failed to launch Chromium")
}

fn set_viewport(tab: &Tab) -> Result<()> {
    tab.call_method(SetDeviceMetricsOverride {
        width: VIEWPORT_W,
        height: VIEWPORT_H,
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

fn path_to_file_url(path: &Path) -> Result<String> {
    let canonical = path.canonicalize()?;
    let s = canonical.to_string_lossy().replace('\\', "/");
    let s = s.strip_prefix("//?/").unwrap_or(&s);
    Ok(format!("file:///{s}"))
}

// ─── Job loading ─────────────────────────────────────────────────────────────

struct RenderJob {
    name: String,
    node: NodeConfig,
    palette: ColorPalette,
    output_dir: PathBuf,
}

fn load_jobs(testdata_dir: &Path, filter: &[&str]) -> Result<Vec<RenderJob>> {
    let mut jobs = Vec::new();

    let mut entries: Vec<_> = std::fs::read_dir(testdata_dir)
        .context("cannot read testdata directory")?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();

        if !filter.is_empty() && !filter.iter().any(|f| *f == name) {
            continue;
        }

        let input_path = entry.path().join("input.json");
        if !input_path.exists() {
            continue;
        }

        let json = std::fs::read_to_string(&input_path)
            .with_context(|| format!("failed to read {}", input_path.display()))?;
        let input: LayoutInput = serde_json::from_str(&json)
            .with_context(|| format!("failed to parse {}", input_path.display()))?;

        jobs.push(RenderJob {
            name,
            node: input.node,
            palette: input.palette,
            output_dir: testdata_dir.to_path_buf(),
        });
    }

    Ok(jobs)
}

// ─── HTML generation ─────────────────────────────────────────────────────────
//
// Generates HTML that matches what react-native-web renders. React Native uses
// Yoga under the hood, which differs from CSS flexbox defaults. We apply the
// Yoga defaults explicitly so the browser reproduces RN layout:
//
//   - Every element is display:flex (RN View is always a flex container)
//   - flexDirection: column (CSS default is row)
//   - flexShrink: 0 (CSS default is 1)
//   - alignContent: flex-start (CSS default is stretch)
//   - box-sizing: border-box (always in RN)

fn generate_html(root: &NodeConfig, palette: ColorPalette) -> String {
    let mut buf = String::from(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<style>html,body{margin:0;padding:0;height:100%;width:100%;background:rgba(28,28,43,1)}</style>
</head>
<body>
"#,
    );
    emit_node(&mut buf, root, 0, &mut 0, palette);
    buf.push_str("\n</body>\n</html>\n");
    buf
}

fn emit_node(
    buf: &mut String,
    node: &NodeConfig,
    depth: usize,
    leaf_idx: &mut usize,
    palette: ColorPalette,
) {
    let pad = "  ".repeat(depth);
    let is_leaf = node.children.is_empty();

    let bg = if is_leaf {
        let (r, g, b) = palette_color(palette, *leaf_idx);
        *leaf_idx += 1;
        format!(
            "rgb({}, {}, {})",
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
        )
    } else {
        "rgba(28, 28, 43, 1)".into()
    };

    let mut styles = Vec::new();

    // RN Views are always flex containers
    styles.push("display: flex".into());

    // RN defaults: flexDirection column, flexShrink 0, alignContent flex-start
    // We always emit flexDirection because CSS default (row) differs from RN (column).
    styles.push(format!(
        "flex-direction: {}",
        css_direction(node.flex_direction)
    ));

    if !node.visible {
        styles.push("opacity: 0".into());
    }
    if !matches!(node.flex_wrap, config::FlexWrap::NoWrap) {
        styles.push(format!("flex-wrap: {}", css_wrap(node.flex_wrap)));
    }
    if !matches!(
        node.justify_content,
        config::JustifyContent::Default
            | config::JustifyContent::FlexStart
            | config::JustifyContent::Start
    ) {
        styles.push(format!(
            "justify-content: {}",
            css_justify(node.justify_content)
        ));
    }
    if !matches!(
        node.align_items,
        config::AlignItems::Default | config::AlignItems::Stretch
    ) {
        styles.push(format!(
            "align-items: {}",
            css_align_items(node.align_items)
        ));
    }
    // RN default alignContent is flex-start (not stretch like CSS)
    if !matches!(
        node.align_content,
        config::AlignContent::Default
            | config::AlignContent::FlexStart
            | config::AlignContent::Start
    ) {
        styles.push(format!(
            "align-content: {}",
            css_align_content(node.align_content)
        ));
    } else {
        // Override CSS default (stretch) with RN default (flex-start)
        styles.push("align-content: flex-start".into());
    }
    if !matches!(node.row_gap, ValueConfig::Auto)
        && !matches!(node.row_gap, ValueConfig::Px(v) if v == 0.0)
    {
        styles.push(format!("row-gap: {}", css_value(&node.row_gap)));
    }
    if !matches!(node.column_gap, ValueConfig::Auto)
        && !matches!(node.column_gap, ValueConfig::Px(v) if v == 0.0)
    {
        styles.push(format!("column-gap: {}", css_value(&node.column_gap)));
    }
    if node.flex_grow != 0.0 {
        styles.push(format!("flex-grow: {}", format_num(node.flex_grow)));
    }
    // RN default flexShrink is 0 (CSS default is 1). Always emit to override CSS.
    styles.push(format!("flex-shrink: {}", format_num(node.flex_shrink)));
    if !matches!(node.flex_basis, ValueConfig::Auto) {
        styles.push(format!("flex-basis: {}", css_value(&node.flex_basis)));
    }
    if !matches!(node.align_self, config::AlignSelf::Auto) {
        styles.push(format!("align-self: {}", css_align_self(node.align_self)));
    }
    if !matches!(node.width, ValueConfig::Auto) {
        styles.push(format!("width: {}", css_value(&node.width)));
    }
    if !matches!(node.height, ValueConfig::Auto) {
        styles.push(format!("height: {}", css_value(&node.height)));
    }
    if !matches!(node.min_width, ValueConfig::Auto) {
        styles.push(format!("min-width: {}", css_value(&node.min_width)));
    }
    if !matches!(node.min_height, ValueConfig::Auto) {
        styles.push(format!("min-height: {}", css_value(&node.min_height)));
    }
    if !matches!(node.max_width, ValueConfig::Auto) {
        styles.push(format!("max-width: {}", css_value(&node.max_width)));
    }
    if !matches!(node.max_height, ValueConfig::Auto) {
        styles.push(format!("max-height: {}", css_value(&node.max_height)));
    }
    if !matches!(node.padding.first(), ValueConfig::Px(v) if v == 0.0) {
        styles.push(format!("padding: {}", css_value(&node.padding.first())));
    }
    if !matches!(node.margin.first(), ValueConfig::Px(v) if v == 0.0) {
        styles.push(format!("margin: {}", css_value(&node.margin.first())));
    }
    styles.push(format!("background: {bg}"));
    styles.push("box-sizing: border-box".into());
    if is_leaf {
        styles.push("color: rgba(13, 13, 26, 0.85)".into());
        styles.push("font-size: 26px".into());
    }

    let style_str = styles.join("; ");

    if is_leaf {
        buf.push_str(&format!(
            "{pad}<div style=\"{style_str}\">{}</div>\n",
            node.label
        ));
    } else {
        buf.push_str(&format!("{pad}<div style=\"{style_str}\">\n"));
        // RN has no CSS `order` — children are always in source order.
        // The codegen sorts children by order, so we do the same here.
        let mut sorted: Vec<&NodeConfig> = node.children.iter().collect();
        sorted.sort_by_key(|c| c.order);
        for child in sorted {
            emit_node(buf, child, depth + 1, leaf_idx, palette);
        }
        buf.push_str(&format!("{pad}</div>\n"));
    }
}

// ─── CSS value helpers ───────────────────────────────────────────────────────

fn format_num(v: f32) -> String {
    if (v - v.round()).abs() < 0.005 {
        format!("{}", v as i32)
    } else {
        format!("{v:.1}")
    }
}

fn css_value(v: &ValueConfig) -> String {
    match v {
        ValueConfig::Auto => "auto".into(),
        ValueConfig::Px(n) => format!("{n:.1}px"),
        ValueConfig::Percent(n) => format!("{n:.1}%"),
        ValueConfig::Vw(n) => format!("{n:.1}vw"),
        ValueConfig::Vh(n) => format!("{n:.1}vh"),
    }
}

fn css_direction(d: config::FlexDirection) -> &'static str {
    match d {
        config::FlexDirection::Row => "row",
        config::FlexDirection::Column => "column",
        config::FlexDirection::RowReverse => "row-reverse",
        config::FlexDirection::ColumnReverse => "column-reverse",
    }
}

fn css_wrap(w: config::FlexWrap) -> &'static str {
    match w {
        config::FlexWrap::NoWrap => "nowrap",
        config::FlexWrap::Wrap => "wrap",
        config::FlexWrap::WrapReverse => "wrap-reverse",
    }
}

fn css_justify(j: config::JustifyContent) -> &'static str {
    match j {
        config::JustifyContent::FlexStart => "flex-start",
        config::JustifyContent::FlexEnd => "flex-end",
        config::JustifyContent::Center => "center",
        config::JustifyContent::SpaceBetween => "space-between",
        config::JustifyContent::SpaceAround => "space-around",
        config::JustifyContent::SpaceEvenly => "space-evenly",
        config::JustifyContent::Stretch => "stretch",
        config::JustifyContent::Start => "start",
        config::JustifyContent::End => "end",
        config::JustifyContent::Default => "flex-start",
    }
}

fn css_align_items(a: config::AlignItems) -> &'static str {
    match a {
        config::AlignItems::FlexStart => "flex-start",
        config::AlignItems::FlexEnd => "flex-end",
        config::AlignItems::Center => "center",
        config::AlignItems::Baseline => "baseline",
        config::AlignItems::Stretch => "stretch",
        config::AlignItems::Start => "start",
        config::AlignItems::End => "end",
        config::AlignItems::Default => "stretch",
    }
}

fn css_align_content(a: config::AlignContent) -> &'static str {
    match a {
        config::AlignContent::FlexStart => "flex-start",
        config::AlignContent::FlexEnd => "flex-end",
        config::AlignContent::Center => "center",
        config::AlignContent::SpaceBetween => "space-between",
        config::AlignContent::SpaceAround => "space-around",
        config::AlignContent::SpaceEvenly => "space-evenly",
        config::AlignContent::Stretch => "stretch",
        config::AlignContent::Start => "start",
        config::AlignContent::End => "end",
        config::AlignContent::Default => "flex-start",
    }
}

fn css_align_self(a: config::AlignSelf) -> &'static str {
    match a {
        config::AlignSelf::Auto => "auto",
        config::AlignSelf::FlexStart => "flex-start",
        config::AlignSelf::FlexEnd => "flex-end",
        config::AlignSelf::Center => "center",
        config::AlignSelf::Baseline => "baseline",
        config::AlignSelf::Stretch => "stretch",
        config::AlignSelf::Start => "start",
        config::AlignSelf::End => "end",
    }
}

// ─── Palette colors (mirrors flexplore's art::palette_color) ────────────────

fn palette_color(palette: ColorPalette, idx: usize) -> (f32, f32, f32) {
    let c = match palette {
        ColorPalette::Pastel1 => colorous::PASTEL1[idx % colorous::PASTEL1.len()],
        ColorPalette::Pastel2 => colorous::PASTEL2[idx % colorous::PASTEL2.len()],
        ColorPalette::Set1 => colorous::SET1[idx % colorous::SET1.len()],
        ColorPalette::Set2 => colorous::SET2[idx % colorous::SET2.len()],
        ColorPalette::Set3 => colorous::SET3[idx % colorous::SET3.len()],
        ColorPalette::Tableau10 => colorous::TABLEAU10[idx % colorous::TABLEAU10.len()],
        ColorPalette::Category10 => colorous::CATEGORY10[idx % colorous::CATEGORY10.len()],
        ColorPalette::Accent => colorous::ACCENT[idx % colorous::ACCENT.len()],
        ColorPalette::Dark2 => colorous::DARK2[idx % colorous::DARK2.len()],
        ColorPalette::Paired => colorous::PAIRED[idx % colorous::PAIRED.len()],
    };
    (c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0)
}
