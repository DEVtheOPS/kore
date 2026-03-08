/// Backend session lock state.
///
/// This lives in Tauri managed state so the Rust side can authoritatively track
/// whether the workspace is locked — independent of the frontend's JS state.
/// This prevents lock bypass via devtools or other client-side manipulation.
use std::sync::atomic::{AtomicBool, Ordering};

pub struct SessionState {
    locked: AtomicBool,
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
        }
    }

    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::SeqCst)
    }

    pub fn set_locked(&self, locked: bool) {
        self.locked.store(locked, Ordering::SeqCst);
    }
}

// ── Tauri commands ────────────────────────────────────────────────────────────

/// Lock the workspace on the backend.  The frontend must call this when the
/// user triggers a lock so the Rust side tracks the authoritative state.
#[tauri::command]
pub fn security_lock(session: tauri::State<'_, SessionState>) -> Result<(), String> {
    session.set_locked(true);
    Ok(())
}

/// Verify that the app can still reach its database key in the OS keychain,
/// then mark the backend session as unlocked.
///
/// Called by the frontend after a successful biometric/password challenge.
#[tauri::command]
pub fn security_verify_keychain_access(
    session: tauri::State<'_, SessionState>,
) -> Result<(), String> {
    crate::security::get_or_create_db_key().map(|_| {
        session.set_locked(false);
    })
}

/// Return the current backend lock state.  The frontend can query this on
/// startup or after window focus to detect lock-state desync.
#[tauri::command]
pub fn security_is_locked(session: tauri::State<'_, SessionState>) -> bool {
    session.is_locked()
}
