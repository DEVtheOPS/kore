<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { headerStore } from "$lib/stores/header.svelte";
  import { activeClusterStore } from "$lib/stores/activeCluster.svelte";
  import DataTable, { type Column } from "$lib/components/ui/DataTable.svelte";
  import Button from "$lib/components/ui/Button.svelte";

  interface HelmAvailability {
    available: boolean;
    version?: string;
    message?: string;
  }

  interface HelmChartSummary {
    id: string;
    name: string;
    chart: string;
    version: string;
    app_version: string;
    description: string;
    status: string;
  }

  let data = $state<HelmChartSummary[]>([]);
  let loading = $state(false);
  let search = $state("");
  let error = $state<string | null>(null);
  let helm = $state<HelmAvailability | null>(null);

  const columns: Column[] = [
    { id: "chart", label: "Chart", sortable: true },
    { id: "version", label: "Version", sortable: true },
    { id: "app_version", label: "App Version", sortable: true },
    { id: "description", label: "Description", sortable: true },
  ];

  $effect(() => {
    headerStore.setTitle("Helm Charts");
  });

  $effect(() => {
    if (activeClusterStore.clusterId) {
      loadData();
    }
  });

  async function loadData() {
    if (!activeClusterStore.clusterId) return;
    loading = true;
    error = null;

    try {
      helm = await invoke<HelmAvailability>("cluster_check_helm_available");
      if (!helm.available) {
        data = [];
        error = helm.message || "Helm CLI is not available.";
        return;
      }

      data = await invoke<HelmChartSummary[]>("cluster_list_helm_charts", {
        clusterId: activeClusterStore.clusterId,
      });
    } catch (e) {
      console.error("Failed to load Helm charts", e);
      error = `Failed to load Helm charts: ${e}`;
    } finally {
      loading = false;
    }
  }
</script>

<div class="h-full space-y-4">
  {#if helm?.available}
    <div class="text-xs text-text-muted">Helm: {helm.version}</div>
  {/if}

  {#if error}
    <div class="p-3 bg-error/10 text-error rounded-md border border-error/20 flex items-center justify-between gap-3">
      <span>{error}</span>
      <Button variant="outline" size="sm" onclick={loadData}>Retry</Button>
    </div>
  {/if}

  <DataTable
    {data}
    {columns}
    bind:search
    {loading}
    onRefresh={loadData}
    storageKey="helm-charts"
    emptyMessage="No Helm charts found in configured repositories."
  >
    {#snippet children({ column, value })}
      {#if column.id === "description"}
        <span class="truncate block max-w-[500px]" title={value}>{value}</span>
      {:else}
        {value}
      {/if}
    {/snippet}
  </DataTable>
</div>
