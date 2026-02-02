// ... imports ...
use crate::cluster_manager::ClusterManagerState;
use crate::config;
use futures::StreamExt;
use k8s_openapi::api::apps::v1::{DaemonSet, Deployment, ReplicaSet, StatefulSet};
use k8s_openapi::api::autoscaling::v1::HorizontalPodAutoscaler;
use k8s_openapi::api::batch::v1::{CronJob, Job};
use k8s_openapi::api::core::v1::{
    ConfigMap, Endpoints, Event, LimitRange, Node, PersistentVolume, PersistentVolumeClaim, Pod,
    ResourceQuota, Secret, Service, ServiceAccount,
};
use k8s_openapi::api::networking::v1::{Ingress, NetworkPolicy};
use k8s_openapi::api::policy::v1::PodDisruptionBudget;
use k8s_openapi::api::rbac::v1::{ClusterRole, Role};
use k8s_openapi::api::storage::v1::StorageClass;
use kube::config::Kubeconfig;
use kube::runtime::watcher;
use kube::{Api, Client, Config};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::async_runtime::JoinHandle;
use tauri::{Emitter, State, Window};

pub struct WatcherState(pub Arc<Mutex<HashMap<String, JoinHandle<()>>>>);

impl Default for WatcherState {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }
}

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
async fn create_client_for_cluster(
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
    use k8s_openapi::api::core::v1::Namespace;
    use kube::api::ListParams;

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

fn probe_to_info(probe_type: &str, probe: &k8s_openapi::api::core::v1::Probe) -> ProbeInfo {
    let (handler_type, details) = if let Some(http) = probe.http_get.as_ref() {
        let path = http.path.clone().unwrap_or_else(|| "/".to_string());
        let port = match &http.port {
            k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(n) => n.to_string(),
            k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::String(s) => s.clone(),
        };
        let scheme = http.scheme.clone().unwrap_or_else(|| "HTTP".to_string());
        (
            "httpGet".to_string(),
            format!("{}://{}:{}{}", scheme, "localhost", port, path),
        )
    } else if let Some(tcp) = probe.tcp_socket.as_ref() {
        let port = match &tcp.port {
            k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(n) => n.to_string(),
            k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::String(s) => s.clone(),
        };
        ("tcpSocket".to_string(), format!(":{}", port))
    } else if let Some(exec) = probe.exec.as_ref() {
        let command = exec
            .command
            .as_ref()
            .map(|c| c.join(" "))
            .unwrap_or_default();
        ("exec".to_string(), command)
    } else {
        ("unknown".to_string(), "".to_string())
    };

    ProbeInfo {
        probe_type: probe_type.to_string(),
        handler_type,
        details,
        initial_delay_seconds: probe.initial_delay_seconds.unwrap_or(0),
        period_seconds: probe.period_seconds.unwrap_or(10),
        timeout_seconds: probe.timeout_seconds.unwrap_or(1),
        success_threshold: probe.success_threshold.unwrap_or(1),
        failure_threshold: probe.failure_threshold.unwrap_or(3),
    }
}

fn map_pod_to_summary(p: Pod) -> PodSummary {
    let status = p
        .status
        .as_ref()
        .map(|s| s.phase.clone().unwrap_or_default())
        .unwrap_or_default();
    let name = p.metadata.name.clone().unwrap_or_default();
    let namespace = p.metadata.namespace.clone().unwrap_or_default();
    let age = p
        .metadata
        .creation_timestamp
        .as_ref()
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

    let creation_timestamp = p
        .metadata
        .creation_timestamp
        .as_ref()
        .map(|t| t.0.to_string());

    let node = p
        .spec
        .as_ref()
        .and_then(|s| s.node_name.clone())
        .unwrap_or_default();

    let container_statuses = p
        .status
        .as_ref()
        .and_then(|s| s.container_statuses.as_ref());
    let containers = container_statuses.map(|s| s.len()).unwrap_or(0);
    let restarts: i32 = container_statuses
        .map(|s| s.iter().map(|cs| cs.restart_count).sum())
        .unwrap_or(0);

    let qos = p
        .status
        .as_ref()
        .and_then(|s| s.qos_class.clone())
        .unwrap_or_default();

    let controlled_by = p
        .metadata
        .owner_references
        .as_ref()
        .and_then(|refs| refs.first())
        .map(|r| format!("{}/{}", r.kind, r.name))
        .unwrap_or_else(|| "-".to_string());

    // Labels and annotations
    let labels = p.metadata.labels.clone().unwrap_or_default();
    let annotations = p.metadata.annotations.clone().unwrap_or_default();

    // Network info
    let pod_ip = p
        .status
        .as_ref()
        .and_then(|s| s.pod_ip.clone())
        .unwrap_or_else(|| "-".to_string());
    let host_ip = p
        .status
        .as_ref()
        .and_then(|s| s.host_ip.clone())
        .unwrap_or_else(|| "-".to_string());

    // Service account
    let service_account = p
        .spec
        .as_ref()
        .and_then(|s| s.service_account_name.clone())
        .unwrap_or_else(|| "default".to_string());

    // Priority class
    let priority_class = p
        .spec
        .as_ref()
        .and_then(|s| s.priority_class_name.clone())
        .unwrap_or_else(|| "-".to_string());

    // Container details
    let mut container_details = Vec::new();
    if let Some(spec) = p.spec.as_ref() {
        for container in &spec.containers {
            let container_status = container_statuses
                .and_then(|statuses| statuses.iter().find(|s| s.name == container.name))
                .cloned();

            let ready = container_status.as_ref().map(|s| s.ready).unwrap_or(false);
            let restart_count = container_status
                .as_ref()
                .map(|s| s.restart_count)
                .unwrap_or(0);

            let state = if let Some(cs) = container_status.as_ref() {
                if cs.state.as_ref().and_then(|s| s.running.as_ref()).is_some() {
                    "Running".to_string()
                } else if cs.state.as_ref().and_then(|s| s.waiting.as_ref()).is_some() {
                    let reason = cs
                        .state
                        .as_ref()
                        .and_then(|s| s.waiting.as_ref())
                        .and_then(|w| w.reason.clone())
                        .unwrap_or_else(|| "Waiting".to_string());
                    format!("Waiting: {}", reason)
                } else if cs
                    .state
                    .as_ref()
                    .and_then(|s| s.terminated.as_ref())
                    .is_some()
                {
                    let reason = cs
                        .state
                        .as_ref()
                        .and_then(|s| s.terminated.as_ref())
                        .and_then(|t| t.reason.clone())
                        .unwrap_or_else(|| "Terminated".to_string());
                    format!("Terminated: {}", reason)
                } else {
                    "Unknown".to_string()
                }
            } else {
                "Unknown".to_string()
            };

            let resources = container.resources.as_ref();
            let cpu_request = resources
                .and_then(|r| r.requests.as_ref())
                .and_then(|req| req.get("cpu"))
                .map(|q| q.0.clone());
            let cpu_limit = resources
                .and_then(|r| r.limits.as_ref())
                .and_then(|lim| lim.get("cpu"))
                .map(|q| q.0.clone());
            let memory_request = resources
                .and_then(|r| r.requests.as_ref())
                .and_then(|req| req.get("memory"))
                .map(|q| q.0.clone());
            let memory_limit = resources
                .and_then(|r| r.limits.as_ref())
                .and_then(|lim| lim.get("memory"))
                .map(|q| q.0.clone());

            // Ports
            let ports = container
                .ports
                .as_ref()
                .map(|ports| {
                    ports
                        .iter()
                        .map(|p| ContainerPort {
                            name: p.name.clone(),
                            container_port: p.container_port,
                            host_port: p.host_port,
                            protocol: p.protocol.clone().unwrap_or_else(|| "TCP".to_string()),
                        })
                        .collect()
                })
                .unwrap_or_default();

            // Environment variables
            let env = container
                .env
                .as_ref()
                .map(|envs| {
                    envs.iter()
                        .map(|e| {
                            let value_from = if e.value_from.is_some() {
                                Some("(from ConfigMap/Secret)".to_string())
                            } else {
                                None
                            };
                            EnvVar {
                                name: e.name.clone(),
                                value: e.value.clone(),
                                value_from,
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();

            // Volume mounts
            let volume_mounts = container
                .volume_mounts
                .as_ref()
                .map(|mounts| {
                    mounts
                        .iter()
                        .map(|m| VolumeMount {
                            name: m.name.clone(),
                            mount_path: m.mount_path.clone(),
                            sub_path: m.sub_path.clone(),
                            read_only: m.read_only.unwrap_or(false),
                        })
                        .collect()
                })
                .unwrap_or_default();

            // Probes
            let mut probes = Vec::new();
            if let Some(liveness) = container.liveness_probe.as_ref() {
                probes.push(probe_to_info("liveness", liveness));
            }
            if let Some(readiness) = container.readiness_probe.as_ref() {
                probes.push(probe_to_info("readiness", readiness));
            }
            if let Some(startup) = container.startup_probe.as_ref() {
                probes.push(probe_to_info("startup", startup));
            }

            let image_pull_policy = container
                .image_pull_policy
                .clone()
                .unwrap_or_else(|| "IfNotPresent".to_string());

            container_details.push(ContainerInfo {
                name: container.name.clone(),
                image: container.image.clone().unwrap_or_default(),
                image_pull_policy,
                ready,
                restart_count,
                state,
                cpu_request,
                cpu_limit,
                memory_request,
                memory_limit,
                ports,
                env,
                volume_mounts,
                probes,
            });
        }
    }

    // Volumes
    let mut volumes = Vec::new();
    if let Some(spec) = p.spec.as_ref() {
        if let Some(vols) = spec.volumes.as_ref() {
            for vol in vols {
                let volume_type = if vol.config_map.is_some() {
                    "ConfigMap".to_string()
                } else if vol.secret.is_some() {
                    "Secret".to_string()
                } else if vol.empty_dir.is_some() {
                    "EmptyDir".to_string()
                } else if vol.host_path.is_some() {
                    "HostPath".to_string()
                } else if vol.persistent_volume_claim.is_some() {
                    "PersistentVolumeClaim".to_string()
                } else if vol.projected.is_some() {
                    "Projected".to_string()
                } else if vol.downward_api.is_some() {
                    "DownwardAPI".to_string()
                } else {
                    "Other".to_string()
                };

                volumes.push(VolumeInfo {
                    name: vol.name.clone(),
                    volume_type,
                });
            }
        }
    }

    // Conditions
    let mut conditions = Vec::new();
    if let Some(status) = p.status.as_ref() {
        if let Some(conds) = status.conditions.as_ref() {
            for cond in conds {
                conditions.push(PodCondition {
                    condition_type: cond.type_.clone(),
                    status: cond.status.clone(),
                    reason: cond.reason.clone(),
                    message: cond.message.clone(),
                    last_transition_time: cond
                        .last_transition_time
                        .as_ref()
                        .map(|t| t.0.to_string()),
                });
            }
        }
    }

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
        labels,
        annotations,
        pod_ip,
        host_ip,
        service_account,
        priority_class,
        container_details,
        volumes,
        conditions,
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

    let pod_list = pods
        .list(&lp)
        .await
        .map_err(|e| format!("Failed to list pods: {}", e))?;

    let summaries = pod_list.items.into_iter().map(map_pod_to_summary).collect();

    Ok(summaries)
}

#[tauri::command]
pub async fn delete_pod(
    context_name: String,
    namespace: String,
    pod_name: String,
) -> Result<(), String> {
    use kube::api::DeleteParams;

    let client = create_client_for_context(&context_name).await?;
    let pods: Api<Pod> = Api::namespaced(client, &namespace);

    pods.delete(&pod_name, &DeleteParams::default())
        .await
        .map_err(|e| format!("Failed to delete pod: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn get_pod_events(
    context_name: String,
    namespace: String,
    pod_name: String,
) -> Result<Vec<PodEventInfo>, String> {
    use k8s_openapi::api::core::v1::Event;
    use kube::api::ListParams;

    let client = create_client_for_context(&context_name).await?;
    let events: Api<Event> = Api::namespaced(client, &namespace);

    let lp = ListParams::default().fields(&format!("involvedObject.name={}", pod_name));

    let event_list = events
        .list(&lp)
        .await
        .map_err(|e| format!("Failed to list events: {}", e))?;

    let mut event_infos: Vec<PodEventInfo> = event_list
        .items
        .into_iter()
        .map(|e| {
            let source = e
                .source
                .as_ref()
                .and_then(|s| s.component.clone())
                .unwrap_or_else(|| "unknown".to_string());

            PodEventInfo {
                event_type: e.type_.unwrap_or_else(|| "Normal".to_string()),
                reason: e.reason.unwrap_or_default(),
                message: e.message.unwrap_or_default(),
                count: e.count.unwrap_or(1),
                first_timestamp: e.first_timestamp.as_ref().map(|t| t.0.to_string()),
                last_timestamp: e.last_timestamp.as_ref().map(|t| t.0.to_string()),
                source,
            }
        })
        .collect();

    // Sort by last_timestamp descending (most recent first)
    event_infos.sort_by(|a, b| b.last_timestamp.as_ref().cmp(&a.last_timestamp.as_ref()));

    Ok(event_infos)
}

#[tauri::command]
pub async fn stream_container_logs(
    window: Window,
    context_name: String,
    namespace: String,
    pod_name: String,
    container_name: String,
    stream_id: String,
) -> Result<(), String> {
    use futures::{AsyncBufReadExt, TryStreamExt};
    use k8s_openapi::api::core::v1::Pod;
    use kube::api::LogParams;

    let client = create_client_for_context(&context_name).await?;
    let pods: Api<Pod> = Api::namespaced(client, &namespace);

    let log_params = LogParams {
        follow: true,
        tail_lines: Some(1000),
        container: Some(container_name.clone()),
        ..Default::default()
    };

    // Spawn a task to stream logs
    tauri::async_runtime::spawn(async move {
        match pods.log_stream(&pod_name, &log_params).await {
            Ok(stream) => {
                let mut lines = stream.lines();
                loop {
                    match lines.try_next().await {
                        Ok(Some(line)) => {
                            let event_name = format!("container_logs_{}", stream_id);
                            if let Err(e) = window.emit(&event_name, line) {
                                println!("Failed to emit log line: {}", e);
                                break;
                            }
                        }
                        Ok(None) => {
                            // Stream ended
                            break;
                        }
                        Err(e) => {
                            println!("Error reading log line: {}", e);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                println!("Failed to open log stream: {}", e);
            }
        }
    });

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
    namespace: String,
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
pub struct ContainerPort {
    name: Option<String>,
    container_port: i32,
    host_port: Option<i32>,
    protocol: String,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct EnvVar {
    name: String,
    value: Option<String>,
    value_from: Option<String>,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct VolumeMount {
    name: String,
    mount_path: String,
    sub_path: Option<String>,
    read_only: bool,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct ProbeInfo {
    probe_type: String,   // "liveness", "readiness", "startup"
    handler_type: String, // "httpGet", "tcpSocket", "exec"
    details: String,
    initial_delay_seconds: i32,
    period_seconds: i32,
    timeout_seconds: i32,
    success_threshold: i32,
    failure_threshold: i32,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct ContainerInfo {
    name: String,
    image: String,
    image_pull_policy: String,
    ready: bool,
    restart_count: i32,
    state: String,
    cpu_request: Option<String>,
    cpu_limit: Option<String>,
    memory_request: Option<String>,
    memory_limit: Option<String>,
    ports: Vec<ContainerPort>,
    env: Vec<EnvVar>,
    volume_mounts: Vec<VolumeMount>,
    probes: Vec<ProbeInfo>,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct VolumeInfo {
    name: String,
    volume_type: String,
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
    // Extended details
    labels: std::collections::BTreeMap<String, String>,
    annotations: std::collections::BTreeMap<String, String>,
    pod_ip: String,
    host_ip: String,
    service_account: String,
    priority_class: String,
    container_details: Vec<ContainerInfo>,
    volumes: Vec<VolumeInfo>,
    conditions: Vec<PodCondition>,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct PodCondition {
    condition_type: String,
    status: String,
    reason: Option<String>,
    message: Option<String>,
    last_transition_time: Option<String>,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct PodEventInfo {
    event_type: String, // "Normal", "Warning"
    reason: String,
    message: String,
    count: i32,
    first_timestamp: Option<String>,
    last_timestamp: Option<String>,
    source: String,
}

// NEW: Cluster-based commands using cluster IDs

#[tauri::command]
pub async fn cluster_list_namespaces(
    cluster_id: String,
    state: State<'_, ClusterManagerState>,
) -> Result<Vec<String>, String> {
    use k8s_openapi::api::core::v1::Namespace;
    use kube::api::ListParams;

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

#[tauri::command]
pub async fn cluster_list_pods(
    cluster_id: String,
    namespace: String,
    state: State<'_, ClusterManagerState>,
) -> Result<Vec<PodSummary>, String> {
    let client = create_client_for_cluster(&cluster_id, &state).await?;

    let pods: Api<Pod> = if namespace == "all" {
        Api::all(client)
    } else {
        Api::namespaced(client, &namespace)
    };

    let lp = kube::api::ListParams::default();
    let list = pods
        .list(&lp)
        .await
        .map_err(|e| format!("Failed to list pods: {}", e))?;

    let summaries = list
        .items
        .iter()
        .map(|p| map_pod_to_summary(p.clone()))
        .collect();
    Ok(summaries)
}

#[tauri::command]
pub async fn cluster_delete_pod(
    cluster_id: String,
    namespace: String,
    pod_name: String,
    state: State<'_, ClusterManagerState>,
) -> Result<(), String> {
    let client = create_client_for_cluster(&cluster_id, &state).await?;
    let pods: Api<Pod> = Api::namespaced(client, &namespace);

    pods.delete(&pod_name, &kube::api::DeleteParams::default())
        .await
        .map_err(|e| format!("Failed to delete pod: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn cluster_get_pod_events(
    cluster_id: String,
    namespace: String,
    pod_name: String,
    state: State<'_, ClusterManagerState>,
) -> Result<Vec<PodEventInfo>, String> {
    use k8s_openapi::api::core::v1::Event;
    use kube::api::ListParams;

    let client = create_client_for_cluster(&cluster_id, &state).await?;
    let events_api: Api<Event> = Api::namespaced(client, &namespace);

    let field_selector = format!("involvedObject.name={}", pod_name);
    let lp = ListParams::default().fields(&field_selector);

    let events_list = events_api
        .list(&lp)
        .await
        .map_err(|e| format!("Failed to list events: {}", e))?;

    let mut event_infos: Vec<PodEventInfo> = events_list
        .items
        .iter()
        .map(|event| {
            let event_type = event
                .type_
                .as_ref()
                .unwrap_or(&"Unknown".to_string())
                .clone();
            let reason = event
                .reason
                .as_ref()
                .unwrap_or(&"Unknown".to_string())
                .clone();
            let message = event.message.as_ref().unwrap_or(&"".to_string()).clone();
            let count = event.count.unwrap_or(1);
            let first_timestamp = event.first_timestamp.as_ref().map(|t| t.0.to_string());
            let last_timestamp = event.last_timestamp.as_ref().map(|t| t.0.to_string());
            let source = event
                .source
                .as_ref()
                .and_then(|s| s.component.as_ref())
                .cloned()
                .unwrap_or_default();

            PodEventInfo {
                event_type,
                reason,
                message,
                count,
                first_timestamp,
                last_timestamp,
                source,
            }
        })
        .collect();

    event_infos.sort_by(|a, b| b.last_timestamp.cmp(&a.last_timestamp));

    Ok(event_infos)
}

#[tauri::command]
pub async fn cluster_stream_container_logs(
    cluster_id: String,
    namespace: String,
    pod_name: String,
    container_name: String,
    stream_id: String,
    window: Window,
    state: State<'_, ClusterManagerState>,
    watcher_state: State<'_, WatcherState>,
) -> Result<(), String> {
    use futures::{AsyncBufReadExt, TryStreamExt};
    use kube::api::LogParams;

    let client = create_client_for_cluster(&cluster_id, &state).await?;
    let pods: Api<Pod> = Api::namespaced(client, &namespace);

    let log_params = LogParams {
        follow: true,
        tail_lines: Some(1000),
        container: Some(container_name.clone()),
        ..Default::default()
    };

    let key = format!("logs:{}", stream_id);

    // Abort existing if any
    {
        let mut watchers = watcher_state.0.lock().unwrap();
        if let Some(handle) = watchers.remove(&key) {
            handle.abort();
        }
    }

    let watchers = watcher_state.inner().0.clone();
    let key_clone = key.clone();

    let handle = tauri::async_runtime::spawn(async move {
        match pods.log_stream(&pod_name, &log_params).await {
            Ok(stream) => {
                let mut lines = stream.lines();
                loop {
                    match lines.try_next().await {
                        Ok(Some(line)) => {
                            let event_name = format!("container_logs_{}", stream_id);
                            if let Err(e) = window.emit(&event_name, line) {
                                println!("Failed to emit log line: {}", e);
                                break;
                            }
                        }
                        Ok(None) => break,
                        Err(e) => {
                            println!("Error reading log line: {}", e);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                println!("Failed to open log stream: {}", e);
            }
        }

        // Cleanup
        let mut watchers = watchers.lock().unwrap();
        watchers.remove(&key_clone);
    });

    // Store new handle
    {
        let mut watchers = watcher_state.0.lock().unwrap();
        watchers.insert(key, handle);
    }

    Ok(())
}

#[tauri::command]
pub async fn cluster_start_pod_watch(
    cluster_id: String,
    namespace: String,
    window: Window,
    state: State<'_, ClusterManagerState>,
    watcher_state: State<'_, WatcherState>,
) -> Result<(), String> {
    use kube::runtime::watcher::Config as WatchConfig;

    let client = create_client_for_cluster(&cluster_id, &state).await?;

    let api: Api<Pod> = if namespace == "all" {
        Api::all(client)
    } else {
        Api::namespaced(client, &namespace)
    };

    let config = WatchConfig::default();
    let key = format!("pod_watch:{}:{}", cluster_id, namespace);

    // Abort existing if any
    {
        let mut watchers = watcher_state.0.lock().unwrap();
        if let Some(handle) = watchers.remove(&key) {
            handle.abort();
        }
    }

    let watchers = watcher_state.inner().0.clone();
    let key_clone = key.clone();

    let handle = tauri::async_runtime::spawn(async move {
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
                        println!("Failed to emit event: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    println!("Watch error: {}", e);
                }
            }
        }

        // Cleanup
        let mut watchers = watchers.lock().unwrap();
        watchers.remove(&key_clone);
    });

    // Store new handle
    {
        let mut watchers = watcher_state.0.lock().unwrap();
        watchers.insert(key, handle);
    }

    Ok(())
}

// --- Dashboard Metrics ---

#[derive(serde::Serialize, Default, Debug)]
pub struct ResourceStats {
    pub capacity: f64,
    pub allocatable: f64,
    pub requests: f64,
    pub limits: f64,
    pub usage: f64,
}

#[derive(serde::Serialize, Default, Debug)]
pub struct ClusterMetrics {
    pub cpu: ResourceStats,
    pub memory: ResourceStats,
    pub pods: ResourceStats,
}

#[derive(serde::Serialize, Debug)]
pub struct WarningEvent {
    pub message: String,
    pub object: String,
    pub type_: String,
    pub age: String,
    pub count: i32,
}

fn parse_cpu(q: &str) -> f64 {
    if q.ends_with('m') {
        q.trim_end_matches('m').parse::<f64>().unwrap_or(0.0) / 1000.0
    } else {
        q.parse::<f64>().unwrap_or(0.0)
    }
}

fn parse_memory(q: &str) -> f64 {
    let q = q.trim();
    if let Some(val) = q.strip_suffix("Ki") {
        val.parse::<f64>().unwrap_or(0.0) * 1024.0
    } else if let Some(val) = q.strip_suffix("Mi") {
        val.parse::<f64>().unwrap_or(0.0) * 1024.0f64.powi(2)
    } else if let Some(val) = q.strip_suffix("Gi") {
        val.parse::<f64>().unwrap_or(0.0) * 1024.0f64.powi(3)
    } else if let Some(val) = q.strip_suffix("Ti") {
        val.parse::<f64>().unwrap_or(0.0) * 1024.0f64.powi(4)
    } else if let Some(val) = q.strip_suffix("m") {
        val.parse::<f64>().unwrap_or(0.0) / 1000.0
    } else {
        q.parse::<f64>().unwrap_or(0.0)
    }
}

#[tauri::command]
pub async fn cluster_get_metrics(
    cluster_id: String,
    state: State<'_, ClusterManagerState>,
) -> Result<ClusterMetrics, String> {
    let client = create_client_for_cluster(&cluster_id, &state).await?;

    let nodes: Api<Node> = Api::all(client.clone());
    let pods: Api<Pod> = Api::all(client.clone());

    let node_list = nodes
        .list(&Default::default())
        .await
        .map_err(|e| e.to_string())?;
    let pod_list = pods
        .list(&Default::default())
        .await
        .map_err(|e| e.to_string())?;

    let mut metrics = ClusterMetrics::default();

    // Node Capacity & Allocatable
    for node in node_list.items {
        if let Some(status) = node.status {
            if let Some(cap) = status.capacity {
                if let Some(cpu) = cap.get("cpu") {
                    metrics.cpu.capacity += parse_cpu(&cpu.0);
                }
                if let Some(mem) = cap.get("memory") {
                    metrics.memory.capacity += parse_memory(&mem.0);
                }
                if let Some(p) = cap.get("pods") {
                    metrics.pods.capacity += parse_cpu(&p.0);
                }
            }
            if let Some(alloc) = status.allocatable {
                if let Some(cpu) = alloc.get("cpu") {
                    metrics.cpu.allocatable += parse_cpu(&cpu.0);
                }
                if let Some(mem) = alloc.get("memory") {
                    metrics.memory.allocatable += parse_memory(&mem.0);
                }
                if let Some(p) = alloc.get("pods") {
                    metrics.pods.allocatable += parse_cpu(&p.0);
                }
            }
        }
    }

    // Pod Requests & Limits
    for pod in pod_list.items {
        // Skip finished pods
        if let Some(status) = &pod.status {
            if let Some(phase) = &status.phase {
                if phase == "Succeeded" || phase == "Failed" {
                    continue;
                }
            }
        }

        metrics.pods.usage += 1.0;

        if let Some(spec) = pod.spec {
            for container in spec.containers {
                if let Some(reqs) = container
                    .resources
                    .as_ref()
                    .and_then(|r| r.requests.as_ref())
                {
                    if let Some(cpu) = reqs.get("cpu") {
                        metrics.cpu.requests += parse_cpu(&cpu.0);
                    }
                    if let Some(mem) = reqs.get("memory") {
                        metrics.memory.requests += parse_memory(&mem.0);
                    }
                }
                if let Some(lims) = container.resources.as_ref().and_then(|r| r.limits.as_ref()) {
                    if let Some(cpu) = lims.get("cpu") {
                        metrics.cpu.limits += parse_cpu(&cpu.0);
                    }
                    if let Some(mem) = lims.get("memory") {
                        metrics.memory.limits += parse_memory(&mem.0);
                    }
                }
            }
        }
    }

    Ok(metrics)
}

#[tauri::command]
pub async fn cluster_get_events(
    cluster_id: String,
    state: State<'_, ClusterManagerState>,
) -> Result<Vec<WarningEvent>, String> {
    let client = create_client_for_cluster(&cluster_id, &state).await?;
    let events: Api<Event> = Api::all(client);

    let lp = kube::api::ListParams::default();
    let event_list = events.list(&lp).await.map_err(|e| e.to_string())?;

    let mut warnings = Vec::new();
    let now = chrono::Utc::now();

    for e in event_list.items {
        if e.type_.as_deref() == Some("Warning") {
            let age = if let Some(last_ts) = &e.last_timestamp {
                let last_ts_str = last_ts.0.to_string();
                let last_ts_parsed = chrono::DateTime::parse_from_rfc3339(&last_ts_str)
                    .unwrap()
                    .with_timezone(&chrono::Utc);
                let duration = now.signed_duration_since(last_ts_parsed);
                if duration.num_days() > 0 {
                    format!("{}d", duration.num_days())
                } else if duration.num_hours() > 0 {
                    format!("{}h", duration.num_hours())
                } else if duration.num_minutes() > 0 {
                    format!("{}m", duration.num_minutes())
                } else {
                    format!("{}s", duration.num_seconds())
                }
            } else {
                "-".to_string()
            };

            warnings.push(WarningEvent {
                message: e.message.unwrap_or_default(),
                object: format!(
                    "{}/{}",
                    e.involved_object.kind.unwrap_or_default(),
                    e.involved_object.name.unwrap_or_default()
                ),
                type_: e.type_.unwrap_or_default(),
                age,
                count: e.count.unwrap_or(1),
            });
        }
    }

    // Limit to 50 most recent warnings
    warnings.reverse();
    warnings.truncate(50);

    Ok(warnings)
}

// --- Workload Resources ---

#[derive(serde::Serialize, Clone, Debug)]
pub struct WorkloadSummary {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub age: String,
    pub labels: std::collections::BTreeMap<String, String>,
    pub status: String,
    pub images: Vec<String>,
    pub created_at: i64,
}

fn calculate_age(
    timestamp: Option<&k8s_openapi::apimachinery::pkg::apis::meta::v1::Time>,
) -> String {
    if let Some(ts) = timestamp {
        let now = chrono::Utc::now();
        // Convert k8s Time (jiff/chrono wrapper) to chrono DateTime
        // Using string parsing as reliable fallback
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&ts.0.to_string()) {
            let duration = now.signed_duration_since(dt.with_timezone(&chrono::Utc));
            if duration.num_days() > 0 {
                format!("{}d", duration.num_days())
            } else if duration.num_hours() > 0 {
                format!("{}h", duration.num_hours())
            } else if duration.num_minutes() > 0 {
                format!("{}m", duration.num_minutes())
            } else {
                format!("{}s", duration.num_seconds())
            }
        } else {
            "-".to_string()
        }
    } else {
        "-".to_string()
    }
}

fn get_created_at(timestamp: Option<&k8s_openapi::apimachinery::pkg::apis::meta::v1::Time>) -> i64 {
    if let Some(ts) = timestamp {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&ts.0.to_string()) {
            return dt.timestamp();
        }
    }
    0
}

fn map_deployment_to_summary(d: Deployment) -> WorkloadSummary {
    let meta = d.metadata;
    let spec = d.spec.unwrap_or_default();
    let status = d.status.unwrap_or_default();

    let _replicas = status.replicas.unwrap_or(0);
    let ready = status.ready_replicas.unwrap_or(0);
    let status_str = format!("{}/{}", ready, spec.replicas.unwrap_or(1));

    let images = if let Some(template) = spec.template.spec {
        template
            .containers
            .into_iter()
            .map(|c| c.image.unwrap_or_default())
            .collect()
    } else {
        vec![]
    };

    WorkloadSummary {
        id: meta.uid.clone().unwrap_or_default(),
        name: meta.name.clone().unwrap_or_default(),
        namespace: meta.namespace.clone().unwrap_or_default(),
        age: calculate_age(meta.creation_timestamp.as_ref()),
        created_at: get_created_at(meta.creation_timestamp.as_ref()),
        labels: meta.labels.unwrap_or_default(),
        status: status_str,
        images,
    }
}

fn map_statefulset_to_summary(s: StatefulSet) -> WorkloadSummary {
    let meta = s.metadata;
    let spec = s.spec.unwrap_or_default();
    let status = s.status.unwrap_or_default();

    let ready = status.ready_replicas.unwrap_or(0);
    let replicas = spec.replicas.unwrap_or(1);
    let status_str = format!("{}/{}", ready, replicas);

    let images = if let Some(template) = spec.template.spec {
        template
            .containers
            .into_iter()
            .map(|c| c.image.unwrap_or_default())
            .collect()
    } else {
        vec![]
    };

    WorkloadSummary {
        id: meta.uid.clone().unwrap_or_default(),
        name: meta.name.clone().unwrap_or_default(),
        namespace: meta.namespace.clone().unwrap_or_default(),
        age: calculate_age(meta.creation_timestamp.as_ref()),
        created_at: get_created_at(meta.creation_timestamp.as_ref()),
        labels: meta.labels.unwrap_or_default(),
        status: status_str,
        images,
    }
}

fn map_daemonset_to_summary(d: DaemonSet) -> WorkloadSummary {
    let meta = d.metadata;
    let spec = d.spec.unwrap_or_default();
    let status = d.status.unwrap_or_default();

    let desired = status.desired_number_scheduled;
    let ready = status.number_ready;
    let status_str = format!("{}/{}", ready, desired);

    let images = if let Some(template) = spec.template.spec {
        template
            .containers
            .into_iter()
            .map(|c| c.image.unwrap_or_default())
            .collect()
    } else {
        vec![]
    };

    WorkloadSummary {
        id: meta.uid.clone().unwrap_or_default(),
        name: meta.name.clone().unwrap_or_default(),
        namespace: meta.namespace.clone().unwrap_or_default(),
        age: calculate_age(meta.creation_timestamp.as_ref()),
        created_at: get_created_at(meta.creation_timestamp.as_ref()),
        labels: meta.labels.unwrap_or_default(),
        status: status_str,
        images,
    }
}

fn map_replicaset_to_summary(r: ReplicaSet) -> WorkloadSummary {
    let meta = r.metadata;
    let spec = r.spec.unwrap_or_default();
    let status = r.status.unwrap_or_default();

    let ready = status.ready_replicas.unwrap_or(0);
    let replicas = spec.replicas.unwrap_or(1);
    let status_str = format!("{}/{}", ready, replicas);

    let images = if let Some(template) = spec.template {
        if let Some(tspec) = template.spec {
            tspec
                .containers
                .into_iter()
                .map(|c| c.image.unwrap_or_default())
                .collect()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    WorkloadSummary {
        id: meta.uid.clone().unwrap_or_default(),
        name: meta.name.clone().unwrap_or_default(),
        namespace: meta.namespace.clone().unwrap_or_default(),
        age: calculate_age(meta.creation_timestamp.as_ref()),
        created_at: get_created_at(meta.creation_timestamp.as_ref()),
        labels: meta.labels.unwrap_or_default(),
        status: status_str,
        images,
    }
}

fn map_job_to_summary(j: Job) -> WorkloadSummary {
    let meta = j.metadata;
    let spec = j.spec.unwrap_or_default();
    let status = j.status.unwrap_or_default();

    let succeeded = status.succeeded.unwrap_or(0);
    let completions = spec.completions.unwrap_or(1);
    let status_str = format!("{}/{}", succeeded, completions);

    let images = if let Some(template) = spec.template.spec {
        template
            .containers
            .into_iter()
            .map(|c| c.image.unwrap_or_default())
            .collect()
    } else {
        vec![]
    };

    WorkloadSummary {
        id: meta.uid.clone().unwrap_or_default(),
        name: meta.name.clone().unwrap_or_default(),
        namespace: meta.namespace.clone().unwrap_or_default(),
        age: calculate_age(meta.creation_timestamp.as_ref()),
        created_at: get_created_at(meta.creation_timestamp.as_ref()),
        labels: meta.labels.unwrap_or_default(),
        status: status_str,
        images,
    }
}

fn map_cronjob_to_summary(c: CronJob) -> WorkloadSummary {
    let meta = c.metadata;
    let spec = c.spec.unwrap_or_default();
    let status = c.status.unwrap_or_default();

    let active = status.active.map(|a| a.len()).unwrap_or(0);
    let status_str = if active > 0 { "Active" } else { "Suspended" }; // Simplified

    let images = if let Some(job_template) = spec.job_template.spec {
        if let Some(template) = job_template.template.spec {
            template
                .containers
                .into_iter()
                .map(|c| c.image.unwrap_or_default())
                .collect()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    WorkloadSummary {
        id: meta.uid.clone().unwrap_or_default(),
        name: meta.name.clone().unwrap_or_default(),
        namespace: meta.namespace.clone().unwrap_or_default(),
        age: calculate_age(meta.creation_timestamp.as_ref()),
        created_at: get_created_at(meta.creation_timestamp.as_ref()),
        labels: meta.labels.unwrap_or_default(),
        status: status_str.to_string(),
        images,
    }
}

macro_rules! impl_workload_commands {
    ($resource:ty, $list_fn:ident, $delete_fn:ident, $map_fn:ident) => {
        #[tauri::command]
        pub async fn $list_fn(
            cluster_id: String,
            namespace: Option<String>,
            state: State<'_, ClusterManagerState>,
        ) -> Result<Vec<WorkloadSummary>, String> {
            let client = create_client_for_cluster(&cluster_id, &state).await?;
            let api: Api<$resource> = if let Some(ns) = namespace {
                Api::namespaced(client, &ns)
            } else {
                Api::all(client)
            };

            let list = api
                .list(&Default::default())
                .await
                .map_err(|e| e.to_string())?;
            Ok(list.items.into_iter().map($map_fn).collect())
        }

        #[tauri::command]
        pub async fn $delete_fn(
            cluster_id: String,
            namespace: String,
            name: String,
            state: State<'_, ClusterManagerState>,
        ) -> Result<(), String> {
            let client = create_client_for_cluster(&cluster_id, &state).await?;
            let api: Api<$resource> = Api::namespaced(client, &namespace);
            api.delete(&name, &Default::default())
                .await
                .map_err(|e| e.to_string())?;
            Ok(())
        }
    };
}

impl_workload_commands!(
    Deployment,
    cluster_list_deployments,
    cluster_delete_deployment,
    map_deployment_to_summary
);
impl_workload_commands!(
    StatefulSet,
    cluster_list_statefulsets,
    cluster_delete_statefulset,
    map_statefulset_to_summary
);
impl_workload_commands!(
    DaemonSet,
    cluster_list_daemonsets,
    cluster_delete_daemonset,
    map_daemonset_to_summary
);
impl_workload_commands!(
    ReplicaSet,
    cluster_list_replicasets,
    cluster_delete_replicaset,
    map_replicaset_to_summary
);
impl_workload_commands!(
    Job,
    cluster_list_jobs,
    cluster_delete_job,
    map_job_to_summary
);
impl_workload_commands!(
    CronJob,
    cluster_list_cronjobs,
    cluster_delete_cronjob,
    map_cronjob_to_summary
);

// --- Additional Resources ---

// Macro for cluster-scoped resources (PV, StorageClass, etc.)
macro_rules! impl_cluster_resource_commands {
    ($resource:ty, $list_fn:ident, $delete_fn:ident, $map_fn:ident) => {
        #[tauri::command]
        pub async fn $list_fn(
            cluster_id: String,
            _namespace: Option<String>,
            state: State<'_, ClusterManagerState>,
        ) -> Result<Vec<WorkloadSummary>, String> {
            let client = create_client_for_cluster(&cluster_id, &state).await?;
            let api: Api<$resource> = Api::all(client);

            let list = api
                .list(&Default::default())
                .await
                .map_err(|e| e.to_string())?;
            Ok(list.items.into_iter().map($map_fn).collect())
        }

        #[tauri::command]
        pub async fn $delete_fn(
            cluster_id: String,
            _namespace: String,
            name: String,
            state: State<'_, ClusterManagerState>,
        ) -> Result<(), String> {
            let client = create_client_for_cluster(&cluster_id, &state).await?;
            let api: Api<$resource> = Api::all(client);
            api.delete(&name, &Default::default())
                .await
                .map_err(|e| e.to_string())?;
            Ok(())
        }
    };
}

// Config Maps
fn map_configmap_to_summary(c: ConfigMap) -> WorkloadSummary {
    let meta = c.metadata;
    let count = c.data.map(|d| d.len()).unwrap_or(0) + c.binary_data.map(|d| d.len()).unwrap_or(0);

    WorkloadSummary {
        id: meta.uid.clone().unwrap_or_default(),
        name: meta.name.clone().unwrap_or_default(),
        namespace: meta.namespace.clone().unwrap_or_default(),
        age: calculate_age(meta.creation_timestamp.as_ref()),
        created_at: get_created_at(meta.creation_timestamp.as_ref()),
        labels: meta.labels.unwrap_or_default(),
        status: format!("{} items", count),
        images: vec![],
    }
}

// Secrets
fn map_secret_to_summary(s: Secret) -> WorkloadSummary {
    let meta = s.metadata;
    let count = s.data.map(|d| d.len()).unwrap_or(0) + s.string_data.map(|d| d.len()).unwrap_or(0);

    WorkloadSummary {
        id: meta.uid.clone().unwrap_or_default(),
        name: meta.name.clone().unwrap_or_default(),
        namespace: meta.namespace.clone().unwrap_or_default(),
        age: calculate_age(meta.creation_timestamp.as_ref()),
        created_at: get_created_at(meta.creation_timestamp.as_ref()),
        labels: meta.labels.unwrap_or_default(),
        status: format!(
            "{} ({} items)",
            s.type_.unwrap_or_else(|| "Opaque".to_string()),
            count
        ),
        images: vec![],
    }
}

// Resource Quotas
fn map_resource_quota_to_summary(r: ResourceQuota) -> WorkloadSummary {
    let meta = r.metadata;

    WorkloadSummary {
        id: meta.uid.clone().unwrap_or_default(),
        name: meta.name.clone().unwrap_or_default(),
        namespace: meta.namespace.clone().unwrap_or_default(),
        age: calculate_age(meta.creation_timestamp.as_ref()),
        created_at: get_created_at(meta.creation_timestamp.as_ref()),
        labels: meta.labels.unwrap_or_default(),
        status: "Active".to_string(),
        images: vec![],
    }
}

// Limit Ranges
fn map_limit_range_to_summary(l: LimitRange) -> WorkloadSummary {
    let meta = l.metadata;
    WorkloadSummary {
        id: meta.uid.clone().unwrap_or_default(),
        name: meta.name.clone().unwrap_or_default(),
        namespace: meta.namespace.clone().unwrap_or_default(),
        age: calculate_age(meta.creation_timestamp.as_ref()),
        created_at: get_created_at(meta.creation_timestamp.as_ref()),
        labels: meta.labels.unwrap_or_default(),
        status: "Active".to_string(),
        images: vec![],
    }
}

// HPA
fn map_hpa_to_summary(h: HorizontalPodAutoscaler) -> WorkloadSummary {
    let meta = h.metadata;
    let spec = h.spec.unwrap_or_default();
    let status = h.status.unwrap_or_default();

    let current = status.current_replicas;
    let desired = status.desired_replicas;
    let min = spec.min_replicas.unwrap_or(1);
    let max = spec.max_replicas;

    let status_str = format!("{}/{} (min: {}, max: {})", current, desired, min, max);

    WorkloadSummary {
        id: meta.uid.clone().unwrap_or_default(),
        name: meta.name.clone().unwrap_or_default(),
        namespace: meta.namespace.clone().unwrap_or_default(),
        age: calculate_age(meta.creation_timestamp.as_ref()),
        created_at: get_created_at(meta.creation_timestamp.as_ref()),
        labels: meta.labels.unwrap_or_default(),
        status: status_str,
        images: vec![],
    }
}

// PDB
fn map_pdb_to_summary(p: PodDisruptionBudget) -> WorkloadSummary {
    let meta = p.metadata;
    let status = p.status.unwrap_or_default();
    let allowed = status.disruptions_allowed;

    WorkloadSummary {
        id: meta.uid.clone().unwrap_or_default(),
        name: meta.name.clone().unwrap_or_default(),
        namespace: meta.namespace.clone().unwrap_or_default(),
        age: calculate_age(meta.creation_timestamp.as_ref()),
        created_at: get_created_at(meta.creation_timestamp.as_ref()),
        labels: meta.labels.unwrap_or_default(),
        status: format!("Allowed: {}", allowed),
        images: vec![],
    }
}

// Services
fn map_service_to_summary(s: Service) -> WorkloadSummary {
    let meta = s.metadata;
    let spec = s.spec.unwrap_or_default();

    let type_ = spec.type_.unwrap_or_else(|| "ClusterIP".to_string());
    let cluster_ip = spec.cluster_ip.unwrap_or_else(|| "-".to_string());
    let ports = spec
        .ports
        .unwrap_or_default()
        .iter()
        .map(|p| format!("{}", p.port))
        .collect::<Vec<_>>()
        .join(",");

    WorkloadSummary {
        id: meta.uid.clone().unwrap_or_default(),
        name: meta.name.clone().unwrap_or_default(),
        namespace: meta.namespace.clone().unwrap_or_default(),
        age: calculate_age(meta.creation_timestamp.as_ref()),
        created_at: get_created_at(meta.creation_timestamp.as_ref()),
        labels: meta.labels.unwrap_or_default(),
        status: format!("{} ({})", type_, cluster_ip),
        images: vec![ports], // Hijacking images field for ports/info
    }
}

// Endpoints
fn map_endpoints_to_summary(e: Endpoints) -> WorkloadSummary {
    let meta = e.metadata;
    let count = e
        .subsets
        .map(|s| {
            s.iter()
                .map(|ss| ss.addresses.as_ref().map(|a| a.len()).unwrap_or(0))
                .sum::<usize>()
        })
        .unwrap_or(0);

    WorkloadSummary {
        id: meta.uid.clone().unwrap_or_default(),
        name: meta.name.clone().unwrap_or_default(),
        namespace: meta.namespace.clone().unwrap_or_default(),
        age: calculate_age(meta.creation_timestamp.as_ref()),
        created_at: get_created_at(meta.creation_timestamp.as_ref()),
        labels: meta.labels.unwrap_or_default(),
        status: format!("{} endpoints", count),
        images: vec![],
    }
}

// Ingresses
fn map_ingress_to_summary(i: Ingress) -> WorkloadSummary {
    let meta = i.metadata;
    let lbs = i
        .status
        .and_then(|s| s.load_balancer)
        .and_then(|lb| lb.ingress)
        .map(|ing| {
            ing.iter()
                .map(|ip| ip.ip.clone().or(ip.hostname.clone()).unwrap_or_default())
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_default();

    WorkloadSummary {
        id: meta.uid.clone().unwrap_or_default(),
        name: meta.name.clone().unwrap_or_default(),
        namespace: meta.namespace.clone().unwrap_or_default(),
        age: calculate_age(meta.creation_timestamp.as_ref()),
        created_at: get_created_at(meta.creation_timestamp.as_ref()),
        labels: meta.labels.unwrap_or_default(),
        status: lbs,
        images: vec![],
    }
}

// Network Policies
fn map_network_policy_to_summary(n: NetworkPolicy) -> WorkloadSummary {
    let meta = n.metadata;
    WorkloadSummary {
        id: meta.uid.clone().unwrap_or_default(),
        name: meta.name.clone().unwrap_or_default(),
        namespace: meta.namespace.clone().unwrap_or_default(),
        age: calculate_age(meta.creation_timestamp.as_ref()),
        created_at: get_created_at(meta.creation_timestamp.as_ref()),
        labels: meta.labels.unwrap_or_default(),
        status: "Active".to_string(),
        images: vec![],
    }
}

// PVC
fn map_pvc_to_summary(p: PersistentVolumeClaim) -> WorkloadSummary {
    let meta = p.metadata;
    let status = p.status.unwrap_or_default();
    let phase = status.phase.unwrap_or_default();
    let capacity = status
        .capacity
        .and_then(|c| c.get("storage").map(|q| q.0.clone()))
        .unwrap_or_default();

    WorkloadSummary {
        id: meta.uid.clone().unwrap_or_default(),
        name: meta.name.clone().unwrap_or_default(),
        namespace: meta.namespace.clone().unwrap_or_default(),
        age: calculate_age(meta.creation_timestamp.as_ref()),
        created_at: get_created_at(meta.creation_timestamp.as_ref()),
        labels: meta.labels.unwrap_or_default(),
        status: format!("{} ({})", phase, capacity),
        images: vec![],
    }
}

// PV (Cluster Scoped)
fn map_pv_to_summary(p: PersistentVolume) -> WorkloadSummary {
    let meta = p.metadata;
    let status = p.status.unwrap_or_default();
    let phase = status.phase.unwrap_or_default();
    let spec = p.spec.unwrap_or_default();
    let capacity = spec
        .capacity
        .and_then(|c| c.get("storage").map(|q| q.0.clone()))
        .unwrap_or_default();

    WorkloadSummary {
        id: meta.uid.clone().unwrap_or_default(),
        name: meta.name.clone().unwrap_or_default(),
        namespace: "-".to_string(),
        age: calculate_age(meta.creation_timestamp.as_ref()),
        created_at: get_created_at(meta.creation_timestamp.as_ref()),
        labels: meta.labels.unwrap_or_default(),
        status: format!("{} ({})", phase, capacity),
        images: vec![],
    }
}

// Storage Classes (Cluster Scoped)
fn map_storage_class_to_summary(s: StorageClass) -> WorkloadSummary {
    let meta = s.metadata;
    let provisioner = s.provisioner;

    WorkloadSummary {
        id: meta.uid.clone().unwrap_or_default(),
        name: meta.name.clone().unwrap_or_default(),
        namespace: "-".to_string(),
        age: calculate_age(meta.creation_timestamp.as_ref()),
        created_at: get_created_at(meta.creation_timestamp.as_ref()),
        labels: meta.labels.unwrap_or_default(),
        status: "Active".to_string(),
        images: vec![provisioner],
    }
}

// Service Accounts
fn map_service_account_to_summary(s: ServiceAccount) -> WorkloadSummary {
    let meta = s.metadata;

    WorkloadSummary {
        id: meta.uid.clone().unwrap_or_default(),
        name: meta.name.clone().unwrap_or_default(),
        namespace: meta.namespace.clone().unwrap_or_default(),
        age: calculate_age(meta.creation_timestamp.as_ref()),
        created_at: get_created_at(meta.creation_timestamp.as_ref()),
        labels: meta.labels.unwrap_or_default(),
        status: "Active".to_string(),
        images: vec![],
    }
}

// Roles
fn map_role_to_summary(r: Role) -> WorkloadSummary {
    let meta = r.metadata;
    WorkloadSummary {
        id: meta.uid.clone().unwrap_or_default(),
        name: meta.name.clone().unwrap_or_default(),
        namespace: meta.namespace.clone().unwrap_or_default(),
        age: calculate_age(meta.creation_timestamp.as_ref()),
        created_at: get_created_at(meta.creation_timestamp.as_ref()),
        labels: meta.labels.unwrap_or_default(),
        status: "Active".to_string(),
        images: vec![],
    }
}

// Cluster Roles (Cluster Scoped)
fn map_cluster_role_to_summary(r: ClusterRole) -> WorkloadSummary {
    let meta = r.metadata;
    WorkloadSummary {
        id: meta.uid.clone().unwrap_or_default(),
        name: meta.name.clone().unwrap_or_default(),
        namespace: "-".to_string(),
        age: calculate_age(meta.creation_timestamp.as_ref()),
        created_at: get_created_at(meta.creation_timestamp.as_ref()),
        labels: meta.labels.unwrap_or_default(),
        status: "Active".to_string(),
        images: vec![],
    }
}

impl_workload_commands!(
    ConfigMap,
    cluster_list_config_maps,
    cluster_delete_config_map,
    map_configmap_to_summary
);
impl_workload_commands!(
    Secret,
    cluster_list_secrets,
    cluster_delete_secret,
    map_secret_to_summary
);
impl_workload_commands!(
    ResourceQuota,
    cluster_list_resource_quotas,
    cluster_delete_resource_quota,
    map_resource_quota_to_summary
);
impl_workload_commands!(
    LimitRange,
    cluster_list_limit_ranges,
    cluster_delete_limit_range,
    map_limit_range_to_summary
);
impl_workload_commands!(
    HorizontalPodAutoscaler,
    cluster_list_hpa,
    cluster_delete_hpa,
    map_hpa_to_summary
);
impl_workload_commands!(
    PodDisruptionBudget,
    cluster_list_pdb,
    cluster_delete_pdb,
    map_pdb_to_summary
);
impl_workload_commands!(
    Service,
    cluster_list_services,
    cluster_delete_service,
    map_service_to_summary
);
impl_workload_commands!(
    Endpoints,
    cluster_list_endpoints,
    cluster_delete_endpoint,
    map_endpoints_to_summary
);
impl_workload_commands!(
    Ingress,
    cluster_list_ingresses,
    cluster_delete_ingress,
    map_ingress_to_summary
);
impl_workload_commands!(
    NetworkPolicy,
    cluster_list_network_policies,
    cluster_delete_network_policy,
    map_network_policy_to_summary
);
impl_workload_commands!(
    PersistentVolumeClaim,
    cluster_list_pvc,
    cluster_delete_pvc,
    map_pvc_to_summary
);
impl_workload_commands!(
    ServiceAccount,
    cluster_list_service_accounts,
    cluster_delete_service_account,
    map_service_account_to_summary
);
impl_workload_commands!(
    Role,
    cluster_list_roles,
    cluster_delete_role,
    map_role_to_summary
);

// Cluster Scoped
impl_cluster_resource_commands!(
    PersistentVolume,
    cluster_list_pv,
    cluster_delete_pv,
    map_pv_to_summary
);
impl_cluster_resource_commands!(
    StorageClass,
    cluster_list_storage_classes,
    cluster_delete_storage_class,
    map_storage_class_to_summary
);
impl_cluster_resource_commands!(
    ClusterRole,
    cluster_list_cluster_roles,
    cluster_delete_cluster_role,
    map_cluster_role_to_summary
);

// --- Deployment Details ---

/// Detailed information about a Kubernetes Deployment
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeploymentDetails {
    pub name: String,
    pub namespace: String,
    pub uid: String,
    pub created_at: String,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub replicas_desired: i32,
    pub replicas_updated: i32,
    pub replicas_total: i32,
    pub replicas_available: i32,
    pub replicas_unavailable: i32,
    pub strategy_type: String,
    pub selector: HashMap<String, String>,
    pub conditions: Vec<DeploymentCondition>,
    pub images: Vec<String>,
}

/// Condition of a Kubernetes Deployment
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeploymentCondition {
    pub condition_type: String,
    pub status: String,
    pub reason: Option<String>,
    pub message: Option<String>,
    pub last_transition_time: Option<String>,
}

/// Get detailed information about a specific deployment
#[tauri::command]
pub async fn cluster_get_deployment_details(
    cluster_id: String,
    namespace: String,
    name: String,
    state: State<'_, ClusterManagerState>,
) -> Result<DeploymentDetails, String> {
    use kube::api::Api;

    let client = create_client_for_cluster(&cluster_id, &state).await?;
    let deployments: Api<Deployment> = Api::namespaced(client, &namespace);

    let deployment = deployments
        .get(&name)
        .await
        .map_err(|e| format!("Failed to get deployment '{}': {}", name, e))?;

    let meta = deployment.metadata;
    let spec = deployment.spec.unwrap_or_default();
    let status = deployment.status.unwrap_or_default();

    // Extract labels and annotations as HashMap
    let labels: HashMap<String, String> = meta
        .labels
        .unwrap_or_default()
        .into_iter()
        .collect();

    let annotations: HashMap<String, String> = meta
        .annotations
        .unwrap_or_default()
        .into_iter()
        .collect();

    // Extract selector
    let selector: HashMap<String, String> = spec
        .selector
        .match_labels
        .unwrap_or_default()
        .into_iter()
        .collect();

    // Extract conditions
    let conditions: Vec<DeploymentCondition> = status
        .conditions
        .unwrap_or_default()
        .into_iter()
        .map(|c| DeploymentCondition {
            condition_type: c.type_,
            status: c.status,
            reason: c.reason,
            message: c.message,
            last_transition_time: c.last_transition_time.map(|t| t.0.to_string()),
        })
        .collect();

    // Extract images from pod template
    let images: Vec<String> = spec
        .template
        .spec
        .map(|pod_spec| {
            pod_spec
                .containers
                .into_iter()
                .filter_map(|c| c.image)
                .collect()
        })
        .unwrap_or_default();

    // Extract strategy type
    let strategy_type = spec
        .strategy
        .and_then(|s| s.type_)
        .unwrap_or_else(|| "RollingUpdate".to_string());

    // Extract created_at timestamp
    let created_at = meta
        .creation_timestamp
        .map(|t| t.0.to_string())
        .unwrap_or_default();

    Ok(DeploymentDetails {
        name: meta.name.unwrap_or_default(),
        namespace: meta.namespace.unwrap_or_default(),
        uid: meta.uid.unwrap_or_default(),
        created_at,
        labels,
        annotations,
        replicas_desired: spec.replicas.unwrap_or(1),
        replicas_updated: status.updated_replicas.unwrap_or(0),
        replicas_total: status.replicas.unwrap_or(0),
        replicas_available: status.available_replicas.unwrap_or(0),
        replicas_unavailable: status.unavailable_replicas.unwrap_or(0),
        strategy_type,
        selector,
        conditions,
        images,
    })
}

// --- Deployment Pods ---

/// Information about a pod belonging to a deployment
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeploymentPodInfo {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub age: String,
    pub ready: String,
    pub restarts: i32,
    pub node: String,
    pub pod_ip: String,
}

/// Helper function to format age from creation timestamp
fn format_age_from_timestamp(creation_timestamp: &Option<k8s_openapi::apimachinery::pkg::apis::meta::v1::Time>) -> String {
    creation_timestamp
        .as_ref()
        .map(|t| {
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
        .unwrap_or_else(|| "-".to_string())
}

/// Helper function to map a Pod to DeploymentPodInfo
pub fn map_pod_to_deployment_pod_info(pod: &Pod) -> DeploymentPodInfo {
    let meta = &pod.metadata;
    let spec = pod.spec.as_ref();
    let status = pod.status.as_ref();

    let name = meta.name.clone().unwrap_or_default();
    let namespace = meta.namespace.clone().unwrap_or_default();

    // Get pod phase/status
    let pod_status = status
        .and_then(|s| s.phase.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    // Calculate age
    let age = format_age_from_timestamp(&meta.creation_timestamp);

    // Calculate ready containers (e.g., "1/2")
    let container_statuses = status.and_then(|s| s.container_statuses.as_ref());
    let total_containers = container_statuses.map(|cs| cs.len()).unwrap_or(0);
    let ready_containers = container_statuses
        .map(|cs| cs.iter().filter(|c| c.ready).count())
        .unwrap_or(0);
    let ready = format!("{}/{}", ready_containers, total_containers);

    // Sum restarts from all containers
    let restarts: i32 = container_statuses
        .map(|cs| cs.iter().map(|c| c.restart_count).sum())
        .unwrap_or(0);

    // Get node name
    let node = spec
        .and_then(|s| s.node_name.clone())
        .unwrap_or_else(|| "-".to_string());

    // Get pod IP
    let pod_ip = status
        .and_then(|s| s.pod_ip.clone())
        .unwrap_or_else(|| "-".to_string());

    DeploymentPodInfo {
        name,
        namespace,
        status: pod_status,
        age,
        ready,
        restarts,
        node,
        pod_ip,
    }
}

/// Get all pods matching a deployment's selector labels
#[tauri::command]
pub async fn cluster_get_deployment_pods(
    cluster_id: String,
    namespace: String,
    deployment_name: String,
    state: State<'_, ClusterManagerState>,
) -> Result<Vec<DeploymentPodInfo>, String> {
    use kube::api::ListParams;

    let client = create_client_for_cluster(&cluster_id, &state).await?;

    // First, get the deployment to retrieve its selector labels
    let deployments_api: Api<Deployment> = Api::namespaced(client.clone(), &namespace);
    let deployment = deployments_api
        .get(&deployment_name)
        .await
        .map_err(|e| format!("Failed to get deployment '{}': {}", deployment_name, e))?;

    // Extract selector labels from deployment spec
    let selector_labels = deployment
        .spec
        .as_ref()
        .and_then(|s| s.selector.match_labels.clone())
        .unwrap_or_default();

    if selector_labels.is_empty() {
        return Ok(vec![]);
    }

    // Build label selector string (e.g., "app=nginx,env=prod")
    let label_selector: String = selector_labels
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join(",");

    // List pods with the label selector
    let pods_api: Api<Pod> = Api::namespaced(client, &namespace);
    let lp = ListParams::default().labels(&label_selector);

    let pods_list = pods_api
        .list(&lp)
        .await
        .map_err(|e| format!("Failed to list pods: {}", e))?;

    // Map pods to DeploymentPodInfo
    let pod_infos: Vec<DeploymentPodInfo> = pods_list
        .items
        .iter()
        .map(map_pod_to_deployment_pod_info)
        .collect();

    Ok(pod_infos)
}

// --- Deployment ReplicaSets ---

/// Information about a ReplicaSet owned by a Deployment
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ReplicaSetInfo {
    pub name: String,
    pub namespace: String,
    pub revision: String,
    pub desired: i32,
    pub current: i32,
    pub ready: i32,
    pub age: String,
    pub images: Vec<String>,
    pub created_at: String,
}

/// Extract revision number from ReplicaSet annotations
fn extract_revision(rs: &ReplicaSet) -> String {
    rs.metadata
        .annotations
        .as_ref()
        .and_then(|annotations| annotations.get("deployment.kubernetes.io/revision"))
        .cloned()
        .unwrap_or_else(|| "0".to_string())
}

/// Map a ReplicaSet to ReplicaSetInfo
fn map_replicaset_to_info(rs: &ReplicaSet) -> ReplicaSetInfo {
    let meta = &rs.metadata;
    let spec = rs.spec.as_ref();
    let status = rs.status.as_ref();

    let name = meta.name.clone().unwrap_or_default();
    let namespace = meta.namespace.clone().unwrap_or_default();
    let revision = extract_revision(rs);

    let desired = spec.and_then(|s| s.replicas).unwrap_or(0);
    let current = status.map(|s| s.replicas).unwrap_or(0);
    let ready = status.and_then(|s| s.ready_replicas).unwrap_or(0);

    let age = calculate_age(meta.creation_timestamp.as_ref());
    let created_at = meta
        .creation_timestamp
        .as_ref()
        .map(|t| t.0.to_string())
        .unwrap_or_default();

    let images = spec
        .and_then(|s| s.template.as_ref())
        .and_then(|t| t.spec.as_ref())
        .map(|pod_spec| {
            pod_spec
                .containers
                .iter()
                .filter_map(|c| c.image.clone())
                .collect()
        })
        .unwrap_or_default();

    ReplicaSetInfo {
        name,
        namespace,
        revision,
        desired,
        current,
        ready,
        age,
        images,
        created_at,
    }
}

/// Fetches ReplicaSets (revision history) for a specific deployment
#[tauri::command]
pub async fn cluster_get_deployment_replicasets(
    cluster_id: String,
    namespace: String,
    deployment_name: String,
    state: State<'_, ClusterManagerState>,
) -> Result<Vec<ReplicaSetInfo>, String> {
    use kube::api::ListParams;

    let client = create_client_for_cluster(&cluster_id, &state).await?;

    // 1. Get the deployment to find its UID
    let deployments_api: Api<Deployment> = Api::namespaced(client.clone(), &namespace);
    let deployment = deployments_api
        .get(&deployment_name)
        .await
        .map_err(|e| format!("Failed to get deployment '{}': {}", deployment_name, e))?;

    let deployment_uid = deployment
        .metadata
        .uid
        .ok_or_else(|| "Deployment has no UID".to_string())?;

    // 2. List all ReplicaSets in the namespace
    let replicasets_api: Api<ReplicaSet> = Api::namespaced(client, &namespace);
    let rs_list = replicasets_api
        .list(&ListParams::default())
        .await
        .map_err(|e| format!("Failed to list replicasets: {}", e))?;

    // 3. Filter by owner reference matching deployment and map to info
    let mut rs_infos: Vec<ReplicaSetInfo> = rs_list
        .items
        .iter()
        .filter(|rs| {
            rs.metadata
                .owner_references
                .as_ref()
                .map(|refs| {
                    refs.iter().any(|owner| {
                        owner.kind == "Deployment" && owner.uid == deployment_uid
                    })
                })
                .unwrap_or(false)
        })
        .map(map_replicaset_to_info)
        .collect();

    // 4. Sort by revision (newest first)
    rs_infos.sort_by(|a, b| {
        let rev_a: i64 = a.revision.parse().unwrap_or(0);
        let rev_b: i64 = b.revision.parse().unwrap_or(0);
        rev_b.cmp(&rev_a)
    });

    Ok(rs_infos)
}

// --- Deployment Events ---

/// Event information for Kubernetes resources (deployments, etc.)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct K8sEventInfo {
    pub event_type: String,
    pub reason: String,
    pub message: String,
    pub count: i32,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    pub source: String,
}

/// Helper function to filter and map events for a specific deployment
pub fn filter_deployment_events(
    events: Vec<Event>,
    deployment_name: &str,
    deployment_uid: Option<&str>,
) -> Vec<K8sEventInfo> {
    let mut event_infos: Vec<K8sEventInfo> = events
        .into_iter()
        .filter(|event| {
            let involved = &event.involved_object;
            let name_matches = involved.name.as_deref() == Some(deployment_name);
            let kind_matches = involved.kind.as_deref() == Some("Deployment");
            let uid_matches = deployment_uid
                .map(|uid| involved.uid.as_deref() == Some(uid))
                .unwrap_or(true);

            name_matches && kind_matches && uid_matches
        })
        .map(|event| {
            let source = event
                .source
                .as_ref()
                .and_then(|s| s.component.clone())
                .unwrap_or_else(|| "unknown".to_string());

            K8sEventInfo {
                event_type: event.type_.unwrap_or_else(|| "Normal".to_string()),
                reason: event.reason.unwrap_or_default(),
                message: event.message.unwrap_or_default(),
                count: event.count.unwrap_or(1),
                first_timestamp: event.first_timestamp.as_ref().map(|t| t.0.to_string()),
                last_timestamp: event.last_timestamp.as_ref().map(|t| t.0.to_string()),
                source,
            }
        })
        .collect();

    // Sort by last_timestamp descending (most recent first)
    event_infos.sort_by(|a, b| b.last_timestamp.cmp(&a.last_timestamp));

    event_infos
}

/// Fetches events related to a specific deployment
#[tauri::command]
pub async fn cluster_get_deployment_events(
    cluster_id: String,
    namespace: String,
    deployment_name: String,
    state: State<'_, ClusterManagerState>,
) -> Result<Vec<K8sEventInfo>, String> {
    use kube::api::ListParams;

    let client = create_client_for_cluster(&cluster_id, &state).await?;

    // First, get the deployment to retrieve its UID
    let deployments_api: Api<Deployment> = Api::namespaced(client.clone(), &namespace);
    let deployment = deployments_api
        .get(&deployment_name)
        .await
        .map_err(|e| format!("Failed to get deployment '{}': {}", deployment_name, e))?;

    let deployment_uid = deployment.metadata.uid.as_deref();

    // List all events in the namespace
    let events_api: Api<Event> = Api::namespaced(client, &namespace);
    let lp = ListParams::default();

    let events_list = events_api
        .list(&lp)
        .await
        .map_err(|e| format!("Failed to list events: {}", e))?;

    // Filter events for this deployment
    let event_infos = filter_deployment_events(events_list.items, &deployment_name, deployment_uid);

    Ok(event_infos)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- K8sEventInfo and filter_deployment_events tests ---

    // Helper to create a mock Event for testing
    fn create_mock_event(
        name: &str,
        kind: &str,
        uid: Option<&str>,
        event_type: &str,
        reason: &str,
        message: &str,
        count: i32,
        last_timestamp: Option<&str>,
    ) -> Event {
        use k8s_openapi::api::core::v1::{EventSource, ObjectReference};
        use k8s_openapi::apimachinery::pkg::apis::meta::v1::Time;

        Event {
            metadata: Default::default(),
            involved_object: ObjectReference {
                name: Some(name.to_string()),
                kind: Some(kind.to_string()),
                uid: uid.map(|s| s.to_string()),
                ..Default::default()
            },
            type_: Some(event_type.to_string()),
            reason: Some(reason.to_string()),
            message: Some(message.to_string()),
            count: Some(count),
            first_timestamp: last_timestamp.map(|ts| {
                Time(
                    ts.parse::<k8s_openapi::jiff::Timestamp>()
                        .unwrap(),
                )
            }),
            last_timestamp: last_timestamp.map(|ts| {
                Time(
                    ts.parse::<k8s_openapi::jiff::Timestamp>()
                        .unwrap(),
                )
            }),
            source: Some(EventSource {
                component: Some("deployment-controller".to_string()),
                host: None,
            }),
            ..Default::default()
        }
    }

    #[test]
    fn test_k8s_event_info_struct_fields() {
        let event_info = K8sEventInfo {
            event_type: "Warning".to_string(),
            reason: "FailedCreate".to_string(),
            message: "Error creating pods".to_string(),
            count: 3,
            first_timestamp: Some("2024-01-01T00:00:00Z".to_string()),
            last_timestamp: Some("2024-01-01T01:00:00Z".to_string()),
            source: "deployment-controller".to_string(),
        };

        assert_eq!(event_info.event_type, "Warning");
        assert_eq!(event_info.reason, "FailedCreate");
        assert_eq!(event_info.message, "Error creating pods");
        assert_eq!(event_info.count, 3);
        assert_eq!(
            event_info.first_timestamp,
            Some("2024-01-01T00:00:00Z".to_string())
        );
        assert_eq!(
            event_info.last_timestamp,
            Some("2024-01-01T01:00:00Z".to_string())
        );
        assert_eq!(event_info.source, "deployment-controller");
    }

    #[test]
    fn test_filter_deployment_events_with_multiple_events() {
        let events = vec![
            create_mock_event(
                "my-deployment",
                "Deployment",
                Some("uid-123"),
                "Normal",
                "ScalingReplicaSet",
                "Scaled up replica set my-deployment-abc to 3",
                1,
                Some("2024-01-01T02:00:00Z"),
            ),
            create_mock_event(
                "my-deployment",
                "Deployment",
                Some("uid-123"),
                "Warning",
                "FailedCreate",
                "Error creating pods",
                2,
                Some("2024-01-01T01:00:00Z"),
            ),
            create_mock_event(
                "my-deployment",
                "Deployment",
                Some("uid-123"),
                "Normal",
                "ScalingReplicaSet",
                "Scaled down replica set my-deployment-xyz to 0",
                1,
                Some("2024-01-01T03:00:00Z"),
            ),
        ];

        let result = filter_deployment_events(events, "my-deployment", Some("uid-123"));

        assert_eq!(result.len(), 3);
        // Should be sorted by last_timestamp descending (newest first)
        assert_eq!(result[0].reason, "ScalingReplicaSet");
        assert!(result[0].message.contains("Scaled down"));
        assert_eq!(result[1].reason, "ScalingReplicaSet");
        assert!(result[1].message.contains("Scaled up"));
        assert_eq!(result[2].reason, "FailedCreate");
    }

    #[test]
    fn test_filter_deployment_events_with_no_events() {
        let events: Vec<Event> = vec![];

        let result = filter_deployment_events(events, "my-deployment", Some("uid-123"));

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_filter_deployment_events_filters_by_involved_object() {
        let events = vec![
            // Event for our deployment
            create_mock_event(
                "my-deployment",
                "Deployment",
                Some("uid-123"),
                "Normal",
                "ScalingReplicaSet",
                "Scaled up",
                1,
                Some("2024-01-01T01:00:00Z"),
            ),
            // Event for a different deployment (should be filtered out)
            create_mock_event(
                "other-deployment",
                "Deployment",
                Some("uid-456"),
                "Normal",
                "ScalingReplicaSet",
                "Other scaled up",
                1,
                Some("2024-01-01T02:00:00Z"),
            ),
            // Event for a Pod (should be filtered out)
            create_mock_event(
                "my-deployment-pod-abc",
                "Pod",
                Some("uid-789"),
                "Normal",
                "Scheduled",
                "Successfully assigned",
                1,
                Some("2024-01-01T03:00:00Z"),
            ),
            // Event for a ReplicaSet (should be filtered out)
            create_mock_event(
                "my-deployment-rs-abc",
                "ReplicaSet",
                Some("uid-101"),
                "Normal",
                "SuccessfulCreate",
                "Created pod",
                1,
                Some("2024-01-01T04:00:00Z"),
            ),
        ];

        let result = filter_deployment_events(events, "my-deployment", Some("uid-123"));

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].message, "Scaled up");
    }

    #[test]
    fn test_filter_deployment_events_handles_event_types() {
        let events = vec![
            create_mock_event(
                "my-deployment",
                "Deployment",
                Some("uid-123"),
                "Normal",
                "ScalingReplicaSet",
                "Scaled up",
                1,
                Some("2024-01-01T01:00:00Z"),
            ),
            create_mock_event(
                "my-deployment",
                "Deployment",
                Some("uid-123"),
                "Warning",
                "FailedCreate",
                "Error creating",
                5,
                Some("2024-01-01T02:00:00Z"),
            ),
        ];

        let result = filter_deployment_events(events, "my-deployment", Some("uid-123"));

        assert_eq!(result.len(), 2);

        // Find the Warning event
        let warning_event = result.iter().find(|e| e.event_type == "Warning").unwrap();
        assert_eq!(warning_event.reason, "FailedCreate");
        assert_eq!(warning_event.count, 5);

        // Find the Normal event
        let normal_event = result.iter().find(|e| e.event_type == "Normal").unwrap();
        assert_eq!(normal_event.reason, "ScalingReplicaSet");
        assert_eq!(normal_event.count, 1);
    }

    #[test]
    fn test_filter_deployment_events_timestamp_sorting() {
        let events = vec![
            create_mock_event(
                "my-deployment",
                "Deployment",
                Some("uid-123"),
                "Normal",
                "Event1",
                "First event",
                1,
                Some("2024-01-01T01:00:00Z"),
            ),
            create_mock_event(
                "my-deployment",
                "Deployment",
                Some("uid-123"),
                "Normal",
                "Event3",
                "Third event (newest)",
                1,
                Some("2024-01-01T03:00:00Z"),
            ),
            create_mock_event(
                "my-deployment",
                "Deployment",
                Some("uid-123"),
                "Normal",
                "Event2",
                "Second event",
                1,
                Some("2024-01-01T02:00:00Z"),
            ),
        ];

        let result = filter_deployment_events(events, "my-deployment", Some("uid-123"));

        assert_eq!(result.len(), 3);
        // Verify descending order (newest first)
        assert_eq!(result[0].reason, "Event3");
        assert_eq!(result[1].reason, "Event2");
        assert_eq!(result[2].reason, "Event1");
    }

    #[test]
    fn test_filter_deployment_events_without_uid_filter() {
        // When UID is not provided, should still filter by name and kind
        let events = vec![
            create_mock_event(
                "my-deployment",
                "Deployment",
                Some("uid-123"),
                "Normal",
                "ScalingReplicaSet",
                "Event 1",
                1,
                Some("2024-01-01T01:00:00Z"),
            ),
            create_mock_event(
                "my-deployment",
                "Deployment",
                Some("uid-456"), // Different UID
                "Normal",
                "ScalingReplicaSet",
                "Event 2",
                1,
                Some("2024-01-01T02:00:00Z"),
            ),
        ];

        // When no UID filter is provided, both events should match
        let result = filter_deployment_events(events, "my-deployment", None);

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_k8s_event_info_serialization() {
        let event_info = K8sEventInfo {
            event_type: "Warning".to_string(),
            reason: "FailedCreate".to_string(),
            message: "Error creating pods".to_string(),
            count: 3,
            first_timestamp: Some("2024-01-01T00:00:00Z".to_string()),
            last_timestamp: Some("2024-01-01T01:00:00Z".to_string()),
            source: "deployment-controller".to_string(),
        };

        let json = serde_json::to_string(&event_info).expect("Serialization should work");
        assert!(json.contains("\"event_type\":\"Warning\""));
        assert!(json.contains("\"reason\":\"FailedCreate\""));
        assert!(json.contains("\"count\":3"));
    }

    #[test]
    fn test_k8s_event_info_deserialization() {
        let json = r#"{
            "event_type": "Normal",
            "reason": "ScalingReplicaSet",
            "message": "Scaled up",
            "count": 1,
            "first_timestamp": "2024-01-01T00:00:00Z",
            "last_timestamp": "2024-01-01T01:00:00Z",
            "source": "deployment-controller"
        }"#;

        let event_info: K8sEventInfo =
            serde_json::from_str(json).expect("Deserialization should work");
        assert_eq!(event_info.event_type, "Normal");
        assert_eq!(event_info.reason, "ScalingReplicaSet");
        assert_eq!(event_info.count, 1);
    }

    #[test]
    fn test_filter_deployment_events_handles_missing_fields() {
        // Test with an event that has minimal fields set
        let event = Event {
            metadata: Default::default(),
            involved_object: k8s_openapi::api::core::v1::ObjectReference {
                name: Some("my-deployment".to_string()),
                kind: Some("Deployment".to_string()),
                uid: Some("uid-123".to_string()),
                ..Default::default()
            },
            type_: None,          // Missing type
            reason: None,         // Missing reason
            message: None,        // Missing message
            count: None,          // Missing count
            first_timestamp: None,
            last_timestamp: None,
            source: None, // Missing source
            ..Default::default()
        };

        let result = filter_deployment_events(vec![event], "my-deployment", Some("uid-123"));

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].event_type, "Normal"); // Default value
        assert_eq!(result[0].reason, "");           // Default empty
        assert_eq!(result[0].message, "");          // Default empty
        assert_eq!(result[0].count, 1);             // Default 1
        assert_eq!(result[0].source, "unknown");    // Default unknown
        assert!(result[0].first_timestamp.is_none());
        assert!(result[0].last_timestamp.is_none());
    }

    // --- DeploymentDetails struct tests ---

    #[test]
    fn test_deployment_details_serialization() {
        let details = DeploymentDetails {
            name: "nginx-deployment".to_string(),
            namespace: "default".to_string(),
            uid: "abc-123-def-456".to_string(),
            created_at: "2024-01-15T10:30:00Z".to_string(),
            labels: HashMap::from([
                ("app".to_string(), "nginx".to_string()),
                ("env".to_string(), "production".to_string()),
            ]),
            annotations: HashMap::from([
                ("kubectl.kubernetes.io/last-applied-configuration".to_string(), "{}".to_string()),
            ]),
            replicas_desired: 3,
            replicas_updated: 3,
            replicas_total: 3,
            replicas_available: 3,
            replicas_unavailable: 0,
            strategy_type: "RollingUpdate".to_string(),
            selector: HashMap::from([("app".to_string(), "nginx".to_string())]),
            conditions: vec![
                DeploymentCondition {
                    condition_type: "Available".to_string(),
                    status: "True".to_string(),
                    reason: Some("MinimumReplicasAvailable".to_string()),
                    message: Some("Deployment has minimum availability.".to_string()),
                    last_transition_time: Some("2024-01-15T10:35:00Z".to_string()),
                },
            ],
            images: vec!["nginx:1.19".to_string()],
        };

        // Test JSON serialization
        let json = serde_json::to_string(&details).expect("Should serialize to JSON");
        assert!(json.contains("nginx-deployment"));
        assert!(json.contains("default"));
        assert!(json.contains("abc-123-def-456"));

        // Test deserialization
        let deserialized: DeploymentDetails = serde_json::from_str(&json).expect("Should deserialize from JSON");
        assert_eq!(deserialized.name, "nginx-deployment");
        assert_eq!(deserialized.namespace, "default");
        assert_eq!(deserialized.replicas_desired, 3);
        assert_eq!(deserialized.replicas_available, 3);
    }

    #[test]
    fn test_deployment_details_with_empty_fields() {
        let details = DeploymentDetails {
            name: "minimal".to_string(),
            namespace: "default".to_string(),
            uid: "".to_string(),
            created_at: "".to_string(),
            labels: HashMap::new(),
            annotations: HashMap::new(),
            replicas_desired: 1,
            replicas_updated: 0,
            replicas_total: 0,
            replicas_available: 0,
            replicas_unavailable: 1,
            strategy_type: "".to_string(),
            selector: HashMap::new(),
            conditions: vec![],
            images: vec![],
        };

        let json = serde_json::to_string(&details).expect("Should serialize empty fields");
        let deserialized: DeploymentDetails = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized.labels.len(), 0);
        assert_eq!(deserialized.conditions.len(), 0);
        assert_eq!(deserialized.images.len(), 0);
    }

    #[test]
    fn test_deployment_condition_serialization() {
        let condition = DeploymentCondition {
            condition_type: "Progressing".to_string(),
            status: "True".to_string(),
            reason: Some("NewReplicaSetAvailable".to_string()),
            message: Some("ReplicaSet has successfully progressed.".to_string()),
            last_transition_time: Some("2024-01-15T10:35:00Z".to_string()),
        };

        let json = serde_json::to_string(&condition).expect("Should serialize condition");
        assert!(json.contains("Progressing"));
        assert!(json.contains("NewReplicaSetAvailable"));

        let deserialized: DeploymentCondition = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.condition_type, "Progressing");
        assert_eq!(deserialized.status, "True");
        assert!(deserialized.reason.is_some());
    }

    #[test]
    fn test_deployment_condition_with_none_fields() {
        let condition = DeploymentCondition {
            condition_type: "Available".to_string(),
            status: "False".to_string(),
            reason: None,
            message: None,
            last_transition_time: None,
        };

        let json = serde_json::to_string(&condition).expect("Should serialize with None fields");
        let deserialized: DeploymentCondition = serde_json::from_str(&json).expect("Should deserialize");

        assert!(deserialized.reason.is_none());
        assert!(deserialized.message.is_none());
        assert!(deserialized.last_transition_time.is_none());
    }

    #[test]
    fn test_deployment_details_multiple_images() {
        let details = DeploymentDetails {
            name: "multi-container".to_string(),
            namespace: "default".to_string(),
            uid: "uid-123".to_string(),
            created_at: "2024-01-15T10:30:00Z".to_string(),
            labels: HashMap::new(),
            annotations: HashMap::new(),
            replicas_desired: 2,
            replicas_updated: 2,
            replicas_total: 2,
            replicas_available: 2,
            replicas_unavailable: 0,
            strategy_type: "RollingUpdate".to_string(),
            selector: HashMap::new(),
            conditions: vec![],
            images: vec![
                "nginx:1.19".to_string(),
                "redis:6.0".to_string(),
                "fluent/fluentd:v1.12".to_string(),
            ],
        };

        assert_eq!(details.images.len(), 3);
        assert!(details.images.contains(&"nginx:1.19".to_string()));
        assert!(details.images.contains(&"redis:6.0".to_string()));
    }

    #[test]
    fn test_deployment_details_multiple_conditions() {
        let details = DeploymentDetails {
            name: "test-deploy".to_string(),
            namespace: "default".to_string(),
            uid: "uid-456".to_string(),
            created_at: "2024-01-15T10:30:00Z".to_string(),
            labels: HashMap::new(),
            annotations: HashMap::new(),
            replicas_desired: 3,
            replicas_updated: 3,
            replicas_total: 3,
            replicas_available: 3,
            replicas_unavailable: 0,
            strategy_type: "RollingUpdate".to_string(),
            selector: HashMap::new(),
            conditions: vec![
                DeploymentCondition {
                    condition_type: "Available".to_string(),
                    status: "True".to_string(),
                    reason: Some("MinimumReplicasAvailable".to_string()),
                    message: None,
                    last_transition_time: None,
                },
                DeploymentCondition {
                    condition_type: "Progressing".to_string(),
                    status: "True".to_string(),
                    reason: Some("NewReplicaSetAvailable".to_string()),
                    message: None,
                    last_transition_time: None,
                },
            ],
            images: vec!["nginx:latest".to_string()],
        };

        assert_eq!(details.conditions.len(), 2);
        assert!(details.conditions.iter().any(|c| c.condition_type == "Available"));
        assert!(details.conditions.iter().any(|c| c.condition_type == "Progressing"));
    }

    #[test]
    fn test_deployment_details_recreate_strategy() {
        let details = DeploymentDetails {
            name: "recreate-deploy".to_string(),
            namespace: "staging".to_string(),
            uid: "uid-789".to_string(),
            created_at: "2024-01-15T10:30:00Z".to_string(),
            labels: HashMap::new(),
            annotations: HashMap::new(),
            replicas_desired: 1,
            replicas_updated: 1,
            replicas_total: 1,
            replicas_available: 1,
            replicas_unavailable: 0,
            strategy_type: "Recreate".to_string(),
            selector: HashMap::new(),
            conditions: vec![],
            images: vec![],
        };

        assert_eq!(details.strategy_type, "Recreate");
    }

    #[test]
    fn test_deployment_details_replica_mismatch() {
        // Test a deployment in the middle of a rollout
        let details = DeploymentDetails {
            name: "rolling-deploy".to_string(),
            namespace: "default".to_string(),
            uid: "uid-abc".to_string(),
            created_at: "2024-01-15T10:30:00Z".to_string(),
            labels: HashMap::new(),
            annotations: HashMap::new(),
            replicas_desired: 5,
            replicas_updated: 3,
            replicas_total: 6, // During rollout, may have more pods
            replicas_available: 4,
            replicas_unavailable: 2,
            strategy_type: "RollingUpdate".to_string(),
            selector: HashMap::new(),
            conditions: vec![],
            images: vec![],
        };

        // Verify the replica counts are preserved correctly
        assert_eq!(details.replicas_desired, 5);
        assert_eq!(details.replicas_updated, 3);
        assert_eq!(details.replicas_total, 6);
        assert_eq!(details.replicas_available, 4);
        assert_eq!(details.replicas_unavailable, 2);
    }

    #[test]
    fn test_deployment_details_labels_and_selector_match() {
        let labels = HashMap::from([
            ("app".to_string(), "myapp".to_string()),
            ("version".to_string(), "v2".to_string()),
            ("team".to_string(), "backend".to_string()),
        ]);

        let selector = HashMap::from([
            ("app".to_string(), "myapp".to_string()),
        ]);

        let details = DeploymentDetails {
            name: "label-test".to_string(),
            namespace: "default".to_string(),
            uid: "uid-def".to_string(),
            created_at: "2024-01-15T10:30:00Z".to_string(),
            labels: labels.clone(),
            annotations: HashMap::new(),
            replicas_desired: 1,
            replicas_updated: 1,
            replicas_total: 1,
            replicas_available: 1,
            replicas_unavailable: 0,
            strategy_type: "RollingUpdate".to_string(),
            selector: selector.clone(),
            conditions: vec![],
            images: vec![],
        };

        // Verify labels contain all entries
        assert_eq!(details.labels.len(), 3);
        assert_eq!(details.labels.get("app"), Some(&"myapp".to_string()));

        // Verify selector is a subset of labels
        assert_eq!(details.selector.len(), 1);
        assert!(details.labels.contains_key("app"));
    }

    // --- Helper function tests ---

    #[test]
    fn test_parse_cpu_millicores() {
        assert_eq!(parse_cpu("100m"), 0.1);
        assert_eq!(parse_cpu("500m"), 0.5);
        assert_eq!(parse_cpu("1000m"), 1.0);
    }

    #[test]
    fn test_parse_cpu_cores() {
        assert_eq!(parse_cpu("1"), 1.0);
        assert_eq!(parse_cpu("2"), 2.0);
        assert_eq!(parse_cpu("0.5"), 0.5);
    }

    #[test]
    fn test_parse_cpu_invalid() {
        assert_eq!(parse_cpu("invalid"), 0.0);
        assert_eq!(parse_cpu(""), 0.0);
    }

    #[test]
    fn test_parse_memory_ki() {
        let result = parse_memory("1024Ki");
        assert_eq!(result, 1024.0 * 1024.0);
    }

    #[test]
    fn test_parse_memory_mi() {
        let result = parse_memory("256Mi");
        assert_eq!(result, 256.0 * 1024.0 * 1024.0);
    }

    #[test]
    fn test_parse_memory_gi() {
        let result = parse_memory("2Gi");
        assert_eq!(result, 2.0 * 1024.0 * 1024.0 * 1024.0);
    }

    #[test]
    fn test_parse_memory_ti() {
        let result = parse_memory("1Ti");
        assert_eq!(result, 1024.0_f64.powi(4));
    }

    #[test]
    fn test_parse_memory_bytes() {
        let result = parse_memory("1000000");
        assert_eq!(result, 1000000.0);
    }

    #[test]
    fn test_parse_memory_invalid() {
        assert_eq!(parse_memory("invalid"), 0.0);
        assert_eq!(parse_memory(""), 0.0);
    }

    #[test]
    fn test_parse_memory_millibytes() {
        // Edge case: memory in millibytes (rare but valid)
        let result = parse_memory("1000m");
        assert_eq!(result, 1.0);
    }

    // --- DeploymentPodInfo struct tests ---

    #[test]
    fn test_deployment_pod_info_serialization() {
        let pod_info = DeploymentPodInfo {
            name: "nginx-deployment-abc123".to_string(),
            namespace: "default".to_string(),
            status: "Running".to_string(),
            age: "5d".to_string(),
            ready: "1/1".to_string(),
            restarts: 0,
            node: "worker-node-1".to_string(),
            pod_ip: "10.244.0.5".to_string(),
        };

        // Test JSON serialization
        let json = serde_json::to_string(&pod_info).expect("Should serialize to JSON");
        assert!(json.contains("nginx-deployment-abc123"));
        assert!(json.contains("default"));
        assert!(json.contains("Running"));
        assert!(json.contains("10.244.0.5"));

        // Test deserialization
        let deserialized: DeploymentPodInfo =
            serde_json::from_str(&json).expect("Should deserialize from JSON");
        assert_eq!(deserialized.name, "nginx-deployment-abc123");
        assert_eq!(deserialized.namespace, "default");
        assert_eq!(deserialized.status, "Running");
        assert_eq!(deserialized.restarts, 0);
    }

    #[test]
    fn test_deployment_pod_info_with_restarts() {
        let pod_info = DeploymentPodInfo {
            name: "crashloop-pod-xyz789".to_string(),
            namespace: "production".to_string(),
            status: "CrashLoopBackOff".to_string(),
            age: "2h".to_string(),
            ready: "0/1".to_string(),
            restarts: 15,
            node: "worker-node-2".to_string(),
            pod_ip: "10.244.1.10".to_string(),
        };

        assert_eq!(pod_info.restarts, 15);
        assert_eq!(pod_info.ready, "0/1");
        assert_eq!(pod_info.status, "CrashLoopBackOff");
    }

    #[test]
    fn test_deployment_pod_info_pending_status() {
        let pod_info = DeploymentPodInfo {
            name: "pending-pod-def456".to_string(),
            namespace: "staging".to_string(),
            status: "Pending".to_string(),
            age: "30s".to_string(),
            ready: "0/2".to_string(),
            restarts: 0,
            node: "-".to_string(),
            pod_ip: "-".to_string(),
        };

        assert_eq!(pod_info.status, "Pending");
        assert_eq!(pod_info.node, "-");
        assert_eq!(pod_info.pod_ip, "-");
    }

    #[test]
    fn test_deployment_pod_info_multi_container() {
        let pod_info = DeploymentPodInfo {
            name: "multi-container-pod".to_string(),
            namespace: "default".to_string(),
            status: "Running".to_string(),
            age: "1d".to_string(),
            ready: "3/3".to_string(),
            restarts: 2,
            node: "worker-node-3".to_string(),
            pod_ip: "10.244.2.15".to_string(),
        };

        assert_eq!(pod_info.ready, "3/3");
        assert_eq!(pod_info.restarts, 2);
    }

    #[test]
    fn test_deployment_pod_info_empty_fields() {
        let pod_info = DeploymentPodInfo {
            name: "".to_string(),
            namespace: "".to_string(),
            status: "Unknown".to_string(),
            age: "-".to_string(),
            ready: "0/0".to_string(),
            restarts: 0,
            node: "-".to_string(),
            pod_ip: "-".to_string(),
        };

        let json = serde_json::to_string(&pod_info).expect("Should serialize empty fields");
        let deserialized: DeploymentPodInfo =
            serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized.name, "");
        assert_eq!(deserialized.status, "Unknown");
        assert_eq!(deserialized.ready, "0/0");
    }

    // --- map_pod_to_deployment_pod_info tests ---

    // Helper function to create a mock Pod for testing
    fn create_mock_pod(
        name: &str,
        namespace: &str,
        phase: &str,
        ready_containers: usize,
        total_containers: usize,
        restarts: i32,
        node: Option<&str>,
        pod_ip: Option<&str>,
    ) -> Pod {
        use k8s_openapi::api::core::v1::{ContainerStatus, PodSpec, PodStatus};
        use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

        let container_statuses: Vec<ContainerStatus> = (0..total_containers)
            .map(|i| ContainerStatus {
                name: format!("container-{}", i),
                ready: i < ready_containers,
                restart_count: if i == 0 { restarts } else { 0 },
                image: "nginx:latest".to_string(),
                image_id: "sha256:abc123".to_string(),
                state: None,
                last_state: None,
                container_id: Some(format!("docker://container-{}", i)),
                started: Some(true),
                allocated_resources: None,
                resources: None,
                volume_mounts: None,
                user: None,
                allocated_resources_status: None,
            })
            .collect();

        Pod {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                namespace: Some(namespace.to_string()),
                creation_timestamp: Some(k8s_openapi::apimachinery::pkg::apis::meta::v1::Time(
                    k8s_openapi::jiff::Timestamp::now(),
                )),
                ..Default::default()
            },
            spec: Some(PodSpec {
                node_name: node.map(|n| n.to_string()),
                containers: vec![],
                ..Default::default()
            }),
            status: Some(PodStatus {
                phase: Some(phase.to_string()),
                pod_ip: pod_ip.map(|ip| ip.to_string()),
                container_statuses: if total_containers > 0 {
                    Some(container_statuses)
                } else {
                    None
                },
                ..Default::default()
            }),
        }
    }

    #[test]
    fn test_map_pod_to_deployment_pod_info_running() {
        let pod = create_mock_pod(
            "nginx-abc123",
            "default",
            "Running",
            1,
            1,
            0,
            Some("worker-1"),
            Some("10.244.0.5"),
        );

        let info = map_pod_to_deployment_pod_info(&pod);

        assert_eq!(info.name, "nginx-abc123");
        assert_eq!(info.namespace, "default");
        assert_eq!(info.status, "Running");
        assert_eq!(info.ready, "1/1");
        assert_eq!(info.restarts, 0);
        assert_eq!(info.node, "worker-1");
        assert_eq!(info.pod_ip, "10.244.0.5");
    }

    #[test]
    fn test_map_pod_to_deployment_pod_info_pending() {
        let pod = create_mock_pod("pending-pod", "staging", "Pending", 0, 2, 0, None, None);

        let info = map_pod_to_deployment_pod_info(&pod);

        assert_eq!(info.name, "pending-pod");
        assert_eq!(info.status, "Pending");
        assert_eq!(info.ready, "0/2");
        assert_eq!(info.node, "-");
        assert_eq!(info.pod_ip, "-");
    }

    #[test]
    fn test_map_pod_to_deployment_pod_info_with_restarts() {
        let pod = create_mock_pod(
            "crash-pod",
            "production",
            "Running",
            1,
            1,
            5,
            Some("worker-2"),
            Some("10.244.1.10"),
        );

        let info = map_pod_to_deployment_pod_info(&pod);

        assert_eq!(info.restarts, 5);
        assert_eq!(info.ready, "1/1");
    }

    #[test]
    fn test_map_pod_to_deployment_pod_info_multi_container() {
        let pod = create_mock_pod(
            "multi-container",
            "default",
            "Running",
            2,
            3,
            3,
            Some("worker-3"),
            Some("10.244.2.20"),
        );

        let info = map_pod_to_deployment_pod_info(&pod);

        assert_eq!(info.ready, "2/3");
        assert_eq!(info.restarts, 3);
    }

    #[test]
    fn test_map_pod_to_deployment_pod_info_no_containers() {
        let pod = create_mock_pod("empty-pod", "default", "Pending", 0, 0, 0, None, None);

        let info = map_pod_to_deployment_pod_info(&pod);

        assert_eq!(info.ready, "0/0");
        assert_eq!(info.restarts, 0);
    }

    #[test]
    fn test_map_pod_to_deployment_pod_info_age_format() {
        // Test that age is formatted (exact value depends on current time, but format should be valid)
        let pod = create_mock_pod(
            "test-pod",
            "default",
            "Running",
            1,
            1,
            0,
            Some("worker-1"),
            Some("10.244.0.1"),
        );

        let info = map_pod_to_deployment_pod_info(&pod);

        // Age should be in format like "1d", "2h", "30m", "45s", or "-"
        assert!(
            info.age.ends_with('d')
                || info.age.ends_with('h')
                || info.age.ends_with('m')
                || info.age.ends_with('s')
                || info.age == "-"
        );
    }

    #[test]
    fn test_deployment_pod_info_vec_serialization() {
        // Test that a vector of DeploymentPodInfo serializes correctly
        let pods = vec![
            DeploymentPodInfo {
                name: "pod-1".to_string(),
                namespace: "default".to_string(),
                status: "Running".to_string(),
                age: "1d".to_string(),
                ready: "1/1".to_string(),
                restarts: 0,
                node: "node-1".to_string(),
                pod_ip: "10.0.0.1".to_string(),
            },
            DeploymentPodInfo {
                name: "pod-2".to_string(),
                namespace: "default".to_string(),
                status: "Running".to_string(),
                age: "2d".to_string(),
                ready: "2/2".to_string(),
                restarts: 1,
                node: "node-2".to_string(),
                pod_ip: "10.0.0.2".to_string(),
            },
        ];

        let json = serde_json::to_string(&pods).expect("Should serialize vec");
        let deserialized: Vec<DeploymentPodInfo> =
            serde_json::from_str(&json).expect("Should deserialize vec");

        assert_eq!(deserialized.len(), 2);
        assert_eq!(deserialized[0].name, "pod-1");
        assert_eq!(deserialized[1].name, "pod-2");
    }

    #[test]
    fn test_deployment_pod_info_all_fields_populated() {
        // Test that all fields are properly captured
        let pod_info = DeploymentPodInfo {
            name: "full-test-pod-abc123xyz".to_string(),
            namespace: "kube-system".to_string(),
            status: "Running".to_string(),
            age: "30d".to_string(),
            ready: "5/5".to_string(),
            restarts: 100,
            node: "master-node-01.cluster.local".to_string(),
            pod_ip: "192.168.1.100".to_string(),
        };

        // Verify all fields
        assert!(!pod_info.name.is_empty());
        assert!(!pod_info.namespace.is_empty());
        assert!(!pod_info.status.is_empty());
        assert!(!pod_info.age.is_empty());
        assert!(!pod_info.ready.is_empty());
        assert!(pod_info.restarts >= 0);
        assert!(!pod_info.node.is_empty());
        assert!(!pod_info.pod_ip.is_empty());
    }
}
