<script lang="ts">
  import { page } from "$app/stores";
  import { onMount } from "svelte";
  import ResourceSidebar from "$lib/components/ResourceSidebar.svelte";
  import BottomDrawer from "$lib/components/BottomDrawer.svelte";
  import { clustersStore } from "$lib/stores/clusters.svelte";
  import { activeClusterStore } from "$lib/stores/activeCluster.svelte";
  import { headerStore } from "$lib/stores/header.svelte";
  import type { Cluster } from "$lib/stores/clusters.svelte";

  let { children } = $props();

  let cluster = $state<Cluster | null>(null);
  let loading = $state(true);

  const clusterId = $derived($page.params.id);

  // Load cluster data when ID changes
  $effect(() => {
    if (clusterId) {
      loadCluster(clusterId);
    }
  });

  async function loadCluster(id: string | undefined) {
    if (!id) {
      loading = false;
      return;
    }
    
    loading = true;
    try {
      cluster = await clustersStore.get(id);
      
      if (cluster) {
        // Update active cluster store
        await activeClusterStore.setCluster(id);
        
        // Update last accessed timestamp
        await clustersStore.updateLastAccessed(id);
      }
    } catch (e) {
      console.error("Failed to load cluster", e);
    } finally {
      loading = false;
    }
  }

  function handleNamespaceChange(ns: string) {
    activeClusterStore.setNamespace(ns);
  }
</script>

{#if loading}
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
        {@render children()}
      </div>

      <!-- Global Bottom Drawer -->
      <BottomDrawer />
    </main>
  </div>
{/if}
