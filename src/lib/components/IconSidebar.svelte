<script lang="ts">
  import { Home, MoreVertical, Plus, Settings as SettingsIcon } from 'lucide-svelte';
  import { page } from '$app/stores';

  import Menu from '$lib/components/ui/Menu.svelte';
  import type { MenuItem } from '$lib/components/ui/Menu.svelte';
  import { contextsStore } from '$lib/stores/contexts.svelte';

  let { onAddCluster } = $props<{
    onAddCluster: () => void;
  }>();

  let draggedIndex = $state<number | null>(null);
  let dragOverIndex = $state<number | null>(null);

  const pinnedContexts = $derived(contextsStore.pinnedContexts);

  function handleDragStart(event: DragEvent, index: number) {
    draggedIndex = index;
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = 'move';
    }
  }

  function handleDragOver(event: DragEvent, index: number) {
    event.preventDefault();
    dragOverIndex = index;
  }

  async function handleDragEnd() {
    if (draggedIndex !== null && dragOverIndex !== null && draggedIndex !== dragOverIndex) {
      const reordered = [...pinnedContexts];
      const [moved] = reordered.splice(draggedIndex, 1);
      reordered.splice(dragOverIndex, 0, moved);
      await contextsStore.reorderPinned(reordered.map((context) => context.id));
    }

    draggedIndex = null;
    dragOverIndex = null;
  }

  function handleDrop(event: DragEvent) {
    event.preventDefault();
    void handleDragEnd();
  }

  function getMenuItems(contextId: string): MenuItem[] {
    return [
      {
        label: 'Unpin from Sidebar',
        action: () => {
          void contextsStore.togglePinned(contextId);
        },
      },
      {
        label: 'Open',
        action: () => {
          window.location.href = `/context/${contextId}`;
        },
      },
      {
        label: 'Settings',
        action: () => {
          window.location.href = `/context/${contextId}/settings`;
        },
      },
    ];
  }

  function isActive(path: string): boolean {
    return $page.url.pathname === path || $page.url.pathname.startsWith(`${path}/`);
  }

  function getColorForString(str: string): string {
    let hash = 0;
    for (let i = 0; i < str.length; i += 1) {
      hash = str.charCodeAt(i) + ((hash << 5) - hash);
    }
    const hue = hash % 360;
    return `hsl(${hue}, 60%, 50%)`;
  }
</script>

<aside class="flex h-full w-16 flex-col border-r border-border-main bg-bg-sidebar">
  <a
    href="/"
    class="group relative flex h-16 items-center justify-center transition-colors hover:bg-bg-main"
    class:bg-bg-main={isActive('/')}
    title="Overview"
  >
    <Home size={28} class={isActive('/') && !$page.url.pathname.startsWith('/context') ? 'text-primary' : 'text-text-main'} />
  </a>

  <button
    onclick={onAddCluster}
    class="flex h-16 items-center justify-center transition-colors hover:bg-bg-main"
    title="Add Context"
  >
    <Plus size={28} />
  </button>

  <div class="mx-2 my-1 h-px bg-border-subtle"></div>

  <div class="flex-1 overflow-y-auto">
    {#each pinnedContexts as context, index (context.id)}
      <div
        class="group relative"
        draggable="true"
        ondragstart={(event) => handleDragStart(event, index)}
        ondragover={(event) => handleDragOver(event, index)}
        ondragend={() => void handleDragEnd()}
        ondrop={handleDrop}
        role="listitem"
      >
        {#if dragOverIndex === index && draggedIndex !== index}
          <div class="absolute inset-x-0 top-0 h-0.5 bg-primary"></div>
        {/if}

        <a
          href="/context/{context.id}"
          class="relative flex h-16 items-center justify-center transition-colors hover:bg-bg-main"
          class:bg-bg-main={isActive(`/context/${context.id}`)}
          title={context.display_name}
        >
          {#if context.icon}
            {#if context.icon.startsWith('http') || context.icon.startsWith('data:')}
              <img
                src={context.icon}
                alt={context.display_name}
                class="h-10 w-10 rounded-full border-2 object-contain"
                style:border-color={context.icon_ring_color || 'var(--color-primary)'}
              />
            {:else}
              <div
                class="flex h-10 w-10 items-center justify-center rounded-full border-2 text-3xl"
                style:border-color={context.icon_ring_color || 'var(--color-primary)'}
              >
                {context.icon}
              </div>
            {/if}
          {:else}
            <div
              class="flex h-10 w-10 items-center justify-center rounded-full border-2 text-sm font-bold text-white"
              style="background-color: {getColorForString(context.display_name)}; border-color: {context.icon_ring_color || 'var(--color-primary)'};"
            >
              {context.display_name.charAt(0).toUpperCase()}
            </div>
          {/if}

          {#if isActive(`/context/${context.id}`)}
            <div class="absolute left-0 top-2 bottom-2 w-0.5 bg-primary"></div>
          {/if}
        </a>

        <div class="absolute top-1 right-1 opacity-0 transition-opacity group-hover:opacity-100">
          <Menu items={getMenuItems(context.id)} align="left" />
        </div>
      </div>
    {/each}
  </div>

  <div class="mx-2 my-1 h-px bg-border-subtle"></div>

  <a
    href="/settings"
    class="relative flex h-16 items-center justify-center transition-colors hover:bg-bg-main"
    class:bg-bg-main={isActive('/settings')}
    title="Settings"
  >
    <SettingsIcon size={28} class={isActive('/settings') ? 'text-primary' : 'text-text-main'} />
  </a>
</aside>

<style>
  .overflow-y-auto::-webkit-scrollbar {
    width: 4px;
  }

  .overflow-y-auto::-webkit-scrollbar-track {
    background: transparent;
  }

  .overflow-y-auto::-webkit-scrollbar-thumb {
    background: var(--border-main);
    border-radius: 2px;
  }

  .overflow-y-auto::-webkit-scrollbar-thumb:hover {
    background: var(--text-muted);
  }
</style>
