<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { headerStore } from "$lib/stores/header.svelte";
  import { activeClusterStore } from "$lib/stores/activeCluster.svelte";
  import Chart from "$lib/components/ui/Chart.svelte";
  import {
    Chart as ChartJS,
    Title,
    Tooltip,
    Legend,
    LineElement,
    LinearScale,
    PointElement,
    CategoryScale,
    ArcElement,
    Filler
  } from "chart.js";
  import DataTable from "$lib/components/ui/DataTable.svelte";
  import type { Column } from "$lib/components/ui/DataTable.svelte";

  ChartJS.register(
    Title,
    Tooltip,
    Legend,
    LineElement,
    LinearScale,
    PointElement,
    CategoryScale,
    ArcElement,
    Filler
  );

  interface ResourceStats {
    capacity: number;
    allocatable: number;
    requests: number;
    limits: number;
    usage: number;
  }

  interface ClusterMetrics {
    cpu: ResourceStats;
    memory: ResourceStats;
    pods: ResourceStats;
  }

  interface WarningEvent {
    message: string;
    object: string;
    type_: string;
    age: string;
    count: number;
  }

  let metrics = $state<ClusterMetrics | null>(null);
  let events = $state<WarningEvent[]>([]);
  let loading = $state(true);
  let interval: ReturnType<typeof setInterval>;

  // History for charts
  let historyMaxLen = 20;
  let historyCpu = $state<number[]>(new Array(historyMaxLen).fill(0));
  let historyMem = $state<number[]>(new Array(historyMaxLen).fill(0));
  let historyLabels = $state<string[]>(new Array(historyMaxLen).fill(""));

  const eventColumns: Column[] = [
    { id: "type_", label: "Type", sortable: true },
    { id: "object", label: "Object", sortable: true },
    { id: "message", label: "Message", sortable: true },
    { id: "age", label: "Age", sortable: true },
    { id: "count", label: "Count", sortable: true },
  ];

  $effect(() => {
    headerStore.setTitle("Cluster Dashboard");
  });

  async function loadData() {
    if (!activeClusterStore.clusterId) return;
    
    try {
      // Fetch Metrics
      const m = await invoke<ClusterMetrics>("cluster_get_metrics", {
        clusterId: activeClusterStore.clusterId,
      });
      metrics = m;

      // Update History
      const now = new Date();
      const timeLabel = `${now.getHours()}:${now.getMinutes()}:${now.getSeconds()}`;
      
      // Calculate percentages (allocatable vs usage/requests)
      // Fallback to requests if usage is 0 (missing metrics-server)
      const cpuVal = m.cpu.usage > 0 ? m.cpu.usage : m.cpu.requests;
      const memVal = m.memory.usage > 0 ? m.memory.usage : m.memory.requests;

      // Normalize to percentage of capacity or allocatable
      const cpuPercent = m.cpu.capacity > 0 ? (cpuVal / m.cpu.capacity) * 100 : 0;
      const memPercent = m.memory.capacity > 0 ? (memVal / m.memory.capacity) * 100 : 0;

      historyCpu = [...historyCpu.slice(1), cpuPercent];
      historyMem = [...historyMem.slice(1), memPercent];
      historyLabels = [...historyLabels.slice(1), timeLabel];

      // Fetch Events
      const e = await invoke<WarningEvent[]>("cluster_get_events", {
        clusterId: activeClusterStore.clusterId,
      });
      events = e;

    } catch (err) {
      console.error("Failed to load dashboard data", err);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadData();
    interval = setInterval(loadData, 5000); // Refresh every 5s
  });

  onDestroy(() => {
    clearInterval(interval);
  });

  // Chart Configs
  function getLineData(label: string, data: number[], color: string) {
    return {
      labels: historyLabels,
      datasets: [
        {
          label,
          data,
          fill: true,
          borderColor: color,
          backgroundColor: color + "33", // hex alpha
          tension: 0.4,
          pointRadius: 0,
        },
      ],
    };
  }

  const lineOptions = {
    responsive: true,
    maintainAspectRatio: false,
    plugins: { legend: { display: false } },
    scales: {
      x: { display: false },
      y: { min: 0, max: 100, display: true, ticks: { callback: (v: string | number) => v + "%" } },
    },
    animation: { duration: 0 },
  };

  function getDoughnutData(usage: number, requests: number, limits: number, allocatable: number, capacity: number, type: 'cpu' | 'memory' | 'pods') {
    // Simplified for now: Allocatable vs (Used/Requests)
    // The "gauge" look is often done with a doughnut chart where the remaining part is gray
    
    // Usage (or Requests if usage 0)
    let val = usage > 0 ? usage : requests;
    // Cap at capacity
    if (val > capacity) val = capacity;
    
    const remaining = capacity - val;

    return {
      labels: ["Used", "Free"],
      datasets: [
        {
          data: [val, remaining],
          backgroundColor: ["#326CE5", "#2D3748"], // Primary, Dark Gray
          borderWidth: 0,
        },
      ],
    };
  }
  
  const doughnutOptions = {
    responsive: true,
    maintainAspectRatio: false,
    cutout: "80%",
    plugins: { legend: { display: false }, tooltip: { enabled: false } },
    animation: { duration: 0 },
  };

  function formatCpu(v: number) {
    return v.toFixed(2);
  }

  function formatMem(v: number) {
    return (v / (1024 * 1024 * 1024)).toFixed(1) + " GiB";
  }

