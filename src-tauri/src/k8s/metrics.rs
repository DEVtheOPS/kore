use crate::cluster_manager::ClusterManagerState;
use crate::k8s::client::create_client_for_cluster;
use crate::k8s::common::{calculate_age, get_created_at};
use k8s_openapi::api::core::v1::{Event, Node, Pod};
use kube::api::Api;
use tauri::State;

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

#[derive(serde::Serialize, Debug)]
pub struct ClusterEventSummary {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub age: String,
    pub labels: std::collections::BTreeMap<String, String>,
    pub status: String,
    pub images: Vec<String>,
    pub created_at: i64,
    pub event_type: String,
    pub reason: String,
    pub message: String,
    pub object: String,
    pub count: i32,
}

#[derive(serde::Serialize, Debug)]
pub struct NodeSummary {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub age: String,
    pub labels: std::collections::BTreeMap<String, String>,
    pub status: String,
    pub images: Vec<String>,
    pub created_at: i64,
    pub roles: String,
    pub version: String,
    pub internal_ip: String,
    pub os_image: String,
    pub kernel_version: String,
    pub container_runtime: String,
    pub taints: Vec<String>,
    pub capacity_cpu: String,
    pub capacity_memory: String,
    pub capacity_pods: String,
    pub allocatable_cpu: String,
    pub allocatable_memory: String,
    pub allocatable_pods: String,
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

fn parse_rfc3339_ts(ts: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(ts)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Utc))
}

fn format_event_age(
    last_timestamp: Option<&k8s_openapi::apimachinery::pkg::apis::meta::v1::Time>,
) -> String {
    let Some(last_ts) = last_timestamp else {
        return "-".to_string();
    };

    let Some(last_ts_parsed) = parse_rfc3339_ts(&last_ts.0.to_string()) else {
        return "-".to_string();
    };

    let duration = chrono::Utc::now().signed_duration_since(last_ts_parsed);
    if duration.num_days() > 0 {
        format!("{}d", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{}h", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{}m", duration.num_minutes())
    } else {
        format!("{}s", duration.num_seconds())
    }
}

