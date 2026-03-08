use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::db::{now_secs, AppDbState};
use crate::input_validation::{
    validate_description, validate_display_name, validate_icon, validate_icon_ring_color,
    validate_tags,
};

// ── Types ────────────────────────────────────────────────────────────────────

/// A Kore context record — the primary unit of navigation.
///
/// `config` is a JSON-serialised `kube::config::NamedContext` — the verbatim
/// kubeconfig context entry (name + cluster ref + user ref + optional namespace).
///
/// `cluster_display_name` and `user_display_name` are joined from their
/// respective tables for display convenience; they are not stored in this table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub id: String,
    pub display_name: String,
    pub cluster_id: String,
    pub user_id: String,
    pub config: String, // JSON NamedContext blob
    pub icon: Option<String>,
    pub icon_ring_color: Option<String>,
    pub description: Option<String>,
    pub tags: String, // JSON array string
    pub is_pinned: bool,
    pub pin_order: Option<i64>,
    pub created_at: i64,
    pub last_accessed: i64,
    // Joined fields (populated on read)
    pub cluster_display_name: String,
    pub user_display_name: String,
}

// ── CRUD ─────────────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub fn create_context(
    db: &crate::db::AppDb,
    display_name: String,
    cluster_id: String,
    user_id: String,
    config: String,
    icon: Option<String>,
    icon_ring_color: Option<String>,
    description: Option<String>,
    tags: Vec<String>,
) -> Result<Context, String> {
    let display_name = validate_display_name(display_name)?;
    let icon = validate_icon(icon)?;
    let icon_ring_color = validate_icon_ring_color(icon_ring_color)?;
    let description = validate_description(description)?;
    let tags = validate_tags(tags)?;
    let tags_json =
        serde_json::to_string(&tags).map_err(|e| format!("Failed to serialize tags: {}", e))?;

    let _: serde_json::Value =
        serde_json::from_str(&config).map_err(|e| format!("Invalid context config JSON: {}", e))?;

    let id = Uuid::new_v4().to_string();
    let now = now_secs();

    let conn = db.lock()?;

    // Validate that cluster and user exist
    let cluster_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM clusters WHERE id = ?1",
            [&cluster_id],
            |r| r.get::<_, i64>(0),
        )
        .map(|n| n > 0)
        .map_err(|e| format!("Failed to verify cluster: {}", e))?;

    if !cluster_exists {
        return Err(format!("Cluster '{}' not found", cluster_id));
    }

    let user_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM users WHERE id = ?1",
            [&user_id],
            |r| r.get::<_, i64>(0),
        )
        .map(|n| n > 0)
        .map_err(|e| format!("Failed to verify user: {}", e))?;

    if !user_exists {
        return Err(format!("User '{}' not found", user_id));
    }

    // Fetch display names for joined fields
    let cluster_display_name: String = conn
        .query_row(
            "SELECT display_name FROM clusters WHERE id = ?1",
            [&cluster_id],
            |r| r.get(0),
        )
        .map_err(|e| format!("Failed to fetch cluster display name: {}", e))?;

    let user_display_name: String = conn
        .query_row(
            "SELECT display_name FROM users WHERE id = ?1",
            [&user_id],
            |r| r.get(0),
        )
        .map_err(|e| format!("Failed to fetch user display name: {}", e))?;

    conn.execute(
        "INSERT INTO contexts
             (id, display_name, cluster_id, user_id, config,
              icon, icon_ring_color, description, tags,
              is_pinned, pin_order, created_at, last_accessed)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, NULL, ?10, ?11)",
        params![
            id,
            display_name,
            cluster_id,
            user_id,
            config,
            icon,
            icon_ring_color,
            description,
            tags_json,
            now,
            now,
        ],
    )
    .map_err(|e| format!("Failed to insert context: {}", e))?;

    Ok(Context {
        id,
        display_name,
        cluster_id,
        user_id,
        config,
        icon,
        icon_ring_color,
        description,
        tags: tags_json,
        is_pinned: false,
        pin_order: None,
        created_at: now,
        last_accessed: now,
        cluster_display_name,
        user_display_name,
    })
}

