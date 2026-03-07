import { invoke } from '@tauri-apps/api/core';

interface NamedContextConfig {
  name?: string;
  context?: {
    namespace?: string;
  };
}

export interface ContextRecord {
  id: string;
  display_name: string;
  cluster_id: string;
  user_id: string;
  config: string;
  icon?: string;
  icon_ring_color?: string;
  description?: string;
  tags: string;
  is_pinned: boolean;
  pin_order?: number;
  created_at: number;
  last_accessed: number;
  cluster_display_name: string;
  user_display_name: string;
  name: string;
  namespace?: string;
}

function parseContext(record: Omit<ContextRecord, 'name' | 'namespace'>): ContextRecord {
  try {
    const config = JSON.parse(record.config) as NamedContextConfig;
    return {
      ...record,
      name: config.name ?? record.display_name,
      namespace: config.context?.namespace,
    };
  } catch {
    return {
      ...record,
      name: record.display_name,
    };
  }
}

class ContextsStore {
  contexts = $state<ContextRecord[]>([]);
  loading = $state(false);

  constructor() {
    void this.load();
  }

  async load(clusterId?: string) {
    this.loading = true;
    try {
      const records = await invoke<Array<Omit<ContextRecord, 'name' | 'namespace'>>>('db_list_contexts', {
        clusterId,
      });
      this.contexts = records.map(parseContext);
    } catch (error) {
      console.error('Failed to load contexts', error);
      this.contexts = [];
    } finally {
      this.loading = false;
    }
  }

  async get(id: string): Promise<ContextRecord | null> {
    try {
      const record = await invoke<Omit<ContextRecord, 'name' | 'namespace'> | null>('db_get_context', { id });
      return record ? parseContext(record) : null;
    } catch (error) {
      console.error('Failed to get context', error);
      return null;
    }
  }

  async update(
    id: string,
    updates: {
      displayName?: string;
      icon?: string | null;
      iconRingColor?: string | null;
      description?: string | null;
      tags?: string[];
    },
  ) {
    await invoke('db_update_context', {
      id,
      displayName: updates.displayName,
      icon: updates.icon !== undefined ? updates.icon : undefined,
      iconRingColor: updates.iconRingColor !== undefined ? updates.iconRingColor : undefined,
      description: updates.description !== undefined ? updates.description : undefined,
      tags: updates.tags,
    });
    await this.load();
  }

  async updateLastAccessed(id: string) {
    await invoke('db_update_context_last_accessed', { id });
    const context = this.contexts.find((item) => item.id === id);
    if (context) {
      context.last_accessed = Date.now() / 1000;
    }
  }

  async remove(id: string) {
    await invoke('db_delete_context', { id });
    await this.load();
  }

  async togglePinned(id: string) {
    const context = this.contexts.find((item) => item.id === id);
    if (!context) {
      return;
    }

    if (context.is_pinned) {
      await invoke('db_unpin_context', { id });
    } else {
      await invoke('db_pin_context', { id });
    }

    await this.load();
  }

  async reorderPinned(orderedIds: string[]) {
    await invoke('db_reorder_pinned_contexts', { orderedIds });
    await this.load();
  }

  get pinnedContexts(): ContextRecord[] {
    return [...this.contexts]
      .filter((context) => context.is_pinned)
      .sort((a, b) => (a.pin_order ?? Number.MAX_SAFE_INTEGER) - (b.pin_order ?? Number.MAX_SAFE_INTEGER));
  }

  getTags(context: ContextRecord): string[] {
    try {
      return JSON.parse(context.tags);
    } catch {
      return [];
    }
  }
}

export const contextsStore = new ContextsStore();
