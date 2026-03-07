import { invoke } from '@tauri-apps/api/core';

interface NamedUserConfig {
  name?: string;
}

export interface UserRecord {
  id: string;
  display_name: string;
  config: string;
  created_at: number;
  updated_at: number;
  context_count: number;
  name: string;
}

function parseUser(record: Omit<UserRecord, 'name'>): UserRecord {
  try {
    const config = JSON.parse(record.config) as NamedUserConfig;
    return {
      ...record,
      name: config.name ?? record.display_name,
    };
  } catch {
    return {
      ...record,
      name: record.display_name,
    };
  }
}

class UsersStore {
  users = $state<UserRecord[]>([]);
  loading = $state(false);

  constructor() {
    void this.load();
  }

  async load() {
    this.loading = true;
    try {
      const users = await invoke<Array<Omit<UserRecord, 'name'>>>('db_list_users');
      this.users = users.map(parseUser);
    } catch (e) {
      console.error('Failed to load users', e);
      this.users = [];
    } finally {
      this.loading = false;
    }
  }

  async remove(id: string, force = false) {
    await invoke('db_delete_user', { id, force });
    await this.load();
  }

  async update(id: string, displayName: string) {
    await invoke('db_update_user', { id, displayName });
    await this.load();
  }
}

export const usersStore = new UsersStore();
