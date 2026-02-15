<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { headerStore } from "$lib/stores/header.svelte";
  import { activeClusterStore } from "$lib/stores/activeCluster.svelte";
  import { clustersStore } from "$lib/stores/clusters.svelte";

  const clusterId = $derived($page.params.id);
  const cluster = $derived(clustersStore.clusters.find((c) => c.id === clusterId));
  let podCount = $state<number | null>(null);
  let cpuUsage = $state<number | null>(null);
  let memoryUsageGi = $state<number | null>(null);

  $effect(() => {
    headerStore.setTitle("Cluster Overview");
  });

  onMount(async () => {
    if (!activeClusterStore.clusterId) return;
    try {
      const metrics = await invoke<any>("cluster_get_metrics", {
        clusterId: activeClusterStore.clusterId,
      });
      podCount = Math.round(metrics?.pods?.usage || 0);
      cpuUsage = metrics?.cpu?.usage || metrics?.cpu?.requests || 0;
      memoryUsageGi = (metrics?.memory?.usage || metrics?.memory?.requests || 0) / (1024 ** 3);
    } catch (e) {
      console.error("Failed to load cluster overview metrics", e);
    }
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
      <p class="text-2xl font-bold text-primary"><a href="/cluster/{clusterId}/nodes" class="hover:underline">View</a></p>
      <p class="text-xs text-text-muted mt-1">Open the full nodes inventory</p>
    </div>

    <div class="bg-bg-main p-4 rounded-lg border border-border-main">
      <h3 class="font-semibold mb-2">Pods</h3>
      <p class="text-2xl font-bold text-primary">{podCount ?? "-"}</p>
      <p class="text-xs text-text-muted mt-1">Running + pending workloads</p>
    </div>

    <div class="bg-bg-main p-4 rounded-lg border border-border-main">
      <h3 class="font-semibold mb-2">CPU / Memory</h3>
      <p class="text-2xl font-bold text-primary">
        {#if cpuUsage !== null && memoryUsageGi !== null}
          {cpuUsage.toFixed(2)} cores / {memoryUsageGi.toFixed(1)} GiB
        {:else}
          -
        {/if}
      </p>
      <p class="text-xs text-text-muted mt-1">Current requested/used resources</p>
    </div>
  </div>
</div>
