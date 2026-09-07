<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { confirm } from "@tauri-apps/plugin-dialog";
  import { onMount, onDestroy } from "svelte";
  import { page } from "$app/stores";
  import DataTable from "$lib/components/ui/DataTable.svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import PodDetailDrawer from "$lib/components/PodDetailDrawer.svelte";
  import { Trash2, FileText, FilePenLine, Eye } from "lucide-svelte";
  import { activeClusterStore } from "$lib/stores/activeCluster.svelte";
  import { headerStore } from "$lib/stores/header.svelte";
  import { bottomDrawerStore } from "$lib/stores/bottomDrawer.svelte";
  import type { MenuItem } from "$lib/components/ui/Menu.svelte";

  interface ContainerPort {
    name?: string;
    container_port: number;
    host_port?: number;
    protocol: string;
  }

  interface EnvVar {
    name: string;
    value?: string;
    value_from?: string;
  }

  interface VolumeMount {
    name: string;
    mount_path: string;
    sub_path?: string;
    read_only: boolean;
  }

  interface ProbeInfo {
    probe_type: string;
    handler_type: string;
    details: string;
    initial_delay_seconds: number;
    period_seconds: number;
    timeout_seconds: number;
    success_threshold: number;
    failure_threshold: number;
  }

  interface ContainerInfo {
    name: string;
    image: string;
    image_pull_policy: string;
    ready: boolean;
    restart_count: number;
    state: string;
    cpu_request?: string;
    cpu_limit?: string;
    memory_request?: string;
    memory_limit?: string;
    ports: ContainerPort[];
    env: EnvVar[];
    volume_mounts: VolumeMount[];
    probes: ProbeInfo[];
  }

  interface VolumeInfo {
    name: string;
    volume_type: string;
  }

  interface PodCondition {
    condition_type: string;
    status: string;
    reason?: string;
    message?: string;
    last_transition_time?: string;
  }

  interface PodEventInfo {
    event_type: string;
    reason: string;
    message: string;
    count: number;
    first_timestamp?: string;
    last_timestamp?: string;
    source: string;
  }

  interface Pod {
    uid: string;
    name: string;
    namespace: string;
    status: string;
    age: string;
    containers: number;
    restarts: number;
    node: string;
    qos: string;
    controlled_by: string;
    creation_timestamp?: string;
    labels: Record<string, string>;
    annotations: Record<string, string>;
    pod_ip: string;
    host_ip: string;
    service_account: string;
    priority_class: string;
    container_details: ContainerInfo[];
    volumes: VolumeInfo[];
    conditions: PodCondition[];
  }

  let pods = $state<Pod[]>([]);
  let loading = $state(false);
  let error = $state("");
  let search = $state("");
  let selectedPod = $state<Pod | null>(null);
  let podEvents = $state<PodEventInfo[]>([]);
  let loadingEvents = $state(false);
  let isDrawerOpen = $state(false);
  let unlisten: (() => void) | null = null;
  let now = $state(Date.now());
  let interval: ReturnType<typeof setInterval> | undefined;
  // Monotonic counter so overlapping startWatch() calls (fast namespace switches,
  // refresh during load) can't apply a stale list or start a stale watch.
  let watchGeneration = 0;

  // Define Columns
  let columns = $state([
    { id: "name", label: "Name", sortable: true, visible: true },
    { id: "namespace", label: "Namespace", sortable: true, visible: true },
    { id: "containers", label: "Containers", sortable: true, visible: true },
    { id: "restarts", label: "Restarts", sortable: true, visible: true },
    { id: "controlled_by", label: "Controlled By", sortable: true, visible: true },
    { id: "node", label: "Node", sortable: true, visible: true },
    { id: "qos", label: "QoS", sortable: true, visible: true },
    { id: "age", label: "Age", sortable: true, visible: true, sortKey: "creation_timestamp" },
    { id: "status", label: "Status", sortable: true, visible: true },
  ]);

  async function startWatch() {
    const clusterId = activeClusterStore.clusterId;
    const namespace = activeClusterStore.activeNamespace;
    if (!clusterId) return;

    const generation = ++watchGeneration;
    loading = true;
    error = "";

    // Fetch the initial list for immediate feedback; the watch keeps it fresh afterwards.
    try {
      const list = await invoke<Pod[]>("cluster_list_pods", { clusterId, namespace });
      if (generation !== watchGeneration) return; // superseded
      pods = list;
    } catch (e) {
      if (generation !== watchGeneration) return;
      console.error(e);
      error = `Failed to load pods: ${e}`;
    } finally {
      if (generation === watchGeneration) loading = false;
    }

    // Start (or restart) the watch. The backend aborts any previous pod watch.
    invoke("cluster_start_pod_watch", { clusterId, namespace }).catch((e) =>
      console.error("Watch failed to start", e)
    );
  }

  onMount(async () => {
    // Listen for backend events
    unlisten = await listen("pod_event", (event: any) => {
      const payload = event.payload;
      console.log("Pod Event:", payload);

      if (payload.type === "Restarted") {
        pods = payload.payload;
      } else if (payload.type === "Added" || payload.type === "Modified") {
        const newPod = payload.payload;
        const idx = pods.findIndex((p) => p.uid === newPod.uid);
        if (idx >= 0) {
          pods[idx] = newPod;
        } else {
          pods.push(newPod);
        }
      } else if (payload.type === "Deleted") {
        const deletedPod = payload.payload;
        pods = pods.filter((p) => p.uid !== deletedPod.uid);
      }
    });

    // Update 'now' every second for active age
    interval = setInterval(() => {
      now = Date.now();
    }, 1000);

    // Check for deep-link query params to auto-open a pod
    const podName = $page.url.searchParams.get('pod');
    const podNamespace = $page.url.searchParams.get('namespace');
    if (podName && podNamespace) {
      // Wait for pods to load, then open the matching pod
      const checkAndOpen = setInterval(() => {
        const pod = pods.find((p) => p.name === podName && p.namespace === podNamespace);
        if (pod) {
          handleRowClick(pod);
          clearInterval(checkAndOpen);
          // Clear the query params from URL without reloading
          window.history.replaceState({}, '', $page.url.pathname);
        }
      }, 100);
      // Stop checking after 5 seconds
      setTimeout(() => clearInterval(checkAndOpen), 5000);
    }
  });

  $effect(() => {
    headerStore.setTitle("Pods");
  });

  onDestroy(() => {
    if (unlisten) unlisten();
    if (interval) clearInterval(interval);
    invoke("cluster_stop_pod_watch").catch((e) => console.error("Failed to stop pod watch", e));
  });

  // (Re)start the watch whenever the active cluster or namespace changes.
  $effect(() => {
    if (activeClusterStore.clusterId && activeClusterStore.activeNamespace) {
      startWatch();
    }
  });

  const filteredPods = $derived(pods.filter((p) => p.name.toLowerCase().includes(search.toLowerCase())));

  function getStatusVariant(status: string) {
    switch (status.toLowerCase()) {
      case "running":
        return "success";
      case "pending":
        return "warning";
      case "failed":
      case "error":
        return "error";
      default:
        return "neutral";
    }
  }

  async function handleRowClick(row: any) {
    selectedPod = row;
    isDrawerOpen = true;

    // Load events for this pod
    loadingEvents = true;
    podEvents = [];
    try {
      podEvents = await invoke<PodEventInfo[]>("cluster_get_pod_events", {
        clusterId: activeClusterStore.clusterId,
        namespace: row.namespace,
        podName: row.name,
      });
    } catch (e) {
      console.error("Failed to load pod events:", e);
    } finally {
      loadingEvents = false;
    }
  }

  function formatAge(creationTimestamp: string | undefined): string {
    if (!creationTimestamp) return "-";

    const created = new Date(creationTimestamp).getTime();
    const diff = Math.max(0, now - created);

    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (days > 0) return `${days}d`;
    if (hours > 0) return `${hours}h`;
    if (minutes > 0) return `${minutes}m`;
    return `${seconds}s`;
  }

  async function handleDelete(pod: Pod) {
    const confirmed = await confirm(`Are you sure you want to delete pod ${pod.name}?`, {
      title: "Delete Pod",
      kind: "warning",
    });

    if (!confirmed) return;

    try {
      await invoke("cluster_delete_pod", {
        clusterId: activeClusterStore.clusterId,
        namespace: pod.namespace,
        podName: pod.name,
      });
      // UI update will happen via watch event
    } catch (e) {
      error = `Failed to delete pod ${pod.name}: ${e}`;
    }
  }

  function openLogs(pod: Pod, containerName: string) {
    const clusterId = activeClusterStore.clusterId;
    if (!clusterId) return;

    const streamId = `${clusterId}-${pod.namespace}-${pod.name}-${containerName}`;
    bottomDrawerStore.openTab({
      id: streamId,
      title: `${containerName}.log`,
      type: "logs",
      data: {
        clusterId,
        namespace: pod.namespace,
        podName: pod.name,
        containerName,
        streamId,
      },
    });
  }

  function handleViewLogs(pod: Pod) {
    const containers = pod.container_details ?? [];
    if (containers.length === 1) {
      openLogs(pod, containers[0].name);
    } else {
      // Multiple containers: open the detail drawer where each container has its own logs button.
      handleRowClick(pod);
    }
  }

  function handleEditYaml(pod: Pod) {
    const clusterId = activeClusterStore.clusterId;
    if (!clusterId) return;

    bottomDrawerStore.openTab({
      id: `edit-pod-${clusterId}-${pod.namespace}-${pod.name}`,
      title: `${pod.name}.yaml`,
      type: "edit",
      data: {
        clusterId,
        kind: "pod",
        name: pod.name,
        namespace: pod.namespace,
      },
    });
  }

  function getActions(row: Pod): MenuItem[] {
    return [
      { label: "View Details", icon: Eye, action: () => handleRowClick(row) },
      { label: "View Logs", icon: FileText, action: () => handleViewLogs(row) },
      { label: "Edit YAML", icon: FilePenLine, action: () => handleEditYaml(row) },
      { label: "Delete", icon: Trash2, action: () => handleDelete(row), danger: true },
    ];
  }

  const batchActions = [
    {
      label: "Delete Selected",
      icon: Trash2,
      danger: true,
      action: async (selectedIds: string[]) => {
        const confirmed = await confirm(`Are you sure you want to delete ${selectedIds.length} pods?`, {
          title: "Delete Pods",
          kind: "warning",
        });

        if (!confirmed) return;

        const targets = pods.filter((p) => selectedIds.includes(p.uid));
        const results = await Promise.allSettled(
          targets.map((pod) =>
            invoke("cluster_delete_pod", {
              clusterId: activeClusterStore.clusterId,
              namespace: pod.namespace,
              podName: pod.name,
            })
          )
        );
        const failed = results.filter((r) => r.status === "rejected").length;
        if (failed > 0) {
          error = `Failed to delete ${failed} of ${targets.length} pods.`;
        }
      },
    },
  ];
</script>

<div class="space-y-6 h-full flex flex-col">
  {#if error}
    <div class="p-4 bg-error/10 text-error rounded-md border border-error/20 flex items-center justify-between gap-3">
      <span>{error}</span>
      <button class="text-sm underline" onclick={() => (error = "")}>Dismiss</button>
    </div>
  {/if}

  <div class="flex-1 overflow-hidden">
    <DataTable
      data={filteredPods}
      bind:columns
      keyField="uid"
      onRowClick={handleRowClick}
      storageKey="pods-table"
      bind:search
      onRefresh={startWatch}
      {loading}
      actions={getActions}
      {batchActions}
    >
      {#snippet children({ row, column, value })}
        {#if column.id === "status"}
          <Badge variant={getStatusVariant(value)}>{value}</Badge>
        {:else if column.id === "age"}
          {formatAge(row.creation_timestamp) || value}
        {:else}
          {value}
        {/if}
      {/snippet}
    </DataTable>
  </div>

  <PodDetailDrawer bind:open={isDrawerOpen} bind:pod={selectedPod} bind:events={podEvents} bind:loadingEvents />
</div>
