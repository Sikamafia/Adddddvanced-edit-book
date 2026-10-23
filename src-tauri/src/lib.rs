//! Native layer of "Watermark Offline".
//!
//! All PDF work (parsing, Arabic shaping, font embedding, watermark drawing) stays in the
//! bundled JavaScript (pdf-lib + PDF.js) because it already works and has no drop-in Rust
//! equivalent. Rust provides: the Android file-save integration (dialog + fs plugins) and a
//! durable on-device store for settings and named presets, so they survive even if the
//! WebView storage is cleared.

use std::{fs, io::ErrorKind, path::PathBuf};
use tauri::{AppHandle, Manager};

const STATE_FILE: &str = "watermark-state.json";
const MAX_STATE_BYTES: usize = 20 * 1024 * 1024;

fn state_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join(STATE_FILE))
}

/// Returns the stored JSON text, or `None` on first launch.
#[tauri::command]
fn state_load(app: AppHandle) -> Result<Option<String>, String> {
    let path = state_path(&app)?;
    match fs::read_to_string(&path) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// Validates the JSON and writes it atomically (temp file + rename).
#[tauri::command]
fn state_save(app: AppHandle, data: String) -> Result<(), String> {
    if data.len() > MAX_STATE_BYTES {
        return Err("state too large".into());
    }
    serde_json::from_str::<serde_json::Value>(&data).map_err(|e| format!("invalid JSON: {e}"))?;
    let path = state_path(&app)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, data.as_bytes()).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![state_load, state_save])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
