use crate::cluster_manager::ClusterManagerState;
use crate::config;
use kube::config::Kubeconfig;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredContext {
    pub context_name: String,
    pub cluster_name: String,
    pub user_name: String,
    pub namespace: Option<String>,
    pub source_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportCandidate {
    pub context: DiscoveredContext,
    pub suggested_name: String,
    pub icon: Option<String>,
    pub description: Option<String>,
}

// Trait for future extensibility (AWS, GKE, etc.)
#[allow(dead_code)]
pub trait ImportSource {
    fn discover(&self) -> Result<Vec<DiscoveredContext>, String>;
    fn import(&self, context: &DiscoveredContext) -> Result<Kubeconfig, String>;
}

/// Parse a kubeconfig file and return all discovered contexts
pub fn discover_contexts_in_file(path: &Path) -> Result<Vec<DiscoveredContext>, String> {
    let kubeconfig = Kubeconfig::read_from(path)
        .map_err(|e| format!("Failed to read kubeconfig from {:?}: {}", path, e))?;

    let mut discovered = Vec::new();
    let source_file = path.to_string_lossy().to_string();

    for context in kubeconfig.contexts {
        if let Some(ctx) = context.context {
            discovered.push(DiscoveredContext {
                context_name: context.name.clone(),
                cluster_name: ctx.cluster,
                user_name: ctx.user.unwrap_or_default(),
                namespace: ctx.namespace,
                source_file: source_file.clone(),
            });
        }
    }

    Ok(discovered)
}

/// Recursively discover all kubeconfig files in a directory
pub fn discover_contexts_in_folder(path: &Path) -> Result<Vec<DiscoveredContext>, String> {
    let mut all_contexts = Vec::new();

    if !path.exists() {
        return Err(format!("Path does not exist: {:?}", path));
    }

    if !path.is_dir() {
        return Err(format!("Path is not a directory: {:?}", path));
    }

    visit_dirs(path, &mut all_contexts)?;

    Ok(all_contexts)
}

fn visit_dirs(dir: &Path, contexts: &mut Vec<DiscoveredContext>) -> Result<(), String> {
    if dir.is_dir() {
        let entries = std::fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory {:?}: {}", dir, e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            if path.is_dir() {
                visit_dirs(&path, contexts)?;
            } else if path.is_file() {
                // Try to parse as kubeconfig
                if let Ok(discovered) = discover_contexts_in_file(&path) {
                    contexts.extend(discovered);
                }
                // If it fails, silently skip (not a valid kubeconfig)
            }
        }
    }

    Ok(())
}

/// Extract a single context from a kubeconfig file and create a new single-context config
pub fn extract_context(
    source_path: &Path,
    context_name: &str,
    cluster_id: &str,
) -> Result<PathBuf, String> {
    let kubeconfig = Kubeconfig::read_from(source_path)
        .map_err(|e| format!("Failed to read kubeconfig: {}", e))?;

    // Find the context
    let context = kubeconfig
        .contexts
        .iter()
        .find(|c| c.name == context_name)
        .ok_or_else(|| format!("Context '{}' not found", context_name))?;

    let ctx = context
        .context
        .as_ref()
        .ok_or_else(|| "Context has no context field".to_string())?;

    // Find the associated cluster and user
    let cluster = kubeconfig
        .clusters
        .iter()
        .find(|c| c.name == ctx.cluster)
        .ok_or_else(|| format!("Cluster '{}' not found", ctx.cluster))?;

    let user_name = ctx.user.as_ref().ok_or_else(|| "Context has no user".to_string())?;
    let user = kubeconfig
        .auth_infos
        .iter()
        .find(|u| &u.name == user_name)
        .ok_or_else(|| format!("User '{}' not found", user_name))?;

    // Create a new kubeconfig with only this context
    let new_kubeconfig = Kubeconfig {
        api_version: kubeconfig.api_version.clone(),
        kind: kubeconfig.kind.clone(),
        preferences: kubeconfig.preferences.clone(),
        clusters: vec![cluster.clone()],
        auth_infos: vec![user.clone()],
        contexts: vec![context.clone()],
        current_context: Some(context_name.to_string()),
        extensions: kubeconfig.extensions.clone(),
    };

    // Save to ~/.kore/kubeconfigs/<cluster_id>.yaml
    let dest_path = config::get_kubeconfigs_dir().join(format!("{}.yaml", cluster_id));
    std::fs::write(
        &dest_path,
        serde_yaml::to_string(&new_kubeconfig)
            .map_err(|e| format!("Failed to serialize kubeconfig: {}", e))?,
    )
    .map_err(|e| format!("Failed to write kubeconfig: {}", e))?;

    Ok(dest_path)
}

// Tauri commands

#[tauri::command]
pub async fn import_discover_file(path: String) -> Result<Vec<ImportCandidate>, String> {
    let path = PathBuf::from(path);
    let contexts = discover_contexts_in_file(&path)?;

    let candidates = contexts
        .into_iter()
        .map(|ctx| ImportCandidate {
            suggested_name: ctx.context_name.clone(),
            icon: None,
            description: Some(format!(
                "Cluster: {}, User: {}",
                ctx.cluster_name, ctx.user_name
            )),
            context: ctx,
        })
        .collect();

    Ok(candidates)
}

#[tauri::command]
pub async fn import_discover_folder(path: String) -> Result<Vec<ImportCandidate>, String> {
    let path = PathBuf::from(path);
    let contexts = discover_contexts_in_folder(&path)?;

    let candidates = contexts
        .into_iter()
        .map(|ctx| ImportCandidate {
            suggested_name: ctx.context_name.clone(),
            icon: None,
            description: Some(format!(
                "Cluster: {}, User: {}, File: {}",
                ctx.cluster_name, ctx.user_name, ctx.source_file
            )),
            context: ctx,
        })
        .collect();

    Ok(candidates)
}

#[tauri::command]
pub async fn import_add_cluster(
    name: String,
    context_name: String,
    source_file: String,
    icon: Option<String>,
    description: Option<String>,
    tags: Vec<String>,
    state: State<'_, ClusterManagerState>,
) -> Result<String, String> {
    // Generate cluster ID
    let cluster_id = uuid::Uuid::new_v4().to_string();

    // Extract context to isolated config file
    let source_path = PathBuf::from(source_file);
    let config_path = extract_context(&source_path, &context_name, &cluster_id)?;

    // Add to database
    let manager = state.0.lock().unwrap();
    let cluster = manager.add_cluster(name, context_name, config_path, icon, description, tags)?;

    Ok(cluster.id)
}
