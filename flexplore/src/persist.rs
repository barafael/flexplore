use flexplore::config::FlexConfig;

// ─── Native persistence ──────────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
pub fn auto_save(cfg: &FlexConfig) {
    let Some(dir) = dirs::config_dir() else {
        return;
    };
    let dir = dir.join("flexplore");
    if let Err(e) = std::fs::create_dir_all(&dir) {
        bevy::log::warn!("autosave: cannot create {}: {e}", dir.display());
        return;
    }
    let path = dir.join("autosave.json");
    match serde_json::to_string_pretty(cfg) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&path, json) {
                bevy::log::warn!("autosave: cannot write {}: {e}", path.display());
            }
        }
        Err(e) => bevy::log::warn!("autosave: cannot serialize config: {e}"),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn auto_load() -> Option<FlexConfig> {
    let dir = dirs::config_dir()?.join("flexplore");
    let path = dir.join("autosave.json");
    let data = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data)
        .inspect_err(|e| bevy::log::warn!("autosave: ignoring {}: {e}", path.display()))
        .ok()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn export_json(cfg: &FlexConfig) -> Option<String> {
    serde_json::to_string_pretty(cfg).ok()
}

// ─── WASM persistence ────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

#[cfg(target_arch = "wasm32")]
pub fn auto_save(cfg: &FlexConfig) {
    let Some(storage) = local_storage() else {
        return;
    };
    match serde_json::to_string(cfg) {
        Ok(json) => {
            if storage.set_item("flexplore_config", &json).is_err() {
                bevy::log::warn!("autosave: localStorage write failed (quota or disabled)");
            }
        }
        Err(e) => bevy::log::warn!("autosave: cannot serialize config: {e}"),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn auto_load() -> Option<FlexConfig> {
    // Try URL hash first (shared layout link), then localStorage
    if let Some(cfg) = load_from_url_hash() {
        return Some(cfg);
    }
    let storage = local_storage()?;
    let json = storage.get_item("flexplore_config").ok()??;
    serde_json::from_str(&json)
        .inspect_err(|e| bevy::log::warn!("autosave: ignoring stored config: {e}"))
        .ok()
}

#[cfg(target_arch = "wasm32")]
pub fn export_json(cfg: &FlexConfig) -> Option<String> {
    serde_json::to_string_pretty(cfg).ok()
}

#[cfg(target_arch = "wasm32")]
pub fn trigger_download(json: &str) {
    use wasm_bindgen::JsCast;

    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let document = match window.document() {
        Some(d) => d,
        None => return,
    };

    let blob_parts = js_sys::Array::new();
    blob_parts.push(&wasm_bindgen::JsValue::from_str(json));

    let opts = web_sys::BlobPropertyBag::new();
    opts.set_type("application/json");

    let blob = match web_sys::Blob::new_with_str_sequence_and_options(&blob_parts, &opts) {
        Ok(b) => b,
        Err(_) => return,
    };

    let url = match web_sys::Url::create_object_url_with_blob(&blob) {
        Ok(u) => u,
        Err(_) => return,
    };

    if let Ok(el) = document.create_element("a") {
        if let Some(anchor) = el.dyn_ref::<web_sys::HtmlAnchorElement>() {
            anchor.set_href(&url);
            anchor.set_download("flexplore-layout.json");
            anchor.click();
        }
    }

    let _ = web_sys::Url::revoke_object_url(&url);
}

// ─── WASM URL sharing ────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
pub fn make_share_url(cfg: &FlexConfig) -> Option<String> {
    let json = serde_json::to_string(cfg).ok()?;
    let window = web_sys::window()?;
    // `btoa` only accepts Latin-1; percent-encode first so labels with any
    // Unicode (emoji, accents) survive the round trip.
    let ascii: String = js_sys::encode_uri_component(&json).into();
    let encoded: String = window.btoa(&ascii).ok()?;
    let location = window.location();
    let origin = location.origin().ok()?;
    let pathname = location.pathname().ok()?;
    // A layout link is a template to import, not an invite: it names no room.
    Some(format!("{origin}{pathname}#layout={encoded}"))
}

#[cfg(target_arch = "wasm32")]
fn load_from_url_hash() -> Option<FlexConfig> {
    let window = web_sys::window()?;
    let fragment = flexplore_net::read_fragment();
    let encoded = flexplore_net::fragment_param(&fragment, "layout")?;
    let decoded: String = window.atob(&encoded).ok()?;
    // New links are percent-encoded; older links carry raw JSON, which only
    // fails to decode when it contains a literal `%` (e.g. a "25%" label).
    let json: String = js_sys::decode_uri_component(&decoded)
        .map(String::from)
        .unwrap_or(decoded);
    // Drop the layout from the fragment after loading so a refresh doesn't
    // re-apply it; the room, if any, stays.
    flexplore_net::write_fragment(&flexplore_net::with_fragment_param(
        &fragment, "layout", None,
    ));
    serde_json::from_str(&json).ok()
}

/// Try to parse a JSON string as FlexConfig.
pub fn import_json(json: &str) -> Option<FlexConfig> {
    serde_json::from_str(json).ok()
}
