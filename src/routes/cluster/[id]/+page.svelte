<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { headerStore } from "$lib/stores/header.svelte";
  import { clustersStore } from "$lib/stores/clusters.svelte";

  const clusterId = $derived($page.params.id);
  const cluster = $derived(clustersStore.clusters.find((c) => c.id === clusterId));

  $effect(() => {
    headerStore.setTitle("Nodes");
  });
</script>

<div class="space-y-4">
  <div class="bg-bg-main p-6 rounded-lg border border-border-main">
    <h2 class="text-xl font-semibold mb-4">Cluster Dashboard</h2>
    <p class="text-text-muted">
      {#if cluster}
        Welcome to {cluster.name}. Select a resource type from the sidebar to get started.
      {:else}
        Loading cluster information...
      {/if}
    </p>
  </div>

  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
    <div class="bg-bg-main p-4 rounded-lg border border-border-main">
      <h3 class="font-semibold mb-2">Nodes</h3>
      <p class="text-2xl font-bold text-primary">-</p>
      <p class="text-xs text-text-muted mt-1">Coming soon</p>
    </div>

    <div class="bg-bg-main p-4 rounded-lg border border-border-main">
      <h3 class="font-semibold mb-2">Pods</h3>
      <p class="text-2xl font-bold text-primary">-</p>
      <p class="text-xs text-text-muted mt-1">Coming soon</p>
    </div>

    <div class="bg-bg-main p-4 rounded-lg border border-border-main">
      <h3 class="font-semibold mb-2">Services</h3>
      <p class="text-2xl font-bold text-primary">-</p>
      <p class="text-xs text-text-muted mt-1">Coming soon</p>
    </div>
  </div>
</div>
