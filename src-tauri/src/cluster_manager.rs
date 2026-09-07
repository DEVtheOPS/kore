use crate::input_validation::{
    validate_cluster_name, validate_context_name, validate_description, validate_tags,
};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cluster {
    pub id: String,
    pub name: String,
    pub context_name: String,
    pub config_path: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub tags: String, // JSON-encoded array
    pub created_at: i64,
    pub last_accessed: i64,
}

pub struct ClusterManager {
    conn: Mutex<Connection>,
}

impl ClusterManager {
    pub fn new(db_path: PathBuf) -> Result<Self, String> {
        let conn =
            Connection::open(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;

        // Create clusters table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS clusters (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                context_name TEXT NOT NULL,
                config_path TEXT NOT NULL,
                icon TEXT,
                description TEXT,
                tags TEXT NOT NULL DEFAULT '[]',
                created_at INTEGER NOT NULL,
                last_accessed INTEGER NOT NULL
            )",
            [],
        )
        .map_err(|e| format!("Failed to create clusters table: {}", e))?;

        let manager = ClusterManager {
            conn: Mutex::new(conn),
        };
        // Best-effort: a failed rewrite must not prevent the app from starting.
        if let Err(e) = manager.migrate_config_paths() {
            eprintln!("Warning: failed to migrate legacy config paths: {}", e);
        }
        Ok(manager)
    }

    /// Rewrite stored kubeconfig paths that still point at the pre-rebrand
    /// `~/.rustylens` directory so they resolve under `~/.kore`.
    fn migrate_config_paths(&self) -> Result<usize, String> {
        let legacy = crate::config::get_legacy_app_config_dir();
        let current = crate::config::get_app_config_dir();
        self.rewrite_config_path_prefix(&legacy, &current)
    }

    /// Re-root every `config_path` under `from` to live under `to`.
    ///
    /// Path handling is done in Rust with `Path::strip_prefix` (exact,
    /// case-sensitive, platform-aware separators) rather than SQL `LIKE`, which
    /// is case-insensitive, treats `_`/`%` as wildcards, and would miss
    /// Windows backslash paths.
    fn rewrite_config_path_prefix(&self, from: &Path, to: &Path) -> Result<usize, String> {
        if from == to {
            return Ok(0);
        }

        let mut conn = self
            .conn
            .lock()
            .map_err(|e| format!("Database lock poisoned: {}", e))?;

        let rows: Vec<(String, String)> = {
            let mut stmt = conn
                .prepare("SELECT id, config_path FROM clusters")
                .map_err(|e| format!("Failed to read config paths: {}", e))?;
            let iter = stmt
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .map_err(|e| format!("Failed to read config paths: {}", e))?;
            iter.collect::<Result<_, _>>()
                .map_err(|e| format!("Failed to read config paths: {}", e))?
        };

        let updates: Vec<(String, String)> = rows
            .into_iter()
            .filter_map(|(id, config_path)| {
                Path::new(&config_path)
                    .strip_prefix(from)
                    .ok()
                    .map(|rest| (id, to.join(rest).to_string_lossy().to_string()))
            })
            .collect();

        if updates.is_empty() {
            return Ok(0);
        }

        let tx = conn
            .transaction()
            .map_err(|e| format!("Failed to start migration transaction: {}", e))?;
        for (id, new_path) in &updates {
            tx.execute(
                "UPDATE clusters SET config_path = ?1 WHERE id = ?2",
                rusqlite::params![new_path, id],
            )
            .map_err(|e| format!("Failed to migrate config path: {}", e))?;
        }
        tx.commit()
            .map_err(|e| format!("Failed to commit config path migration: {}", e))?;

        Ok(updates.len())
    }

    pub fn add_cluster(
        &self,
        name: String,
        context_name: String,
        config_path: PathBuf,
        icon: Option<String>,
        description: Option<String>,
        tags: Vec<String>,
    ) -> Result<Cluster, String> {
        let name = validate_cluster_name(name)?;
        let context_name = validate_context_name(context_name)?;
        let description = validate_description(description)?;
        let tags = validate_tags(tags)?;

        let id = Uuid::new_v4().to_string();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs() as i64;

        let tags_json =
            serde_json::to_string(&tags).map_err(|e| format!("Failed to serialize tags: {}", e))?;

        let config_path_str = config_path.to_string_lossy().to_string();

        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Database lock poisoned: {}", e))?;
        conn.execute(
            "INSERT INTO clusters (id, name, context_name, config_path, icon, description, tags, created_at, last_accessed)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                &id,
                &name,
                &context_name,
                &config_path_str,
                &icon,
                &description,
                &tags_json,
                now,
                now,
            ],
        )
        .map_err(|e| format!("Failed to insert cluster: {}", e))?;

        Ok(Cluster {
            id,
            name,
            context_name,
            config_path: config_path_str,
            icon,
            description,
            tags: tags_json,
            created_at: now,
            last_accessed: now,
        })
    }

    pub fn list_clusters(&self) -> Result<Vec<Cluster>, String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Database lock poisoned: {}", e))?;
        let mut stmt = conn
            .prepare("SELECT id, name, context_name, config_path, icon, description, tags, created_at, last_accessed FROM clusters ORDER BY last_accessed DESC")
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let clusters = stmt
            .query_map([], |row| {
                Ok(Cluster {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    context_name: row.get(2)?,
                    config_path: row.get(3)?,
                    icon: row.get(4)?,
                    description: row.get(5)?,
                    tags: row.get(6)?,
                    created_at: row.get(7)?,
                    last_accessed: row.get(8)?,
                })
            })
            .map_err(|e| format!("Failed to query clusters: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect clusters: {}", e))?;

        Ok(clusters)
    }

    pub fn get_cluster(&self, id: &str) -> Result<Option<Cluster>, String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Database lock poisoned: {}", e))?;
        let mut stmt = conn
            .prepare("SELECT id, name, context_name, config_path, icon, description, tags, created_at, last_accessed FROM clusters WHERE id = ?1")
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let cluster = stmt
            .query_row([id], |row| {
                Ok(Cluster {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    context_name: row.get(2)?,
                    config_path: row.get(3)?,
                    icon: row.get(4)?,
                    description: row.get(5)?,
                    tags: row.get(6)?,
                    created_at: row.get(7)?,
                    last_accessed: row.get(8)?,
                })
            })
            .optional()
            .map_err(|e| format!("Failed to query cluster: {}", e))?;

        Ok(cluster)
    }

    pub fn update_cluster(
        &self,
        id: &str,
        name: Option<String>,
        icon: Option<Option<String>>,
        description: Option<Option<String>>,
        tags: Option<Vec<String>>,
    ) -> Result<(), String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Database lock poisoned: {}", e))?;

        // Build dynamic UPDATE query based on provided fields
        let mut updates = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(name_val) = name {
            let name_val = validate_cluster_name(name_val)?;
            updates.push("name = ?");
            params.push(Box::new(name_val));
        }

        if let Some(icon_val) = icon {
            updates.push("icon = ?");
            params.push(Box::new(icon_val));
        }

        if let Some(desc_val) = description {
            let desc_val = validate_description(desc_val)?;
            updates.push("description = ?");
            params.push(Box::new(desc_val));
        }

        if let Some(tags_val) = tags {
            let tags_val = validate_tags(tags_val)?;
            let tags_json = serde_json::to_string(&tags_val)
                .map_err(|e| format!("Failed to serialize tags: {}", e))?;
            updates.push("tags = ?");
            params.push(Box::new(tags_json));
        }

        if updates.is_empty() {
            return Ok(());
        }

        let query = format!("UPDATE clusters SET {} WHERE id = ?", updates.join(", "));
        params.push(Box::new(id.to_string()));

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();

        conn.execute(&query, param_refs.as_slice())
            .map_err(|e| format!("Failed to update cluster: {}", e))?;

        Ok(())
    }

    pub fn update_last_accessed(&self, id: &str) -> Result<(), String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs() as i64;

        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Database lock poisoned: {}", e))?;
        conn.execute(
            "UPDATE clusters SET last_accessed = ?1 WHERE id = ?2",
            params![now, id],
        )
        .map_err(|e| format!("Failed to update last_accessed: {}", e))?;

        Ok(())
    }

    pub fn delete_cluster(&self, id: &str) -> Result<(), String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Database lock poisoned: {}", e))?;
        conn.execute("DELETE FROM clusters WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to delete cluster: {}", e))?;

        Ok(())
    }
}

