use kube::config::{Kubeconfig, NamedAuthInfo, NamedCluster, NamedContext};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::State;

use crate::db::AppDbState;

const MAX_DISCOVERY_DEPTH: usize = 8;

// ── Discovery types ───────────────────────────────────────────────────────────

/// A context discovered inside a kubeconfig file, ready to be presented to the
/// user for selection in the import modal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredContext {
    pub context_name: String,
    pub cluster_name: String,
    pub user_name: String,
    pub namespace: Option<String>,
    pub source_file: String,
}

// ── Discovery logic ───────────────────────────────────────────────────────────

/// Discover all contexts in a single kubeconfig file.
pub fn discover_contexts_in_file(path: &Path) -> Result<Vec<DiscoveredContext>, String> {
    let kubeconfig =
        Kubeconfig::read_from(path).map_err(|e| format!("Failed to read kubeconfig: {}", e))?;

    let source_file = path.to_string_lossy().to_string();
    let mut contexts = Vec::new();

    for named_ctx in &kubeconfig.contexts {
        if let Some(ctx) = &named_ctx.context {
            contexts.push(DiscoveredContext {
                context_name: named_ctx.name.clone(),
                cluster_name: ctx.cluster.clone(),
                user_name: ctx.user.clone().unwrap_or_default(),
                namespace: ctx.namespace.clone(),
                source_file: source_file.clone(),
            });
        }
    }

    Ok(contexts)
}

/// Recursively discover all contexts in kubeconfig files within a folder.
pub fn discover_contexts_in_folder(path: &Path) -> Result<Vec<DiscoveredContext>, String> {
    if !path.is_dir() {
        return Err("Path is not a directory".to_string());
    }

    let mut all_contexts = Vec::new();
    visit_dirs(path, 0, &mut all_contexts)?;
    Ok(all_contexts)
}

fn visit_dirs(dir: &Path, depth: usize, contexts: &mut Vec<DiscoveredContext>) -> Result<(), String> {
    if depth > MAX_DISCOVERY_DEPTH {
        return Ok(());
    }

    let entries =
        std::fs::read_dir(dir).map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        let metadata = std::fs::symlink_metadata(&path)
            .map_err(|e| format!("Failed to read metadata: {}", e))?;

        // Skip symlinks — avoids traversal outside scope and cycle-based recursion.
        if metadata.file_type().is_symlink() {
            continue;
        }

        if path.is_dir() {
            if let Some(name) = path.file_name() {
                let name_str = name.to_string_lossy();
                if name_str.starts_with('.') || name_str == "node_modules" {
                    continue;
                }
            }
            visit_dirs(&path, depth + 1, contexts)?;
        } else if path.is_file() {
            if let Ok(file_contexts) = discover_contexts_in_file(&path) {
                contexts.extend(file_contexts);
            }
        }
    }

    Ok(())
}

// ── Import logic ──────────────────────────────────────────────────────────────

/// Extract the JSON config blobs for a single context from a source kubeconfig.
/// Returns `(NamedCluster JSON, NamedAuthInfo JSON, NamedContext JSON)`.
fn extract_config_blobs(
    source_path: &Path,
    context_name: &str,
) -> Result<(String, String, String), String> {
    let kubeconfig = Kubeconfig::read_from(source_path)
        .map_err(|e| format!("Failed to read kubeconfig: {}", e))?;

    let named_ctx = kubeconfig
        .contexts
        .iter()
        .find(|c| c.name == context_name)
        .ok_or_else(|| format!("Context '{}' not found in kubeconfig", context_name))?;

    let ctx = named_ctx
        .context
        .as_ref()
        .ok_or("Context entry has no context field")?;

    let named_cluster = kubeconfig
        .clusters
        .iter()
        .find(|c| c.name == ctx.cluster)
        .ok_or_else(|| format!("Cluster '{}' referenced by context not found", ctx.cluster))?;

    let user_name = ctx.user.as_deref().unwrap_or("");
    let named_user = kubeconfig
        .auth_infos
        .iter()
        .find(|u| u.name == user_name)
        .ok_or_else(|| format!("User '{}' referenced by context not found", user_name))?;

    let cluster_json = serde_json::to_string(named_cluster)
        .map_err(|e| format!("Failed to serialise cluster config: {}", e))?;
    let user_json = serde_json::to_string(named_user)
        .map_err(|e| format!("Failed to serialise user config: {}", e))?;
    let context_json = serde_json::to_string(named_ctx)
        .map_err(|e| format!("Failed to serialise context config: {}", e))?;

    Ok((cluster_json, user_json, context_json))
}