/// Return all contexts, optionally filtered to one cluster.
/// Ordered by `last_accessed DESC` then `display_name ASC`.
pub fn list_contexts(
    db: &crate::db::AppDb,
    cluster_id: Option<&str>,
) -> Result<Vec<Context>, String> {
    let conn = db.lock()?;

    let (where_clause, param): (&str, Option<&str>) = match cluster_id {
        Some(id) => (" WHERE ctx.cluster_id = ?1", Some(id)),
        None => ("", None),
    };

    let sql = format!(
        "SELECT ctx.id, ctx.display_name, ctx.cluster_id, ctx.user_id, ctx.config,
                ctx.icon, ctx.icon_ring_color, ctx.description, ctx.tags,
                ctx.is_pinned, ctx.pin_order, ctx.created_at, ctx.last_accessed,
                c.display_name  AS cluster_display_name,
                u.display_name  AS user_display_name
         FROM contexts ctx
         JOIN clusters c ON c.id = ctx.cluster_id
         JOIN users    u ON u.id = ctx.user_id
         {}
         ORDER BY ctx.last_accessed DESC, ctx.display_name ASC",
        where_clause
    );

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Failed to prepare list_contexts: {}", e))?;

    let map_row = |row: &rusqlite::Row| {
        Ok(Context {
            id: row.get(0)?,
            display_name: row.get(1)?,
            cluster_id: row.get(2)?,
            user_id: row.get(3)?,
            config: row.get(4)?,
            icon: row.get(5)?,
            icon_ring_color: row.get(6)?,
            description: row.get(7)?,
            tags: row.get(8)?,
            is_pinned: row.get::<_, i64>(9)? != 0,
            pin_order: row.get(10)?,
            created_at: row.get(11)?,
            last_accessed: row.get(12)?,
            cluster_display_name: row.get(13)?,
            user_display_name: row.get(14)?,
        })
    };

    let rows = if let Some(id) = param {
        stmt.query_map([id], map_row)
    } else {
        stmt.query_map([], map_row)
    }
    .map_err(|e| format!("Failed to query contexts: {}", e))?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| format!("Failed to collect contexts: {}", e))?;

    Ok(rows)
}

/// Return all pinned contexts ordered by `pin_order ASC`.
pub fn list_pinned_contexts(db: &crate::db::AppDb) -> Result<Vec<Context>, String> {
    let conn = db.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT ctx.id, ctx.display_name, ctx.cluster_id, ctx.user_id, ctx.config,
                    ctx.icon, ctx.icon_ring_color, ctx.description, ctx.tags,
                    ctx.is_pinned, ctx.pin_order, ctx.created_at, ctx.last_accessed,
                    c.display_name  AS cluster_display_name,
                    u.display_name  AS user_display_name
             FROM contexts ctx
             JOIN clusters c ON c.id = ctx.cluster_id
             JOIN users    u ON u.id = ctx.user_id
             WHERE ctx.is_pinned = 1
             ORDER BY ctx.pin_order ASC, ctx.display_name ASC",
        )
        .map_err(|e| format!("Failed to prepare list_pinned_contexts: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Context {
                id: row.get(0)?,
                display_name: row.get(1)?,
                cluster_id: row.get(2)?,
                user_id: row.get(3)?,
                config: row.get(4)?,
                icon: row.get(5)?,
                icon_ring_color: row.get(6)?,
                description: row.get(7)?,
                tags: row.get(8)?,
                is_pinned: row.get::<_, i64>(9)? != 0,
                pin_order: row.get(10)?,
                created_at: row.get(11)?,
                last_accessed: row.get(12)?,
                cluster_display_name: row.get(13)?,
                user_display_name: row.get(14)?,
            })
        })
        .map_err(|e| format!("Failed to query pinned contexts: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect pinned contexts: {}", e))?;

    Ok(rows)
}

