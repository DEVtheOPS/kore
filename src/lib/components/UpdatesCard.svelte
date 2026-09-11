<script lang="ts">
  import Card from '$lib/components/ui/Card.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Checkbox from '$lib/components/ui/Checkbox.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import { Download, RefreshCw, RotateCw, ExternalLink } from 'lucide-svelte';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { updaterStore } from '$lib/stores/updater.svelte';

  function toggleAutoCheck(enabled: boolean) {
    settingsStore.setAutoCheckUpdates(enabled);
    updaterStore.scheduleAutoChecks();
  }

  function formatChecked(d: Date | null): string {
    return d ? d.toLocaleString() : 'never';
  }

  async function openReleases() {
    try {
      await openUrl(updaterStore.releasesUrl);
    } catch (e) {
      console.error('Failed to open releases page', e);
    }
  }
</script>

<Card class="p-6">
  <div class="flex items-start gap-4">
    <div class="p-3 bg-bg-popover rounded-full">
      <Download size={24} class="text-primary" />
    </div>
    <div class="flex-1 space-y-4">
      <div>
        <h3 class="font-bold text-lg mb-1">Updates</h3>
        <p class="text-text-muted text-sm">
          Kore {updaterStore.currentVersion ? `v${updaterStore.currentVersion}` : ''}
          <span class="mx-1">·</span>
          Last checked: {formatChecked(updaterStore.lastChecked)}
        </p>
      </div>

      <label class="flex items-center gap-2 text-sm cursor-pointer">
        <Checkbox checked={settingsStore.value.autoCheckUpdates} onchange={toggleAutoCheck} />
        Check for updates automatically (on launch and every 6 hours)
      </label>

      {#if updaterStore.error}
        <div class="p-3 bg-error/10 text-error rounded-md border border-error/20 text-sm">{updaterStore.error}</div>
      {/if}

      {#if updaterStore.available && updaterStore.info}
        <div class="p-4 bg-bg-panel rounded-md border border-border-subtle space-y-3">
          <div class="flex items-center gap-2">
            <Badge variant="success">v{updaterStore.info.version} available</Badge>
            {#if updaterStore.info.date}
              <span class="text-xs text-text-muted">{new Date(updaterStore.info.date).toLocaleDateString()}</span>
            {/if}
          </div>

          {#if updaterStore.info.body}
            <pre class="text-xs text-text-muted whitespace-pre-wrap max-h-48 overflow-auto font-sans">{updaterStore.info.body}</pre>
          {/if}

          {#if updaterStore.status === 'downloading'}
            <div class="space-y-1">
              <div class="h-1.5 rounded-full bg-bg-main border border-border-subtle overflow-hidden">
                <div class="h-full bg-primary transition-all" style="width: {updaterStore.progress ?? 0}%"></div>
              </div>
              <div class="text-xs text-text-muted">
                Downloading{updaterStore.progress != null ? ` ${updaterStore.progress}%` : '...'}
              </div>
            </div>
          {:else if updaterStore.status === 'ready'}
            <div class="flex items-center gap-3">
              <span class="text-sm">Update installed. Restart to finish.</span>
              <Button size="sm" onclick={() => updaterStore.restart()}>
                <RotateCw size={14} />
                Restart Now
              </Button>
            </div>
          {:else if updaterStore.canSelfUpdate}
            <Button size="sm" onclick={() => updaterStore.downloadAndInstall()}>
              <Download size={14} />
              Install & Restart
            </Button>
          {:else}
            <div class="space-y-2">
              <p class="text-sm text-text-muted">
                This install was made with a system package (.deb/.rpm), so it can't update itself. Download the new
                package from GitHub Releases and install it with your package manager.
              </p>
              <Button size="sm" variant="outline" onclick={openReleases}>
                <ExternalLink size={14} />
                Open Releases
              </Button>
            </div>
          {/if}
        </div>
      {:else}
        <div class="flex items-center gap-3">
          <Button
            size="sm"
            variant="outline"
            onclick={() => updaterStore.check()}
            disabled={updaterStore.status === 'checking'}
          >
            <RefreshCw size={14} class={updaterStore.status === 'checking' ? 'animate-spin' : ''} />
            {updaterStore.status === 'checking' ? 'Checking...' : 'Check Now'}
          </Button>
          {#if updaterStore.status === 'up-to-date'}
            <span class="text-sm text-text-muted">You're on the latest version.</span>
          {/if}
        </div>
      {/if}
    </div>
  </div>
</Card>