/// Find an existing cluster in the DB whose kubeconfig name matches `cluster_name`,
/// or create a new one.  Returns the cluster UUID.
fn find_or_create_cluster(
    db: &crate::db::AppDb,
    cluster_name: &str,
    cluster_json: &str,
    display_name: &str,
) -> Result<String, String> {
    // Look for an existing cluster whose config JSON has the same kubeconfig name.
    let conn = db.lock()?;
    let existing_id: Option<String> = conn
        .query_row(
            "SELECT id FROM clusters WHERE json_extract(config, '$.name') = ?1",
            [cluster_name],
            |r| r.get(0),
        )
        .ok();
    drop(conn);

    if let Some(id) = existing_id {
        return Ok(id);
    }

    // Create a new cluster record.
    let cluster = crate::db::clusters::create_cluster(
        db,
        display_name.to_string(),
        cluster_json.to_string(),
        None,
        None,
        vec![],
    )?;
    Ok(cluster.id)
}

/// Find an existing user in the DB whose kubeconfig name matches `user_name`,
/// or create a new one.  Returns the user UUID.
fn find_or_create_user(
    db: &crate::db::AppDb,
    user_name: &str,
    user_json: &str,
    display_name: &str,
) -> Result<String, String> {
    let conn = db.lock()?;
    let existing_id: Option<String> = conn
        .query_row(
            "SELECT id FROM users WHERE json_extract(config, '$.name') = ?1",
            [user_name],
            |r| r.get(0),
        )
        .ok();
    drop(conn);

    if let Some(id) = existing_id {
        return Ok(id);
    }

    let user = crate::db::users::create_user(db, display_name.to_string(), user_json.to_string())?;
    Ok(user.id)
}

// ── Export types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConflict {
    pub entry_type: String, // "cluster" | "user" | "context"
    pub name: String,
    pub action: String, // "skip" | "overwrite"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPreview {
    pub conflicts: Vec<ExportConflict>,
    pub cluster_count: usize,
    pub user_count: usize,
    pub context_count: usize,
}

// ── Export logic ──────────────────────────────────────────────────────────────

fn build_export_kubeconfig(
    db: &crate::db::AppDb,
    context_ids: &[String],
    current_context: Option<&str>,
    conflicts: &[ExportConflict],
) -> Result<Kubeconfig, String> {
    // Build a set of cluster/user IDs needed for the selected contexts.
    let mut cluster_ids = std::collections::HashSet::new();
    let mut user_ids = std::collections::HashSet::new();
    let mut named_contexts: Vec<NamedContext> = Vec::new();

    for ctx_id in context_ids {
        let ctx = crate::db::contexts::get_context(db, ctx_id)?
            .ok_or_else(|| format!("Context '{}' not found", ctx_id))?;

        let named_ctx: NamedContext = serde_json::from_str(&ctx.config)
            .map_err(|e| format!("Failed to parse context config: {}", e))?;

        // Check conflict resolution
        let skip = conflicts.iter().any(|c| {
            c.entry_type == "context" && c.name == named_ctx.name && c.action == "skip"
        });
        if skip {
            continue;
        }

        cluster_ids.insert(ctx.cluster_id.clone());
        user_ids.insert(ctx.user_id.clone());
        named_contexts.push(named_ctx);
    }

    let mut named_clusters: Vec<NamedCluster> = Vec::new();
    for cluster_id in &cluster_ids {
        let cluster = crate::db::clusters::get_cluster(db, cluster_id)?
            .ok_or_else(|| format!("Cluster '{}' not found", cluster_id))?;
        let named_cluster: NamedCluster = serde_json::from_str(&cluster.config)
            .map_err(|e| format!("Failed to parse cluster config: {}", e))?;
        let skip = conflicts.iter().any(|c| {
            c.entry_type == "cluster" && c.name == named_cluster.name && c.action == "skip"
        });
        if !skip {
            named_clusters.push(named_cluster);
        }
    }

    let mut named_users: Vec<NamedAuthInfo> = Vec::new();
    for user_id in &user_ids {
        let user = crate::db::users::get_user(db, user_id)?
            .ok_or_else(|| format!("User '{}' not found", user_id))?;
        let named_user: NamedAuthInfo = serde_json::from_str(&user.config)
            .map_err(|e| format!("Failed to parse user config: {}", e))?;
        let skip = conflicts.iter().any(|c| {
            c.entry_type == "user" && c.name == named_user.name && c.action == "skip"
        });
        if !skip {
            named_users.push(named_user);
        }
    }

    Ok(Kubeconfig {
        clusters: named_clusters,
        auth_infos: named_users,
        contexts: named_contexts,
        current_context: current_context.map(str::to_string),
        ..Default::default()
    })
}