// Tauri commands
use tauri::State;

pub struct ClusterManagerState(pub Arc<Mutex<ClusterManager>>);

#[tauri::command]
pub fn db_list_clusters(state: State<ClusterManagerState>) -> Result<Vec<Cluster>, String> {
    let manager = state
        .0
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    manager.list_clusters()
}

#[tauri::command]
pub fn db_get_cluster(
    id: String,
    state: State<ClusterManagerState>,
) -> Result<Option<Cluster>, String> {
    let manager = state
        .0
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    manager.get_cluster(&id)
}

#[tauri::command]
pub fn db_update_cluster(
    id: String,
    name: Option<String>,
    icon: Option<Option<String>>,
    description: Option<Option<String>>,
    tags: Option<Vec<String>>,
    state: State<ClusterManagerState>,
) -> Result<(), String> {
    let manager = state
        .0
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    manager.update_cluster(&id, name, icon, description, tags)
}

#[tauri::command]
pub fn db_update_last_accessed(
    id: String,
    state: State<ClusterManagerState>,
) -> Result<(), String> {
    let manager = state
        .0
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    manager.update_last_accessed(&id)
}

#[tauri::command]
pub fn db_delete_cluster(id: String, state: State<ClusterManagerState>) -> Result<(), String> {
    let manager = state
        .0
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    // Get cluster to find config file path
    if let Some(cluster) = manager.get_cluster(&id)? {
        // Delete the config file
        let config_path = PathBuf::from(&cluster.config_path);

        // Validate the path before deletion to prevent path traversal
        if let Ok(validated_path) = crate::config::validate_kubeconfig_path(&config_path) {
            // Attempt to delete the file, ignore NotFound errors (file already deleted)
            match std::fs::remove_file(&validated_path) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(format!("Failed to delete config file: {}", e)),
            }
        } else {
            // Log warning but don't fail the deletion if path validation fails
            eprintln!(
                "Warning: Could not validate path for deletion: {:?}",
                config_path
            );
        }
    }

    // Delete from database
    manager.delete_cluster(&id)
}

