export type Theme = 'kore' | 'kore-light' | 'rusty' | 'rusty-light' | 'dracula' | 'alucard';
export type CodeTheme = 'same-as-app' | 'kore' | 'kore-light' | 'rusty' | 'rusty-light' | 'dracula' | 'alucard';

export interface Settings {
  theme: Theme;
  codeTheme: CodeTheme;
  refreshInterval: number;
}

class SettingsStore {
  value = $state<Settings>({
    theme: 'kore',
    codeTheme: 'same-as-app',
    refreshInterval: 5000,
  });

  constructor() {
    if (typeof localStorage !== 'undefined') {
      const stored = localStorage.getItem('app-settings');
      if (stored) {
        try {
          const parsed = JSON.parse(stored);
          this.value = { ...this.value, ...parsed };
        } catch (e) {
          console.error("Failed to load settings", e);
        }
      }
    }
  }

  setTheme(theme: Theme) {
    this.value.theme = theme;
    this.save();
  }

  setCodeTheme(codeTheme: CodeTheme) {
    this.value.codeTheme = codeTheme;
    this.save();
  }

  setRefreshInterval(ms: number) {
    this.value.refreshInterval = ms;
    this.save();
  }

  get effectiveCodeTheme(): Theme {
    if (this.value.codeTheme === 'same-as-app') {
      return this.value.theme;
    }
    return this.value.codeTheme as Theme;
  }

  save() {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('app-settings', JSON.stringify(this.value));
    }
  }
}

export const settingsStore = new SettingsStore();