fn merge_into_existing(
    existing: Kubeconfig,
    new: Kubeconfig,
    conflicts: &[ExportConflict],
) -> Kubeconfig {
    let mut clusters = existing.clusters;
    for nc in new.clusters {
        let overwrite = conflicts.iter().any(|c| {
            c.entry_type == "cluster" && c.name == nc.name && c.action == "overwrite"
        });
        if !clusters.iter().any(|e| e.name == nc.name) || overwrite {
            clusters.retain(|e| e.name != nc.name);
            clusters.push(nc);
        }
    }

    let mut auth_infos = existing.auth_infos;
    for nu in new.auth_infos {
        let overwrite = conflicts.iter().any(|c| {
            c.entry_type == "user" && c.name == nu.name && c.action == "overwrite"
        });
        if !auth_infos.iter().any(|e| e.name == nu.name) || overwrite {
            auth_infos.retain(|e| e.name != nu.name);
            auth_infos.push(nu);
        }
    }

    let mut contexts = existing.contexts;
    for nctx in new.contexts {
        let overwrite = conflicts.iter().any(|c| {
            c.entry_type == "context" && c.name == nctx.name && c.action == "overwrite"
        });
        if !contexts.iter().any(|e| e.name == nctx.name) || overwrite {
            contexts.retain(|e| e.name != nctx.name);
            contexts.push(nctx);
        }
    }

    Kubeconfig {
        clusters,
        auth_infos,
        contexts,
        current_context: new.current_context.or(existing.current_context),
        ..Default::default()
    }
}

// ── Tauri commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn import_discover_file(path: String) -> Result<Vec<DiscoveredContext>, String> {
    discover_contexts_in_file(&PathBuf::from(path))
}

#[tauri::command]
pub fn import_discover_folder(path: String) -> Result<Vec<DiscoveredContext>, String> {
    discover_contexts_in_folder(&PathBuf::from(path))
}

/// Import a single context (and its cluster + user if not already present) into
/// the encrypted database.  Returns the new context UUID.
#[tauri::command]
pub async fn import_add_context(
    source_file: String,
    context_name: String,
    display_name: String,
    icon: Option<String>,
    icon_ring_color: Option<String>,
    state: State<'_, AppDbState>,
) -> Result<String, String> {
    let source_path = PathBuf::from(&source_file);
    crate::config::validate_import_source(&source_path)?;

    let db = state.0.clone();

    // Extract JSON config blobs from the source kubeconfig.
    let (cluster_json, user_json, context_json) =
        extract_config_blobs(&source_path, &context_name)?;

    // Determine the kubeconfig names from the JSON blobs (for deduplication).
    let cluster_name: String = {
        let v: serde_json::Value = serde_json::from_str(&cluster_json)
            .map_err(|e| format!("Failed to parse cluster JSON: {}", e))?;
        v["name"]
            .as_str()
            .ok_or("Cluster config missing 'name' field")?
            .to_string()
    };
    let user_name: String = {
        let v: serde_json::Value = serde_json::from_str(&user_json)
            .map_err(|e| format!("Failed to parse user JSON: {}", e))?;
        v["name"]
            .as_str()
            .ok_or("User config missing 'name' field")?
            .to_string()
    };

    // Find or create the cluster and user records.
    let cluster_id = find_or_create_cluster(&db, &cluster_name, &cluster_json, &cluster_name)?;
    let user_id = find_or_create_user(&db, &user_name, &user_json, &user_name)?;

    // Create the context record.
    let context = crate::db::contexts::create_context(
        &db,
        display_name,
        cluster_id,
        user_id,
        context_json,
        icon,
        icon_ring_color,
        None,
        vec![],
    )?;

    Ok(context.id)
}

