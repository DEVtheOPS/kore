use crate::db::AppDbState;
use crate::db::{clusters, contexts, users};
use crate::k8s::common::{calculate_age, get_created_at};
use k8s_openapi::api::core::v1::Namespace;
use kube::api::{Api, DeleteParams, ListParams};
use kube::config::{Kubeconfig, NamedAuthInfo, NamedCluster, NamedContext};
use kube::{Client, Config};
use std::path::PathBuf;
use tauri::State;
use tempfile::NamedTempFile;

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

fn parse_named_cluster(config: &str) -> Result<NamedCluster, String> {
    serde_json::from_str(config).map_err(|e| format!("Failed to parse cluster config: {}", e))
}

fn parse_named_user(config: &str) -> Result<NamedAuthInfo, String> {
    serde_json::from_str(config).map_err(|e| format!("Failed to parse user config: {}", e))
}

fn parse_named_context(config: &str) -> Result<NamedContext, String> {
    serde_json::from_str(config).map_err(|e| format!("Failed to parse context config: {}", e))
}

fn build_context_kubeconfig(db: &crate::db::AppDb, context_id: &str) -> Result<(Kubeconfig, String), String> {
    let context = contexts::get_context(db, context_id)?
        .ok_or_else(|| format!("Context '{}' not found", context_id))?;
    let cluster = clusters::get_cluster(db, &context.cluster_id)?
        .ok_or_else(|| format!("Cluster '{}' not found", context.cluster_id))?;
    let user = users::get_user(db, &context.user_id)?
        .ok_or_else(|| format!("User '{}' not found", context.user_id))?;

    let named_context = parse_named_context(&context.config)?;
    let context_name = named_context.name.clone();

    let kubeconfig = Kubeconfig {
        clusters: vec![parse_named_cluster(&cluster.config)?],
        auth_infos: vec![parse_named_user(&user.config)?],
        contexts: vec![named_context],
        current_context: Some(context_name.clone()),
        ..Default::default()
    };

    Ok((kubeconfig, context_name))
}

pub async fn create_temp_kubeconfig_for_cluster(
    cluster_id: &str,
    state: &State<'_, AppDbState>,
) -> Result<(NamedTempFile, String), String> {
    let db = state.0.clone();
    let context_id = cluster_id.to_string();

    tauri::async_runtime::spawn_blocking(move || {
        let (kubeconfig, context_name) = build_context_kubeconfig(&db, &context_id)?;
        let yaml = serde_yaml::to_string(&kubeconfig)
            .map_err(|e| format!("Failed to serialize kubeconfig: {}", e))?;

        let mut file = NamedTempFile::new()
            .map_err(|e| format!("Failed to create temporary kubeconfig: {}", e))?;
        use std::io::Write;
        file.write_all(yaml.as_bytes())
            .map_err(|e| format!("Failed to write temporary kubeconfig: {}", e))?;

        Ok((file, context_name))
    })
    .await
    .map_err(|e| e.to_string())?
}

// Compatibility helper: `cluster_id` is still the parameter name in the command
// surface, but it now refers to a context UUID backed by the encrypted DB.
pub async fn create_client_for_cluster(
    cluster_id: &str,
    state: &State<'_, AppDbState>,
) -> Result<Client, String> {
    let db = state.0.clone();
    let context_id = cluster_id.to_string();

    let (kubeconfig, context_name) = tauri::async_runtime::spawn_blocking(move || {
        build_context_kubeconfig(&db, &context_id)
    })
    .await
    .map_err(|e| e.to_string())??;

    let options = kube::config::KubeConfigOptions {
        context: Some(context_name),
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
    state: State<'_, AppDbState>,
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
    state: State<'_, AppDbState>,
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
    state: State<'_, AppDbState>,
) -> Result<(), String> {
    let client = create_client_for_cluster(&cluster_id, &state).await?;
    let ns_api: Api<Namespace> = Api::all(client);
    ns_api
        .delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| format!("Failed to delete namespace '{}': {}", name, e))?;
    Ok(())
}
