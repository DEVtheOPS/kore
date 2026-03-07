<script lang="ts">
  import { Fingerprint, LockKeyhole } from 'lucide-svelte';

  import Button from '$lib/components/ui/Button.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import { lockStore } from '$lib/stores/lock.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
</script>

<div class="fixed inset-0 z-[110] flex items-center justify-center bg-[radial-gradient(circle_at_top,_var(--color-primary)_0%,_transparent_38%),linear-gradient(180deg,_var(--bg-main),_var(--bg-surface))] px-6">
  <Card class="w-full max-w-md border border-border-main/70 bg-bg-panel/95 p-8 shadow-2xl backdrop-blur-sm">
    <div class="mb-6 flex items-center gap-4">
      <div class="flex h-14 w-14 items-center justify-center rounded-2xl border border-primary/30 bg-primary/10 text-primary">
        <LockKeyhole size={26} />
      </div>
      <div>
        <h2 class="text-xl font-semibold">Kore is locked</h2>
        <p class="text-sm text-text-muted">
          {#if settingsStore.value.requireBiometric}
            Unlock protection is enabled. Desktop biometric integration is planned next.
          {:else}
            Unlock your secure workspace to continue.
          {/if}
        </p>
      </div>
    </div>

    <div class="mb-6 rounded-xl border border-border-subtle bg-bg-main/60 p-4 text-sm text-text-muted">
      <div class="flex items-center gap-2 text-text-main">
        <Fingerprint size={16} class="text-primary" />
        <span>Encrypted config vault protection is active.</span>
      </div>
      <div class="mt-2">
        Lock mode: <span class="text-text-main">{settingsStore.value.lockMode}</span>
      </div>
    </div>

    {#if lockStore.error}
      <div class="mb-4 rounded-xl border border-status-danger/30 bg-status-danger/10 px-4 py-3 text-sm text-status-danger">
        {lockStore.error}
      </div>
    {/if}

    <Button class="w-full" size="lg" onclick={() => lockStore.unlock()} disabled={lockStore.unlocking}>
      {#if lockStore.unlocking}
        Unlocking...
      {:else if settingsStore.value.requireBiometric}
        Unlock workspace
      {:else}
        Unlock workspace
      {/if}
    </Button>
  </Card>
</div>
