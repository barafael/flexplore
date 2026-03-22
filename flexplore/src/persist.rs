use flexplore::config::FlexConfig;

// ─── Native persistence ──────────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
pub fn auto_save(cfg: &FlexConfig) {
    let Some(dir) = dirs::config_dir() else {
        return;
    };
    let dir = dir.join("flexplore");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("autosave.json");
    if let Ok(json) = serde_json::to_string_pretty(cfg) {
        let _ = std::fs::write(path, json);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn auto_load() -> Option<FlexConfig> {
    let dir = dirs::config_dir()?.join("flexplore");
    let path = dir.join("autosave.json");
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
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
    if let Ok(json) = serde_json::to_string(cfg) {
        let _ = storage.set_item("flexplore_config", &json);
    }
}

#[cfg(target_arch = "wasm32")]
pub fn auto_load() -> Option<FlexConfig> {
    let storage = local_storage()?;
    let json = storage.get_item("flexplore_config").ok()??;
    serde_json::from_str(&json).ok()
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

    let mut opts = web_sys::BlobPropertyBag::new();
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

/// Try to parse a JSON string as FlexConfig.
pub fn import_json(json: &str) -> Option<FlexConfig> {
    serde_json::from_str(json).ok()
}
