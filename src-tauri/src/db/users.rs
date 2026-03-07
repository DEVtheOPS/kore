use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::db::{now_secs, AppDbState};

// ── Types ────────────────────────────────────────────────────────────────────

/// A Kore user record (a kubeconfig credential set).
///
/// `config` is a JSON-serialised `kube::config::NamedAuthInfo` — the verbatim
/// kubeconfig user entry (name + credentials: token, client cert, exec plugin, etc.).
/// Users are independent of clusters, matching the kubeconfig spec.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub display_name: String,
    pub config: String, // JSON NamedAuthInfo blob
    pub created_at: i64,
    pub updated_at: i64,
    /// Number of contexts that reference this user (joined on read).
    pub context_count: i64,
}

// ── CRUD ─────────────────────────────────────────────────────────────────────

/// Insert a new user.  Returns the new record.
pub fn create_user(
    db: &crate::db::AppDb,
    display_name: String,
    config: String, // JSON NamedAuthInfo blob from the importer
) -> Result<User, String> {
    // Basic sanity check that config is valid JSON.
    let _: serde_json::Value =
        serde_json::from_str(&config).map_err(|e| format!("Invalid user config JSON: {}", e))?;

    let id = Uuid::new_v4().to_string();
    let now = now_secs();

    let conn = db.lock()?;
    conn.execute(
        "INSERT INTO users (id, display_name, config, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, display_name, config, now, now],
    )
    .map_err(|e| format!("Failed to insert user: {}", e))?;

    Ok(User {
        id,
        display_name,
        config,
        created_at: now,
        updated_at: now,
        context_count: 0,
    })
}

/// Return all users ordered by most-recently updated, with context counts.
pub fn list_users(db: &crate::db::AppDb) -> Result<Vec<User>, String> {
    let conn = db.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT u.id, u.display_name, u.config, u.created_at, u.updated_at,
                    COUNT(ctx.id) AS context_count
             FROM users u
             LEFT JOIN contexts ctx ON ctx.user_id = u.id
             GROUP BY u.id
             ORDER BY u.updated_at DESC",
        )
        .map_err(|e| format!("Failed to prepare list_users: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(User {
                id: row.get(0)?,
                display_name: row.get(1)?,
                config: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                context_count: row.get(5)?,
            })
        })
        .map_err(|e| format!("Failed to query users: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect users: {}", e))?;

    Ok(rows)
}

/// Return a single user by UUID, or `None` if not found.
pub fn get_user(db: &crate::db::AppDb, id: &str) -> Result<Option<User>, String> {
    let conn = db.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT u.id, u.display_name, u.config, u.created_at, u.updated_at,
                    COUNT(ctx.id) AS context_count
             FROM users u
             LEFT JOIN contexts ctx ON ctx.user_id = u.id
             WHERE u.id = ?1
             GROUP BY u.id",
        )
        .map_err(|e| format!("Failed to prepare get_user: {}", e))?;

    let user = stmt
        .query_row([id], |row| {
            Ok(User {
                id: row.get(0)?,
                display_name: row.get(1)?,
                config: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                context_count: row.get(5)?,
            })
        })
        .optional()
        .map_err(|e| format!("Failed to query user: {}", e))?;

    Ok(user)
}

/// Update a user's display name.
pub fn update_user(db: &crate::db::AppDb, id: &str, display_name: String) -> Result<(), String> {
    let now = now_secs();
    let conn = db.lock()?;
    conn.execute(
        "UPDATE users SET display_name = ?1, updated_at = ?2 WHERE id = ?3",
        params![display_name, now, id],
    )
    .map_err(|e| format!("Failed to update user: {}", e))?;
    Ok(())
}

/// Delete a user by UUID.
///
/// Returns an error listing referencing context names if the user is still
/// referenced by any contexts.  Pass `force = true` to delete the user and
/// all its contexts together.
pub fn delete_user(db: &crate::db::AppDb, id: &str, force: bool) -> Result<(), String> {
    let conn = db.lock()?;

    if !force {
        // Check for dependent contexts
        let mut stmt = conn
            .prepare("SELECT display_name FROM contexts WHERE user_id = ?1")
            .map_err(|e| format!("Failed to prepare dependency check: {}", e))?;

        let names: Vec<String> = stmt
            .query_map([id], |row| row.get(0))
            .map_err(|e| format!("Failed to query dependent contexts: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect context names: {}", e))?;

        if !names.is_empty() {
            return Err(format!(
                "Cannot delete user: referenced by {} context(s): {}",
                names.len(),
                names.join(", ")
            ));
        }
    } else {
        // Force-delete all dependent contexts first
        conn.execute("DELETE FROM contexts WHERE user_id = ?1", params![id])
            .map_err(|e| format!("Failed to delete dependent contexts: {}", e))?;
    }

    conn.execute("DELETE FROM users WHERE id = ?1", params![id])
        .map_err(|e| format!("Failed to delete user: {}", e))?;

    Ok(())
}

// ── Tauri commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn db_list_users(state: State<'_, AppDbState>) -> Result<Vec<User>, String> {
    list_users(&state.0)
}

#[tauri::command]
pub fn db_get_user(id: String, state: State<'_, AppDbState>) -> Result<Option<User>, String> {
    get_user(&state.0, &id)
}

#[tauri::command]
pub fn db_update_user(
    id: String,
    display_name: String,
    state: State<'_, AppDbState>,
) -> Result<(), String> {
    update_user(&state.0, &id, display_name)
}

/// Delete a user.
///
/// If `force` is `true`, the user and all its contexts are deleted together.
/// If `force` is `false` (default) and contexts still reference this user,
/// the command returns an error listing the blocking context names.
#[tauri::command]
pub fn db_delete_user(
    id: String,
    force: Option<bool>,
    state: State<'_, AppDbState>,
) -> Result<(), String> {
    delete_user(&state.0, &id, force.unwrap_or(false))
}
