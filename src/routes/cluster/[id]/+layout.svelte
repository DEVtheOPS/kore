<script lang="ts">
  import { page } from "$app/stores";
  import ResourceSidebar from "$lib/components/ResourceSidebar.svelte";
  import BottomDrawer from "$lib/components/BottomDrawer.svelte";
  import { activeContextStore } from "$lib/stores/activeCluster.svelte";
  import { contextsStore, type ContextRecord } from "$lib/stores/contexts.svelte";
  import { headerStore } from "$lib/stores/header.svelte";

  let { children } = $props();

  let context = $state<ContextRecord | null>(null);
  let loading = $state(true);

  const contextId = $derived($page.params.id);

  // Load cluster data when ID changes
  $effect(() => {
    if (contextId) {
      loadContext(contextId);
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
    } catch (e) {
      console.error("Failed to load context", e);
    } finally {
      loading = false;
    }
  }
</script>

{#if loading}
  <div class="flex items-center justify-center h-full w-full">
    <div class="text-text-muted">Loading context...</div>
  </div>
{:else if !context}
  <div class="flex items-center justify-center h-full w-full">
    <div class="text-center space-y-2">
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
      onNamespaceChange={(ns) => activeContextStore.setNamespace(ns)}
    />

    <!-- Main Content Area -->
    <main class="flex-1 flex flex-col h-full overflow-hidden">
      <!-- Header Bar -->
      <header
        class="h-14 border-b border-border-subtle flex items-center justify-between px-6 bg-bg-main"
      >
        <h2 class="font-semibold text-lg">{headerStore.title}</h2>
      </header>

      <!-- Content Area -->
      <div class="flex-1 overflow-auto p-6 bg-bg-panel">
        {@render children()}
      </div>

      <!-- Global Bottom Drawer -->
      <BottomDrawer />
    </main>
  </div>
{/if}
