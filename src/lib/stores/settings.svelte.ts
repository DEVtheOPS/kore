import { invoke } from '@tauri-apps/api/core';

export type Theme = 'kore' | 'kore-light' | 'rusty' | 'rusty-light' | 'dracula' | 'alucard';
export type CodeTheme = 'same-as-app' | 'kore' | 'kore-light' | 'rusty' | 'rusty-light' | 'dracula' | 'alucard';
export type LockMode = 'off' | 'on_open' | 'timeout';

interface BackendSettings {
  theme: Theme;
  code_theme: CodeTheme;
  refresh_interval: number;
  lock_mode: LockMode;
  lock_timeout_minutes: number;
  require_biometric: boolean;
}

export interface Settings {
  theme: Theme;
  codeTheme: CodeTheme;
  refreshInterval: number;
  lockMode: LockMode;
  lockTimeoutMinutes: number;
  requireBiometric: boolean;
}

const defaultSettings: Settings = {
  theme: 'kore',
  codeTheme: 'same-as-app',
  refreshInterval: 5000,
  lockMode: 'off',
  lockTimeoutMinutes: 15,
  requireBiometric: false,
};

function fromBackend(settings: BackendSettings): Settings {
  return {
    theme: settings.theme,
    codeTheme: settings.code_theme,
    refreshInterval: settings.refresh_interval,
    lockMode: settings.lock_mode,
    lockTimeoutMinutes: settings.lock_timeout_minutes,
    requireBiometric: settings.require_biometric,
  };
}

class SettingsStore {
  value = $state<Settings>(defaultSettings);
  loaded = $state(false);
  saving = $state(false);

  async load() {
    if (this.loaded) {
      return;
    }

    const settings = await invoke<BackendSettings>('settings_get');
    this.value = fromBackend(settings);
    this.loaded = true;
  }

  async update(updates: Partial<Settings>) {
    this.saving = true;

    try {
      const settings = await invoke<BackendSettings>('settings_update', {
        theme: updates.theme,
        codeTheme: updates.codeTheme,
        refreshInterval: updates.refreshInterval,
        lockMode: updates.lockMode,
        lockTimeoutMinutes: updates.lockTimeoutMinutes,
        requireBiometric: updates.requireBiometric,
      });

      this.value = fromBackend(settings);
      this.loaded = true;
    } finally {
      this.saving = false;
    }
  }

  setTheme(theme: Theme) {
    return this.update({ theme });
  }

  setCodeTheme(codeTheme: CodeTheme) {
    return this.update({ codeTheme });
  }

  setRefreshInterval(refreshInterval: number) {
    return this.update({ refreshInterval });
  }

  setLockMode(lockMode: LockMode) {
    return this.update({ lockMode });
  }

  setLockTimeoutMinutes(lockTimeoutMinutes: number) {
    return this.update({ lockTimeoutMinutes });
  }

  setRequireBiometric(requireBiometric: boolean) {
    return this.update({ requireBiometric });
  }

  get effectiveCodeTheme(): Theme {
    if (this.value.codeTheme === 'same-as-app') {
      return this.value.theme;
    }
    return this.value.codeTheme as Theme;
  }
}

export const settingsStore = new SettingsStore();
