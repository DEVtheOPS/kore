use serde::{Deserialize, Serialize};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[allow(dead_code)]
pub struct AppConfig {
    #[allow(dead_code)]
    pub kubeconfig_paths: Vec<PathBuf>,
}

/// Name of the application data directory under the user's home (`~/.kore`).
pub const APP_DIR_NAME: &str = ".kore";
/// Legacy (pre-rebrand) data directory name, migrated on startup if present.
pub const LEGACY_APP_DIR_NAME: &str = ".rustylens";

fn home_dir() -> PathBuf {
    match dirs::home_dir() {
        Some(dir) => dir,
        None => {
            eprintln!("Could not find home directory, using /tmp");
            PathBuf::from("/tmp")
        }
    }
}

pub fn get_app_config_dir() -> PathBuf {
    home_dir().join(APP_DIR_NAME)
}

pub fn get_legacy_app_config_dir() -> PathBuf {
    home_dir().join(LEGACY_APP_DIR_NAME)
}

/// Move the pre-rebrand `~/.rustylens` directory to `~/.kore` if the new
/// directory does not exist yet. Returns `Ok(true)` when a migration happened.
///
/// Absolute kubeconfig paths stored in the cluster database are rewritten by
/// `ClusterManager::migrate_config_paths` after the database is opened.
pub fn migrate_legacy_app_dir() -> std::io::Result<bool> {
    migrate_legacy_app_dir_between(&get_legacy_app_config_dir(), &get_app_config_dir())
}

pub(crate) fn migrate_legacy_app_dir_between(
    legacy: &Path,
    current: &Path,
) -> std::io::Result<bool> {
    if current.exists() || !legacy.is_dir() {
        return Ok(false);
    }
    fs::rename(legacy, current)?;
    Ok(true)
}

pub fn get_kubeconfigs_dir() -> PathBuf {
    let mut path = get_app_config_dir();
    path.push("kubeconfigs");
    path
}

pub fn init_directories() -> std::io::Result<()> {
    match migrate_legacy_app_dir() {
        Ok(true) => eprintln!(
            "Migrated legacy data directory {} -> {}",
            LEGACY_APP_DIR_NAME, APP_DIR_NAME
        ),
        Ok(false) => {}
        Err(e) => eprintln!("Warning: failed to migrate legacy data directory: {}", e),
    }

    let app_dir = get_app_config_dir();
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)?;
    }
    set_owner_only_dir_permissions(&app_dir)?;

    let kube_dir = get_kubeconfigs_dir();
    if !kube_dir.exists() {
        fs::create_dir_all(&kube_dir)?;
    }
    set_owner_only_dir_permissions(&kube_dir)?;

    Ok(())
}

