use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::db::{now_secs, AppDbState};
use crate::input_validation::{
    validate_description, validate_display_name, validate_icon, validate_tags,
};

// ── Types ────────────────────────────────────────────────────────────────────

/// Full cluster record used internally (includes the raw kubeconfig blob).
///
/// `config` is a JSON-serialised `kube::config::NamedCluster` — the verbatim
/// kubeconfig cluster entry (name + server URL + CA cert, etc.).
/// Storing the whole entry as a blob means we never lose kubeconfig fields,
/// and the schema never needs migration as the kubeconfig spec evolves.
///
/// **Never serialise this struct over the Tauri IPC bridge** — `config` contains
/// CA certificates and other sensitive material.  Use `ClusterSummary` instead.
#[derive(Debug, Clone)]
pub struct Cluster {
    pub id: String,
    pub display_name: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub tags: String,   // JSON array string e.g. '["prod","eu"]'
    pub config: String, // JSON NamedCluster blob — SENSITIVE, not sent to frontend
    pub created_at: i64,
    pub updated_at: i64,
    pub context_count: i64,
}

/// Redacted cluster view sent over the Tauri IPC bridge.
///
/// The `config` blob (CA cert + server URL) is intentionally omitted.
/// The frontend only needs display / navigation fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterSummary {
    pub id: String,
    pub display_name: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub tags: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub context_count: i64,
}

impl From<Cluster> for ClusterSummary {
    fn from(c: Cluster) -> Self {
        ClusterSummary {
            id: c.id,
            display_name: c.display_name,
            icon: c.icon,
            description: c.description,
            tags: c.tags,
            created_at: c.created_at,
            updated_at: c.updated_at,
            context_count: c.context_count,
        }
    }
}

// ── CRUD ─────────────────────────────────────────────────────────────────────

/// Insert a new cluster.  Returns the new record.
pub fn create_cluster(
    db: &crate::db::AppDb,
    display_name: String,
    config: String, // JSON NamedCluster blob from the importer
    icon: Option<String>,
    description: Option<String>,
    tags: Vec<String>,
) -> Result<Cluster, String> {
    let display_name = validate_display_name(display_name)?;
    let icon = validate_icon(icon)?;
    let description = validate_description(description)?;
    let tags = validate_tags(tags)?;
    let tags_json =
        serde_json::to_string(&tags).map_err(|e| format!("Failed to serialize tags: {}", e))?;

    // Basic sanity check that config is valid JSON.
    let _: serde_json::Value =
        serde_json::from_str(&config).map_err(|e| format!("Invalid cluster config JSON: {}", e))?;

    let id = Uuid::new_v4().to_string();
    let now = now_secs();

    let conn = db.lock()?;
    conn.execute(
        "INSERT INTO clusters (id, display_name, icon, description, tags, config, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id, display_name, icon, description, tags_json, config, now, now],
    )
    .map_err(|e| format!("Failed to insert cluster: {}", e))?;

    Ok(Cluster {
        id,
        display_name,
        icon,
        description,
        tags: tags_json,
        config,
        created_at: now,
        updated_at: now,
        context_count: 0,
    })
}

/// Return all clusters ordered by most-recently updated, with context counts.
pub fn list_clusters(db: &crate::db::AppDb) -> Result<Vec<Cluster>, String> {
    let conn = db.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.display_name, c.icon, c.description, c.tags, c.config,
                    c.created_at, c.updated_at,
                    COUNT(ctx.id) AS context_count
             FROM clusters c
             LEFT JOIN contexts ctx ON ctx.cluster_id = c.id
             GROUP BY c.id
             ORDER BY c.updated_at DESC",
        )
        .map_err(|e| format!("Failed to prepare list_clusters: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Cluster {
                id: row.get(0)?,
                display_name: row.get(1)?,
                icon: row.get(2)?,
                description: row.get(3)?,
                tags: row.get(4)?,
                config: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                context_count: row.get(8)?,
            })
        })
        .map_err(|e| format!("Failed to query clusters: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect clusters: {}", e))?;

    Ok(rows)
}

