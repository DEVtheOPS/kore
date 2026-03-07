import { invoke } from '@tauri-apps/api/core';

interface NamedClusterConfig {
  name?: string;
  cluster?: {
    server?: string;
  };
}

export interface Cluster {
  id: string;
  display_name: string;
  icon?: string;
  description?: string;
  tags: string;
  config: string;
  created_at: number;
  updated_at: number;
  context_count: number;
  name: string;
  server?: string;
}

function parseCluster(record: Omit<Cluster, 'name' | 'server'>): Cluster {
  try {
    const config = JSON.parse(record.config) as NamedClusterConfig;
    return {
      ...record,
      name: config.name ?? record.display_name,
      server: config.cluster?.server,
    };
  } catch {
    return {
      ...record,
      name: record.display_name,
    };
  }
}

class ClustersStore {
  clusters = $state<Cluster[]>([]);
  loading = $state(false);

  constructor() {
    void this.load();
  }

  async load() {
    this.loading = true;
    try {
      const clusters = await invoke<Array<Omit<Cluster, 'name' | 'server'>>>('db_list_clusters');
      this.clusters = clusters.map(parseCluster);
    } catch (e) {
      console.error('Failed to load clusters', e);
      this.clusters = [];
    } finally {
      this.loading = false;
    }
  }

  async get(id: string): Promise<Cluster | null> {
    try {
      const cluster = await invoke<Omit<Cluster, 'name' | 'server'> | null>('db_get_cluster', { id });
      return cluster ? parseCluster(cluster) : null;
    } catch (e) {
      console.error('Failed to get cluster', e);
      return null;
    }
  }

  async update(
    id: string,
    updates: {
      displayName?: string;
      icon?: string | null;
      description?: string | null;
      tags?: string[];
    },
  ) {
    try {
      await invoke('db_update_cluster', {
        id,
        displayName: updates.displayName,
        icon: updates.icon !== undefined ? updates.icon : undefined,
        description: updates.description !== undefined ? updates.description : undefined,
        tags: updates.tags,
      });
      await this.load();
    } catch (e) {
      console.error('Failed to update cluster', e);
      throw e;
    }
  }

  async remove(id: string) {
    try {
      await invoke('db_delete_cluster', { id });
      await this.load();
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
