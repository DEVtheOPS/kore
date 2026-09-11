//! Whether the running install can replace itself via the built-in updater.
//!
//! The updater ships an AppImage for Linux, so .deb/.rpm installs must upgrade
//! through their package manager. Tauri's own updater detects AppImage via the
//! `APPIMAGE` environment variable set by the AppImage runtime.

#[tauri::command]
pub fn updater_can_self_update() -> bool {
    can_self_update(
        cfg!(target_os = "linux"),
        std::env::var_os("APPIMAGE").is_some(),
    )
}

fn can_self_update(is_linux: bool, is_appimage: bool) -> bool {
    !is_linux || is_appimage
}

#[cfg(test)]
mod tests {
    use super::can_self_update;

    #[test]
    fn linux_requires_appimage() {
        assert!(can_self_update(true, true));
        assert!(!can_self_update(true, false));
    }

    #[test]
    fn other_platforms_always_self_update() {
        assert!(can_self_update(false, false));
        assert!(can_self_update(false, true));
    }
}
