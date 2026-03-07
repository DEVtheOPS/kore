<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { headerStore } from "$lib/stores/header.svelte";
  import { activeClusterStore } from "$lib/stores/activeCluster.svelte";
  import DataTable, { type Column } from "$lib/components/ui/DataTable.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Badge from "$lib/components/ui/Badge.svelte";

  interface HelmAvailability {
    available: boolean;
    version?: string;
    message?: string;
  }

  interface HelmReleaseSummary {
    id: string;
    name: string;
    namespace: string;
    status: string;
    revision: string;
    chart: string;
    app_version: string;
    updated: string;
  }

  let data = $state<HelmReleaseSummary[]>([]);
  let loading = $state(false);
  let search = $state("");
  let error = $state<string | null>(null);
  let helm = $state<HelmAvailability | null>(null);

  const columns: Column[] = [
    { id: "name", label: "Release", sortable: true },
    { id: "namespace", label: "Namespace", sortable: true },
    { id: "status", label: "Status", sortable: true },
    { id: "revision", label: "Revision", sortable: true },
    { id: "chart", label: "Chart", sortable: true },
    { id: "app_version", label: "App Version", sortable: true },
    { id: "updated", label: "Updated", sortable: true },
  ];

  $effect(() => {
    headerStore.setTitle("Helm Releases");
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

      data = await invoke<HelmReleaseSummary[]>("cluster_list_helm_releases", {
        clusterId: activeClusterStore.clusterId,
      });
    } catch (e) {
      console.error("Failed to load Helm releases", e);
      error = `Failed to load Helm releases: ${e}`;
    } finally {
      loading = false;
    }
  }

  function getVariant(status: string): "success" | "warning" | "error" | "info" | "neutral" {
    const value = status.toLowerCase();
    if (value.includes("deployed")) return "success";
    if (value.includes("failed")) return "error";
    if (value.includes("pending")) return "warning";
    return "info";
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
    storageKey="helm-releases"
    emptyMessage="No Helm releases found."
  >
    {#snippet children({ column, value })}
      {#if column.id === "status"}
        <Badge variant={getVariant(value)}>{value}</Badge>
      {:else}
        {value}
      {/if}
    {/snippet}
  </DataTable>
</div>
