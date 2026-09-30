//! Headless Chromium helpers shared by the HTML-based backends.

use std::{
    path::Path,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
pub use headless_chrome;
use headless_chrome::{
    Browser, LaunchOptions,
    browser::tab::Tab,
    protocol::cdp::{Emulation::SetDeviceMetricsOverride, Page::CaptureScreenshotFormatOption},
};

use crate::{VIEWPORT_H, VIEWPORT_W};

/// JS expression that is truthy once the document has finished loading.
pub const DOCUMENT_COMPLETE: &str = "document.readyState === 'complete'";

/// Launch a headless Chromium sized to the golden viewport.
pub fn launch_browser() -> Result<Browser> {
    let options = LaunchOptions {
        window_size: Some((VIEWPORT_W as u32, VIEWPORT_H as u32)),
        headless: true,
        ..Default::default()
    };
    Browser::new(options).context("failed to launch Chromium")
}

/// Pin the tab's viewport to exactly the golden size at 1× DPI (no
/// scrollbars, no window chrome).
pub fn set_viewport(tab: &Tab) -> Result<()> {
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

/// `file://` URL for a local file (canonicalized first, so relative paths
/// work regardless of the working directory).
pub fn path_to_file_url(path: &Path) -> Result<String> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("cannot resolve {}", path.display()))?;
    url::Url::from_file_path(&canonical)
        .map(|u| u.into())
        .map_err(|()| anyhow::anyhow!("{} is not an absolute path", canonical.display()))
}

/// Poll `expression` in the page until it evaluates truthy or `timeout`
/// elapses.
pub fn wait_for_js(tab: &Tab, expression: &str, timeout: Duration) -> Result<()> {
    let start = Instant::now();
    loop {
        let truthy = tab
            .evaluate(&format!("Boolean({expression})"), false)?
            .value
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if truthy {
            return Ok(());
        }
        if start.elapsed() > timeout {
            bail!("timed out after {timeout:?} waiting for `{expression}` to become true");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Navigate the tab to `url`, wait until `ready` (a JS expression; defaults to
/// [`DOCUMENT_COMPLETE`]) is truthy, and write a PNG screenshot to `out`.
pub fn screenshot_tab(tab: &Tab, url: &str, out: &Path, ready: Option<&str>) -> Result<()> {
    tab.navigate_to(url)?;
    tab.wait_until_navigated()?;
    wait_for_js(
        tab,
        ready.unwrap_or(DOCUMENT_COMPLETE),
        Duration::from_secs(20),
    )?;
    let png = tab.capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, true)?;
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(out, png).with_context(|| format!("failed to write {}", out.display()))?;
    Ok(())
}

/// Write `html` to `tmp_file`, screenshot it, then delete the temp file
/// (also on failure).
pub fn screenshot_html(tab: &Tab, html: &str, tmp_file: &Path, out: &Path) -> Result<()> {
    std::fs::write(tmp_file, html)
        .with_context(|| format!("failed to write {}", tmp_file.display()))?;
    let result = path_to_file_url(tmp_file).and_then(|url| screenshot_tab(tab, &url, out, None));
    let _ = std::fs::remove_file(tmp_file);
    result
}
