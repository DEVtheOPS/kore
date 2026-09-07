<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { confirm } from '@tauri-apps/plugin-dialog';
  import { onMount } from 'svelte';
  import { RefreshCw, Save } from 'lucide-svelte';
  import CodeEditor from '$lib/components/ui/CodeEditor.svelte';
  import Button from '$lib/components/ui/Button.svelte';

  export interface EditTabData {
    clusterId: string;
    /** kubectl resource kind, e.g. "pod", "deployment" */
    kind: string;
    name: string;
    /** Omit / null for cluster-scoped resources */
    namespace?: string | null;
    /** Called after a successful apply so callers can refresh their lists */
    onApplied?: () => void;
  }

  let { data }: { data: EditTabData } = $props();

  let yamlContent = $state('');
  let loading = $state(false);
  let applying = $state(false);
  let error = $state<string | null>(null);
  let notice = $state<string | null>(null);

  async function loadYaml() {
    loading = true;
    error = null;
    notice = null;
    try {
      yamlContent = await invoke<string>('cluster_get_resource_yaml', {
        clusterId: data.clusterId,
        kind: data.kind,
        name: data.name,
        namespace: data.namespace ?? null,
      });
    } catch (e) {
      console.error('Failed to load YAML', e);
      error = `Failed to load YAML: ${e}`;
    } finally {
      loading = false;
    }
  }

  async function applyYaml() {
    if (!yamlContent.trim()) return;

    const confirmed = await confirm(`Apply changes to ${data.kind}/${data.name}?`, {
      title: 'Apply Resource YAML',
      kind: 'warning',
    });
    if (!confirmed) return;

    applying = true;
    error = null;
    notice = null;
    try {
      const result = await invoke<string>('cluster_apply_resource_yaml', {
        clusterId: data.clusterId,
        yaml: yamlContent,
      });
      notice = result || 'Applied.';
      data.onApplied?.();
    } catch (e) {
      console.error('Failed to apply YAML', e);
      error = `Failed to apply YAML: ${e}`;
    } finally {
      applying = false;
    }
  }

  onMount(loadYaml);
</script>

<div class="flex flex-col h-full">
  <!-- Toolbar -->
  <div class="flex items-center justify-between px-4 py-2 bg-bg-panel gap-3">
    <div class="flex items-center gap-2 min-w-0">
      <span class="text-sm text-text-muted truncate">
        {data.kind}/{data.name}{data.namespace ? ` (${data.namespace})` : ''}
      </span>
      {#if notice}
        <span class="text-xs text-success truncate">{notice}</span>
      {/if}
      {#if error}
        <span class="text-xs text-error truncate" title={error}>{error}</span>
      {/if}
    </div>
    <div class="flex items-center gap-2 shrink-0">
      <Button variant="ghost" size="sm" onclick={loadYaml} disabled={loading || applying} title="Reload from cluster">
        <RefreshCw size={14} class={loading ? 'animate-spin' : ''} />
      </Button>
      <Button size="sm" onclick={applyYaml} disabled={loading || applying || !yamlContent.trim()}>
        <Save size={14} />
        {applying ? 'Applying...' : 'Apply'}
      </Button>
    </div>
  </div>

  <!-- Editor -->
  <div class="flex-1 min-h-0 bg-bg-main">
    {#if loading}
      <div class="p-4 text-sm text-text-muted">Loading YAML...</div>
    {:else}
      <CodeEditor bind:value={yamlContent} />
    {/if}
  </div>
</div>
