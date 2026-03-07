mod config;
mod db;
mod image_utils;
mod import;
mod input_validation;
mod k8s;
mod security;

// Legacy modules kept temporarily during the refactor (Phase 8 will remove these).
mod cluster_manager;

use db::AppDbState;
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 1. Initialise the ~/.kore/ directory.
    if let Err(e) = config::init_directories() {
        eprintln!("Failed to initialise config directories: {}", e);
        std::process::exit(1);
    }

    // 2. Retrieve (or generate on first launch) the SQLCipher encryption key
    //    from the OS keychain.
    let db_key = match security::get_or_create_db_key() {
        Ok(key) => key,
        Err(e) => {
            eprintln!("Failed to obtain database encryption key: {}", e);
            std::process::exit(1);
        }
    };

    // 3. Open the encrypted database.
    let db_path = config::get_db_path();
    let app_db = match db::AppDb::new(db_path, &db_key) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to initialise database: {}", e);
            std::process::exit(1);
        }
    };
    let app_db_state = AppDbState(Arc::new(app_db));

    // 4. Legacy cluster manager (kept for backwards compat during refactor).
    let legacy_db_path = config::get_app_config_dir().join("clusters_legacy.db");
    let cluster_manager = match cluster_manager::ClusterManager::new(legacy_db_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Warning: Failed to initialise legacy cluster manager: {}", e);
            // Don't exit — legacy manager is not critical for new architecture.
            // Create a dummy in-memory manager so the app still starts.
            cluster_manager::ClusterManager::new(std::path::PathBuf::from(":memory:"))
                .expect("In-memory SQLite should always succeed")
        }
    };
    let cluster_manager_state = cluster_manager::ClusterManagerState(Arc::new(
        std::sync::Mutex::new(cluster_manager),
    ));

    tauri::Builder::default()
        .plugin(tauri_plugin_websocket::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        // New encrypted DB state
        .manage(app_db_state)
        // Legacy state (kept during refactor)
        .manage(cluster_manager_state)
        .manage(k8s::WatcherState::default())
        .invoke_handler(tauri::generate_handler![
            // ── New: Cluster CRUD ────────────────────────────────────────────
            db::clusters::db_list_clusters,
            db::clusters::db_get_cluster,
            db::clusters::db_update_cluster,
            db::clusters::db_delete_cluster,
            // ── New: User CRUD ───────────────────────────────────────────────
            db::users::db_list_users,
            db::users::db_get_user,
            db::users::db_update_user,
            db::users::db_delete_user,
            // ── New: Context CRUD ────────────────────────────────────────────
            db::contexts::db_list_contexts,
            db::contexts::db_list_pinned_contexts,
            db::contexts::db_get_context,
            db::contexts::db_update_context,
            db::contexts::db_update_context_last_accessed,
            db::contexts::db_pin_context,
            db::contexts::db_unpin_context,
            db::contexts::db_reorder_pinned_contexts,
            db::contexts::db_delete_context,
            // ── New: Settings ────────────────────────────────────────────────
            config::settings::settings_get,
            config::settings::settings_update,
            // ── New: Security ────────────────────────────────────────────────
            security::lock::security_biometrics_available,
            security::lock::security_verify_keychain_access,
            // ── New: Import (updated) ─────────────────────────────────────────
            import::import_discover_file,
            import::import_discover_folder,
            import::import_add_context,
            // ── New: Export ───────────────────────────────────────────────────
            import::export_kubeconfig,
            import::export_preview,
            // ── Image processing ──────────────────────────────────────────────
            image_utils::process_icon_file,
            // ── Legacy k8s commands (kept during refactor) ───────────────────
            k8s::list_contexts,
            k8s::list_namespaces,
            k8s::list_pods,
            k8s::delete_pod,
            k8s::get_pod_events,
            k8s::stream_container_logs,
            k8s::stop_stream_logs,
            k8s::start_pod_watch,
            k8s::cluster_list_namespaces,
            k8s::cluster_list_namespaces_detailed,
            k8s::cluster_delete_namespace,
            k8s::cluster_list_pods,
            k8s::cluster_delete_pod,
            k8s::cluster_get_pod_events,
            k8s::cluster_stream_container_logs,
            k8s::cluster_start_pod_watch,
            k8s::cluster_get_metrics,
            k8s::cluster_get_events,
            k8s::cluster_list_events,
            k8s::cluster_list_nodes,
            k8s::cluster_get_resource_yaml,
            k8s::cluster_apply_resource_yaml,
            k8s::cluster_scale_workload,
            k8s::cluster_restart_workload,
            k8s::cluster_list_deployments,
            k8s::cluster_delete_deployment,
            k8s::cluster_list_statefulsets,
            k8s::cluster_delete_statefulset,
            k8s::cluster_list_daemonsets,
            k8s::cluster_delete_daemonset,
            k8s::cluster_list_replicasets,
            k8s::cluster_delete_replicaset,
            k8s::cluster_list_jobs,
            k8s::cluster_delete_job,
            k8s::cluster_list_cronjobs,
            k8s::cluster_delete_cronjob,
            k8s::cluster_list_config_maps,
            k8s::cluster_delete_config_map,
            k8s::cluster_list_secrets,
            k8s::cluster_delete_secret,
            k8s::cluster_list_resource_quotas,
            k8s::cluster_delete_resource_quota,
            k8s::cluster_list_limit_ranges,
            k8s::cluster_delete_limit_range,
            k8s::cluster_list_hpa,
            k8s::cluster_delete_hpa,
            k8s::cluster_list_pdb,
            k8s::cluster_delete_pdb,
            k8s::cluster_list_services,
            k8s::cluster_delete_service,
            k8s::cluster_list_endpoints,
            k8s::cluster_delete_endpoint,
            k8s::cluster_list_ingresses,
            k8s::cluster_delete_ingress,
            k8s::cluster_list_network_policies,
            k8s::cluster_delete_network_policy,
            k8s::cluster_list_pvc,
            k8s::cluster_delete_pvc,
            k8s::cluster_list_pv,
            k8s::cluster_delete_pv,
            k8s::cluster_list_storage_classes,
            k8s::cluster_delete_storage_class,
            k8s::cluster_list_service_accounts,
            k8s::cluster_delete_service_account,
            k8s::cluster_list_roles,
            k8s::cluster_delete_role,
            k8s::cluster_list_role_bindings,
            k8s::cluster_delete_role_binding,
            k8s::cluster_list_cluster_roles,
            k8s::cluster_delete_cluster_role,
            k8s::cluster_list_cluster_role_bindings,
            k8s::cluster_delete_cluster_role_binding,
            k8s::cluster_list_crds,
            k8s::cluster_delete_crd,
            k8s::cluster_check_helm_available,
            k8s::cluster_list_helm_releases,
            k8s::cluster_list_helm_charts,
            k8s::cluster_get_deployment_details,
            k8s::cluster_get_deployment_pods,
            k8s::cluster_get_deployment_replicasets,
            k8s::cluster_get_deployment_events,
            k8s::cluster_get_statefulset_details,
            k8s::cluster_get_statefulset_pods,
            k8s::cluster_get_statefulset_events,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("Error running tauri application: {}", e);
            std::process::exit(1);
        });
}
