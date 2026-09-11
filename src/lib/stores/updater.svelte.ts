import { check, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { getVersion } from '@tauri-apps/api/app';
import { invoke } from '@tauri-apps/api/core';
import { settingsStore } from './settings.svelte';

export type UpdaterStatus =
  | 'idle'
  | 'checking'
  | 'up-to-date'
  | 'available'
  | 'downloading'
  | 'ready'
  | 'error';

export interface UpdateInfo {
  version: string;
  currentVersion: string;
  date?: string;
  body?: string;
}

const CHECK_INTERVAL_MS = 6 * 60 * 60 * 1000;
const STARTUP_DELAY_MS = 15_000;
const RELEASES_URL = 'https://github.com/DEVtheOPS/kore/releases/latest';

class UpdaterStore {
  status = $state<UpdaterStatus>('idle');
  info = $state<UpdateInfo | null>(null);
  error = $state<string | null>(null);
  currentVersion = $state<string>('');
  lastChecked = $state<Date | null>(null);
  downloaded = $state(0);
  contentLength = $state<number | null>(null);
  /** False when the running binary cannot replace itself (e.g. Linux .deb/.rpm). */
  canSelfUpdate = $state(true);
  releasesUrl = RELEASES_URL;

  private update: Update | null = null;
  private timer: ReturnType<typeof setInterval> | null = null;
  private startupTimer: ReturnType<typeof setTimeout> | null = null;

  get available(): boolean {
    return this.status === 'available' || this.status === 'downloading' || this.status === 'ready';
  }

  get progress(): number | null {
    if (!this.contentLength) return null;
    return Math.min(100, Math.round((this.downloaded / this.contentLength) * 100));
  }

  async init() {
    if (!isTauri()) return;
    try {
      this.currentVersion = await getVersion();
    } catch (e) {
      console.error('Failed to read app version', e);
    }
    try {
      this.canSelfUpdate = await invoke<boolean>('updater_can_self_update');
    } catch (e) {
      console.error('Failed to query self-update support', e);
    }
    this.scheduleAutoChecks();
  }

  /** (Re)start or stop the periodic check according to the user's setting. */
  scheduleAutoChecks() {
    this.stopAutoChecks();
    if (!settingsStore.value.autoCheckUpdates) return;
    this.startupTimer = setTimeout(() => this.check(true), STARTUP_DELAY_MS);
    this.timer = setInterval(() => this.check(true), CHECK_INTERVAL_MS);
  }

  stopAutoChecks() {
    if (this.startupTimer) clearTimeout(this.startupTimer);
    if (this.timer) clearInterval(this.timer);
    this.startupTimer = null;
    this.timer = null;
  }

  /**
   * Check the update endpoints. `silent` suppresses the transient
   * checking / up-to-date / error states so background checks never disturb
   * the UI; only an available update changes what the user sees.
   */
  async check(silent = false) {
    if (!isTauri()) return;
    if (this.status === 'downloading' || this.status === 'ready') return;

    if (!silent) {
      this.status = 'checking';
      this.error = null;
    }

    try {
      const update = await check();
      this.lastChecked = new Date();
      if (update) {
        this.update = update;
        this.info = {
          version: update.version,
          currentVersion: update.currentVersion,
          date: update.date,
          body: update.body,
        };
        this.status = 'available';
      } else {
        this.update = null;
        this.info = null;
        if (!silent) this.status = 'up-to-date';
      }
    } catch (e) {
      console.error('Update check failed', e);
      if (!silent) {
        this.error = `Update check failed: ${e}`;
        this.status = 'error';
      }
    }
  }

  async downloadAndInstall() {
    if (!this.update || !this.canSelfUpdate) return;

    this.status = 'downloading';
    this.error = null;
    this.downloaded = 0;
    this.contentLength = null;

    try {
      await this.update.downloadAndInstall((event) => {
        switch (event.event) {
          case 'Started':
            this.contentLength = event.data.contentLength ?? null;
            break;
          case 'Progress':
            this.downloaded += event.data.chunkLength;
            break;
          case 'Finished':
            if (this.contentLength) this.downloaded = this.contentLength;
            break;
        }
      });
      this.status = 'ready';
    } catch (e) {
      console.error('Update install failed', e);
      this.error = `Update failed: ${e}`;
      this.status = 'error';
    }
  }

  async restart() {
    if (this.status !== 'ready') return;
    try {
      await relaunch();
    } catch (e) {
      console.error('Relaunch failed', e);
      this.error = `Restart failed: ${e}. Please quit and reopen Kore.`;
    }
  }
}

function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export const updaterStore = new UpdaterStore();
