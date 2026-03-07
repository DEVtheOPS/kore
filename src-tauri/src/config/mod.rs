pub mod settings;

use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// Returns `~/.kore/`
pub fn get_app_config_dir() -> PathBuf {
    let mut path = match dirs::home_dir() {
        Some(dir) => dir,
        None => {
            eprintln!("Could not find home directory, using /tmp");
            PathBuf::from("/tmp")
        }
    };
    path.push(".kore");
    path
}

/// Returns `~/.kore/kore.db`
pub fn get_db_path() -> PathBuf {
    get_app_config_dir().join("kore.db")
}

/// Returns `~/.kore/settings.json`
pub fn get_settings_path() -> PathBuf {
    get_app_config_dir().join("settings.json")
}

/// Create the app config directory with secure permissions.
pub fn init_directories() -> std::io::Result<()> {
    let app_dir = get_app_config_dir();
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)?;
    }
    set_owner_only_dir_permissions(&app_dir)?;
    Ok(())
}

pub(crate) fn set_owner_only_dir_permissions(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        if let Err(e) = fs::set_permissions(path, fs::Permissions::from_mode(0o700)) {
            // Some sandboxed or managed filesystems disallow chmod; don't block app startup.
            if e.kind() != std::io::ErrorKind::PermissionDenied {
                return Err(e);
            }
        }
    }
    Ok(())
}

pub(crate) fn set_owner_only_file_permissions(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        if let Err(e) = fs::set_permissions(path, fs::Permissions::from_mode(0o600)) {
            // Some sandboxed or managed filesystems disallow chmod; keep best-effort behavior.
            if e.kind() != std::io::ErrorKind::PermissionDenied {
                return Err(e);
            }
        }
    }
    Ok(())
}

/// Validate that a source path for kubeconfig import exists and is readable.
pub fn validate_import_source(path: &Path) -> Result<PathBuf, String> {
    if !path.exists() {
        return Err("Source file does not exist".to_string());
    }

    if !path.is_file() {
        return Err("Source path is not a file".to_string());
    }

    // Canonicalize to resolve symlinks and relative paths
    let canonical = path
        .canonicalize()
        .map_err(|e| format!("Failed to resolve path: {}", e))?;

    // Check file permissions (readable)
    std::fs::metadata(&canonical).map_err(|e| format!("Cannot read file: {}", e))?;

    Ok(canonical)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn validate_import_source_rejects_directory() {
        let temp = tempfile::TempDir::new().unwrap();
        let err = validate_import_source(temp.path()).unwrap_err();
        assert!(err.contains("not a file"));
    }

    #[cfg(unix)]
    #[test]
    fn set_owner_only_permissions_on_file_and_dir() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let dir = temp_dir.path().join("secure-dir");
        let file = dir.join("secret.yaml");
        fs::create_dir_all(&dir).unwrap();
        fs::write(&file, "secret").unwrap();

        set_owner_only_dir_permissions(&dir).unwrap();
        set_owner_only_file_permissions(&file).unwrap();

        let dir_mode = fs::metadata(&dir).unwrap().permissions().mode() & 0o777;
        let file_mode = fs::metadata(&file).unwrap().permissions().mode() & 0o777;
        assert_eq!(dir_mode, 0o700);
        assert_eq!(file_mode, 0o600);
    }
}
