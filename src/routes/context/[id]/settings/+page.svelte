<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { confirm, open } from '@tauri-apps/plugin-dialog';

  import Button from '$lib/components/ui/Button.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import { contextsStore, type ContextRecord } from '$lib/stores/contexts.svelte';
  import { headerStore } from '$lib/stores/header.svelte';
  import { Image as ImageIcon, Save, Trash2, Upload } from 'lucide-svelte';

  const contextId = $derived($page.params.id);

  let context = $state<ContextRecord | null>(null);
  let loading = $state(true);
  let saving = $state(false);

  let displayName = $state('');
  let icon = $state('');
  let iconRingColor = $state('#3b82f6');
  let description = $state('');
  let tags = $state<string[]>([]);
  let tagInput = $state('');

  const kubeconfigName = $derived.by(() => {
    if (!context) return '-';
    try {
      return (JSON.parse(context.config) as { name?: string }).name ?? context.display_name;
    } catch {
      return context.display_name;
    }
  });

  $effect(() => {
    headerStore.setTitle('Context Settings');
  });

  $effect(() => {
    if (contextId) {
      void loadContext();
    }
  });

  async function loadContext() {
    if (!contextId) {
      loading = false;
      return;
    }

    loading = true;
    try {
      context = await contextsStore.get(contextId);
      if (context) {
        displayName = context.display_name;
        icon = context.icon || '';
        iconRingColor = context.icon_ring_color || '#3b82f6';
        description = context.description || '';
        tags = contextsStore.getTags(context);
      }
    } finally {
      loading = false;
    }
  }

  async function handleSave() {
    if (!context) return;
    saving = true;
    try {
      await contextsStore.update(context.id, {
        displayName,
        icon: icon || null,
        iconRingColor: iconRingColor || null,
        description: description || null,
        tags,
      });
      await loadContext();
    } finally {
      saving = false;
    }
  }

  async function handleDelete() {
    if (!context) return;
    const confirmed = await confirm(`Delete context "${context.display_name}"?`, {
      title: 'Delete Context',
      kind: 'warning',
    });
    if (!confirmed) return;
    await contextsStore.remove(context.id);
    goto('/');
  }

  function handleAddTag() {
    const value = tagInput.trim();
    if (value && !tags.includes(value)) {
      tags = [...tags, value];
      tagInput = '';
    }
  }

  async function handleIconFileSelect() {
    const selected = await open({
      multiple: false,
      title: 'Select Context Icon',
      filters: [{
        name: 'Images',
        extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp', 'bmp', 'ico'],
      }],
    });

    if (!selected) return;
    icon = await invoke<string>('process_icon_file', { path: selected });
  }
</script>

{#if loading}
  <div class="flex h-full items-center justify-center text-text-muted">Loading context settings...</div>
{:else if !context}
  <div class="flex h-full items-center justify-center">
    <div class="space-y-2 text-center">
      <h2 class="text-xl font-semibold">Context Not Found</h2>
      <a href="/" class="text-primary hover:underline">Go to Overview</a>
    </div>
  </div>
{:else}
  <div class="max-w-3xl space-y-6">
    <Card>
      <div class="space-y-4 p-6">
        <h2 class="text-lg font-semibold">General Settings</h2>

        <div>
          <label for="display-name" class="mb-2 block text-sm font-medium">Context Name</label>
          <Input id="display-name" bind:value={displayName} class="w-full" />
        </div>

        <div>
          <label for="context-icon" class="mb-2 block text-sm font-medium">Icon</label>
          <div class="flex items-start gap-3">
            <div class="flex h-16 w-16 items-center justify-center rounded-full border-2 bg-bg-panel overflow-hidden" style:border-color={iconRingColor}>
              {#if icon}
                {#if icon.startsWith('data:image') || icon.startsWith('http')}
                  <img src={icon} alt="Icon" class="h-full w-full object-contain" />
                {:else}
                  <span class="text-3xl">{icon}</span>
                {/if}
              {:else}
                <ImageIcon size={24} class="text-text-muted" />
              {/if}
            </div>
            <div class="flex-1 space-y-2">
              <Button onclick={handleIconFileSelect} variant="outline">
                <Upload size={16} />
                Upload Image
              </Button>
              <Input id="context-icon" bind:value={icon} placeholder="🌐 or paste image URL" class="w-full" />
              <div>
                <label for="ring-color" class="mb-2 block text-sm font-medium">Ring Color</label>
                <input id="ring-color" type="color" bind:value={iconRingColor} class="h-10 w-14 rounded border border-border-main bg-bg-panel p-1" />
              </div>
            </div>
          </div>
        </div>

        <div>
          <label for="description" class="mb-2 block text-sm font-medium">Description</label>
          <Input id="description" bind:value={description} class="w-full" />
        </div>

        <div>
          <label for="tag-input" class="mb-2 block text-sm font-medium">Tags</label>
          <div class="flex gap-2">
            <Input id="tag-input" bind:value={tagInput} onkeydown={(event) => event.key === 'Enter' && (event.preventDefault(), handleAddTag())} class="flex-1" />
            <Button variant="outline" onclick={handleAddTag}>Add</Button>
          </div>
          <div class="mt-2 flex flex-wrap gap-2">
            {#each tags as tag}
              <button class="rounded border border-border-main px-2 py-1 text-xs" onclick={() => (tags = tags.filter((item) => item !== tag))}>
                {tag} ×
              </button>
            {/each}
          </div>
        </div>
      </div>
    </Card>

    <Card>
      <div class="space-y-3 p-6 text-sm">
        <h2 class="text-lg font-semibold">Configuration</h2>
        <div><span class="text-text-muted">Kubeconfig Context:</span> <span class="font-mono">{kubeconfigName}</span></div>
        <div><span class="text-text-muted">Cluster:</span> {context.cluster_display_name}</div>
        <div><span class="text-text-muted">User:</span> {context.user_display_name}</div>
        <div><span class="text-text-muted">Namespace:</span> {context.namespace || '-'}</div>
        <div><span class="text-text-muted">Context ID:</span> <span class="font-mono">{context.id}</span></div>
      </div>
    </Card>

    <div class="flex items-center justify-between">
      <Button variant="outline" onclick={handleDelete}>
        <Trash2 size={16} />
        Delete Context
      </Button>
      <Button onclick={handleSave} disabled={saving}>
        <Save size={16} />
        {saving ? 'Saving...' : 'Save Changes'}
      </Button>
    </div>
  </div>
{/if}
