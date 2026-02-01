import { invoke } from '@tauri-apps/api/core';
import { clustersStore } from './clusters.svelte';

const STORAGE_KEY = 'kore-active-cluster';

class ActiveClusterStore {
  clusterId = $state<string | null>(null);
  namespaces = $state<string[]>([]);
  activeNamespace = $state<string>('all');
  loading = $state(false);

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

  async setCluster(clusterId: string | null) {
    this.clusterId = clusterId;
    this.activeNamespace = 'all';
    this.save();
    
    if (clusterId) {
      await this.fetchNamespaces();
    } else {
      this.namespaces = [];
    }
  }

  setNamespace(namespace: string) {
    this.activeNamespace = namespace;
    this.save();
  }

  async fetchNamespaces() {
    if (!this.clusterId) {
      this.namespaces = [];
      return;
    }

    this.loading = true;
    try {
      const nss = await invoke<string[]>('cluster_list_namespaces', {
        clusterId: this.clusterId,
      });
      this.namespaces = nss.sort();
      
      // Reset to 'all' if current namespace doesn't exist
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

export const activeClusterStore = new ActiveClusterStore();
