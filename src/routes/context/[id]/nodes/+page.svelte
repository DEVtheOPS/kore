<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { headerStore } from "$lib/stores/header.svelte";
  import { activeClusterStore } from "$lib/stores/activeCluster.svelte";
  import DataTable, { type Column } from "$lib/components/ui/DataTable.svelte";
  import Drawer from "$lib/components/ui/Drawer.svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";

  interface NodeSummary {
    id: string;
    name: string;
    status: string;
    roles: string;
    version: string;
    age: string;
    internal_ip: string;
    os_image: string;
    kernel_version: string;
    container_runtime: string;
    taints: string[];
    capacity_cpu: string;
    capacity_memory: string;
    capacity_pods: string;
    allocatable_cpu: string;
    allocatable_memory: string;
    allocatable_pods: string;
    labels: Record<string, string>;
    created_at: number;
  }

  let data = $state<NodeSummary[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let search = $state("");
  let selectedNode = $state<NodeSummary | null>(null);
  let showDrawer = $state(false);

  const columns: Column[] = [
    { id: "name", label: "Name", sortable: true },
    { id: "status", label: "Status", sortable: true },
    { id: "roles", label: "Roles", sortable: true },
    { id: "version", label: "Version", sortable: true },
    { id: "internal_ip", label: "Internal IP", sortable: true },
    { id: "age", label: "Age", sortable: true, sortKey: "created_at" },
  ];

  $effect(() => {
    headerStore.setTitle("Nodes");
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
      data = await invoke<NodeSummary[]>("cluster_list_nodes", {
        clusterId: activeClusterStore.clusterId,
      });
    } catch (e) {
      console.error("Failed to load nodes", e);
      error = "Failed to load nodes.";
    } finally {
      loading = false;
    }
  }

  function getStatusVariant(status: string): "success" | "warning" | "error" | "info" | "neutral" {
    if (status === "Ready") return "success";
    if (status === "NotReady") return "error";
    return "warning";
  }

  function openDetails(node: NodeSummary) {
    selectedNode = node;
    showDrawer = true;
  }
</script>

<div class="h-full">
  {#if error}
    <div class="mb-4 p-3 bg-error/10 text-error rounded-md border border-error/20 flex items-center justify-between gap-3">
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
    onRowClick={openDetails}
    emptyMessage="No nodes found."
    storageKey="nodes"
  >
    {#snippet children({ column, value })}
      {#if column.id === "status"}
        <Badge variant={getStatusVariant(value)}>{value}</Badge>
      {:else}
        {value}
      {/if}
    {/snippet}
  </DataTable>

  <Drawer bind:open={showDrawer} title={selectedNode?.name || "Node Details"}>
    {#if selectedNode}
      <div class="p-4 space-y-4 text-sm">
        <div class="grid grid-cols-2 gap-4">
          <div><span class="text-text-muted">Status:</span> {selectedNode.status}</div>
          <div><span class="text-text-muted">Roles:</span> {selectedNode.roles}</div>
          <div><span class="text-text-muted">Version:</span> {selectedNode.version}</div>
          <div><span class="text-text-muted">Internal IP:</span> {selectedNode.internal_ip}</div>
          <div><span class="text-text-muted">OS:</span> {selectedNode.os_image}</div>
          <div><span class="text-text-muted">Runtime:</span> {selectedNode.container_runtime}</div>
          <div><span class="text-text-muted">Kernel:</span> {selectedNode.kernel_version}</div>
          <div><span class="text-text-muted">Age:</span> {selectedNode.age}</div>
        </div>

        <div>
          <h3 class="font-semibold mb-2">Capacity</h3>
          <div class="grid grid-cols-3 gap-2">
            <div class="p-2 bg-bg-main rounded border border-border-main">CPU: {selectedNode.capacity_cpu}</div>
            <div class="p-2 bg-bg-main rounded border border-border-main">Memory: {selectedNode.capacity_memory}</div>
            <div class="p-2 bg-bg-main rounded border border-border-main">Pods: {selectedNode.capacity_pods}</div>
          </div>
        </div>

        <div>
          <h3 class="font-semibold mb-2">Allocatable</h3>
          <div class="grid grid-cols-3 gap-2">
            <div class="p-2 bg-bg-main rounded border border-border-main">CPU: {selectedNode.allocatable_cpu}</div>
            <div class="p-2 bg-bg-main rounded border border-border-main">Memory: {selectedNode.allocatable_memory}</div>
            <div class="p-2 bg-bg-main rounded border border-border-main">Pods: {selectedNode.allocatable_pods}</div>
          </div>
        </div>

        <div>
          <h3 class="font-semibold mb-2">Taints</h3>
          {#if selectedNode.taints.length > 0}
            <ul class="list-disc list-inside space-y-1">
              {#each selectedNode.taints as taint (taint)}
                <li class="font-mono text-xs">{taint}</li>
              {/each}
            </ul>
          {:else}
            <div class="text-text-muted">No taints</div>
          {/if}
        </div>
      </div>
    {/if}
  </Drawer>
</div>
