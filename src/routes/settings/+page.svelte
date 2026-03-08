<script lang="ts">
  import Card from '$lib/components/ui/Card.svelte';
  import ExportKubeconfigModal from '$lib/components/ExportKubeconfigModal.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import Select from '$lib/components/ui/Select.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import { Download, Palette, Shield } from 'lucide-svelte';
  import { lockStore } from '$lib/stores/lock.svelte';
  import { settingsStore, type Theme, type CodeTheme, type LockMode } from '$lib/stores/settings.svelte';

  const themeOptions: Theme[] = ['kore', 'kore-light', 'rusty', 'rusty-light', 'dracula', 'alucard'];
  const codeThemeOptions: CodeTheme[] = ['same-as-app', 'kore', 'kore-light', 'rusty', 'rusty-light', 'dracula', 'alucard'];
  const lockModeOptions: LockMode[] = ['off', 'on_open', 'timeout'];
  let exportModalOpen = $state(false);

  async function updateLockTimeout(value: string) {
    const minutes = Number.parseInt(value, 10);

    if (Number.isNaN(minutes)) {
      return;
    }

    await settingsStore.setLockTimeoutMinutes(minutes);
    lockStore.applySettings();
  }
</script>

<div class="max-w-3xl space-y-6 p-6 overflow-y-auto h-full">
  <div class="space-y-2">
    <h1 class="text-2xl font-bold">Settings</h1>
    <p class="text-text-muted">
      Manage your application preferences and settings.
    </p>
  </div>

  <Card class="p-6">
    <div class="flex items-start gap-4">
      <div class="p-3 bg-bg-popover rounded-full">
        <Palette size={24} class="text-primary" />
      </div>
      <div class="flex-1">
        <h3 class="font-bold text-lg mb-1">Appearance</h3>
        <p class="text-text-muted text-sm mb-4">
          Customize the look and feel of the application.
        </p>
        <div class="space-y-4">
          <div>
            <label for="theme-select" class="block text-sm font-medium mb-2">App Theme</label>
            <div class="w-64">
              <Select
                id="theme-select"
                options={themeOptions}
                value={settingsStore.value.theme}
                onselect={(val) => settingsStore.setTheme(val as Theme)}
                placeholder="Select Theme"
              />
            </div>
          </div>

          <div>
            <label for="code-theme-select" class="block text-sm font-medium mb-2">Code Editor Theme</label>
            <p class="text-text-muted text-xs mb-2">
              Choose a different theme for code blocks and YAML editors
            </p>
            <div class="w-64">
              <Select
                id="code-theme-select"
                options={codeThemeOptions}
                value={settingsStore.value.codeTheme}
                onselect={(val) => settingsStore.setCodeTheme(val as CodeTheme)}
                placeholder="Select Code Theme"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  </Card>

  <Card class="p-6">
    <div class="flex items-start gap-4">
      <div class="rounded-full bg-bg-popover p-3">
        <Shield size={24} class="text-primary" />
      </div>
      <div class="flex-1">
        <h3 class="mb-1 text-lg font-bold">Security</h3>
        <p class="mb-4 text-sm text-text-muted">
          Control when Kore locks and whether it should ask for system biometric or device credential auth.
        </p>

        <div class="space-y-4">
          <div>
            <label for="lock-mode-select" class="mb-2 block text-sm font-medium">Lock Mode</label>
            <div class="w-64">
              <Select
                id="lock-mode-select"
                options={lockModeOptions}
                value={settingsStore.value.lockMode}
                onselect={async (val) => {
                  await settingsStore.setLockMode(val as LockMode);
                  lockStore.applySettings();
                }}
                placeholder="Select Lock Mode"
              />
            </div>
          </div>

          {#if settingsStore.value.lockMode === 'timeout'}
            <div>
              <label for="lock-timeout" class="mb-2 block text-sm font-medium">Timeout (minutes)</label>
              <div class="w-40">
                <Input
                  id="lock-timeout"
                  type="number"
                  min="1"
                  value={String(settingsStore.value.lockTimeoutMinutes)}
                  oninput={(event) => updateLockTimeout((event.currentTarget as HTMLInputElement).value)}
                />
              </div>
            </div>
          {/if}

          <label class="flex items-start gap-3 rounded-xl border border-border-subtle bg-bg-main/30 px-4 py-3 opacity-60 cursor-not-allowed">
            <input
              type="checkbox"
              disabled
              class="mt-0.5 h-4 w-4 rounded border-border-main bg-bg-panel text-primary cursor-not-allowed"
            />
            <span>
              <span class="block text-sm font-medium">Require system auth on unlock <span class="text-xs text-text-muted font-normal">(coming soon)</span></span>
              <span class="block text-xs text-text-muted">
                Desktop biometric integration (Touch ID / Windows Hello) is not yet implemented.
                The checkbox is disabled to prevent a false sense of security.
              </span>
            </span>
          </label>

          {#if settingsStore.value.lockMode !== 'off'}
            <div class="flex items-center gap-3">
              <Button variant="outline" onclick={() => lockStore.lock()}>
                Lock now
              </Button>
              <span class="text-xs text-text-muted">
                {#if lockStore.biometricAvailable}
                  Biometric auth is available on this device.
                {:else}
                  Biometric auth is not available; Kore can still rely on OS keychain access.
                {/if}
              </span>
            </div>
          {/if}
        </div>
      </div>
    </div>
  </Card>

  <Card class="p-6">
    <div class="flex items-start gap-4">
      <div class="rounded-full bg-bg-popover p-3">
        <Download size={24} class="text-primary" />
      </div>
      <div class="flex-1">
        <h3 class="mb-1 text-lg font-bold">Export</h3>
        <p class="mb-4 text-sm text-text-muted">
          Build a kubeconfig from your encrypted vault, choose the exported current-context, and resolve collisions before writing.
        </p>

        <div class="flex items-center gap-3">
          <Button onclick={() => (exportModalOpen = true)}>
            <Download size={16} />
            Open Export Wizard
          </Button>
          <span class="text-xs text-text-muted">Supports exporting to <code>~/.kube/config</code> or any custom path.</span>
        </div>
      </div>
    </div>
  </Card>
</div>

<ExportKubeconfigModal bind:isOpen={exportModalOpen} onClose={() => (exportModalOpen = false)} />