#[tauri::command]
pub fn db_migrate_legacy_configs(state: State<ClusterManagerState>) -> Result<Vec<String>, String> {
    use crate::import::{discover_contexts_in_folder, extract_context};

    let manager = state
        .0
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    let kubeconfigs_dir = crate::config::get_kubeconfigs_dir();

    if !kubeconfigs_dir.exists() {
        return Ok(vec![]); // No legacy configs to migrate
    }

    // Discover all contexts from the kubeconfigs directory
    let discovered = discover_contexts_in_folder(&kubeconfigs_dir)
        .map_err(|e| format!("Failed to discover legacy configs: {}", e))?;

    let mut migrated = Vec::new();
    let conn = manager
        .conn
        .lock()
        .map_err(|e| format!("Database lock poisoned: {}", e))?;

    for ctx in discovered {
        let validated_context_name = match validate_context_name(ctx.context_name.clone()) {
            Ok(value) => value,
            Err(e) => {
                eprintln!(
                    "Skipping invalid context name '{}': {}",
                    ctx.context_name, e
                );
                continue;
            }
        };
        let validated_name = match validate_cluster_name(ctx.context_name.clone()) {
            Ok(value) => value,
            Err(e) => {
                eprintln!(
                    "Skipping invalid cluster name '{}': {}",
                    ctx.context_name, e
                );
                continue;
            }
        };

        // Check if this context already exists in the database
        let existing = conn
            .query_row(
                "SELECT COUNT(*) FROM clusters WHERE context_name = ?1",
                [&validated_context_name],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0);

        if existing > 0 {
            // Already migrated, skip
            continue;
        }

        // Import this context
        let id = uuid::Uuid::new_v4().to_string();

        // Extract this context to a new file
        let config_path =
            match extract_context(&PathBuf::from(&ctx.source_file), &ctx.context_name, &id) {
                Ok(path) => path,
                Err(e) => {
                    eprintln!("Failed to extract context {}: {}", ctx.context_name, e);
                    continue;
                }
            };

        // Add to database
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs() as i64;

        conn.execute(
            "INSERT INTO clusters (id, name, context_name, config_path, created_at, last_accessed)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                &id,
                &validated_name, // Use context name as display name initially
                &validated_context_name,
                config_path.to_string_lossy().to_string(),
                now,
                now,
            ),
        )
        .map_err(|e| format!("Failed to insert cluster: {}", e))?;

        migrated.push(validated_context_name);
    }

    Ok(migrated)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn rewrite_config_path_prefix_migrates_only_matching_rows() {
        let temp = TempDir::new().unwrap();
        let manager = ClusterManager::new(temp.path().join("clusters.db")).unwrap();

        let legacy = manager
            .add_cluster(
                "legacy".to_string(),
                "ctx-a".to_string(),
                PathBuf::from("/home/u/.rustylens/kubeconfigs/a.yaml"),
                None,
                None,
                vec![],
            )
            .unwrap();
        let other = manager
            .add_cluster(
                "other".to_string(),
                "ctx-b".to_string(),
                PathBuf::from("/elsewhere/.rustylens-backup/b.yaml"),
                None,
                None,
                vec![],
            )
            .unwrap();

        let changed = manager
            .rewrite_config_path_prefix(Path::new("/home/u/.rustylens"), Path::new("/home/u/.kore"))
            .unwrap();
        assert_eq!(changed, 1);

        let legacy = manager.get_cluster(&legacy.id).unwrap().unwrap();
        assert_eq!(legacy.config_path, "/home/u/.kore/kubeconfigs/a.yaml");

        let other = manager.get_cluster(&other.id).unwrap().unwrap();
        assert_eq!(other.config_path, "/elsewhere/.rustylens-backup/b.yaml");

        // Idempotent: nothing left to migrate.
        let changed = manager
            .rewrite_config_path_prefix(Path::new("/home/u/.rustylens"), Path::new("/home/u/.kore"))
            .unwrap();
        assert_eq!(changed, 0);
    }

    #[test]
    fn rewrite_config_path_prefix_is_exact_and_case_sensitive() {
        let temp = TempDir::new().unwrap();
        let manager = ClusterManager::new(temp.path().join("clusters.db")).unwrap();

        // Looks like the legacy prefix under a LIKE match, but is not the same directory.
        let tricky = manager
            .add_cluster(
                "tricky".to_string(),
                "ctx-c".to_string(),
                PathBuf::from("/home/u/.RustyLens/kubeconfigs/c.yaml"),
                None,
                None,
                vec![],
            )
            .unwrap();
        let sibling = manager
            .add_cluster(
                "sibling".to_string(),
                "ctx-d".to_string(),
                PathBuf::from("/home/u/.rustylens2/kubeconfigs/d.yaml"),
                None,
                None,
                vec![],
            )
            .unwrap();

        let changed = manager
            .rewrite_config_path_prefix(Path::new("/home/u/.rustylens"), Path::new("/home/u/.kore"))
            .unwrap();
        assert_eq!(changed, 0);
        assert_eq!(
            manager
                .get_cluster(&tricky.id)
                .unwrap()
                .unwrap()
                .config_path,
            "/home/u/.RustyLens/kubeconfigs/c.yaml"
        );
        assert_eq!(
            manager
                .get_cluster(&sibling.id)
                .unwrap()
                .unwrap()
                .config_path,
            "/home/u/.rustylens2/kubeconfigs/d.yaml"
        );
    }

    #[test]
    fn add_cluster_rejects_invalid_name() {
        let temp = TempDir::new().unwrap();
        let manager = ClusterManager::new(temp.path().join("clusters.db")).unwrap();
        let result = manager.add_cluster(
            "bad\nname".to_string(),
            "valid-context".to_string(),
            PathBuf::from("/tmp/config.yaml"),
            None,
            None,
            vec![],
        );
        assert!(result.is_err());
    }

    #[test]
    fn add_cluster_rejects_duplicate_tags() {
        let temp = TempDir::new().unwrap();
        let manager = ClusterManager::new(temp.path().join("clusters.db")).unwrap();
        let result = manager.add_cluster(
            "valid".to_string(),
            "valid-context".to_string(),
            PathBuf::from("/tmp/config.yaml"),
            None,
            None,
            vec!["prod".to_string(), "prod".to_string()],
        );
        assert!(result.is_err());
    }

    #[test]
    fn update_cluster_rejects_invalid_description() {
        let temp = TempDir::new().unwrap();
        let manager = ClusterManager::new(temp.path().join("clusters.db")).unwrap();
        let cluster = manager
            .add_cluster(
                "valid".to_string(),
                "valid-context".to_string(),
                PathBuf::from("/tmp/config.yaml"),
                None,
                None,
                vec!["prod".to_string()],
            )
            .unwrap();

        let result = manager.update_cluster(
            &cluster.id,
            None,
            None,
            Some(Some("bad\u{0007}".to_string())),
            None,
        );
        assert!(result.is_err());
    }
}