/// Return a single cluster by UUID, or `None` if not found.
pub fn get_cluster(db: &crate::db::AppDb, id: &str) -> Result<Option<Cluster>, String> {
    let conn = db.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.display_name, c.icon, c.description, c.tags, c.config,
                    c.created_at, c.updated_at,
                    COUNT(ctx.id) AS context_count
             FROM clusters c
             LEFT JOIN contexts ctx ON ctx.cluster_id = c.id
             WHERE c.id = ?1
             GROUP BY c.id",
        )
        .map_err(|e| format!("Failed to prepare get_cluster: {}", e))?;

    let cluster = stmt
        .query_row([id], |row| {
            Ok(Cluster {
                id: row.get(0)?,
                display_name: row.get(1)?,
                icon: row.get(2)?,
                description: row.get(3)?,
                tags: row.get(4)?,
                config: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                context_count: row.get(8)?,
            })
        })
        .optional()
        .map_err(|e| format!("Failed to query cluster: {}", e))?;

    Ok(cluster)
}

/// Partially update a cluster's metadata fields.
pub fn update_cluster(
    db: &crate::db::AppDb,
    id: &str,
    display_name: Option<String>,
    icon: Option<Option<String>>,
    description: Option<Option<String>>,
    tags: Option<Vec<String>>,
) -> Result<(), String> {
    let mut parts: Vec<String> = Vec::new();
    let mut values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(name) = display_name {
        let name = validate_display_name(name)?;
        parts.push("display_name = ?".to_string());
        values.push(Box::new(name));
    }
    if let Some(icon_val) = icon {
        let icon_val = validate_icon(icon_val)?;
        parts.push("icon = ?".to_string());
        values.push(Box::new(icon_val));
    }
    if let Some(desc) = description {
        let desc = validate_description(desc)?;
        parts.push("description = ?".to_string());
        values.push(Box::new(desc));
    }
    if let Some(t) = tags {
        let t = validate_tags(t)?;
        let json =
            serde_json::to_string(&t).map_err(|e| format!("Failed to serialize tags: {}", e))?;
        parts.push("tags = ?".to_string());
        values.push(Box::new(json));
    }

    // Nothing to update — return early rather than touching updated_at.
    if parts.is_empty() {
        return Ok(());
    }

    parts.push("updated_at = ?".to_string());
    let now = now_secs();
    values.push(Box::new(now));

    let conn = db.lock()?;
    let query = format!("UPDATE clusters SET {} WHERE id = ?", parts.join(", "));
    values.push(Box::new(id.to_string()));

    let refs: Vec<&dyn rusqlite::ToSql> = values.iter().map(|b| b.as_ref()).collect();
    conn.execute(&query, refs.as_slice())
        .map_err(|e| format!("Failed to update cluster: {}", e))?;

    Ok(())
}

/// Delete a cluster by UUID.
///
/// Contexts that reference this cluster are cascade-deleted by the DB.
/// Users are NOT deleted (they are independent).
pub fn delete_cluster(db: &crate::db::AppDb, id: &str) -> Result<(), String> {
    let conn = db.lock()?;
    conn.execute("DELETE FROM clusters WHERE id = ?1", params![id])
        .map_err(|e| format!("Failed to delete cluster: {}", e))?;
    Ok(())
}

// ── Tauri commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn db_list_clusters(state: State<'_, AppDbState>) -> Result<Vec<ClusterSummary>, String> {
    list_clusters(&state.0).map(|v| v.into_iter().map(ClusterSummary::from).collect())
}

#[tauri::command]
pub fn db_get_cluster(
    id: String,
    state: State<'_, AppDbState>,
) -> Result<Option<ClusterSummary>, String> {
    get_cluster(&state.0, &id).map(|opt| opt.map(ClusterSummary::from))
}

#[tauri::command]
pub fn db_update_cluster(
    id: String,
    display_name: Option<String>,
    icon: Option<Option<String>>,
    description: Option<Option<String>>,
    tags: Option<Vec<String>>,
    state: State<'_, AppDbState>,
) -> Result<(), String> {
    update_cluster(&state.0, &id, display_name, icon, description, tags)
}

#[tauri::command]
pub fn db_delete_cluster(id: String, state: State<'_, AppDbState>) -> Result<(), String> {
    delete_cluster(&state.0, &id)
}
