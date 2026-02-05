use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::async_runtime::JoinHandle;

pub struct WatcherState(pub Arc<Mutex<HashMap<String, JoinHandle<()>>>>);

impl Default for WatcherState {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }
}
