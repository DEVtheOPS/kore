use crate::cluster_manager::ClusterManagerState;
use k8s_openapi::api::apps::v1::{Deployment, ReplicaSet};
use k8s_openapi::api::core::v1::{Container, Pod, ResourceRequirements};
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use kube::api::{ApiResource, DynamicObject, ListParams, ObjectMeta};
use kube::core::{NotUsed, Object, Selector};
use kube::Api;
use std::collections::{BTreeMap, HashMap, HashSet};
use tauri::State;

use super::client::create_client_for_cluster;
use super::usage::{list_pod_metrics, parse_quantity};

#[derive(Default, serde::Serialize)]
pub struct DeploymentResourceUsage {
    pub pods: Vec<DeploymentPodResources>,
    pub usage_error: Option<String>,
}

#[derive(serde::Serialize)]
pub struct DeploymentPodResources {
    pub name: String,
    pub cpu_millicores: Option<f64>,
    pub memory_bytes: Option<f64>,
    pub cpu_request_millicores: Option<f64>,
    pub cpu_limit_millicores: Option<f64>,
    pub memory_request_bytes: Option<f64>,
    pub memory_limit_bytes: Option<f64>,
    pub window: Option<String>,
}

// Kubernetes 1.31's generated PodSpec omits pod-level resource requirements.
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PodResourceSpec {
    containers: Vec<Container>,
    init_containers: Option<Vec<Container>>,
    overhead: Option<BTreeMap<String, Quantity>>,
    resources: Option<ResourceRequirements>,
}

#[derive(Clone, Copy)]
struct ResourceBudget {
    request: Option<f64>,
    limit: Option<f64>,
}

impl ResourceBudget {
    // Identity for both a sum and peak. An absent limit is unbounded.
    const EMPTY: Self = Self {
        request: None,
        limit: Some(0.0),
    };

    fn read(resources: Option<&ResourceRequirements>, resource: &str) -> Self {
        Self {
            request: read_quantity(resources.and_then(|r| r.requests.as_ref()), resource),
            limit: read_quantity(resources.and_then(|r| r.limits.as_ref()), resource),
        }
    }

    fn add(self, other: Self) -> Self {
        Self {
            request: combine_present(self.request, other.request, |a, b| a + b),
            limit: self.limit.zip(other.limit).map(|(a, b)| a + b),
        }
    }

    fn peak(self, other: Self) -> Self {
        Self {
            request: combine_present(self.request, other.request, f64::max),
            limit: self.limit.zip(other.limit).map(|(a, b)| a.max(b)),
        }
    }
}

fn combine_present(
    a: Option<f64>,
    b: Option<f64>,
    combine: impl FnOnce(f64, f64) -> f64,
) -> Option<f64> {
    a.zip(b).map(|(a, b)| combine(a, b)).or(a).or(b)
}

fn read_quantity(quantities: Option<&BTreeMap<String, Quantity>>, resource: &str) -> Option<f64> {
    parse_quantity(&quantities?.get(resource)?.0)
}

fn pod_budget(spec: &PodResourceSpec, resource: &str) -> ResourceBudget {
    let mut sidecars = ResourceBudget::EMPTY;
    let mut init_peak = ResourceBudget::EMPTY;
    for init in spec.init_containers.as_deref().unwrap_or_default() {
        let phase = sidecars.add(ResourceBudget::read(init.resources.as_ref(), resource));
        if init.restart_policy.as_deref() == Some("Always") {
            sidecars = phase;
        }
        init_peak = init_peak.peak(phase);
    }

    let running = spec.containers.iter().fold(sidecars, |budget, container| {
        budget.add(ResourceBudget::read(container.resources.as_ref(), resource))
    });
    let effective = running.peak(init_peak);
    let pod_level = ResourceBudget::read(spec.resources.as_ref(), resource);
    let overhead = read_quantity(spec.overhead.as_ref(), resource);

    ResourceBudget {
        request: combine_present(pod_level.request.or(effective.request), overhead, |a, b| {
            a + b
        }),
        limit: pod_level.limit.or(effective.limit).map(|limit| {
            if limit == 0.0 {
                limit
            } else {
                limit + overhead.unwrap_or(0.0)
            }
        }),
    }
}

fn controller_uid<'a>(metadata: &'a ObjectMeta, kind: &str) -> Option<&'a str> {
    metadata
        .owner_references
        .as_ref()?
        .iter()
        .find(|owner| owner.kind == kind && owner.controller == Some(true))
        .map(|owner| owner.uid.as_str())
}

fn summed_usage(metrics: Option<&DynamicObject>, resource: &str) -> Option<f64> {
    let containers = metrics?.data.get("containers")?.as_array()?;
    if containers.is_empty() {
        return None;
    }
    containers
        .iter()
        .map(|container| parse_quantity(container.get("usage")?.get(resource)?.as_str()?))
        .sum()
}

fn pod_resources(
    name: String,
    spec: &PodResourceSpec,
    metrics: Option<&DynamicObject>,
) -> DeploymentPodResources {
    let cpu = pod_budget(spec, "cpu");
    let memory = pod_budget(spec, "memory");
    DeploymentPodResources {
        name,
        cpu_millicores: summed_usage(metrics, "cpu").map(|cores| cores * 1000.0),
        memory_bytes: summed_usage(metrics, "memory"),
        cpu_request_millicores: cpu.request.map(|cores| cores * 1000.0),
        cpu_limit_millicores: cpu.limit.map(|cores| cores * 1000.0),
        memory_request_bytes: memory.request,
        memory_limit_bytes: memory.limit,
        window: metrics
            .and_then(|metrics| metrics.data.get("window")?.as_str())
            .map(str::to_owned),
    }
}

