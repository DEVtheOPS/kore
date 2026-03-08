import { invoke } from '@tauri-apps/api/core';

import { settingsStore } from './settings.svelte';

class LockStore {
  locked = $state(false);
  initializing = $state(true);
  unlocking = $state(false);
  error = $state<string | null>(null);
  biometricAvailable = $state(false);

  private timeoutId: number | null = null;
  private listenersAttached = false;

  async init() {
    await settingsStore.load();
    this.attachListeners();
    this.applySettings(true);
    this.initializing = false;
  }

  applySettings(initial = false) {
    this.clearTimeout();

    if (settingsStore.value.lockMode === 'off') {
      this.locked = false;
      this.error = null;
      return;
    }

    if (settingsStore.value.lockMode === 'on_open') {
      if (initial) {
        this.lock();
      }
      return;
    }

    if (settingsStore.value.lockMode === 'timeout') {
      if (initial) {
        this.locked = false;
      }
      this.bumpActivity();
    }
  }

  async lock() {
    this.clearTimeout();
    this.locked = true;
    this.error = null;
    // Notify the backend so it tracks the authoritative lock state.
    try {
      await invoke('security_lock');
    } catch {
      // Non-fatal — UI lock is still applied; backend call is belt-and-suspenders.
    }
  }

  async unlock() {
    this.unlocking = true;
    this.error = null;

    try {
      await invoke('security_verify_keychain_access');
      this.locked = false;
      this.bumpActivity();
    } catch (error) {
      this.error = error instanceof Error ? error.message : 'Failed to unlock Kore';
    } finally {
      this.unlocking = false;
    }
  }

  bumpActivity = () => {
    if (this.locked || settingsStore.value.lockMode !== 'timeout') {
      return;
    }

    this.clearTimeout();

    const ms = Math.max(1, settingsStore.value.lockTimeoutMinutes) * 60 * 1000;
    this.timeoutId = window.setTimeout(() => {
      this.lock();
    }, ms);
  };

  private attachListeners() {
    if (this.listenersAttached || typeof window === 'undefined') {
      return;
    }

    const events: Array<keyof WindowEventMap> = ['mousemove', 'mousedown', 'keydown', 'touchstart', 'focus'];
    for (const eventName of events) {
      window.addEventListener(eventName, this.bumpActivity, { passive: true });
    }

    document.addEventListener('visibilitychange', () => {
      if (document.visibilityState === 'visible') {
        this.applySettings();
      } else if (settingsStore.value.lockMode === 'timeout') {
        this.clearTimeout();
      }
    });

    this.listenersAttached = true;
  }

  private clearTimeout() {
    if (this.timeoutId !== null) {
      window.clearTimeout(this.timeoutId);
      this.timeoutId = null;
    }
  }
}

export const lockStore = new LockStore();
