/// Legacy cluster manager — kept only to support existing k8s commands
/// during the refactor. All new Tauri commands live in `db/clusters.rs`.
/// This module will be removed in Phase 8 (cleanup).
use crate::input_validation::{
    validate_cluster_name, validate_context_name, validate_description, validate_tags,
};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
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
    pub tags: String,
    pub created_at: i64,
    pub last_accessed: i64,
}

pub struct ClusterManager {
    pub conn: Mutex<Connection>,
}

impl ClusterManager {
    pub fn new(db_path: PathBuf) -> Result<Self, String> {
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open legacy database: {}", e))?;

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
        .map_err(|e| format!("Failed to create legacy clusters table: {}", e))?;

        Ok(ClusterManager {
            conn: Mutex::new(conn),
        })
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
            .unwrap_or_default()
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
            params![id, name, context_name, config_path_str, icon, description, tags_json, now, now],
        )
        .map_err(|e| format!("Failed to insert legacy cluster: {}", e))?;

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

    pub fn get_cluster(&self, id: &str) -> Result<Option<Cluster>, String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Database lock poisoned: {}", e))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, context_name, config_path, icon, description, tags, created_at, last_accessed
                 FROM clusters WHERE id = ?1",
            )
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

    pub fn update_last_accessed(&self, id: &str) -> Result<(), String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
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

pub struct ClusterManagerState(pub Arc<Mutex<ClusterManager>>);
