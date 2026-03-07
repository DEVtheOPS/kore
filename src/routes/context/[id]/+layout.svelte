<script lang="ts">
  import { page } from '$app/stores';

  import BottomDrawer from '$lib/components/BottomDrawer.svelte';
  import ResourceSidebar from '$lib/components/ResourceSidebar.svelte';
  import { activeContextStore } from '$lib/stores/activeCluster.svelte';
  import { contextsStore, type ContextRecord } from '$lib/stores/contexts.svelte';
  import { headerStore } from '$lib/stores/header.svelte';

  let { children } = $props();

  let context = $state<ContextRecord | null>(null);
  let loading = $state(true);

  const contextId = $derived($page.params.id);

  $effect(() => {
    if (contextId) {
      void loadContext(contextId);
    }
  });

  async function loadContext(id: string | undefined) {
    if (!id) {
      loading = false;
      return;
    }

    loading = true;
    try {
      context = await contextsStore.get(id);
      if (context) {
        await activeContextStore.setContext(id);
        await contextsStore.updateLastAccessed(id);
      }
    } catch (error) {
      console.error('Failed to load context', error);
    } finally {
      loading = false;
    }
  }
</script>

{#if loading}
  <div class="flex h-full w-full items-center justify-center">
    <div class="text-text-muted">Loading context...</div>
  </div>
{:else if !context}
  <div class="flex h-full w-full items-center justify-center">
    <div class="space-y-2 text-center">
      <h2 class="text-xl font-semibold">Context Not Found</h2>
      <p class="text-text-muted">The context you're looking for doesn't exist.</p>
      <a href="/" class="text-primary hover:underline">Go to Overview</a>
    </div>
  </div>
{:else}
  <div class="flex h-full w-full overflow-hidden">
    <ResourceSidebar
      {context}
      namespaces={activeContextStore.namespaces}
      activeNamespace={activeContextStore.activeNamespace}
      onNamespaceChange={(namespace) => activeContextStore.setNamespace(namespace)}
    />

    <main class="flex h-full flex-1 flex-col overflow-hidden">
      <header class="flex h-14 items-center justify-between border-b border-border-subtle bg-bg-main px-6">
        <h2 class="text-lg font-semibold">{headerStore.title}</h2>
      </header>

      <div class="flex-1 overflow-auto bg-bg-panel p-6">
        {@render children()}
      </div>

      <BottomDrawer />
    </main>
  </div>
{/if}
