<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { ChartOptions, ScriptableContext } from 'chart.js';
  import Chart from '$lib/components/ui/Chart.svelte';
  import { activeClusterStore } from '$lib/stores/activeCluster.svelte';
  import { formatBytes, formatMillicores, isMetricsUnavailableError } from '$lib/utils/quantity';

  interface PodUsage {
    name: string;
    cpu_millicores: number | null;
    memory_bytes: number | null;
    cpu_request_millicores?: number | null;
    cpu_limit_millicores?: number | null;
    memory_request_bytes?: number | null;
    memory_limit_bytes?: number | null;
    window?: string | null;
  }

  interface DeploymentResourceUsage {
    pods: PodUsage[];
    usage_error: string | null;
  }

  const resourceSeries = [
    {
      label: 'Usage',
      color: '--color-primary',
      cpu: 'cpu_millicores',
      memory: 'memory_bytes',
      empty: 'Unavailable',
    },
    {
      label: 'Requests',
      color: '--color-success',
      cpu: 'cpu_request_millicores',
      memory: 'memory_request_bytes',
      empty: 'Unset',
    },
    {
      label: 'Limits',
      color: '--color-warning',
      cpu: 'cpu_limit_millicores',
      memory: 'memory_limit_bytes',
      empty: 'Unset',
    },
  ] as const;

  let {
    namespace,
    selector,
    deploymentName,
    refreshMs = 15_000,
  }: {
    namespace: string;
    /** Label selector for workloads that show live usage only. */
    selector?: Record<string, string>;
    deploymentName?: string;
    /** Polling interval; set to 0 to disable auto refresh. */
    refreshMs?: number;
  } = $props();

  let usage = $state<PodUsage[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let usageError = $state<string | null>(null);
  let activeTab = $state<'cpu' | 'memory'>('cpu');

  const labelSelector = $derived(
    Object.entries(selector ?? {})
      .map(([k, v]) => `${k}=${v}`)
      .join(',')
  );

  const unavailable = $derived(!deploymentName && isMetricsUnavailableError(error));
  const metricsWindow = $derived(usage.find((pod) => pod.window)?.window);
  const series = $derived(
    (deploymentName ? resourceSeries : resourceSeries.slice(0, 1)).map(({ label, color, cpu, memory, empty }) => {
      const field = activeTab === 'cpu' ? cpu : memory;
      const values = usage.map((pod) => pod[field] ?? null);
      const reported = values.filter((value) => value !== null);
      const sum = reported.reduce((total, value) => total + value, 0);
      return {
        label,
        color,
        values,
        count: reported.length,
        total: reported.length === 0 ? empty : formatValue(sum),
      };
    })
  );

  function formatValue(value: number) {
    if (activeTab === 'memory') return formatBytes(value);
    if (value > 0 && value < 0.5) return '<1m';
    return value >= 1000 ? `${formatMillicores(value)} cores` : formatMillicores(value);
  }

  function chartColor(context: { chart: { canvas: HTMLCanvasElement } }, variable: string) {
    return getComputedStyle(context.chart.canvas).getPropertyValue(variable).trim();
  }

  function shortPodName(name: string): string {
    // Trim the common replicaset/statefulset prefix noise for chart labels.
    return name.length > 28 ? `…${name.slice(-27)}` : name;
  }

  const chartData = $derived({
    labels: usage.map((p) => shortPodName(p.name)),
    datasets: series.map(({ label, color, values }) => ({
      label,
      data: values.map((value) => value === null || activeTab === 'cpu' ? value : value / 1024 ** 2),
      backgroundColor: (context: ScriptableContext<'bar'>) => chartColor(context, color),
    })),
  });

  const chartOptions = {
    responsive: true,
    maintainAspectRatio: false,
    interaction: { mode: 'index', intersect: false },
    plugins: {
      legend: { display: false },
      tooltip: {
        callbacks: {
          title: ([item]) => usage[item.dataIndex]?.name ?? '',
          label: (item) => {
            const value = series[item.datasetIndex]?.values[item.dataIndex];
            return value == null ? '' : `${item.dataset.label}: ${formatValue(value)}`;
          },
        },
      },
    },
    scales: {
      y: {
        beginAtZero: true,
        ticks: {
          callback: (value) => `${value}${activeTab === 'cpu' ? 'm' : ' MiB'}`,
          color: (context) => chartColor(context, '--text-muted'),
        },
        grid: { color: (context) => chartColor(context, '--border-subtle') },
      },
      x: {
        ticks: { autoSkip: true, maxRotation: 0, color: (context) => chartColor(context, '--text-muted') },
        grid: { display: false },
      },
    },
  } satisfies ChartOptions<'bar'>;

  $effect(() => {
    const target = { clusterId: activeClusterStore.clusterId, namespace };
    const name = deploymentName;
    const labels = labelSelector;
    usage = [];
    error = null;
    usageError = null;
    loading = false;
    if (!target.clusterId || !target.namespace || !(name || labels)) return;

    let active = true;
    let inFlight = false;

    async function load() {
      if (inFlight) return;
      inFlight = true;
      loading = true;
      try {
        const result = name
          ? await invoke<DeploymentResourceUsage>('cluster_get_deployment_resource_usage', {
              ...target,
              deploymentName: name,
            })
          : {
              pods: await invoke<PodUsage[]>('cluster_get_pod_usage', { ...target, labelSelector: labels }),
              usage_error: null,
            };
        if (!active) return;
        usage = result.pods;
        usageError = result.usage_error;
        error = null;
      } catch (e) {
        if (!active) return;
        usage = [];
        usageError = null;
        error = String(e);
      } finally {
        inFlight = false;
        if (active) loading = false;
      }
    }

    void load();
    const timer = refreshMs > 0 ? setInterval(() => {
      if (!unavailable) void load();
    }, refreshMs) : null;

    return () => {
      active = false;
      if (timer !== null) clearInterval(timer);
    };
  });
</script>

<div class="space-y-3">
  <div class="flex items-center justify-between border-b border-border">
    <div class="flex gap-2">
      <button
        aria-pressed={activeTab === 'cpu'}
        class="px-4 py-2 text-sm font-medium transition-colors {activeTab === 'cpu'
          ? 'border-b-2 border-color-primary text-text-main'
          : 'text-text-muted hover:text-text-main'}"
        onclick={() => (activeTab = 'cpu')}
      >
        CPU
      </button>
      <button
        aria-pressed={activeTab === 'memory'}
        class="px-4 py-2 text-sm font-medium transition-colors {activeTab === 'memory'
          ? 'border-b-2 border-color-primary text-text-main'
          : 'text-text-muted hover:text-text-main'}"
        onclick={() => (activeTab = 'memory')}
      >
        Memory
      </button>
    </div>
    {#if metricsWindow || (!deploymentName && usage.length > 0)}
      <div class="text-xs text-text-muted font-mono pr-2">
        {#if !deploymentName}Total: {series[0].total}{/if}
        {#if metricsWindow}{#if !deploymentName} · {/if}window {metricsWindow}{/if}
      </div>
    {/if}
  </div>

  {#if usageError}
    <p class="text-sm text-text-muted">Failed to load live usage: {usageError}</p>
  {/if}

  {#if deploymentName && usage.length > 0}
    <ul class="flex flex-wrap gap-x-5 gap-y-1 text-xs text-text-muted" aria-label="Resource totals">
      {#each series as { label, color, total, count } (label)}
        <li class="inline-flex items-center gap-1.5">
          <span class="size-2.5 rounded-sm" style:background-color="var({color})" aria-hidden="true"></span>
          {label}
          <span class="font-mono text-text-main">{total}</span>
          {#if count > 0 && count < usage.length}
            <span>({count}/{usage.length} pods)</span>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}

  <div class="h-48 bg-bg-panel rounded-md p-4">
    {#if unavailable}
      <div class="h-full flex items-center justify-center text-center text-sm text-text-muted">
        Live usage is unavailable: the Metrics API (metrics-server) is not installed in this cluster.
      </div>
    {:else if error}
      <div class="h-full flex items-center justify-center text-center text-sm text-error">
        Failed to load {deploymentName ? 'deployment resources' : 'pod usage'}: {error}
      </div>
    {:else if usage.length === 0}
      <div class="h-full flex items-center justify-center text-sm text-text-muted">
        {#if deploymentName}
          {loading ? 'Loading deployment resources...' : 'No pods found for this deployment.'}
        {:else}
          {loading ? 'Loading usage...' : 'No usage data reported for these pods yet.'}
        {/if}
      </div>
    {:else}
      <Chart type="bar" data={chartData} options={chartOptions} />
    {/if}
  </div>
</div>
