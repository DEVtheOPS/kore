import { invoke } from '@tauri-apps/api/core';
import { clustersStore } from './clusters.svelte';

const STORAGE_KEY = 'kore-active-cluster';

class ActiveClusterStore {
  clusterId = $state<string | null>(null);
  namespaces = $state<string[]>([]);
  activeNamespace = $state<string>('all');
  loading = $state(false);
  private namespaceRequest = 0;
  private selectionSignal?: AbortSignal;

  get contextName(): string | null {
    if (!this.clusterId) return null;
    const cluster = clustersStore.clusters.find(c => c.id === this.clusterId);
    return cluster?.context_name || null;
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
        this.clusterId = data.clusterId;
        this.activeNamespace = data.activeNamespace || 'all';
      } catch (e) {
        console.error('Failed to parse active cluster', e);
      }
    }
  }

  save() {
    if (typeof localStorage === 'undefined') return;
    
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({
        clusterId: this.clusterId,
        activeNamespace: this.activeNamespace,
      })
    );
  }

  async setCluster(clusterId: string | null, signal?: AbortSignal) {
    if (signal?.aborted) return;
    this.namespaceRequest++;
    this.selectionSignal = signal;
    this.clusterId = clusterId;
    this.activeNamespace = 'all';
    this.namespaces = [];
    this.loading = false;
    this.save();

    if (clusterId) await this.fetchNamespaces();
  }

  setNamespace(namespace: string) {
    this.activeNamespace = namespace;
    this.save();
  }

  async fetchNamespaces() {
    const { clusterId, selectionSignal } = this;
    if (selectionSignal?.aborted) return;
    const request = ++this.namespaceRequest;
    const isCurrent = () => request === this.namespaceRequest && !selectionSignal?.aborted;
    if (!clusterId) {
      this.namespaces = [];
      return;
    }

    this.loading = true;
    try {
      const nss = await invoke<string[]>('cluster_list_namespaces', { clusterId });
      if (!isCurrent()) return;
      this.namespaces = [...nss].sort();
      
      // Reset to 'all' if current namespace doesn't exist
      if (this.activeNamespace !== 'all' && !this.namespaces.includes(this.activeNamespace)) {
        this.activeNamespace = 'all';
        this.save();
      }
    } catch (e) {
      if (!isCurrent()) return;
      console.error('Failed to fetch namespaces', e);
      this.namespaces = [];
    } finally {
      if (isCurrent()) this.loading = false;
    }
  }
}

export const activeClusterStore = new ActiveClusterStore();