fn map_node_to_summary(node: Node) -> NodeSummary {
    let meta = node.metadata;
    let status = node.status.unwrap_or_default();
    let spec = node.spec.unwrap_or_default();

    let ready_status = status
        .conditions
        .as_ref()
        .and_then(|conds| conds.iter().find(|c| c.type_ == "Ready"))
        .map(|c| c.status.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let status_label = match ready_status.as_str() {
        "True" => "Ready",
        "False" => "NotReady",
        _ => "Unknown",
    };

    let labels = meta.labels.unwrap_or_default();
    let mut roles: Vec<String> = labels
        .keys()
        .filter_map(|key| {
            key.strip_prefix("node-role.kubernetes.io/")
                .map(|r| r.to_string())
        })
        .collect();
    if roles.is_empty() {
        roles.push("worker".to_string());
    }
    roles.sort();
    roles.dedup();

    let internal_ip = status
        .addresses
        .as_ref()
        .and_then(|addrs| addrs.iter().find(|a| a.type_ == "InternalIP"))
        .map(|a| a.address.clone())
        .unwrap_or_else(|| "-".to_string());

    let taints = spec
        .taints
        .unwrap_or_default()
        .into_iter()
        .map(|t| {
            if let Some(value) = t.value {
                format!("{}={} ({})", t.key, value, t.effect)
            } else {
                format!("{} ({})", t.key, t.effect)
            }
        })
        .collect::<Vec<_>>();

    let capacity = status.capacity.unwrap_or_default();
    let allocatable = status.allocatable.unwrap_or_default();
    let node_info = status.node_info.unwrap_or_default();

    NodeSummary {
        id: meta.uid.clone().unwrap_or_default(),
        name: meta.name.clone().unwrap_or_default(),
        namespace: "-".to_string(),
        age: calculate_age(meta.creation_timestamp.as_ref()),
        labels,
        status: status_label.to_string(),
        images: vec![],
        created_at: get_created_at(meta.creation_timestamp.as_ref()),
        roles: roles.join(","),
        version: node_info.kubelet_version,
        internal_ip,
        os_image: node_info.os_image,
        kernel_version: node_info.kernel_version,
        container_runtime: node_info.container_runtime_version,
        taints,
        capacity_cpu: capacity
            .get("cpu")
            .map(|v| v.0.clone())
            .unwrap_or_else(|| "-".to_string()),
        capacity_memory: capacity
            .get("memory")
            .map(|v| v.0.clone())
            .unwrap_or_else(|| "-".to_string()),
        capacity_pods: capacity
            .get("pods")
            .map(|v| v.0.clone())
            .unwrap_or_else(|| "-".to_string()),
        allocatable_cpu: allocatable
            .get("cpu")
            .map(|v| v.0.clone())
            .unwrap_or_else(|| "-".to_string()),
        allocatable_memory: allocatable
            .get("memory")
            .map(|v| v.0.clone())
            .unwrap_or_else(|| "-".to_string()),
        allocatable_pods: allocatable
            .get("pods")
            .map(|v| v.0.clone())
            .unwrap_or_else(|| "-".to_string()),
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

    for e in event_list.items {
        if e.type_.as_deref() == Some("Warning") {
            let age = format_event_age(e.last_timestamp.as_ref());

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

#[tauri::command]
pub async fn cluster_list_events(
    cluster_id: String,
    namespace: Option<String>,
    include_normal: Option<bool>,
    state: State<'_, ClusterManagerState>,
) -> Result<Vec<ClusterEventSummary>, String> {
    let client = create_client_for_cluster(&cluster_id, &state).await?;
    let events: Api<Event> = if let Some(ns) = namespace.clone() {
        Api::namespaced(client, &ns)
    } else {
        Api::all(client)
    };

    let include_normal = include_normal.unwrap_or(true);
    let event_list = events
        .list(&kube::api::ListParams::default())
        .await
        .map_err(|e| e.to_string())?;

    let mut summaries: Vec<ClusterEventSummary> = event_list
        .items
        .into_iter()
        .filter(|e| include_normal || e.type_.as_deref() == Some("Warning"))
        .map(|e| {
            let meta = e.metadata;
            let name = meta.name.clone().unwrap_or_default();
            let event_type = e.type_.clone().unwrap_or_else(|| "Normal".to_string());
            let reason = e.reason.clone().unwrap_or_default();
            let message = e.message.clone().unwrap_or_default();
            let object = format!(
                "{}/{}",
                e.involved_object.kind.clone().unwrap_or_default(),
                e.involved_object.name.clone().unwrap_or_default()
            );
            let event_namespace = meta
                .namespace
                .clone()
                .or(e.involved_object.namespace.clone())
                .unwrap_or_else(|| "-".to_string());
            let age = format_event_age(e.last_timestamp.as_ref().or(e.first_timestamp.as_ref()));
            let created_at = get_created_at(meta.creation_timestamp.as_ref());

            ClusterEventSummary {
                id: meta.uid.clone().unwrap_or_else(|| name.clone()),
                name,
                namespace: event_namespace,
                age,
                labels: meta.labels.unwrap_or_default(),
                status: format!("{}: {}", event_type, reason),
                images: vec![],
                created_at,
                event_type,
                reason,
                message,
                object,
                count: e.count.unwrap_or(1),
            }
        })
        .collect();

    summaries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(summaries)
}

#[tauri::command]
pub async fn cluster_list_nodes(
    cluster_id: String,
    state: State<'_, ClusterManagerState>,
) -> Result<Vec<NodeSummary>, String> {
    let client = create_client_for_cluster(&cluster_id, &state).await?;
    let nodes: Api<Node> = Api::all(client);
    let mut list = nodes
        .list(&Default::default())
        .await
        .map_err(|e| format!("Failed to list nodes: {}", e))?
        .items
        .into_iter()
        .map(map_node_to_summary)
        .collect::<Vec<_>>();
    list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(list)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
