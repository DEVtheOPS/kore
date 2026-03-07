/// Lock-related Tauri commands.
///
/// The biometric *prompt* is driven from the JS side via @tauri-apps/plugin-biometric
/// (Touch ID / Windows Hello / etc.).  These commands handle backend bookkeeping.

/// Verify that the app can still reach its database key in the OS keychain.
/// Called by the frontend after a successful biometric/password challenge to
/// confirm the encryption key is intact before proceeding.
#[tauri::command]
pub fn security_verify_keychain_access() -> Result<(), String> {
    crate::security::get_or_create_db_key().map(|_| ())
}