#[tauri::command]
pub async fn import_kubeconfig(path: String) -> Result<String, String> {
    let source = PathBuf::from(path);

    // Validate the source path
    let validated_source = validate_import_source(&source)?;

    let file_name = validated_source
        .file_name()
        .ok_or("Invalid file name")?
        .to_string_lossy()
        .to_string();

    // Add timestamp to prevent overwrites or just unique ID
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_else(|_| std::time::Duration::from_secs(0))
        .as_secs();

    let dest_name = format!("{}_{}", file_name, timestamp);
    let dest_path = get_kubeconfigs_dir().join(&dest_name);

    // Validate destination path
    let validated_dest = validate_kubeconfig_path(&dest_path)?;

    fs::copy(&validated_source, &validated_dest)
        .map_err(|e| format!("Failed to copy config: {}", e))?;
    set_owner_only_file_permissions(&validated_dest)
        .map_err(|e| format!("Failed to set secure permissions: {}", e))?;

    Ok(dest_name)
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

/// Validate that a path is within the allowed kubeconfigs directory
/// Returns the canonicalized path if valid, otherwise returns an error
pub fn validate_kubeconfig_path(path: &Path) -> Result<PathBuf, String> {
    // Get the canonical path of the kubeconfigs directory
    let allowed_dir = get_kubeconfigs_dir()
        .canonicalize()
        .map_err(|e| format!("Failed to resolve kubeconfigs directory: {}", e))?;

    // Attempt to canonicalize the provided path
    // If the file doesn't exist yet, we need to check the parent directory
    let canonical = if path.exists() {
        path.canonicalize()
            .map_err(|e| format!("Invalid path: {}", e))?
    } else {
        // For non-existent files, validate the parent directory
        let parent = path
            .parent()
            .ok_or_else(|| "Path has no parent directory".to_string())?;
        let canonical_parent = parent
            .canonicalize()
            .map_err(|e| format!("Invalid parent directory: {}", e))?;
        canonical_parent.join(
            path.file_name()
                .ok_or_else(|| "Path has no filename".to_string())?,
        )
    };

    // Check if the canonical path is within the allowed directory
    if !canonical.starts_with(&allowed_dir) {
        return Err(format!(
            "Path traversal detected: path must be within {:?}",
            allowed_dir
        ));
    }

    Ok(canonical)
}

/// Validate that a source path for import exists and is readable
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
    fn legacy_dir_migration_renames_when_target_missing() {
        let temp = tempfile::TempDir::new().unwrap();
        let legacy = temp.path().join(".rustylens");
        let current = temp.path().join(".kore");
        fs::create_dir_all(legacy.join("kubeconfigs")).unwrap();
        fs::write(legacy.join("clusters.db"), b"db").unwrap();

        assert!(migrate_legacy_app_dir_between(&legacy, &current).unwrap());
        assert!(!legacy.exists());
        assert!(current.join("kubeconfigs").is_dir());
        assert_eq!(fs::read(current.join("clusters.db")).unwrap(), b"db");
    }

    #[test]
    fn legacy_dir_migration_is_noop_when_target_exists_or_legacy_missing() {
        let temp = tempfile::TempDir::new().unwrap();
        let legacy = temp.path().join(".rustylens");
        let current = temp.path().join(".kore");

        // Neither exists
        assert!(!migrate_legacy_app_dir_between(&legacy, &current).unwrap());

        // Both exist: never clobber the current directory
        fs::create_dir_all(&legacy).unwrap();
        fs::create_dir_all(&current).unwrap();
        fs::write(current.join("keep"), b"x").unwrap();
        assert!(!migrate_legacy_app_dir_between(&legacy, &current).unwrap());
        assert!(legacy.exists());
        assert!(current.join("keep").exists());
    }

    #[test]
    fn validate_kubeconfig_path_rejects_parent_traversal() {
        init_directories().unwrap();
        let path = get_kubeconfigs_dir().join("../outside-config.yaml");
        let err = validate_kubeconfig_path(&path).unwrap_err();
        assert!(err.contains("Path traversal detected"));
    }

    #[test]
    fn validate_import_source_rejects_directory() {
        init_directories().unwrap();
        let err = validate_import_source(&get_kubeconfigs_dir()).unwrap_err();
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

    #[cfg(unix)]
    #[test]
    #[allow(clippy::panic)]
    fn validate_kubeconfig_path_rejects_symlink_escape() {
        use std::os::unix::fs::symlink;
        use std::time::{SystemTime, UNIX_EPOCH};

        init_directories().unwrap();
        let allowed = get_kubeconfigs_dir();
        let outside_target = std::env::temp_dir().join("kore-security-outside.yaml");
        std::fs::write(&outside_target, "apiVersion: v1\nkind: Config\n").unwrap();

        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_nanos();
        let link_path = allowed.join(format!("kore-test-link-{}.yaml", suffix));
        if let Err(e) = symlink(&outside_target, &link_path) {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                // Some CI/sandbox environments disallow creating symlinks in this location.
                let _ = std::fs::remove_file(outside_target);
                return;
            }
            panic!("failed to create symlink: {}", e);
        }

        let err = validate_kubeconfig_path(&link_path).unwrap_err();
        assert!(err.contains("Path traversal detected"));

        let _ = std::fs::remove_file(link_path);
        let _ = std::fs::remove_file(outside_target);
    }
}