/// Export selected contexts (and their clusters/users) to a kubeconfig file.
///
/// If the destination file already exists, conflicts are resolved according to
/// the `conflicts` list supplied by the frontend (each entry specifies "skip"
/// or "overwrite" for a named entry).
#[tauri::command]
pub async fn export_kubeconfig(
    context_ids: Vec<String>,
    destination: String,
    current_context: Option<String>,
    conflicts: Vec<ExportConflict>,
    state: State<'_, AppDbState>,
) -> Result<(), String> {
    let db = &state.0;
    let dest_path = PathBuf::from(&destination);

    // Build the export kubeconfig from selected contexts.
    let export_kb =
        build_export_kubeconfig(db, &context_ids, current_context.as_deref(), &conflicts)?;

    // Merge into existing file if present.
    let final_kb = if dest_path.exists() {
        let existing = Kubeconfig::read_from(&dest_path)
            .map_err(|e| format!("Failed to read existing kubeconfig: {}", e))?;
        merge_into_existing(existing, export_kb, &conflicts)
    } else {
        export_kb
    };

    // Serialise and write.
    let yaml = serde_yaml::to_string(&final_kb)
        .map_err(|e| format!("Failed to serialise kubeconfig: {}", e))?;

    // Ensure the parent directory exists.
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create destination directory: {}", e))?;
    }

    std::fs::write(&dest_path, yaml)
        .map_err(|e| format!("Failed to write kubeconfig to {:?}: {}", dest_path, e))?;

    // Apply secure permissions if the destination is a new file.
    let _ = crate::config::set_owner_only_file_permissions(&dest_path);

    Ok(())
}

