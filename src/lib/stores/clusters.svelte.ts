import { invoke } from '@tauri-apps/api/core';

export interface Cluster {
  id: string;
  name: string;
  context_name: string;
  icon?: string;
  description?: string;
  tags: string; // JSON-encoded array
  created_at: number;
  last_accessed: number;
}

class ClustersStore {
  clusters = $state<Cluster[]>([]);
  loading = $state(false);

  constructor() {
    this.load();
  }

  async load() {
    this.loading = true;
    try {
      const clusters = await invoke<Cluster[]>('db_list_clusters');
      this.clusters = clusters;
    } catch (e) {
      console.error('Failed to load clusters', e);
      this.clusters = [];
    } finally {
      this.loading = false;
    }
  }

  async get(id: string): Promise<Cluster | null> {
    try {
      const cluster = await invoke<Cluster | null>('db_get_cluster', { id });
      return cluster;
    } catch (e) {
      console.error('Failed to get cluster', e);
      return null;
    }
  }

  async update(
    id: string,
    updates: {
      name?: string;
      icon?: string | null;
      description?: string | null;
      tags?: string[];
    }
  ) {
    try {
      await invoke('db_update_cluster', {
        id,
        name: updates.name,
        icon: updates.icon !== undefined ? updates.icon : undefined,
        description: updates.description !== undefined ? updates.description : undefined,
        tags: updates.tags,
      });
      await this.load(); // Reload to get updated data
    } catch (e) {
      console.error('Failed to update cluster', e);
      throw e;
    }
  }

  async updateLastAccessed(id: string) {
    try {
      await invoke('db_update_last_accessed', { id });
      // Update local state
      const cluster = this.clusters.find((c) => c.id === id);
      if (cluster) {
        cluster.last_accessed = Date.now() / 1000;
      }
    } catch (e) {
      console.error('Failed to update last accessed', e);
    }
  }

  async remove(id: string) {
    try {
      await invoke('db_delete_cluster', { id });
      await this.load(); // Reload list
    } catch (e) {
      console.error('Failed to delete cluster', e);
      throw e;
    }
  }

  getTags(cluster: Cluster): string[] {
    try {
      return JSON.parse(cluster.tags);
    } catch {
      return [];
    }
  }
}

export const clustersStore = new ClustersStore();
