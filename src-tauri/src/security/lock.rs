/// Lock-related Tauri commands.
///
/// The lock screen itself lives in the Svelte frontend; these commands provide
/// the backend verification layer. Biometric integration (tauri-plugin-biometric)
/// will be wired in Phase 2 of the refactor.

/// Check whether the OS supports biometric authentication on this device.
/// Returns false if biometrics are not configured or not available.
#[tauri::command]
pub fn security_biometrics_available() -> bool {
    // Phase 2: integrate tauri-plugin-biometric for real biometric checks.
    // For now, report unavailable so the frontend falls back to OS password.
    false
}

/// Verify that the application can still access its database key from the
/// OS keychain. Returns Ok(()) if accessible, Err if the key is gone
/// (e.g., user deleted it, or permissions changed).
#[tauri::command]
pub fn security_verify_keychain_access() -> Result<(), String> {
    crate::security::get_or_create_db_key().map(|_| ())
}
