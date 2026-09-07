use crate::cluster_manager::ClusterManagerState;
use crate::k8s::common::{calculate_age, get_created_at};
use k8s_openapi::api::core::v1::Namespace;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::{Api, DeleteParams, ListParams, PostParams};
use kube::config::Kubeconfig;
use kube::{Client, Config};
use std::path::PathBuf;
use tauri::State;

/// Create a kube client for a cluster stored in the cluster database.
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

    namespaces.sort_by_key(|ns| std::cmp::Reverse(ns.created_at));
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

/// Validate a Kubernetes namespace name (RFC 1123 DNS label).
pub fn validate_namespace_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Namespace name cannot be empty".to_string());
    }
    if name.len() > 63 {
        return Err("Namespace name must be 63 characters or fewer".to_string());
    }
    let bytes = name.as_bytes();
    let valid_char = |c: u8| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-';
    if !bytes.iter().all(|&c| valid_char(c)) {
        return Err(
            "Namespace name may only contain lowercase letters, digits, and '-'".to_string(),
        );
    }
    if !bytes[0].is_ascii_alphanumeric() || !bytes[bytes.len() - 1].is_ascii_alphanumeric() {
        return Err("Namespace name must start and end with an alphanumeric character".to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn cluster_create_namespace(
    cluster_id: String,
    name: String,
    state: State<'_, ClusterManagerState>,
) -> Result<(), String> {
    let name = name.trim().to_string();
    validate_namespace_name(&name)?;

    let client = create_client_for_cluster(&cluster_id, &state).await?;
    let ns_api: Api<Namespace> = Api::all(client);

    let ns = Namespace {
        metadata: ObjectMeta {
            name: Some(name.clone()),
            ..Default::default()
        },
        ..Default::default()
    };

    ns_api
        .create(&PostParams::default(), &ns)
        .await
        .map_err(|e| format!("Failed to create namespace '{}': {}", name, e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_namespace_name;

    #[test]
    fn accepts_valid_names() {
        for name in ["default", "kube-system", "team-a1", "a", "abc123"] {
            assert!(
                validate_namespace_name(name).is_ok(),
                "{name} should be valid"
            );
        }
    }

    #[test]
    fn rejects_empty_and_too_long() {
        assert!(validate_namespace_name("").is_err());
        assert!(validate_namespace_name(&"a".repeat(64)).is_err());
        assert!(validate_namespace_name(&"a".repeat(63)).is_ok());
    }

    #[test]
    fn rejects_invalid_characters_and_edges() {
        for name in [
            "Prod",
            "my_ns",
            "ns.name",
            "-lead",
            "trail-",
            "with space",
            "ünï",
        ] {
            assert!(
                validate_namespace_name(name).is_err(),
                "{name} should be invalid"
            );
        }
    }
}

/// Live tests against the current kubeconfig context. Opt-in via
/// `KORE_LIVE_TESTS=1 cargo test live_`.
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod live_tests {
    use super::*;

    fn enabled() -> bool {
        std::env::var("KORE_LIVE_TESTS")
            .map(|v| v == "1")
            .unwrap_or(false)
    }

    #[tokio::test]
    async fn live_namespace_create_and_delete_roundtrip() {
        if !enabled() {
            return;
        }
        let client = kube::Client::try_default().await.expect("kube client");
        let ns_api: Api<Namespace> = Api::all(client);

        let name = format!("kore-live-test-{}", uuid::Uuid::new_v4().simple());
        validate_namespace_name(&name).unwrap();

        let ns = Namespace {
            metadata: ObjectMeta {
                name: Some(name.clone()),
                ..Default::default()
            },
            ..Default::default()
        };
        ns_api
            .create(&PostParams::default(), &ns)
            .await
            .expect("create namespace");

        let fetched = ns_api.get(&name).await.expect("get namespace");
        assert_eq!(fetched.metadata.name.as_deref(), Some(name.as_str()));

        // The name is 15 + 32 = 47 chars, under the 63 limit
        assert!(name.len() <= 63);

        ns_api
            .delete(&name, &DeleteParams::default())
            .await
            .expect("delete namespace");
        eprintln!("created and deleted namespace {name}");
    }
}
