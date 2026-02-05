use crate::cluster_manager::ClusterManagerState;
use crate::config;
use k8s_openapi::api::core::v1::Namespace;
use kube::api::{Api, ListParams};
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
            let manager = manager.lock().unwrap();
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