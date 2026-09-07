<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onDestroy } from 'svelte';
  import Chart from '$lib/components/ui/Chart.svelte';
  import { activeClusterStore } from '$lib/stores/activeCluster.svelte';
  import { formatBytes, formatMillicores, isMetricsUnavailableError } from '$lib/utils/quantity';

  interface ContainerUsage {
    name: string;
    cpu_millicores: number;
    memory_bytes: number;
  }

  interface PodUsage {
    name: string;
    namespace: string;
    cpu_millicores: number;
    memory_bytes: number;
    containers: ContainerUsage[];
    timestamp?: string;
    window?: string;
  }

  let {
    namespace,
    selector,
    refreshMs = 15_000,
  }: {
    namespace: string;
    /** Label selector map (e.g. the workload's `spec.selector.matchLabels`). */
    selector: Record<string, string>;
    /** Polling interval; set to 0 to disable auto refresh. */
    refreshMs?: number;
  } = $props();

  let usage = $state<PodUsage[]>([]);
  let loading = $state(false);
  let unavailable = $state(false);
  let error = $state<string | null>(null);
  let activeTab = $state<'cpu' | 'memory'>('cpu');
  let timer: ReturnType<typeof setInterval> | null = null;

  const labelSelector = $derived(
    Object.entries(selector ?? {})
      .map(([k, v]) => `${k}=${v}`)
      .join(',')
  );

  const totalCpu = $derived(usage.reduce((sum, p) => sum + p.cpu_millicores, 0));
  const totalMemory = $derived(usage.reduce((sum, p) => sum + p.memory_bytes, 0));

  function shortPodName(name: string): string {
    // Trim the common replicaset/statefulset prefix noise for chart labels.
    return name.length > 28 ? `…${name.slice(-27)}` : name;
  }

  const chartData = $derived({
    labels: usage.map((p) => shortPodName(p.name)),
    datasets: [
      {
        label: activeTab === 'cpu' ? 'CPU (millicores)' : 'Memory (MiB)',
        data: usage.map((p) =>
          activeTab === 'cpu' ? Math.round(p.cpu_millicores) : Math.round(p.memory_bytes / 1024 ** 2)
        ),
        backgroundColor: 'rgba(50, 108, 229, 0.6)',
        borderColor: 'rgb(50, 108, 229)',
        borderWidth: 1,
      },
    ],
  });

  const chartOptions = {
    responsive: true,
    maintainAspectRatio: false,
    plugins: { legend: { display: false } },
    scales: {
      y: { beginAtZero: true, ticks: { precision: 0 } },
      x: { ticks: { autoSkip: true, maxRotation: 0 } },
    },
  };

  async function load() {
    const clusterId = activeClusterStore.clusterId;
    if (!clusterId || !namespace || !labelSelector) return;

    loading = true;
    try {
      usage = await invoke<PodUsage[]>('cluster_get_pod_usage', {
        clusterId,
        namespace,
        labelSelector,
      });
      unavailable = false;
      error = null;
    } catch (e) {
      usage = [];
      if (isMetricsUnavailableError(e)) {
        unavailable = true;
        error = null;
      } else {
        unavailable = false;
        error = String(e);
        console.error('Failed to load pod usage', e);
      }
    } finally {
      loading = false;
    }
  }

  // Reload when the target changes; poll while metrics are available.
  $effect(() => {
    // Track dependencies explicitly
    void namespace;
    void labelSelector;
    void activeClusterStore.clusterId;

    if (timer) clearInterval(timer);
    timer = null;
    load();
    if (refreshMs > 0) {
      timer = setInterval(() => {
        if (!unavailable) load();
      }, refreshMs);
    }
  });

  onDestroy(() => {
    if (timer) clearInterval(timer);
  });
</script>

<div class="space-y-3">
  <div class="flex items-center justify-between border-b border-border">
    <div class="flex gap-2">
      <button
        class="px-4 py-2 text-sm font-medium transition-colors {activeTab === 'cpu'
          ? 'border-b-2 border-color-primary text-text-main'
          : 'text-text-muted hover:text-text-main'}"
        onclick={() => (activeTab = 'cpu')}
      >
        CPU
      </button>
      <button
        class="px-4 py-2 text-sm font-medium transition-colors {activeTab === 'memory'
          ? 'border-b-2 border-color-primary text-text-main'
          : 'text-text-muted hover:text-text-main'}"
        onclick={() => (activeTab = 'memory')}
      >
        Memory
      </button>
    </div>
    {#if usage.length > 0}
      <div class="text-xs text-text-muted font-mono pr-2">
        Total: {activeTab === 'cpu' ? formatMillicores(totalCpu) : formatBytes(totalMemory)}
        {#if usage[0]?.window}<span class="opacity-70"> · window {usage[0].window}</span>{/if}
      </div>
    {/if}
  </div>

  <div class="h-48 bg-bg-panel rounded-md p-4">
    {#if unavailable}
      <div class="h-full flex items-center justify-center text-center text-sm text-text-muted">
        Live usage is unavailable: the Metrics API (metrics-server) is not installed in this cluster.
      </div>
    {:else if error}
      <div class="h-full flex items-center justify-center text-center text-sm text-error">{error}</div>
    {:else if usage.length === 0}
      <div class="h-full flex items-center justify-center text-sm text-text-muted">
        {loading ? 'Loading usage...' : 'No usage data reported for these pods yet.'}
      </div>
    {:else}
      <Chart type="bar" data={chartData} options={chartOptions} />
    {/if}
  </div>
</div>