/// Return a single context by UUID, or `None` if not found.
pub fn get_context(db: &crate::db::AppDb, id: &str) -> Result<Option<Context>, String> {
    let conn = db.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT ctx.id, ctx.display_name, ctx.cluster_id, ctx.user_id, ctx.config,
                    ctx.icon, ctx.icon_ring_color, ctx.description, ctx.tags,
                    ctx.is_pinned, ctx.pin_order, ctx.created_at, ctx.last_accessed,
                    c.display_name  AS cluster_display_name,
                    u.display_name  AS user_display_name
             FROM contexts ctx
             JOIN clusters c ON c.id = ctx.cluster_id
             JOIN users    u ON u.id = ctx.user_id
             WHERE ctx.id = ?1",
        )
        .map_err(|e| format!("Failed to prepare get_context: {}", e))?;

    let ctx = stmt
        .query_row([id], |row| {
            Ok(Context {
                id: row.get(0)?,
                display_name: row.get(1)?,
                cluster_id: row.get(2)?,
                user_id: row.get(3)?,
                config: row.get(4)?,
                icon: row.get(5)?,
                icon_ring_color: row.get(6)?,
                description: row.get(7)?,
                tags: row.get(8)?,
                is_pinned: row.get::<_, i64>(9)? != 0,
                pin_order: row.get(10)?,
                created_at: row.get(11)?,
                last_accessed: row.get(12)?,
                cluster_display_name: row.get(13)?,
                user_display_name: row.get(14)?,
            })
        })
        .optional()
        .map_err(|e| format!("Failed to query context: {}", e))?;

    Ok(ctx)
}

#[allow(clippy::too_many_arguments)]
pub fn update_context(
    db: &crate::db::AppDb,
    id: &str,
    display_name: Option<String>,
    icon: Option<Option<String>>,
    icon_ring_color: Option<Option<String>>,
    description: Option<Option<String>>,
    tags: Option<Vec<String>>,
) -> Result<(), String> {
    let conn = db.lock()?;

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
    if let Some(ring) = icon_ring_color {
        let ring = validate_icon_ring_color(ring)?;
        parts.push("icon_ring_color = ?".to_string());
        values.push(Box::new(ring));
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

    if parts.is_empty() {
        return Ok(());
    }

    let query = format!("UPDATE contexts SET {} WHERE id = ?", parts.join(", "));
    values.push(Box::new(id.to_string()));
    let refs: Vec<&dyn rusqlite::ToSql> = values.iter().map(|b| b.as_ref()).collect();
    conn.execute(&query, refs.as_slice())
        .map_err(|e| format!("Failed to update context: {}", e))?;

    Ok(())
}

pub fn update_context_last_accessed(db: &crate::db::AppDb, id: &str) -> Result<(), String> {
    let now = now_secs();
    let conn = db.lock()?;
    conn.execute(
        "UPDATE contexts SET last_accessed = ?1 WHERE id = ?2",
        params![now, id],
    )
    .map_err(|e| format!("Failed to update last_accessed: {}", e))?;
    Ok(())
}

pub fn pin_context(db: &crate::db::AppDb, id: &str) -> Result<(), String> {
    // Determine next pin_order value
    let conn = db.lock()?;
    let max_order: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(pin_order), -1) FROM contexts WHERE is_pinned = 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(-1);

    conn.execute(
        "UPDATE contexts SET is_pinned = 1, pin_order = ?1 WHERE id = ?2",
        params![max_order + 1, id],
    )
    .map_err(|e| format!("Failed to pin context: {}", e))?;

    Ok(())
}

