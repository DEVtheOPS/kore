<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { save } from '@tauri-apps/plugin-dialog';
  import { homeDir, join } from '@tauri-apps/api/path';
  import { Download, FileDown, Loader2, X } from 'lucide-svelte';

  import Button from '$lib/components/ui/Button.svelte';
  import Checkbox from '$lib/components/ui/Checkbox.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import Select from '$lib/components/ui/Select.svelte';
  import { contextsStore } from '$lib/stores/contexts.svelte';

  interface ExportConflict {
    entry_type: string;
    name: string;
    action: 'skip' | 'overwrite';
  }

  interface ExportPreview {
    conflicts: ExportConflict[];
    cluster_count: number;
    user_count: number;
    context_count: number;
  }

  let { isOpen = $bindable(), onClose } = $props<{
    isOpen: boolean;
    onClose: () => void;
  }>();

  let selectedIds = $state<Set<string>>(new Set());
  let destination = $state('');
  let currentContextId = $state('');
  let preview = $state<ExportPreview | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let success = $state<string | null>(null);
  let initialized = $state(false);

  const currentContextOptions = $derived(
    ['(none)', ...contextsStore.contexts
      .filter((context) => selectedIds.has(context.id))
      .map((context) => context.id)],
  );

  function displayContextLabel(contextId: string): string {
    if (contextId === '(none)') {
      return '(none)';
    }

    const context = contextsStore.contexts.find((item) => item.id === contextId);
    return context ? `${context.display_name} (${context.cluster_display_name})` : contextId;
  }

  async function ensureInitialized() {
    if (initialized) {
      return;
    }

    await contextsStore.load();
    selectedIds = new Set(contextsStore.contexts.map((context) => context.id));

    try {
      const home = await homeDir();
      destination = await join(home, '.kube', 'config');
    } catch {
      destination = '';
    }

    currentContextId = '(none)';
    initialized = true;
  }

  $effect(() => {
    if (isOpen) {
      void ensureInitialized();
    }
  });

  function resetState() {
    preview = null;
    error = null;
    success = null;
  }

  function toggleContext(id: string, checked: boolean) {
    const next = new Set(selectedIds);

    if (checked) {
      next.add(id);
    } else {
      next.delete(id);
    }

    selectedIds = next;
    preview = null;

    if (currentContextId !== '(none)' && !next.has(currentContextId)) {
      currentContextId = '(none)';
    }
  }

  async function chooseDestination() {
    const selected = await save({
      title: 'Export kubeconfig',
      defaultPath: destination || undefined,
    });

    if (selected) {
      destination = selected;
      preview = null;
    }
  }

  function updateConflict(name: string, entryType: string, action: 'skip' | 'overwrite') {
    if (!preview) {
      return;
    }

    preview = {
      ...preview,
      conflicts: preview.conflicts.map((conflict) =>
        conflict.name === name && conflict.entry_type === entryType
          ? { ...conflict, action }
          : conflict,
      ),
    };
  }

  async function previewExport() {
    loading = true;
    error = null;
    success = null;

    try {
      preview = await invoke<ExportPreview>('export_preview', {
        contextIds: [...selectedIds],
        destination,
      });
    } catch (previewError) {
      error = previewError instanceof Error ? previewError.message : String(previewError);
    } finally {
      loading = false;
    }
  }

  async function exportSelected() {
    if (selectedIds.size === 0) {
      error = 'Select at least one context to export.';
      return;
    }

    if (!destination.trim()) {
      error = 'Choose a destination path first.';
      return;
    }

    if (!preview) {
      await previewExport();
      if (!preview) {
        return;
      }
    }

    loading = true;
    error = null;
    success = null;

    try {
      await invoke('export_kubeconfig', {
        contextIds: [...selectedIds],
        destination,
        currentContext: currentContextId === '(none)' ? null : selectedContextName,
        conflicts: preview.conflicts,
      });

      success = `Exported ${selectedIds.size} context${selectedIds.size === 1 ? '' : 's'} to ${destination}`;
      preview = null;
    } catch (exportError) {
      error = exportError instanceof Error ? exportError.message : String(exportError);
    } finally {
      loading = false;
    }
  }

  const selectedContextName = $derived.by(() => {
    if (currentContextId === '(none)') {
      return null;
    }

    const context = contextsStore.contexts.find((item) => item.id === currentContextId);
    if (!context) {
      return null;
    }

    try {
      const parsed = JSON.parse(context.config) as { name?: string };
      return parsed.name ?? context.display_name;
    } catch {
      return context.display_name;
    }
  });

  function handleClose() {
    resetState();
    onClose();
  }
