import { invoke } from '@tauri-apps/api/core';
import { contextsStore } from './contexts.svelte';

// Active context is kept in-memory only — no localStorage persistence.
// The active context is restored from the URL route on navigation,
// and the backend tracks last_accessed via db_update_context_last_accessed.
// Persisting to localStorage would expose cluster activity metadata in
// plaintext outside the encrypted vault.

class ActiveContextStore {
  contextId = $state<string | null>(null);
  namespaces = $state<string[]>([]);
  activeNamespace = $state<string>('all');
  loading = $state(false);

  get clusterId(): string | null {
    return this.contextId;
  }

  get contextName(): string | null {
    if (!this.contextId) return null;
    const context = contextsStore.contexts.find((item) => item.id === this.contextId);
    return context?.name || null;
  }

  async setCluster(contextId: string | null) {
    this.contextId = contextId;
    this.activeNamespace = 'all';

    if (contextId) {
      await this.fetchNamespaces();
    } else {
      this.namespaces = [];
    }
  }

  async setContext(contextId: string | null) {
    await this.setCluster(contextId);
  }

  setNamespace(namespace: string) {
    this.activeNamespace = namespace;
  }

  async fetchNamespaces() {
    if (!this.contextId) {
      this.namespaces = [];
      return;
    }

    this.loading = true;
    try {
      const nss = await invoke<string[]>('cluster_list_namespaces', {
        clusterId: this.contextId,
      });
      this.namespaces = nss.sort();

      if (this.activeNamespace !== 'all' && !this.namespaces.includes(this.activeNamespace)) {
        this.activeNamespace = 'all';
      }
    } catch (e) {
      console.error('Failed to fetch namespaces', e);
      this.namespaces = [];
    } finally {
      this.loading = false;
    }
  }
}

export const activeContextStore = new ActiveContextStore();
export const activeClusterStore = activeContextStore;