#[tauri::command]
pub async fn cluster_get_deployment_resource_usage(
    cluster_id: String,
    namespace: String,
    deployment_name: String,
    state: State<'_, ClusterManagerState>,
) -> Result<DeploymentResourceUsage, String> {
    let client = create_client_for_cluster(&cluster_id, &state).await?;
    let deployments: Api<Deployment> = Api::namespaced(client.clone(), &namespace);
    let deployment = deployments
        .get(&deployment_name)
        .await
        .map_err(|e| format!("Failed to get deployment '{deployment_name}': {e}"))?;
    let deployment_uid = deployment.metadata.uid.ok_or("Deployment has no UID")?;
    let spec = deployment.spec.ok_or("Deployment has no spec")?;
    let selector = Selector::try_from(spec.selector)
        .map_err(|e| format!("Invalid selector for deployment '{deployment_name}': {e}"))?;
    if selector.selects_all() {
        return Ok(DeploymentResourceUsage::default());
    }

    let pod_resource = ApiResource::erase::<Pod>(&());
    let pods: Api<Object<PodResourceSpec, NotUsed>> =
        Api::namespaced_with(client.clone(), &namespace, &pod_resource);
    let replica_sets: Api<ReplicaSet> = Api::namespaced(client.clone(), &namespace);
    let selected = ListParams::default().labels_from(&selector);
    let all = ListParams::default();
    let (pods, replica_sets, metrics) = tokio::join!(
        pods.list(&selected),
        replica_sets.list_metadata(&all),
        list_pod_metrics(client, Some(&namespace), &selected)
    );
    let pods = pods.map_err(|e| format!("Failed to list pods: {e}"))?;
    let replica_sets = replica_sets.map_err(|e| format!("Failed to list replicasets: {e}"))?;
    let replica_set_uids: HashSet<_> = replica_sets
        .items
        .iter()
        .filter(|rs| controller_uid(&rs.metadata, "Deployment") == Some(deployment_uid.as_str()))
        .filter_map(|rs| rs.metadata.uid.as_deref())
        .collect();

    let mut response = DeploymentResourceUsage::default();
    let metrics = match metrics {
        Ok(metrics) => metrics,
        Err(error) => {
            response.usage_error = Some(error);
            Vec::new()
        }
    };
    let metrics_by_name: HashMap<_, _> = metrics
        .iter()
        .filter_map(|pod| Some((pod.metadata.name.as_deref()?, pod)))
        .collect();

    response.pods = pods
        .items
        .into_iter()
        .filter(|pod| {
            controller_uid(&pod.metadata, "ReplicaSet")
                .is_some_and(|uid| replica_set_uids.contains(uid))
        })
        .filter_map(|pod| {
            let name = pod.metadata.name?;
            let metrics = metrics_by_name.get(name.as_str()).copied();
            Some(pod_resources(name, &pod.spec, metrics))
        })
        .collect();
    response.pods.sort_unstable_by(|a, b| a.name.cmp(&b.name));
    if response.pods.is_empty() {
        response.usage_error = None;
    }
    Ok(response)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn maps_spec_resources_without_a_usage_sample() {
        let spec = serde_json::from_value(json!({
            "containers": [
                {"name": "app", "resources": {
                    "requests": {"cpu": "250m", "memory": "64Mi"},
                    "limits": {"cpu": "1", "memory": "128Mi"}
                }},
                {"name": "helper", "resources": {"requests": {"cpu": "125m"}}}
            ]
        }))
        .unwrap();
        let pod = pod_resources("example".into(), &spec, None);

        assert_eq!(pod.cpu_millicores, None);
        assert_eq!(pod.memory_bytes, None);
        assert_eq!(pod.cpu_request_millicores, Some(375.0));
        assert_eq!(pod.memory_request_bytes, Some(64.0 * 1024.0 * 1024.0));
        assert_eq!(pod.cpu_limit_millicores, None);
        assert_eq!(pod.memory_limit_bytes, None);
    }

    #[test]
    fn handles_init_peaks_overhead_and_pod_level_budgets() {
        let spec = serde_json::from_value(json!({
            "containers": [{"name": "app", "resources": {
                "requests": {"cpu": "250m"}, "limits": {"cpu": "500m"}
            }}],
            "initContainers": [
                {"name": "sidecar", "restartPolicy": "Always", "resources": {
                    "requests": {"cpu": "250m"}, "limits": {"cpu": "500m"}
                }},
                {"name": "init", "resources": {
                    "requests": {"cpu": "500m"}, "limits": {"cpu": "1"}
                }}
            ],
            "resources": {"requests": {"memory": "0"}, "limits": {"memory": "1Gi"}},
            "overhead": {"cpu": "250m"}
        }))
        .unwrap();
        let metrics = serde_json::from_value(json!({
            "metadata": {"name": "example"},
            "window": "15s",
            "containers": [
                {"name": "app", "usage": {"cpu": "250000000n", "memory": "64Mi"}},
                {"name": "sidecar", "usage": {"cpu": "0", "memory": "1Mi"}}
            ]
        }))
        .unwrap();
        let pod = pod_resources("example".into(), &spec, Some(&metrics));

        assert_eq!(pod.cpu_request_millicores, Some(1000.0));
        assert_eq!(pod.cpu_limit_millicores, Some(1750.0));
        assert_eq!(pod.memory_request_bytes, Some(0.0));
        assert_eq!(pod.memory_limit_bytes, Some(1024.0 * 1024.0 * 1024.0));
        assert_eq!(pod.cpu_millicores, Some(250.0));
        assert_eq!(pod.memory_bytes, Some(65.0 * 1024.0 * 1024.0));
        assert_eq!(pod.window.as_deref(), Some("15s"));
    }
}
