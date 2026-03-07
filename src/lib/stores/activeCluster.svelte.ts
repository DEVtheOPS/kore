import { invoke } from '@tauri-apps/api/core';
import { contextsStore } from './contexts.svelte';

const STORAGE_KEY = 'kore-active-context';

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

  constructor() {
    this.loadFromStorage();
  }

  loadFromStorage() {
    if (typeof localStorage === 'undefined') return;

    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved) {
      try {
        const data = JSON.parse(saved);
        this.contextId = data.contextId || data.clusterId || null;
        this.activeNamespace = data.activeNamespace || 'all';
      } catch (e) {
        console.error('Failed to parse active context', e);
      }
    }
  }

  save() {
    if (typeof localStorage === 'undefined') return;

    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({
        contextId: this.contextId,
        activeNamespace: this.activeNamespace,
      }),
    );
  }

  async setCluster(contextId: string | null) {
    this.contextId = contextId;
    this.activeNamespace = 'all';
    this.save();

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
    this.save();
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
        this.save();
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
