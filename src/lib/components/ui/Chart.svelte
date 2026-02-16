<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import {
    Chart,
    type ChartConfiguration,
    type ChartType,
    LineController,
    LineElement,
    PointElement,
    LinearScale,
    CategoryScale,
    Title,
    Tooltip,
    Legend,
    Filler
  } from 'chart.js';

  // Register Chart.js components
  Chart.register(
    LineController,
    LineElement,
    PointElement,
    LinearScale,
    CategoryScale,
    Title,
    Tooltip,
    Legend,
    Filler
  );

  let { type, data, options } = $props<{
    type: ChartType;
    data: ChartConfiguration['data'];
    options?: ChartConfiguration['options'];
  }>();

  let canvas = $state<HTMLCanvasElement | null>(null);
  let chart: Chart | null = null;

  onMount(() => {
    if (canvas) {
      chart = new Chart(canvas, {
        type,
        data,
        options
      });
    }
  });

  onDestroy(() => {
    if (chart) {
      chart.destroy();
      chart = null;
    }
  });

  // Reactively update chart when data/options change
  $effect(() => {
    if (chart && data) {
      chart.data = data;
      if (options) {
        chart.options = options;
      }
      chart.update();
    }
  });
</script>

<div class="relative w-full h-full">
  <canvas bind:this={canvas}></canvas>
</div>