pub fn unpin_context(db: &crate::db::AppDb, id: &str) -> Result<(), String> {
    let conn = db.lock()?;
    conn.execute(
        "UPDATE contexts SET is_pinned = 0, pin_order = NULL WHERE id = ?1",
        params![id],
    )
    .map_err(|e| format!("Failed to unpin context: {}", e))?;
    Ok(())
}

/// Reorder pinned contexts.  `ordered_ids` is the new desired order
/// (only IDs of pinned contexts should be included).
pub fn reorder_pinned_contexts(
    db: &crate::db::AppDb,
    ordered_ids: &[String],
) -> Result<(), String> {
    const MAX_PINNED: usize = 100;
    if ordered_ids.len() > MAX_PINNED {
        return Err(format!(
            "Cannot reorder more than {} pinned contexts at once",
            MAX_PINNED
        ));
    }

    let conn = db.lock()?;

    // Wrap in a transaction so a mid-loop failure leaves order consistent.
    conn.execute_batch("BEGIN")
        .map_err(|e| format!("Failed to begin transaction: {}", e))?;

    for (i, id) in ordered_ids.iter().enumerate() {
        conn.execute(
            "UPDATE contexts SET pin_order = ?1 WHERE id = ?2 AND is_pinned = 1",
            params![i as i64, id],
        )
        .map_err(|e| {
            let _ = conn.execute_batch("ROLLBACK");
            format!("Failed to reorder context '{}': {}", id, e)
        })?;
    }

    conn.execute_batch("COMMIT")
        .map_err(|e| format!("Failed to commit reorder transaction: {}", e))?;

    Ok(())
}

pub fn delete_context(db: &crate::db::AppDb, id: &str) -> Result<(), String> {
    let conn = db.lock()?;
    conn.execute("DELETE FROM contexts WHERE id = ?1", params![id])
        .map_err(|e| format!("Failed to delete context: {}", e))?;
    Ok(())
}

// ── Tauri commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn db_list_contexts(
    cluster_id: Option<String>,
    state: State<'_, AppDbState>,
) -> Result<Vec<Context>, String> {
    list_contexts(&state.0, cluster_id.as_deref())
}

#[tauri::command]
pub fn db_list_pinned_contexts(state: State<'_, AppDbState>) -> Result<Vec<Context>, String> {
    list_pinned_contexts(&state.0)
}

#[tauri::command]
pub fn db_get_context(id: String, state: State<'_, AppDbState>) -> Result<Option<Context>, String> {
    get_context(&state.0, &id)
}

#[tauri::command]
pub fn db_update_context(
    id: String,
    display_name: Option<String>,
    icon: Option<Option<String>>,
    icon_ring_color: Option<Option<String>>,
    description: Option<Option<String>>,
    tags: Option<Vec<String>>,
    state: State<'_, AppDbState>,
) -> Result<(), String> {
    update_context(
        &state.0,
        &id,
        display_name,
        icon,
        icon_ring_color,
        description,
        tags,
    )
}

#[tauri::command]
pub fn db_update_context_last_accessed(
    id: String,
    state: State<'_, AppDbState>,
) -> Result<(), String> {
    update_context_last_accessed(&state.0, &id)
}

#[tauri::command]
pub fn db_pin_context(id: String, state: State<'_, AppDbState>) -> Result<(), String> {
    pin_context(&state.0, &id)
}

#[tauri::command]
pub fn db_unpin_context(id: String, state: State<'_, AppDbState>) -> Result<(), String> {
    unpin_context(&state.0, &id)
}

#[tauri::command]
pub fn db_reorder_pinned_contexts(
    ordered_ids: Vec<String>,
    state: State<'_, AppDbState>,
) -> Result<(), String> {
    reorder_pinned_contexts(&state.0, &ordered_ids)
}

#[tauri::command]
pub fn db_delete_context(id: String, state: State<'_, AppDbState>) -> Result<(), String> {
    delete_context(&state.0, &id)
}