</script>

<div class="space-y-6">
  {#if loading && !metrics}
    <div class="flex items-center justify-center h-64">
      <div class="text-text-muted">Loading metrics...</div>
    </div>
  {:else if metrics}
    <!-- Top Row: Metrics -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
      
      <!-- CPU -->
      <div class="bg-bg-main rounded-lg border border-border-main p-4 flex flex-col h-80">
        <h3 class="text-lg font-semibold mb-4 text-center">CPU</h3>
        
        <!-- Gauge/Donut -->
        <div class="relative h-32 w-32 mx-auto mb-4">
          <Chart type="doughnut" data={getDoughnutData(metrics.cpu.usage, metrics.cpu.requests, metrics.cpu.limits, metrics.cpu.allocatable, metrics.cpu.capacity, 'cpu')} options={doughnutOptions} />
          <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
            <span class="text-xl font-bold">
              {metrics.cpu.capacity > 0 ? Math.round(((metrics.cpu.usage > 0 ? metrics.cpu.usage : metrics.cpu.requests) / metrics.cpu.capacity) * 100) : 0}%
            </span>
          </div>
        </div>

        <!-- Legend -->
        <div class="grid grid-cols-2 gap-x-4 gap-y-1 text-xs text-text-muted mb-4 px-4">
            <div class="flex justify-between"><span>Usage:</span> <span class="text-text-main">{formatCpu(metrics.cpu.usage)}</span></div>
            <div class="flex justify-between"><span>Requests:</span> <span class="text-text-main">{formatCpu(metrics.cpu.requests)}</span></div>
            <div class="flex justify-between"><span>Limits:</span> <span class="text-text-main">{formatCpu(metrics.cpu.limits)}</span></div>
            <div class="flex justify-between"><span>Capacity:</span> <span class="text-text-main">{formatCpu(metrics.cpu.capacity)}</span></div>
        </div>

        <!-- History Line -->
        <div class="flex-1 min-h-0 w-full">
           <Chart type="line" data={getLineData("CPU", historyCpu, "#326CE5")} options={lineOptions} />
        </div>
      </div>

      <!-- Memory -->
      <div class="bg-bg-main rounded-lg border border-border-main p-4 flex flex-col h-80">
        <h3 class="text-lg font-semibold mb-4 text-center">Memory</h3>
        
        <!-- Gauge/Donut -->
        <div class="relative h-32 w-32 mx-auto mb-4">
           <Chart type="doughnut" data={getDoughnutData(metrics.memory.usage, metrics.memory.requests, metrics.memory.limits, metrics.memory.allocatable, metrics.memory.capacity, 'memory')} options={doughnutOptions} />
           <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
            <span class="text-xl font-bold">
              {metrics.memory.capacity > 0 ? Math.round(((metrics.memory.usage > 0 ? metrics.memory.usage : metrics.memory.requests) / metrics.memory.capacity) * 100) : 0}%
            </span>
          </div>
        </div>

        <!-- Legend -->
        <div class="grid grid-cols-2 gap-x-4 gap-y-1 text-xs text-text-muted mb-4 px-4">
            <div class="flex justify-between"><span>Usage:</span> <span class="text-text-main">{formatMem(metrics.memory.usage)}</span></div>
            <div class="flex justify-between"><span>Requests:</span> <span class="text-text-main">{formatMem(metrics.memory.requests)}</span></div>
            <div class="flex justify-between"><span>Limits:</span> <span class="text-text-main">{formatMem(metrics.memory.limits)}</span></div>
            <div class="flex justify-between"><span>Capacity:</span> <span class="text-text-main">{formatMem(metrics.memory.capacity)}</span></div>
        </div>

        <!-- History Line -->
        <div class="flex-1 min-h-0 w-full">
            <Chart type="line" data={getLineData("Memory", historyMem, "#805AD5")} options={lineOptions} />
        </div>
      </div>

      <!-- Pods -->
      <div class="bg-bg-main rounded-lg border border-border-main p-4 flex flex-col h-80">
        <h3 class="text-lg font-semibold mb-4 text-center">Pods</h3>
        
         <!-- Gauge/Donut -->
        <div class="relative h-32 w-32 mx-auto mb-4">
           <Chart type="doughnut" data={getDoughnutData(metrics.pods.usage, 0, 0, metrics.pods.allocatable, metrics.pods.capacity, 'pods')} options={doughnutOptions} />
           <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
            <span class="text-xl font-bold">
              {metrics.pods.capacity > 0 ? Math.round((metrics.pods.usage / metrics.pods.capacity) * 100) : 0}%
            </span>
          </div>
        </div>

         <!-- Legend -->
        <div class="grid grid-cols-1 gap-y-1 text-xs text-text-muted mb-4 px-10">
            <div class="flex justify-between"><span>Usage:</span> <span class="text-text-main">{metrics.pods.usage}</span></div>
            <div class="flex justify-between"><span>Capacity:</span> <span class="text-text-main">{metrics.pods.capacity}</span></div>
        </div>
      </div>

    </div>

    <!-- Bottom Row: Events -->
    <div class="bg-bg-main rounded-lg border border-border-main flex flex-col overflow-hidden h-[400px]">
        <div class="p-4 border-b border-border-subtle flex items-center gap-2">
            <div class="w-2 h-2 rounded-full bg-amber-500 animate-pulse"></div>
            <h3 class="font-semibold">Warnings ({events.length})</h3>
        </div>
        <div class="flex-1 overflow-hidden p-4">
            <DataTable 
                data={events} 
                columns={eventColumns} 
                showSearch={true}
                storageKey="dashboard-events"
            >
             {#snippet children({ row: event, column })}
                {#if column.id === "type_"}
                    <span class="text-amber-500 font-medium">{event.type_}</span>
                {:else if column.id === "object"}
                    <span class="font-mono text-xs">{event.object}</span>
                {:else if column.id === "message"}
                    <span class="text-sm">{event.message}</span>
                {:else if column.id === "age"}
                    <span class="text-text-muted text-xs">{event.age}</span>
                {:else if column.id === "count"}
                    <span class="text-text-muted text-xs font-mono">{event.count}</span>
                {/if}
             {/snippet}
            </DataTable>
        </div>
    </div>
  {/if}
</div>
