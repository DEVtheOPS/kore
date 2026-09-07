//! Live resource usage from the Kubernetes Metrics API (`metrics.k8s.io/v1beta1`).
//!
//! These commands require metrics-server (or a compatible provider) to be
//! installed in the cluster. When the API is not available they return an
//! error so the frontend can degrade gracefully instead of showing fake data.

use crate::cluster_manager::ClusterManagerState;
use crate::k8s::client::create_client_for_cluster;
use kube::api::{ApiResource, DynamicObject, ListParams};
use kube::Api;
use tauri::State;

const METRICS_GROUP: &str = "metrics.k8s.io";
const METRICS_VERSION: &str = "v1beta1";

#[derive(Debug, Clone, serde::Serialize)]
pub struct ContainerUsage {
    pub name: String,
    pub cpu_millicores: f64,
    pub memory_bytes: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PodUsage {
    pub name: String,
    pub namespace: String,
    pub cpu_millicores: f64,
    pub memory_bytes: f64,
    pub containers: Vec<ContainerUsage>,
    pub timestamp: Option<String>,
    pub window: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct NodeUsage {
    pub name: String,
    pub cpu_millicores: f64,
    pub memory_bytes: f64,
    pub timestamp: Option<String>,
    pub window: Option<String>,
}

/// Parse a Kubernetes resource quantity into a plain `f64` in base units
/// (cores for CPU, bytes for memory).
///
/// Supports SI suffixes (`n`, `u`, `m`, `k`, `M`, `G`, `T`, `P`, `E`), binary
/// suffixes (`Ki`, `Mi`, `Gi`, `Ti`, `Pi`, `Ei`) and exponent notation
/// (`1e3`, `2.5E6`). Unparseable input yields `None`.
pub fn parse_quantity(q: &str) -> Option<f64> {
    let q = q.trim();
    if q.is_empty() {
        return None;
    }

    const BINARY: [(&str, f64); 6] = [
        ("Ki", 1024.0),
        ("Mi", 1024.0 * 1024.0),
        ("Gi", 1024.0 * 1024.0 * 1024.0),
        ("Ti", 1024.0 * 1024.0 * 1024.0 * 1024.0),
        ("Pi", 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0),
        ("Ei", 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0),
    ];
    for (suffix, mult) in BINARY {
        if let Some(num) = q.strip_suffix(suffix) {
            return num.parse::<f64>().ok().map(|v| v * mult);
        }
    }

    const DECIMAL: [(char, f64); 8] = [
        ('n', 1e-9),
        ('u', 1e-6),
        ('m', 1e-3),
        ('k', 1e3),
        ('M', 1e6),
        ('G', 1e9),
        ('T', 1e12),
        ('P', 1e15),
    ];
    if let Some(last) = q.chars().last() {
        for (suffix, mult) in DECIMAL {
            if last == suffix {
                let num = &q[..q.len() - suffix.len_utf8()];
                return num.parse::<f64>().ok().map(|v| v * mult);
            }
        }
        // 'E' is ambiguous with exponent notation (e.g. "1E3"); only treat it
        // as the exa suffix when the remainder is a plain number.
        if last == 'E' {
            let num = &q[..q.len() - 1];
            if let Ok(v) = num.parse::<f64>() {
                return Some(v * 1e18);
            }
        }
    }

    q.parse::<f64>().ok()
}

/// CPU quantity -> millicores.
pub fn cpu_millicores(q: &str) -> f64 {
    parse_quantity(q).map(|cores| cores * 1000.0).unwrap_or(0.0)
}

/// Memory quantity -> bytes.
pub fn memory_bytes(q: &str) -> f64 {
    parse_quantity(q).unwrap_or(0.0)
}

fn usage_field(usage: Option<&serde_json::Value>, key: &str) -> String {
    usage
        .and_then(|u| u.get(key))
        .and_then(|v| v.as_str())
        .unwrap_or("0")
        .to_string()
}

fn opt_str(v: Option<&serde_json::Value>) -> Option<String> {
    v.and_then(|v| v.as_str()).map(|s| s.to_string())
}

fn metrics_api_resource(kind: &str, plural: &str) -> ApiResource {
    ApiResource {
        group: METRICS_GROUP.to_string(),
        version: METRICS_VERSION.to_string(),
        api_version: format!("{}/{}", METRICS_GROUP, METRICS_VERSION),
        kind: kind.to_string(),
        plural: plural.to_string(),
    }
}

fn metrics_error(what: &str, e: kube::Error) -> String {
    match &e {
        kube::Error::Api(err) if err.code == 404 => format!(
            "Metrics API not available ({} usage requires metrics-server to be installed)",
            what
        ),
        _ => format!("Failed to fetch {} metrics: {}", what, e),
    }
}

pub(crate) fn map_pod_metrics(obj: DynamicObject) -> PodUsage {
    let name = obj.metadata.name.clone().unwrap_or_default();
    let namespace = obj.metadata.namespace.clone().unwrap_or_default();
    let data = &obj.data;

    let containers: Vec<ContainerUsage> = data
        .get("containers")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .map(|c| {
                    let usage = c.get("usage");
                    ContainerUsage {
                        name: c
                            .get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        cpu_millicores: cpu_millicores(&usage_field(usage, "cpu")),
                        memory_bytes: memory_bytes(&usage_field(usage, "memory")),
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    PodUsage {
        name,
        namespace,
        cpu_millicores: containers.iter().map(|c| c.cpu_millicores).sum(),
        memory_bytes: containers.iter().map(|c| c.memory_bytes).sum(),
        containers,
        timestamp: opt_str(data.get("timestamp")),
        window: opt_str(data.get("window")),
    }
}

pub(crate) fn map_node_metrics(obj: DynamicObject) -> NodeUsage {
    let name = obj.metadata.name.clone().unwrap_or_default();
    let data = &obj.data;
    let usage = data.get("usage");
    NodeUsage {
        name,
        cpu_millicores: cpu_millicores(&usage_field(usage, "cpu")),
        memory_bytes: memory_bytes(&usage_field(usage, "memory")),
        timestamp: opt_str(data.get("timestamp")),
        window: opt_str(data.get("window")),
    }
}

#[tauri::command]
pub async fn cluster_get_node_usage(
    cluster_id: String,
    state: State<'_, ClusterManagerState>,
) -> Result<Vec<NodeUsage>, String> {
    let client = create_client_for_cluster(&cluster_id, &state).await?;
    let ar = metrics_api_resource("NodeMetrics", "nodes");
    let api: Api<DynamicObject> = Api::all_with(client, &ar);

    let list = api
        .list(&ListParams::default())
        .await
        .map_err(|e| metrics_error("node", e))?;

    Ok(list.items.into_iter().map(map_node_metrics).collect())
}

#[tauri::command]
pub async fn cluster_get_pod_usage(
    cluster_id: String,
    namespace: Option<String>,
    label_selector: Option<String>,
    state: State<'_, ClusterManagerState>,
) -> Result<Vec<PodUsage>, String> {
    let client = create_client_for_cluster(&cluster_id, &state).await?;
    let ar = metrics_api_resource("PodMetrics", "pods");

    let api: Api<DynamicObject> = match namespace.as_deref() {
        Some(ns) if !ns.is_empty() && ns != "all" => Api::namespaced_with(client, ns, &ar),
        _ => Api::all_with(client, &ar),
    };

    let mut lp = ListParams::default();
    if let Some(selector) = label_selector.filter(|s| !s.trim().is_empty()) {
        lp = lp.labels(&selector);
    }

    let list = api.list(&lp).await.map_err(|e| metrics_error("pod", e))?;

    Ok(list.items.into_iter().map(map_pod_metrics).collect())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use serde_json::json;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6 * b.abs().max(1.0)
    }

    #[test]
    fn parse_quantity_plain_and_exponent() {
        assert_eq!(parse_quantity("2"), Some(2.0));
        assert_eq!(parse_quantity("0.5"), Some(0.5));
        assert_eq!(parse_quantity("1e3"), Some(1000.0));
        assert_eq!(parse_quantity("2.5E6"), Some(2_500_000.0));
    }

    #[test]
    fn parse_quantity_decimal_suffixes() {
        assert!(approx(parse_quantity("250m").unwrap(), 0.25));
        assert!(approx(parse_quantity("1500u").unwrap(), 0.0015));
        assert!(approx(parse_quantity("123456789n").unwrap(), 0.123456789));
        assert_eq!(parse_quantity("2k"), Some(2000.0));
        assert_eq!(parse_quantity("3M"), Some(3_000_000.0));
        assert_eq!(parse_quantity("1G"), Some(1e9));
        assert_eq!(parse_quantity("1E"), Some(1e18));
    }

    #[test]
    fn parse_quantity_binary_suffixes() {
        assert_eq!(parse_quantity("1Ki"), Some(1024.0));
        assert_eq!(parse_quantity("512Mi"), Some(512.0 * 1024.0 * 1024.0));
        assert_eq!(parse_quantity("2Gi"), Some(2.0 * 1024f64.powi(3)));
        assert_eq!(parse_quantity("1Ti"), Some(1024f64.powi(4)));
    }

    #[test]
    fn parse_quantity_invalid() {
        assert_eq!(parse_quantity(""), None);
        assert_eq!(parse_quantity("abc"), None);
        assert_eq!(parse_quantity("Mi"), None);
        assert_eq!(cpu_millicores("junk"), 0.0);
        assert_eq!(memory_bytes("junk"), 0.0);
    }

    #[test]
    fn cpu_and_memory_helpers() {
        assert!(approx(cpu_millicores("250m"), 250.0));
        assert!(approx(cpu_millicores("1"), 1000.0));
        assert!(approx(cpu_millicores("123456789n"), 123.456789));
        assert_eq!(memory_bytes("100Mi"), 100.0 * 1024.0 * 1024.0);
    }

    fn dynamic(json: serde_json::Value) -> DynamicObject {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn maps_pod_metrics_and_sums_containers() {
        let obj = dynamic(json!({
            "apiVersion": "metrics.k8s.io/v1beta1",
            "kind": "PodMetrics",
            "metadata": { "name": "api-1", "namespace": "prod" },
            "timestamp": "2026-01-01T00:00:00Z",
            "window": "15s",
            "containers": [
                { "name": "app", "usage": { "cpu": "250m", "memory": "100Mi" } },
                { "name": "sidecar", "usage": { "cpu": "50000000n", "memory": "20Mi" } }
            ]
        }));
        let usage = map_pod_metrics(obj);
        assert_eq!(usage.name, "api-1");
        assert_eq!(usage.namespace, "prod");
        assert_eq!(usage.containers.len(), 2);
        assert!(approx(usage.cpu_millicores, 300.0));
        assert_eq!(usage.memory_bytes, 120.0 * 1024.0 * 1024.0);
        assert_eq!(usage.window.as_deref(), Some("15s"));
    }

    #[test]
    fn maps_node_metrics() {
        let obj = dynamic(json!({
            "apiVersion": "metrics.k8s.io/v1beta1",
            "kind": "NodeMetrics",
            "metadata": { "name": "node-a" },
            "timestamp": "2026-01-01T00:00:00Z",
            "window": "20s",
            "usage": { "cpu": "1500m", "memory": "4Gi" }
        }));
        let usage = map_node_metrics(obj);
        assert_eq!(usage.name, "node-a");
        assert!(approx(usage.cpu_millicores, 1500.0));
        assert_eq!(usage.memory_bytes, 4.0 * 1024f64.powi(3));
    }

    #[test]
    fn maps_missing_usage_to_zero() {
        let obj = dynamic(json!({
            "apiVersion": "metrics.k8s.io/v1beta1",
            "kind": "NodeMetrics",
            "metadata": { "name": "node-b" }
        }));
        let usage = map_node_metrics(obj);
        assert_eq!(usage.cpu_millicores, 0.0);
        assert_eq!(usage.memory_bytes, 0.0);
        assert!(usage.timestamp.is_none());
    }
}

/// Live tests against the current kubeconfig context. Opt-in via
/// `KORE_LIVE_TESTS=1 cargo test live_` so CI (no cluster) is unaffected.
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod live_tests {
    use super::*;

    fn enabled() -> bool {
        std::env::var("KORE_LIVE_TESTS")
            .map(|v| v == "1")
            .unwrap_or(false)
    }

    async fn client() -> kube::Client {
        kube::Client::try_default()
            .await
            .expect("kubeconfig with a reachable current context")
    }

    #[tokio::test]
    async fn live_node_usage_returns_data_or_clear_unavailable_error() {
        if !enabled() {
            return;
        }
        let ar = metrics_api_resource("NodeMetrics", "nodes");
        let api: Api<DynamicObject> = Api::all_with(client().await, &ar);
        match api.list(&ListParams::default()).await {
            Ok(list) => {
                let usage: Vec<NodeUsage> = list.items.into_iter().map(map_node_metrics).collect();
                assert!(
                    !usage.is_empty(),
                    "metrics-server present but returned no nodes"
                );
                for n in &usage {
                    assert!(!n.name.is_empty());
                    assert!(n.cpu_millicores >= 0.0 && n.memory_bytes >= 0.0);
                }
                eprintln!("node usage: {:?}", usage);
            }
            Err(e) => {
                let msg = metrics_error("node", e);
                assert!(
                    msg.contains("Metrics API not available"),
                    "unexpected error shape: {msg}"
                );
                eprintln!("{msg}");
            }
        }
    }

    #[tokio::test]
    async fn live_pod_usage_with_selector_does_not_panic() {
        if !enabled() {
            return;
        }
        let ar = metrics_api_resource("PodMetrics", "pods");
        let api: Api<DynamicObject> = Api::namespaced_with(client().await, "kube-system", &ar);
        let lp = ListParams::default().labels("k8s-app=kube-dns");
        match api.list(&lp).await {
            Ok(list) => {
                let usage: Vec<PodUsage> = list.items.into_iter().map(map_pod_metrics).collect();
                for p in &usage {
                    assert_eq!(p.namespace, "kube-system");
                    assert!(!p.containers.is_empty());
                }
                eprintln!("pod usage: {} pods", usage.len());
            }
            Err(e) => {
                let msg = metrics_error("pod", e);
                assert!(msg.contains("Metrics API not available"), "{msg}");
            }
        }
    }
}
