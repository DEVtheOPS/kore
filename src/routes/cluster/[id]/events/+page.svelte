<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { headerStore } from "$lib/stores/header.svelte";
  import { activeClusterStore } from "$lib/stores/activeCluster.svelte";
  import DataTable, { type Column } from "$lib/components/ui/DataTable.svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";

  interface ClusterEventSummary {
    id: string;
    name: string;
    namespace: string;
    age: string;
    status: string;
    event_type: string;
    reason: string;
    message: string;
    object: string;
    count: number;
    created_at: number;
  }

  let data = $state<ClusterEventSummary[]>([]);
  let loading = $state(false);
  let search = $state("");
  let error = $state<string | null>(null);
  let includeNormal = $state(true);

  const columns: Column[] = [
    { id: "event_type", label: "Type", sortable: true },
    { id: "reason", label: "Reason", sortable: true },
    { id: "object", label: "Object", sortable: true },
    { id: "namespace", label: "Namespace", sortable: true },
    { id: "message", label: "Message", sortable: true },
    { id: "count", label: "Count", sortable: true },
    { id: "age", label: "Age", sortable: true, sortKey: "created_at" },
  ];

  $effect(() => {
    headerStore.setTitle("Events");
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
      data = await invoke<ClusterEventSummary[]>("cluster_list_events", {
        clusterId: activeClusterStore.clusterId,
        namespace: activeClusterStore.activeNamespace === "all" ? null : activeClusterStore.activeNamespace,
        includeNormal,
      });
    } catch (e) {
      console.error("Failed to load events", e);
      error = "Failed to load events.";
    } finally {
      loading = false;
    }
  }

  function getEventVariant(type: string): "success" | "warning" | "error" | "info" | "neutral" {
    if (type === "Warning") return "error";
    if (type === "Normal") return "info";
    return "neutral";
  }
</script>

<div class="h-full">
  {#if error}
    <div class="mb-4 p-3 bg-error/10 text-error rounded-md border border-error/20 flex items-center justify-between gap-3">
      <span>{error}</span>
      <Button variant="outline" size="sm" onclick={loadData}>Retry</Button>
    </div>
  {/if}

  <div class="mb-4 flex items-center justify-end gap-2">
    <Button
      variant={includeNormal ? "secondary" : "outline"}
      size="sm"
      onclick={() => {
        includeNormal = !includeNormal;
        loadData();
      }}
    >
      {includeNormal ? "Showing All Events" : "Showing Warnings Only"}
    </Button>
  </div>

  <DataTable
    {data}
    {columns}
    bind:search
    {loading}
    onRefresh={loadData}
    emptyMessage="No events found for current filters."
    storageKey="cluster-events"
  >
    {#snippet children({ column, value })}
      {#if column.id === "event_type"}
        <Badge variant={getEventVariant(value)}>{value}</Badge>
      {:else if column.id === "message"}
        <span class="truncate block max-w-[450px]" title={value}>{value}</span>
      {:else}
        {value}
      {/if}
    {/snippet}
  </DataTable>
</div>
