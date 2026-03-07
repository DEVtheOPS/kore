pub mod clusters;
pub mod contexts;
pub mod users;

use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Shared, encrypted SQLite database handle.
///
/// The `Mutex<Connection>` serialises all access (SQLite is single-writer).
/// `Arc` allows the handle to be cloned cheaply across threads / Tauri State.
pub struct AppDb {
    conn: Mutex<Connection>,
}

impl AppDb {
    /// Open (or create) the SQLCipher-encrypted database at `db_path`.
    ///
    /// `key` is the hex string retrieved from the OS keychain; it is passed
    /// directly to `PRAGMA key` which SQLCipher uses for AES-256 encryption.
    pub fn new(db_path: PathBuf, key: &str) -> Result<Self, String> {
        let conn =
            Connection::open(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;

        // Apply the SQLCipher encryption key immediately after opening.
        conn.execute_batch(&format!("PRAGMA key = '{}';", key))
            .map_err(|e| format!("Failed to apply database encryption key: {}", e))?;

        // Enable referential integrity enforcement.
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|e| format!("Failed to enable foreign key constraints: {}", e))?;

        create_schema(&conn)?;

        Ok(AppDb {
            conn: Mutex::new(conn),
        })
    }

    /// Acquire the database connection lock.
    pub fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, String> {
        self.conn
            .lock()
            .map_err(|e| format!("Database lock poisoned: {}", e))
    }
}

// SAFETY: Connection is Send in rusqlite; Mutex makes it Sync.
unsafe impl Send for AppDb {}
unsafe impl Sync for AppDb {}

/// Tauri-managed state wrapper.  Clone is cheap (increments the Arc refcount).
#[derive(Clone)]
pub struct AppDbState(pub Arc<AppDb>);

// ── Schema ──────────────────────────────────────────────────────────────────

fn create_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        -- Physical Kubernetes API server endpoints.
        -- `config` is a JSON blob containing the full kubeconfig NamedCluster entry
        -- (name + cluster object), stored verbatim from the imported kubeconfig.
        CREATE TABLE IF NOT EXISTS clusters (
            id           TEXT PRIMARY KEY,
            display_name TEXT NOT NULL,
            icon         TEXT,
            description  TEXT,
            tags         TEXT NOT NULL DEFAULT '[]',
            config       TEXT NOT NULL,
            created_at   INTEGER NOT NULL,
            updated_at   INTEGER NOT NULL
        );

        -- Credential sets (tokens, client certs, exec plugins, etc.).
        -- `config` is a JSON blob containing the full kubeconfig NamedAuthInfo entry.
        -- Users are independent of clusters, matching the kubeconfig spec.
        CREATE TABLE IF NOT EXISTS users (
            id           TEXT PRIMARY KEY,
            display_name TEXT NOT NULL,
            config       TEXT NOT NULL,
            created_at   INTEGER NOT NULL,
            updated_at   INTEGER NOT NULL
        );

        -- Contexts: the unit of navigation.  Each context binds a cluster + user
        -- and optional default namespace.
        -- `config` is a JSON blob containing the full kubeconfig NamedContext entry.
        -- Deleting a cluster cascades to its contexts.
        -- Deleting a user does NOT cascade (users are independent).
        CREATE TABLE IF NOT EXISTS contexts (
            id              TEXT PRIMARY KEY,
            display_name    TEXT NOT NULL,
            cluster_id      TEXT NOT NULL REFERENCES clusters(id) ON DELETE CASCADE,
            user_id         TEXT NOT NULL REFERENCES users(id),
            config          TEXT NOT NULL,
            icon            TEXT,
            icon_ring_color TEXT,
            description     TEXT,
            tags            TEXT NOT NULL DEFAULT '[]',
            is_pinned       INTEGER NOT NULL DEFAULT 0,
            pin_order       INTEGER,
            created_at      INTEGER NOT NULL,
            last_accessed   INTEGER NOT NULL
        );
        ",
    )
    .map_err(|e| format!("Failed to create database schema: {}", e))
}

// ── Shared helpers ───────────────────────────────────────────────────────────

/// Current Unix epoch in seconds.
pub fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
