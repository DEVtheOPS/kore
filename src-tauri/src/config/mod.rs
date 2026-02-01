use std::path::PathBuf;
use std::fs;
use serde::{Serialize, Deserialize};
use notify::{Watcher, RecursiveMode, RecommendedWatcher, Config};
use tauri::{Emitter, AppHandle};
use std::sync::mpsc::channel;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct AppConfig {
    #[allow(dead_code)]
    pub kubeconfig_paths: Vec<PathBuf>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            kubeconfig_paths: vec![],
        }
    }
}

pub fn get_app_config_dir() -> PathBuf {
    let mut path = dirs::home_dir().expect("Could not find home directory");
    path.push(".rustylens");
    path
}

pub fn get_kubeconfigs_dir() -> PathBuf {
    let mut path = get_app_config_dir();
    path.push("kubeconfigs");
    path
}

pub fn init_directories() -> std::io::Result<()> {
    let app_dir = get_app_config_dir();
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)?;
    }
    
    let kube_dir = get_kubeconfigs_dir();
    if !kube_dir.exists() {
        fs::create_dir_all(&kube_dir)?;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn import_kubeconfig(path: String) -> Result<String, String> {
    let source = PathBuf::from(path);
    if !source.exists() {
        return Err("Source file does not exist".to_string());
    }
    
    let file_name = source.file_name()
        .ok_or("Invalid file name")?
        .to_string_lossy()
        .to_string();
        
    // Add timestamp to prevent overwrites or just unique ID
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
        
    let dest_name = format!("{}_{}", timestamp, file_name);
    let dest_path = get_kubeconfigs_dir().join(&dest_name);
    
    fs::copy(&source, &dest_path)
        .map_err(|e| format!("Failed to copy config: {}", e))?;
        
    Ok(dest_name)
}

use std::time::{Duration, Instant};

pub fn start_watcher(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let (tx, rx) = channel();
        
        let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();
        
        let path = get_kubeconfigs_dir();
        if let Err(e) = watcher.watch(&path, RecursiveMode::NonRecursive) {
            eprintln!("Failed to watch kubeconfigs dir: {}", e);
            return;
        }

        let mut last_event_time = Instant::now();
        let debounce_duration = Duration::from_millis(500);

        for res in rx {
            match res {
                Ok(event) => {
                    // Simple debounce
                    if last_event_time.elapsed() < debounce_duration {
                        continue;
                    }

                    // Filter for meaningful events
                    if event.kind.is_create() || event.kind.is_remove() || event.kind.is_modify() {
                        let _ = app_handle.emit("kubeconfig_update", ());
                        last_event_time = Instant::now();
                    }
                },
                Err(e) => eprintln!("Watch error: {:?}", e),
            }
        }
    });
}