</script>

{#if isOpen}
  <div
    class="fixed inset-0 z-[120] flex items-center justify-center bg-black/55 px-4"
    onclick={handleClose}
    onkeydown={(event) => event.key === 'Escape' && handleClose()}
    role="button"
    tabindex="-1"
  >
    <div
      class="flex max-h-[85vh] w-full max-w-4xl flex-col overflow-hidden rounded-card border border-border-main bg-bg-main shadow-2xl"
      onclick={(event) => event.stopPropagation()}
      onkeydown={(event) => event.stopPropagation()}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <div class="flex items-center justify-between border-b border-border-subtle px-5 py-4">
        <div>
          <h2 class="text-lg font-semibold">Export Kubeconfig</h2>
          <p class="text-sm text-text-muted">Choose which contexts to export and how to handle conflicts.</p>
        </div>
        <button class="rounded-md p-1 transition-colors hover:bg-bg-panel" onclick={handleClose}>
          <X size={18} />
        </button>
      </div>

      <div class="grid flex-1 gap-6 overflow-auto p-5 lg:grid-cols-[1.3fr_1fr]">
        <div class="space-y-5">
          <div>
            <div class="mb-2 flex items-center justify-between">
              <h3 class="font-medium">Contexts</h3>
              <span class="text-xs text-text-muted">{selectedIds.size} selected</span>
            </div>

            <div class="max-h-[22rem] space-y-2 overflow-auto rounded-card border border-border-subtle bg-bg-panel/40 p-3">
              {#if contextsStore.loading}
                <div class="flex items-center gap-2 text-sm text-text-muted">
                  <Loader2 size={16} class="animate-spin" />
                  Loading contexts...
                </div>
              {:else if contextsStore.contexts.length === 0}
                <div class="text-sm text-text-muted">No contexts available to export yet.</div>
              {:else}
                {#each contextsStore.contexts as context (context.id)}
                  <label class="flex cursor-pointer items-start gap-3 rounded-xl border border-border-subtle bg-bg-main/70 px-3 py-3 transition-colors hover:border-border-main">
                    <Checkbox checked={selectedIds.has(context.id)} onchange={(checked) => toggleContext(context.id, checked)} />
                    <div class="min-w-0 flex-1">
                      <div class="flex items-center gap-2">
                        <span class="font-medium">{context.display_name}</span>
                        {#if context.is_pinned}
                          <span class="rounded-full border border-border-main px-2 py-0.5 text-[10px] uppercase tracking-wide text-text-muted">Pinned</span>
                        {/if}
                      </div>
                      <div class="mt-1 text-xs text-text-muted">
                        <div>Cluster: {context.cluster_display_name}</div>
                        <div>User: {context.user_display_name}</div>
                      </div>
                    </div>
                  </label>
                {/each}
              {/if}
            </div>
          </div>

          {#if preview && preview.conflicts.length > 0}
            <div>
              <div class="mb-2 flex items-center gap-2">
                <FileDown size={16} class="text-primary" />
                <h3 class="font-medium">Conflict Resolution</h3>
              </div>
              <div class="space-y-2 rounded-card border border-border-subtle bg-bg-panel/40 p-3">
                {#each preview.conflicts as conflict (`${conflict.entry_type}-${conflict.name}`)}
                  <div class="flex flex-col gap-2 rounded-xl border border-border-subtle bg-bg-main/70 px-3 py-3 md:flex-row md:items-center md:justify-between">
                    <div>
                      <div class="font-medium">{conflict.name}</div>
                      <div class="text-xs uppercase tracking-wide text-text-muted">{conflict.entry_type} already exists at destination</div>
                    </div>
                    <div class="w-full md:w-40">
                      <Select
                        options={['skip', 'overwrite']}
                        value={conflict.action}
                        onselect={(value) => updateConflict(conflict.name, conflict.entry_type, value as 'skip' | 'overwrite')}
                      />
                    </div>
                  </div>
                {/each}
              </div>
            </div>
          {/if}
        </div>

        <div class="space-y-5">
          <div class="rounded-card border border-border-subtle bg-bg-panel/40 p-4">
            <h3 class="mb-3 font-medium">Destination</h3>
            <div class="space-y-3">
              <Input bind:value={destination} placeholder="Choose export path..." />
              <div class="flex gap-2">
                <Button variant="secondary" class="flex-1" onclick={chooseDestination}>Browse</Button>
                <Button
                  variant="outline"
                  onclick={async () => {
                    const home = await homeDir();
                    destination = await join(home, '.kube', 'config');
                    preview = null;
                  }}
                >
                  Use ~/.kube/config
                </Button>
              </div>
            </div>
          </div>

          <div class="rounded-card border border-border-subtle bg-bg-panel/40 p-4">
            <h3 class="mb-3 font-medium">Export Options</h3>
            <div class="space-y-3">
              <div>
                <label for="export-current-context" class="mb-2 block text-sm font-medium">current-context</label>
                <Select
                  id="export-current-context"
                  options={currentContextOptions.map(displayContextLabel)}
                  value={displayContextLabel(currentContextId || '(none)')}
                  onselect={(label) => {
                    const entry = currentContextOptions.find((id) => displayContextLabel(id) === label);
                    currentContextId = entry ?? '(none)';
                  }}
                />
              </div>

              {#if preview}
                <div class="rounded-xl border border-border-subtle bg-bg-main/70 p-3 text-sm text-text-muted">
                  <div>{preview.cluster_count} cluster{preview.cluster_count === 1 ? '' : 's'}</div>
                  <div>{preview.user_count} user{preview.user_count === 1 ? '' : 's'}</div>
                  <div>{preview.context_count} context{preview.context_count === 1 ? '' : 's'}</div>
                  <div>{preview.conflicts.length} conflict{preview.conflicts.length === 1 ? '' : 's'}</div>
                </div>
              {/if}
            </div>
          </div>

          {#if error}
            <div class="rounded-xl border border-status-danger/30 bg-status-danger/10 px-4 py-3 text-sm text-status-danger">
              {error}
            </div>
          {/if}

          {#if success}
            <div class="rounded-xl border border-primary/30 bg-primary/10 px-4 py-3 text-sm text-primary">
              {success}
            </div>
          {/if}
        </div>
      </div>

      <div class="flex items-center justify-between border-t border-border-subtle px-5 py-4">
        <div class="text-xs text-text-muted">
          Preview runs automatically before export so conflicts can be resolved cleanly.
        </div>
        <div class="flex gap-2">
          <Button variant="outline" onclick={handleClose}>Close</Button>
          <Button variant="secondary" onclick={previewExport} disabled={loading || selectedIds.size === 0}>Preview</Button>
          <Button onclick={exportSelected} disabled={loading || selectedIds.size === 0}>
            {#if loading}
              <Loader2 size={16} class="animate-spin" />
              Exporting...
            {:else}
              <Download size={16} />
              Export
            {/if}
          </Button>
        </div>
      </div>
    </div>
  </div>
{/if}
