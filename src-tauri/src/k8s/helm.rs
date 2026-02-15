use crate::cluster_manager::ClusterManagerState;
use serde_json::Value;
use std::process::Command;
use tauri::State;

#[derive(serde::Serialize, Debug)]
pub struct HelmReleaseSummary {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub age: String,
    pub labels: std::collections::BTreeMap<String, String>,
    pub status: String,
    pub images: Vec<String>,
    pub created_at: i64,
    pub revision: String,
    pub chart: String,
    pub app_version: String,
    pub updated: String,
}

#[derive(serde::Serialize, Debug)]
pub struct HelmChartSummary {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub age: String,
    pub labels: std::collections::BTreeMap<String, String>,
    pub status: String,
    pub images: Vec<String>,
    pub created_at: i64,
    pub chart: String,
    pub version: String,
    pub app_version: String,
    pub description: String,
}

#[derive(serde::Serialize, Debug)]
pub struct HelmAvailability {
    pub available: bool,
    pub version: Option<String>,
    pub message: Option<String>,
}

fn get_cluster_kubeconfig_and_context(
    cluster_id: &str,
    state: &State<'_, ClusterManagerState>,
) -> Result<(String, String), String> {
    let manager = state
        .0
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    let cluster = manager
        .get_cluster(cluster_id)?
        .ok_or_else(|| format!("Cluster '{}' not found", cluster_id))?;
    Ok((cluster.config_path, cluster.context_name))
}

#[tauri::command]
pub async fn cluster_check_helm_available() -> Result<HelmAvailability, String> {
    let output = Command::new("helm").arg("version").arg("--short").output();

    match output {
        Ok(out) if out.status.success() => {
            let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
            Ok(HelmAvailability {
                available: true,
                version: Some(version),
                message: None,
            })
        }
        Ok(out) => Ok(HelmAvailability {
            available: false,
            version: None,
            message: Some(String::from_utf8_lossy(&out.stderr).trim().to_string()),
        }),
        Err(err) => Ok(HelmAvailability {
            available: false,
            version: None,
            message: Some(format!("Failed to execute helm: {}", err)),
        }),
    }
}

#[tauri::command]
pub async fn cluster_list_helm_releases(
    cluster_id: String,
    state: State<'_, ClusterManagerState>,
) -> Result<Vec<HelmReleaseSummary>, String> {
    let (kubeconfig, context_name) = get_cluster_kubeconfig_and_context(&cluster_id, &state)?;

    let output = Command::new("helm")
        .args([
            "list",
            "--all-namespaces",
            "-o",
            "json",
            "--kubeconfig",
            &kubeconfig,
            "--kube-context",
            &context_name,
        ])
        .output()
        .map_err(|e| format!("Failed to execute helm list: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "helm list failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let parsed: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse helm list output: {}", e))?;

    let mut releases = Vec::new();
    if let Some(items) = parsed.as_array() {
        for item in items {
            let name = item
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let namespace = item
                .get("namespace")
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string();
            let revision = item
                .get("revision")
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string();
            let updated = item
                .get("updated")
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string();
            let status = item
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let chart = item
                .get("chart")
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string();
            let app_version = item
                .get("app_version")
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string();

            releases.push(HelmReleaseSummary {
                id: format!("{}/{}", namespace, name),
                name,
                namespace,
                age: "-".to_string(),
                labels: std::collections::BTreeMap::new(),
                status,
                images: vec![chart.clone()],
                created_at: 0,
                revision,
                chart,
                app_version,
                updated,
            });
        }
    }

    Ok(releases)
}

#[tauri::command]
pub async fn cluster_list_helm_charts(
    _cluster_id: String,
    _state: State<'_, ClusterManagerState>,
) -> Result<Vec<HelmChartSummary>, String> {
    let output = Command::new("helm")
        .args(["search", "repo", "-o", "json"])
        .output()
        .map_err(|e| format!("Failed to execute helm search repo: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "helm search repo failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let parsed: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse helm search output: {}", e))?;

    let mut charts = Vec::new();
    if let Some(items) = parsed.as_array() {
        for item in items {
            let chart = item
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let version = item
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string();
            let app_version = item
                .get("app_version")
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string();
            let description = item
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            charts.push(HelmChartSummary {
                id: format!("{}:{}", chart, version),
                name: chart.split('/').last().unwrap_or_default().to_string(),
                namespace: "-".to_string(),
                age: "-".to_string(),
                labels: std::collections::BTreeMap::new(),
                status: "Available".to_string(),
                images: vec![],
                created_at: 0,
                chart,
                version,
                app_version,
                description,
            });
        }
    }

    Ok(charts)
}
