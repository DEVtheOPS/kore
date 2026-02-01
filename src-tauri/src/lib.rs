// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod k8s;
mod config;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Init directories
    let _ = config::init_directories();

    tauri::Builder::default()
        .plugin(tauri_plugin_websocket::init())
        // .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            config::start_watcher(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            k8s::list_contexts,
            k8s::list_namespaces,
            k8s::list_pods,
            k8s::delete_pod,
            k8s::get_pod_events,
            k8s::start_pod_watch,
            config::import_kubeconfig
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