/// Preview an export — detect conflicts with an existing destination file
/// without writing anything.
#[tauri::command]
pub async fn export_preview(
    context_ids: Vec<String>,
    destination: String,
    state: State<'_, AppDbState>,
) -> Result<ExportPreview, String> {
    let db = &state.0;
    let dest_path = PathBuf::from(&destination);

    let mut cluster_names = std::collections::HashSet::new();
    let mut user_names = std::collections::HashSet::new();
    let mut context_names = std::collections::HashSet::new();

    for ctx_id in &context_ids {
        let ctx = crate::db::contexts::get_context(db, ctx_id)?
            .ok_or_else(|| format!("Context '{}' not found", ctx_id))?;

        let named_ctx: NamedContext = serde_json::from_str(&ctx.config)
            .map_err(|e| format!("Failed to parse context config: {}", e))?;
        context_names.insert(named_ctx.name);

        let cluster = crate::db::clusters::get_cluster(db, &ctx.cluster_id)?
            .ok_or_else(|| format!("Cluster '{}' not found", ctx.cluster_id))?;
        let named_cluster: NamedCluster = serde_json::from_str(&cluster.config)
            .map_err(|e| format!("Failed to parse cluster config: {}", e))?;
        cluster_names.insert(named_cluster.name);

        let user = crate::db::users::get_user(db, &ctx.user_id)?
            .ok_or_else(|| format!("User '{}' not found", ctx.user_id))?;
        let named_user: NamedAuthInfo = serde_json::from_str(&user.config)
            .map_err(|e| format!("Failed to parse user config: {}", e))?;
        user_names.insert(named_user.name);
    }

    let mut conflicts = Vec::new();

    if dest_path.exists() {
        if let Ok(existing) = Kubeconfig::read_from(&dest_path) {
            for ec in &existing.clusters {
                if cluster_names.contains(&ec.name) {
                    conflicts.push(ExportConflict {
                        entry_type: "cluster".to_string(),
                        name: ec.name.clone(),
                        action: "skip".to_string(), // default: skip
                    });
                }
            }
            for eu in &existing.auth_infos {
                if user_names.contains(&eu.name) {
                    conflicts.push(ExportConflict {
                        entry_type: "user".to_string(),
                        name: eu.name.clone(),
                        action: "skip".to_string(),
                    });
                }
            }
            for ectx in &existing.contexts {
                if context_names.contains(&ectx.name) {
                    conflicts.push(ExportConflict {
                        entry_type: "context".to_string(),
                        name: ectx.name.clone(),
                        action: "skip".to_string(),
                    });
                }
            }
        }
    }

    Ok(ExportPreview {
        conflicts,
        cluster_count: cluster_names.len(),
        user_count: user_names.len(),
        context_count: context_names.len(),
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_kubeconfig(
        dir: &Path,
        filename: &str,
        contexts: &[(&str, &str, &str)],
    ) -> PathBuf {
        let config_path = dir.join(filename);
        let mut file = fs::File::create(&config_path).unwrap();

        writeln!(file, "apiVersion: v1").unwrap();
        writeln!(file, "kind: Config").unwrap();
        writeln!(file, "current-context: {}", contexts[0].0).unwrap();
        writeln!(file, "clusters:").unwrap();
        for (_, cluster_name, _) in contexts {
            writeln!(file, "- name: {}", cluster_name).unwrap();
            writeln!(file, "  cluster:").unwrap();
            writeln!(file, "    server: https://example.com").unwrap();
        }
        writeln!(file, "users:").unwrap();
        for (_, _, user_name) in contexts {
            writeln!(file, "- name: {}", user_name).unwrap();
            writeln!(file, "  user:").unwrap();
            writeln!(file, "    token: test-token").unwrap();
        }
        writeln!(file, "contexts:").unwrap();
        for (context_name, cluster_name, user_name) in contexts {
            writeln!(file, "- name: {}", context_name).unwrap();
            writeln!(file, "  context:").unwrap();
            writeln!(file, "    cluster: {}", cluster_name).unwrap();
            writeln!(file, "    user: {}", user_name).unwrap();
        }

        config_path
    }

    #[test]
    fn test_discover_contexts_in_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_test_kubeconfig(
            temp_dir.path(),
            "config",
            &[
                ("prod-context", "prod-cluster", "prod-user"),
                ("dev-context", "dev-cluster", "dev-user"),
            ],
        );

        let contexts = discover_contexts_in_file(&config_path).unwrap();
        assert_eq!(contexts.len(), 2);
        assert_eq!(contexts[0].context_name, "prod-context");
        assert_eq!(contexts[1].context_name, "dev-context");
    }

    #[test]
    fn test_discover_contexts_in_folder() {
        let temp_dir = TempDir::new().unwrap();
        create_test_kubeconfig(temp_dir.path(), "config1", &[("ctx1", "cluster1", "user1")]);
        create_test_kubeconfig(temp_dir.path(), "config2", &[("ctx2", "cluster2", "user2")]);

        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        create_test_kubeconfig(&subdir, "config3", &[("ctx3", "cluster3", "user3")]);

        let contexts = discover_contexts_in_folder(temp_dir.path()).unwrap();
        assert_eq!(contexts.len(), 3);
        assert!(contexts.iter().any(|c| c.context_name == "ctx1"));
        assert!(contexts.iter().any(|c| c.context_name == "ctx3"));
    }

    #[test]
    fn test_extract_config_blobs() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_test_kubeconfig(
            temp_dir.path(),
            "config",
            &[("prod-ctx", "prod-cluster", "prod-user")],
        );

        let (cluster_json, user_json, context_json) =
            extract_config_blobs(&config_path, "prod-ctx").unwrap();

        let cluster: serde_json::Value = serde_json::from_str(&cluster_json).unwrap();
        assert_eq!(cluster["name"], "prod-cluster");

        let user: serde_json::Value = serde_json::from_str(&user_json).unwrap();
        assert_eq!(user["name"], "prod-user");

        let ctx: serde_json::Value = serde_json::from_str(&context_json).unwrap();
        assert_eq!(ctx["name"], "prod-ctx");
    }

    #[test]
    fn test_discover_contexts_respects_max_depth() {
        let temp_dir = TempDir::new().unwrap();
        let mut current = temp_dir.path().to_path_buf();

        for i in 0..MAX_DISCOVERY_DEPTH {
            current = current.join(format!("level-{}", i));
            fs::create_dir(&current).unwrap();
        }

        create_test_kubeconfig(&current, "within-limit.yaml", &[("ctx-within", "c1", "u1")]);

        let too_deep = current.join("too-deep");
        fs::create_dir(&too_deep).unwrap();
        create_test_kubeconfig(&too_deep, "too-deep.yaml", &[("ctx-too-deep", "c2", "u2")]);

        let contexts = discover_contexts_in_folder(temp_dir.path()).unwrap();
        assert!(contexts.iter().any(|c| c.context_name == "ctx-within"));
        assert!(!contexts.iter().any(|c| c.context_name == "ctx-too-deep"));
    }
}
