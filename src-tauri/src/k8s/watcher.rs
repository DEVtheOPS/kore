//! Registry of long-running background tasks (resource watches, log streams)
//! keyed by a string so they can be replaced or aborted on demand.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tauri::async_runtime::JoinHandle;

/// A registered task plus the generation it was inserted with. Generations
/// let a task that exits naturally remove only *its own* entry, so it never
/// evicts a newer task that has since taken over the same key.
pub struct WatcherEntry {
    pub generation: u64,
    pub handle: JoinHandle<()>,
}

#[derive(Clone)]
pub struct WatcherState(pub Arc<Mutex<HashMap<String, WatcherEntry>>>);

static NEXT_GENERATION: AtomicU64 = AtomicU64::new(1);

impl Default for WatcherState {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }
}

impl WatcherState {
    /// Allocate a new unique generation for a task about to be registered.
    pub fn next_generation() -> u64 {
        NEXT_GENERATION.fetch_add(1, Ordering::Relaxed)
    }

    /// Abort and remove whatever task currently owns `key` (if any).
    pub fn abort(&self, key: &str) -> Result<(), String> {
        let mut watchers = self
            .0
            .lock()
            .map_err(|e| format!("Watcher state lock poisoned: {}", e))?;
        if let Some(entry) = watchers.remove(key) {
            entry.handle.abort();
        }
        Ok(())
    }

    /// Register `handle` under `key`, aborting any task that previously owned
    /// the key.
    pub fn insert(
        &self,
        key: String,
        generation: u64,
        handle: JoinHandle<()>,
    ) -> Result<(), String> {
        let mut watchers = self
            .0
            .lock()
            .map_err(|e| format!("Watcher state lock poisoned: {}", e))?;
        if let Some(previous) = watchers.insert(key, WatcherEntry { generation, handle }) {
            previous.handle.abort();
        }
        Ok(())
    }

    /// Remove `key` only if it is still owned by `generation`. Called by a
    /// task when it finishes on its own.
    pub fn remove_if_current(&self, key: &str, generation: u64) {
        match self.0.lock() {
            Ok(mut watchers) => {
                if watchers
                    .get(key)
                    .is_some_and(|e| e.generation == generation)
                {
                    watchers.remove(key);
                }
            }
            Err(_) => eprintln!("Warning: failed to clean up watcher state for {}", key),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn dummy_task() -> JoinHandle<()> {
        tauri::async_runtime::spawn(async {
            // Park forever; the test aborts it.
            std::future::pending::<()>().await;
        })
    }

    #[tokio::test]
    async fn natural_exit_does_not_evict_newer_generation() {
        let state = WatcherState::default();
        let g1 = WatcherState::next_generation();
        let g2 = WatcherState::next_generation();
        state.insert("k".into(), g1, dummy_task()).unwrap();
        state.insert("k".into(), g2, dummy_task()).unwrap();

        // Old task's cleanup runs late: must be a no-op.
        state.remove_if_current("k", g1);
        assert!(state.0.lock().unwrap().contains_key("k"));

        // Current task's cleanup removes it.
        state.remove_if_current("k", g2);
        assert!(!state.0.lock().unwrap().contains_key("k"));
    }

    #[tokio::test]
    async fn abort_removes_entry() {
        let state = WatcherState::default();
        let g = WatcherState::next_generation();
        state.insert("k".into(), g, dummy_task()).unwrap();
        state.abort("k").unwrap();
        assert!(!state.0.lock().unwrap().contains_key("k"));
        // Aborting a missing key is fine.
        state.abort("k").unwrap();
    }
}
