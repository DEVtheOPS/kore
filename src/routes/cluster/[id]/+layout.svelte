<script lang="ts">
  import { page } from "$app/state";
  import ResourceSidebar from "$lib/components/ResourceSidebar.svelte";
  import BottomDrawer from "$lib/components/BottomDrawer.svelte";
  import { clustersStore } from "$lib/stores/clusters.svelte";
  import { activeClusterStore } from "$lib/stores/activeCluster.svelte";
  import { headerStore } from "$lib/stores/header.svelte";
  import type { Cluster } from "$lib/stores/clusters.svelte";
  import { setClusterMetadata } from "./clusterContext";

  let { children } = $props();

  let cluster = $state<Cluster | null>(null);
  let loadedClusterId = $state<string>();

  const clusterId = $derived(page.params.id);
  setClusterMetadata(() => cluster);

  $effect(() => {
    const id = clusterId;
    if (!id) return;
    const selection = new AbortController();
    loadedClusterId = undefined;
    cluster = null;
    void loadCluster(id, selection.signal);
    return () => selection.abort();
  });

  async function loadCluster(id: string, signal: AbortSignal) {
    const isCurrent = () => !signal.aborted && clusterId === id;
    if (!isCurrent()) return;
    try {
      const selected = await clustersStore.get(id);
      if (!isCurrent()) return;
      if (selected?.id === id) {
        await activeClusterStore.setCluster(id, signal);
        if (!isCurrent()) return;
        await clustersStore.updateLastAccessed(id);
        if (isCurrent()) cluster = selected;
      }
    } catch (e) {
      if (isCurrent()) console.error("Failed to load cluster", e);
    } finally {
      if (isCurrent()) loadedClusterId = id;
    }
  }

  function handleNamespaceChange(ns: string) {
    activeClusterStore.setNamespace(ns);
  }
</script>

{#if loadedClusterId !== clusterId}
  <div class="flex items-center justify-center h-full w-full">
    <div class="text-text-muted">Loading cluster...</div>
  </div>
{:else if !cluster}
  <div class="flex items-center justify-center h-full w-full">
    <div class="text-center space-y-2">
      <h2 class="text-xl font-semibold">Cluster Not Found</h2>
      <p class="text-text-muted">The cluster you're looking for doesn't exist.</p>
      <a href="/" class="text-primary hover:underline">Go to Overview</a>
    </div>
  </div>
{:else}
  <div class="flex h-full w-full overflow-hidden">
    <!-- Resource Sidebar -->
    <ResourceSidebar
      {cluster}
      namespaces={activeClusterStore.namespaces}
      activeNamespace={activeClusterStore.activeNamespace}
      onNamespaceChange={handleNamespaceChange}
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
        {#key clusterId}
          {@render children()}
        {/key}
      </div>

      <!-- Global Bottom Drawer -->
      <BottomDrawer />
    </main>
  </div>
{/if}
