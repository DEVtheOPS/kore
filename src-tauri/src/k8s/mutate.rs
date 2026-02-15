use crate::cluster_manager::ClusterManagerState;
use std::io::Write;
use std::process::{Command, Stdio};
use tauri::State;

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
pub async fn cluster_get_resource_yaml(
    cluster_id: String,
    kind: String,
    name: String,
    namespace: Option<String>,
    state: State<'_, ClusterManagerState>,
) -> Result<String, String> {
    let (kubeconfig, context_name) = get_cluster_kubeconfig_and_context(&cluster_id, &state)?;

    let mut cmd = Command::new("kubectl");
    cmd.args([
        "--kubeconfig",
        &kubeconfig,
        "--context",
        &context_name,
        "get",
        &kind,
        &name,
        "-o",
        "yaml",
    ]);

    if let Some(ns) = namespace {
        if !ns.is_empty() && ns != "-" {
            cmd.args(["-n", &ns]);
        }
    }

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to execute kubectl get: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "kubectl get failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[tauri::command]
pub async fn cluster_apply_resource_yaml(
    cluster_id: String,
    yaml: String,
    state: State<'_, ClusterManagerState>,
) -> Result<String, String> {
    let (kubeconfig, context_name) = get_cluster_kubeconfig_and_context(&cluster_id, &state)?;

    let mut child = Command::new("kubectl")
        .args([
            "--kubeconfig",
            &kubeconfig,
            "--context",
            &context_name,
            "apply",
            "-f",
            "-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to execute kubectl apply: {}", e))?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(yaml.as_bytes())
            .map_err(|e| format!("Failed to write yaml to kubectl stdin: {}", e))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to read kubectl apply output: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "kubectl apply failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[tauri::command]
pub async fn cluster_scale_workload(
    cluster_id: String,
    kind: String,
    namespace: String,
    name: String,
    replicas: i32,
    state: State<'_, ClusterManagerState>,
) -> Result<String, String> {
    let (kubeconfig, context_name) = get_cluster_kubeconfig_and_context(&cluster_id, &state)?;

    let output = Command::new("kubectl")
        .args([
            "--kubeconfig",
            &kubeconfig,
            "--context",
            &context_name,
            "scale",
            &format!("{}/{}", kind, name),
            "-n",
            &namespace,
            "--replicas",
            &replicas.to_string(),
        ])
        .output()
        .map_err(|e| format!("Failed to execute kubectl scale: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "kubectl scale failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[tauri::command]
pub async fn cluster_restart_workload(
    cluster_id: String,
    kind: String,
    namespace: String,
    name: String,
    state: State<'_, ClusterManagerState>,
) -> Result<String, String> {
    let (kubeconfig, context_name) = get_cluster_kubeconfig_and_context(&cluster_id, &state)?;

    let output = Command::new("kubectl")
        .args([
            "--kubeconfig",
            &kubeconfig,
            "--context",
            &context_name,
            "rollout",
            "restart",
            &format!("{}/{}", kind, name),
            "-n",
            &namespace,
        ])
        .output()
        .map_err(|e| format!("Failed to execute kubectl rollout restart: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "kubectl rollout restart failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
