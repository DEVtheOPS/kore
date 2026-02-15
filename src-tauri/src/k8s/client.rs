use crate::cluster_manager::ClusterManagerState;
use crate::config;
use crate::k8s::common::{calculate_age, get_created_at};
use k8s_openapi::api::core::v1::Namespace;
use kube::api::{Api, DeleteParams, ListParams};
use kube::config::Kubeconfig;
use kube::{Client, Config};
use std::path::PathBuf;
use tauri::State;

// Helper to find which file contains the context
pub fn find_kubeconfig_path_for_context(context_name: &str) -> Option<PathBuf> {
    // 1. Standard locations
    let mut paths = vec![];
    if let Ok(p) = std::env::var("KUBECONFIG") {
        paths.push(PathBuf::from(p));
    }
    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".kube").join("config"));
    }

    // 2. Custom app config directory
    let app_kube_dir = config::get_kubeconfigs_dir();
    if app_kube_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(app_kube_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    paths.push(path);
                }
            }
        }
    }

    // Check each file
    for path in paths {
        if path.exists() {
            if let Ok(config) = Kubeconfig::read_from(&path) {
                for ctx in config.contexts {
                    if ctx.name == context_name {
                        return Some(path);
                    }
                }
            }
        }
    }

    None
}

// Helper to create client
pub async fn create_client_for_context(context_name: &str) -> Result<Client, String> {
    let config_path = find_kubeconfig_path_for_context(context_name).ok_or_else(|| {
        format!(
            "Context '{}' not found in any kubeconfig file",
            context_name
        )
    })?;

    let kubeconfig = Kubeconfig::read_from(&config_path)
        .map_err(|e| format!("Failed to read kubeconfig {:?}: {}", config_path, e))?;

    let options = kube::config::KubeConfigOptions {
        context: Some(context_name.to_string()),
        ..Default::default()
    };

    let config = Config::from_custom_kubeconfig(kubeconfig, &options)
        .await
        .map_err(|e| format!("Failed to load config: {}", e))?;

    Client::try_from(config).map_err(|e| format!("Failed to create client: {}", e))
}

// NEW: Helper to create client from cluster ID
pub async fn create_client_for_cluster(
    cluster_id: &str,
    state: &State<'_, ClusterManagerState>,
) -> Result<Client, String> {
    let manager = state.0.clone();
    let cluster_id = cluster_id.to_string();

    // 1. Blocking I/O (DB + File Read)
    let kubeconfig = tauri::async_runtime::spawn_blocking(move || {
        // Get config path
        let config_path = {
            let manager = manager
                .lock()
                .map_err(|e| format!("Failed to acquire lock: {}", e))?;
            let cluster = manager
                .get_cluster(&cluster_id)?
                .ok_or_else(|| format!("Cluster '{}' not found", cluster_id))?;
            PathBuf::from(&cluster.config_path)
        };

        if !config_path.exists() {
            return Err(format!("Config file not found: {:?}", config_path));
        }

        let kubeconfig = Kubeconfig::read_from(&config_path)
            .map_err(|e| format!("Failed to read kubeconfig {:?}: {}", config_path, e))?;

        Ok(kubeconfig)
    })
    .await
    .map_err(|e| e.to_string())??;

    // 2. Async Config Loading
    // The extracted config should have only one context, use current_context
    let context_name = kubeconfig
        .current_context
        .as_ref()
        .ok_or_else(|| "No current context in kubeconfig".to_string())?;

    let options = kube::config::KubeConfigOptions {
        context: Some(context_name.clone()),
        ..Default::default()
    };

    let config = Config::from_custom_kubeconfig(kubeconfig, &options)
        .await
        .map_err(|e| format!("Failed to load config: {}", e))?;

    Client::try_from(config).map_err(|e| format!("Failed to create client: {}", e))
}

#[tauri::command]
pub async fn list_contexts() -> Result<Vec<String>, String> {
    let mut paths = vec![];
    if let Ok(p) = std::env::var("KUBECONFIG") {
        paths.push(PathBuf::from(p));
    }
    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".kube").join("config"));
    }

    let app_kube_dir = config::get_kubeconfigs_dir();
    if app_kube_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(app_kube_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    paths.push(path);
                }
            }
        }
    }

    let mut contexts = Vec::new();
    for path in paths {
        if path.exists() {
            if let Ok(config) = Kubeconfig::read_from(&path) {
                for ctx in config.contexts {
                    contexts.push(ctx.name);
                }
            }
        }
    }

    if contexts.is_empty() {
        return Ok(vec![]);
    }

    contexts.sort();
    contexts.dedup();

    Ok(contexts)
}

#[tauri::command]
pub async fn list_namespaces(context_name: String) -> Result<Vec<String>, String> {
    let client = create_client_for_context(&context_name).await?;
    let ns_api: Api<Namespace> = Api::all(client);
    let lp = ListParams::default();

    let list = ns_api
        .list(&lp)
        .await
        .map_err(|e| format!("Failed to list namespaces: {}", e))?;

    let names = list
        .items
        .into_iter()
        .filter_map(|ns| ns.metadata.name)
        .collect();

    Ok(names)
}

#[tauri::command]
pub async fn cluster_list_namespaces(
    cluster_id: String,
    state: State<'_, ClusterManagerState>,
) -> Result<Vec<String>, String> {
    let client = create_client_for_cluster(&cluster_id, &state).await?;
    let ns_api: Api<Namespace> = Api::all(client);
    let lp = ListParams::default();

    let list = ns_api
        .list(&lp)
        .await
        .map_err(|e| format!("Failed to list namespaces: {}", e))?;

    let namespaces: Vec<String> = list
        .items
        .iter()
        .map(|ns| ns.metadata.name.clone().unwrap_or_default())
        .collect();

    Ok(namespaces)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct NamespaceSummary {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub age: String,
    pub labels: std::collections::BTreeMap<String, String>,
    pub status: String,
    pub images: Vec<String>,
    pub created_at: i64,
}

#[tauri::command]
pub async fn cluster_list_namespaces_detailed(
    cluster_id: String,
    state: State<'_, ClusterManagerState>,
) -> Result<Vec<NamespaceSummary>, String> {
    let client = create_client_for_cluster(&cluster_id, &state).await?;
    let ns_api: Api<Namespace> = Api::all(client);

    let list = ns_api
        .list(&ListParams::default())
        .await
        .map_err(|e| format!("Failed to list namespaces: {}", e))?;

    let mut namespaces: Vec<NamespaceSummary> = list
        .items
        .into_iter()
        .map(|ns| {
            let meta = ns.metadata;
            let status = ns
                .status
                .and_then(|s| s.phase)
                .unwrap_or_else(|| "Unknown".to_string());

            NamespaceSummary {
                id: meta.uid.clone().unwrap_or_default(),
                name: meta.name.clone().unwrap_or_default(),
                namespace: "-".to_string(),
                age: calculate_age(meta.creation_timestamp.as_ref()),
                labels: meta.labels.unwrap_or_default(),
                status,
                images: vec![],
                created_at: get_created_at(meta.creation_timestamp.as_ref()),
            }
        })
        .collect();

    namespaces.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(namespaces)
}

#[tauri::command]
pub async fn cluster_delete_namespace(
    cluster_id: String,
    name: String,
    state: State<'_, ClusterManagerState>,
) -> Result<(), String> {
    let client = create_client_for_cluster(&cluster_id, &state).await?;
    let ns_api: Api<Namespace> = Api::all(client);
    ns_api
        .delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| format!("Failed to delete namespace '{}': {}", name, e))?;
    Ok(())
}
