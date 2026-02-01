// ... imports ...
use crate::config;
use kube::config::Kubeconfig;
use std::path::PathBuf;
use tauri::{Emitter, Window};
use futures::StreamExt;
use k8s_openapi::api::core::v1::Pod;
use kube::{Api, Client, Config};
use kube::runtime::watcher;

// Helper to find which file contains the context
fn find_kubeconfig_path_for_context(context_name: &str) -> Option<PathBuf> {
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
async fn create_client_for_context(context_name: &str) -> Result<Client, String> {
    let config_path = find_kubeconfig_path_for_context(context_name)
        .ok_or_else(|| format!("Context '{}' not found in any kubeconfig file", context_name))?;

    let kubeconfig = Kubeconfig::read_from(&config_path)
        .map_err(|e| format!("Failed to read kubeconfig {:?}: {}", config_path, e))?;

    let options = kube::config::KubeConfigOptions {
        context: Some(context_name.to_string()),
        ..Default::default()
    };

    let config = Config::from_custom_kubeconfig(kubeconfig, &options).await
        .map_err(|e| format!("Failed to load config: {}", e))?;
    
    Client::try_from(config)
        .map_err(|e| format!("Failed to create client: {}", e))
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
    use k8s_openapi::api::core::v1::Namespace;
    use kube::api::ListParams;

    let client = create_client_for_context(&context_name).await?;
    let ns_api: Api<Namespace> = Api::all(client);
    let lp = ListParams::default();
    
    let list = ns_api.list(&lp).await
        .map_err(|e| format!("Failed to list namespaces: {}", e))?;
        
    let names = list.items.into_iter()
        .filter_map(|ns| ns.metadata.name)
        .collect();
        
    Ok(names)
}

fn map_pod_to_summary(p: Pod) -> PodSummary {
    let status = p.status.as_ref().map(|s| s.phase.clone().unwrap_or_default()).unwrap_or_default();
    let name = p.metadata.name.unwrap_or_default();
    let namespace = p.metadata.namespace.unwrap_or_default();
    let age = p.metadata.creation_timestamp.as_ref()
        .map(|t| {
            // k8s-openapi 0.27 uses `jiff` by default or `chrono` if configured, but t.0 returns the inner type
            // Convert timestamp string to chrono DateTime to be safe across versions or just parse it
            if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(&t.0.to_string()) {
                let duration = chrono::Utc::now().signed_duration_since(ts);
                let days = duration.num_days();
                if days > 0 {
                    format!("{}d", days)
                } else {
                    let hours = duration.num_hours();
                    if hours > 0 {
                        format!("{}h", hours)
                    } else {
                        let minutes = duration.num_minutes();
                        if minutes > 0 {
                            format!("{}m", minutes)
                        } else {
                            format!("{}s", duration.num_seconds())
                        }
                    }
                }
            } else {
                "-".to_string()
            }
        })
        .unwrap_or_default();
    
    let creation_timestamp = p.metadata.creation_timestamp.as_ref().map(|t| t.0.to_string());

    let node = p.spec.as_ref().and_then(|s| s.node_name.clone()).unwrap_or_default();
    
    let container_statuses = p.status.as_ref().and_then(|s| s.container_statuses.as_ref());
    let containers = container_statuses.map(|s| s.len()).unwrap_or(0);
    let restarts: i32 = container_statuses.map(|s| s.iter().map(|cs| cs.restart_count).sum()).unwrap_or(0);
    
    let qos = p.status.as_ref().and_then(|s| s.qos_class.clone()).unwrap_or_default();
    
    let controlled_by = p.metadata.owner_references.as_ref()
        .and_then(|refs| refs.first())
        .map(|r| r.kind.clone())
        .unwrap_or_else(|| "-".to_string());

    PodSummary {
        name,
        namespace,
        status,
        age,
        creation_timestamp,
        containers,
        restarts,
        node,
        qos,
        controlled_by,
    }
}

#[tauri::command]
pub async fn list_pods(context_name: String, namespace: String) -> Result<Vec<PodSummary>, String> {
    use kube::api::ListParams;

    let client = create_client_for_context(&context_name).await?;

    let pods: Api<Pod> = if namespace == "all" {
        Api::all(client)
    } else {
        Api::namespaced(client, &namespace)
    };
    
    let lp = ListParams::default();
    
    let pod_list = pods.list(&lp).await
        .map_err(|e| format!("Failed to list pods: {}", e))?;

    let summaries = pod_list.items.into_iter().map(map_pod_to_summary).collect();

    Ok(summaries)
}

#[tauri::command]
pub async fn delete_pod(context_name: String, namespace: String, pod_name: String) -> Result<(), String> {
    use kube::api::DeleteParams;

    let client = create_client_for_context(&context_name).await?;
    let pods: Api<Pod> = Api::namespaced(client, &namespace);
    
    pods.delete(&pod_name, &DeleteParams::default()).await
        .map_err(|e| format!("Failed to delete pod: {}", e))?;
        
    Ok(())
}

#[derive(Clone, serde::Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum PodEvent {
    Added(PodSummary),
    #[allow(dead_code)]
    Modified(PodSummary),
    Deleted(PodSummary),
    #[allow(dead_code)]
    Restarted(Vec<PodSummary>),
}

// Global variable or state to manage cancellation would be better, but for this demo/clone
// we will just start a new loop. The frontend should handle deduplication or we should use an ID.
// Note: This naive approach might spawn multiple watchers if called repeatedly.
// In a real app, use Tauri State with a Mutex<HashMap<String, AbortHandle>>.

#[tauri::command]
pub async fn start_pod_watch(
    window: Window, 
    context_name: String, 
    namespace: String
) -> Result<(), String> {
    use kube::runtime::watcher::Config as WatchConfig;

    let client = create_client_for_context(&context_name).await?;
    
    let api: Api<Pod> = if namespace == "all" {
        Api::all(client)
    } else {
        Api::namespaced(client, &namespace)
    };

    let config = WatchConfig::default();
    
    // Spawn a task to watch
    tauri::async_runtime::spawn(async move {
        let mut stream = watcher(api, config).boxed();

        while let Some(result) = stream.next().await {
            match result {
                Ok(event) => {
                    let pod_event = match event {
                        watcher::Event::Apply(pod) => PodEvent::Added(map_pod_to_summary(pod)),
                        watcher::Event::Delete(pod) => PodEvent::Deleted(map_pod_to_summary(pod)),
                        watcher::Event::InitApply(pod) => PodEvent::Added(map_pod_to_summary(pod)),
                        _ => continue,
                    };
                    
                    if let Err(e) = window.emit("pod_event", pod_event) {
                        // Window might be closed
                        println!("Failed to emit event: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    println!("Watch error: {}", e);
                    // Decide whether to break or continue
                }
            }
        }
    });

    Ok(())
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct PodSummary {
    name: String,
    namespace: String,
    status: String,
    age: String,
    creation_timestamp: Option<String>,
    containers: usize,
    restarts: i32,
    node: String,
    qos: String,
    controlled_by: String,
}
