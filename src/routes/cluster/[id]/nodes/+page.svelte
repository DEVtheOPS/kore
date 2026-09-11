<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { headerStore } from "$lib/stores/header.svelte";
  import { activeClusterStore } from "$lib/stores/activeCluster.svelte";
  import DataTable, { type Column } from "$lib/components/ui/DataTable.svelte";
  import Drawer from "$lib/components/ui/Drawer.svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import UsageBar from "$lib/components/ui/UsageBar.svelte";
  import { cpuToMillicores, parseQuantity, formatMillicores, formatBytes, isMetricsUnavailableError } from "$lib/utils/quantity";

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

  interface NodeUsage {
    name: string;
    cpu_millicores: number;
    memory_bytes: number;
    timestamp?: string;
    window?: string;
  }

  /** NodeSummary enriched with live usage (when metrics-server is available). */
  interface NodeRow extends NodeSummary {
    cpu_usage_millicores: number | null;
    memory_usage_bytes: number | null;
    cpu_allocatable_millicores: number | null;
    memory_allocatable_bytes: number | null;
  }

  let nodes = $state<NodeSummary[]>([]);
  let usageByNode = $state<Record<string, NodeUsage>>({});
  let metricsAvailable = $state<boolean | null>(null);
  let usageError = $state<string | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let search = $state("");
  let selectedNode = $state<NodeRow | null>(null);
  let showDrawer = $state(false);
  let refreshTimer: ReturnType<typeof setInterval> | null = null;

  const data = $derived<NodeRow[]>(
    nodes.map((n) => {
      const usage = usageByNode[n.name];
      return {
        ...n,
        cpu_usage_millicores: usage?.cpu_millicores ?? null,
        memory_usage_bytes: usage?.memory_bytes ?? null,
        cpu_allocatable_millicores: cpuToMillicores(n.allocatable_cpu),
        memory_allocatable_bytes: parseQuantity(n.allocatable_memory),
      };
    })
  );

  const columns: Column[] = [
    { id: "name", label: "Name", sortable: true },
    { id: "status", label: "Status", sortable: true },
    { id: "roles", label: "Roles", sortable: true },
    { id: "cpu_usage_millicores", label: "CPU", sortable: true },
    { id: "memory_usage_bytes", label: "Memory", sortable: true },
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

  // Refresh live usage periodically while the page is mounted and metrics are available.
  $effect(() => {
    if (refreshTimer) clearInterval(refreshTimer);
    if (metricsAvailable) {
      refreshTimer = setInterval(loadUsage, 15_000);
    }
    return () => {
      if (refreshTimer) clearInterval(refreshTimer);
      refreshTimer = null;
    };
  });

  async function loadUsage() {
    if (!activeClusterStore.clusterId) return;
    try {
      const usage = await invoke<NodeUsage[]>("cluster_get_node_usage", {
        clusterId: activeClusterStore.clusterId,
      });
      usageByNode = Object.fromEntries(usage.map((u) => [u.name, u]));
      metricsAvailable = true;
      usageError = null;
    } catch (e) {
      if (isMetricsUnavailableError(e)) {
        // Metrics API not served by this cluster: stop polling and show the notice.
        metricsAvailable = false;
        usageByNode = {};
        usageError = null;
      } else {
        // Transient / RBAC / network failure: keep the last values and keep polling.
        console.error("Failed to load node usage", e);
        usageError = `Live usage could not be refreshed: ${e}`;
        if (metricsAvailable === null) metricsAvailable = true;
      }
    }
  }

  async function loadData() {
    if (!activeClusterStore.clusterId) return;
    loading = true;
    error = null;
    try {
      const [nodeList] = await Promise.all([
        invoke<NodeSummary[]>("cluster_list_nodes", {
          clusterId: activeClusterStore.clusterId,
        }),
        loadUsage(),
      ]);
      nodes = nodeList;
    } catch (e) {
      console.error("Failed to load nodes", e);
      error = `Failed to load nodes: ${e}`;
    } finally {
      loading = false;
    }
  }

  function getStatusVariant(status: string): "success" | "warning" | "error" | "info" | "neutral" {
    if (status === "Ready") return "success";
    if (status === "NotReady") return "error";
    return "warning";
  }

  function openDetails(node: NodeRow) {
    selectedNode = node;
    showDrawer = true;
  }
</script>

<div class="h-full">
  {#if metricsAvailable === false}
    <div class="mb-4 p-3 bg-bg-panel text-text-muted text-sm rounded-md border border-border-subtle">
      Live CPU/Memory usage is unavailable: the Metrics API (metrics-server) is not installed in this cluster.
    </div>
  {:else if usageError}
    <div class="mb-4 p-3 bg-warning/10 text-warning text-sm rounded-md border border-warning/20 flex items-center justify-between gap-3">
      <span>{usageError}</span>
      <Button variant="ghost" size="sm" onclick={() => (usageError = null)}>Dismiss</Button>
    </div>
  {/if}

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
    {#snippet children({ row, column, value })}
      {#if column.id === "status"}
        <Badge variant={getStatusVariant(value)}>{value}</Badge>
      {:else if metricsAvailable === false && (column.id === "cpu_usage_millicores" || column.id === "memory_usage_bytes")}
        <span class="text-xs text-text-muted">n/a</span>
      {:else if column.id === "cpu_usage_millicores"}
        <UsageBar
          used={row.cpu_usage_millicores}
          total={row.cpu_allocatable_millicores}
          label={`${formatMillicores(row.cpu_usage_millicores)} / ${formatMillicores(row.cpu_allocatable_millicores)}`}
        />
      {:else if column.id === "memory_usage_bytes"}
        <UsageBar
          used={row.memory_usage_bytes}
          total={row.memory_allocatable_bytes}
          label={`${formatBytes(row.memory_usage_bytes)} / ${formatBytes(row.memory_allocatable_bytes)}`}
        />
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

        {#if selectedNode.cpu_usage_millicores != null || selectedNode.memory_usage_bytes != null}
          <div>
            <h3 class="font-semibold mb-2">Live Usage</h3>
            <div class="space-y-2">
              <div class="flex items-center gap-3">
                <span class="w-16 text-text-muted">CPU</span>
                <UsageBar
                  class="flex-1"
                  used={selectedNode.cpu_usage_millicores}
                  total={selectedNode.cpu_allocatable_millicores}
                  label={`${formatMillicores(selectedNode.cpu_usage_millicores)} / ${formatMillicores(selectedNode.cpu_allocatable_millicores)}`}
                />
              </div>
              <div class="flex items-center gap-3">
                <span class="w-16 text-text-muted">Memory</span>
                <UsageBar
                  class="flex-1"
                  used={selectedNode.memory_usage_bytes}
                  total={selectedNode.memory_allocatable_bytes}
                  label={`${formatBytes(selectedNode.memory_usage_bytes)} / ${formatBytes(selectedNode.memory_allocatable_bytes)}`}
                />
              </div>
            </div>
          </div>
        {/if}

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
