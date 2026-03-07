use serde::{Deserialize, Serialize};
use std::fs;
use tauri::State;

use crate::db::AppDbState;

/// Lock mode controls when Kore requires re-authentication.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LockMode {
    /// Never show a lock screen; OS handles keychain silently.
    Off,
    /// Lock screen every time Kore opens.
    OnOpen,
    /// Lock screen after N minutes of inactivity.
    Timeout,
}

impl Default for LockMode {
    fn default() -> Self {
        LockMode::Off
    }
}

/// Application-level settings stored in `~/.kore/settings.json`.
///
/// This file is intentionally plaintext — it must be readable before the
/// encrypted database is unlocked (lock mode, timeout, etc. are needed first).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub theme: String,
    pub code_theme: String,
    pub refresh_interval: u64,
    pub lock_mode: LockMode,
    pub lock_timeout_minutes: u64,
    pub require_biometric: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            theme: "kore".to_string(),
            code_theme: "same-as-app".to_string(),
            refresh_interval: 5000,
            lock_mode: LockMode::Off,
            lock_timeout_minutes: 15,
            require_biometric: false,
        }
    }
}

/// Read settings from disk, returning defaults if the file is absent or unparsable.
pub fn read_settings() -> AppSettings {
    let path = crate::config::get_settings_path();
    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => AppSettings::default(),
    }
}

/// Persist settings to disk.
pub fn write_settings(settings: &AppSettings) -> Result<(), String> {
    let path = crate::config::get_settings_path();
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Failed to write settings: {}", e))?;
    Ok(())
}

// ── Tauri commands ──────────────────────────────────────────────────────────

#[tauri::command]
pub fn settings_get(_state: State<'_, AppDbState>) -> Result<AppSettings, String> {
    Ok(read_settings())
}

#[tauri::command]
pub fn settings_update(
    theme: Option<String>,
    code_theme: Option<String>,
    refresh_interval: Option<u64>,
    lock_mode: Option<LockMode>,
    lock_timeout_minutes: Option<u64>,
    require_biometric: Option<bool>,
    _state: State<'_, AppDbState>,
) -> Result<AppSettings, String> {
    let mut settings = read_settings();

    if let Some(v) = theme {
        settings.theme = v;
    }
    if let Some(v) = code_theme {
        settings.code_theme = v;
    }
    if let Some(v) = refresh_interval {
        settings.refresh_interval = v;
    }
    if let Some(v) = lock_mode {
        settings.lock_mode = v;
    }
    if let Some(v) = lock_timeout_minutes {
        settings.lock_timeout_minutes = v;
    }
    if let Some(v) = require_biometric {
        settings.require_biometric = v;
    }

    write_settings(&settings)?;
    Ok(settings)
}
