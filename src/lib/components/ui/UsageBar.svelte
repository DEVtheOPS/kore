<script lang="ts">
  import { percentOf } from '$lib/utils/quantity';

  let {
    used,
    total,
    label = '',
    class: className = '',
  }: {
    /** Current usage in the same unit as `total`. */
    used: number | null | undefined;
    /** Capacity / allocatable in the same unit as `used`. */
    total: number | null | undefined;
    /** Pre-formatted text shown next to the bar (e.g. "250m / 4"). */
    label?: string;
    class?: string;
  } = $props();

  const pct = $derived(percentOf(used, total));

  const barClass = $derived.by(() => {
    if (pct == null) return 'bg-text-muted/40';
    if (pct >= 90) return 'bg-error';
    if (pct >= 75) return 'bg-warning';
    return 'bg-primary';
  });
</script>

<div class="flex items-center gap-2 min-w-[140px] {className}" title={pct == null ? 'Usage unavailable' : `${pct.toFixed(1)}%`}>
  <div class="flex-1 h-1.5 rounded-full bg-bg-main border border-border-subtle overflow-hidden">
    <div class="h-full {barClass} transition-all" style="width: {pct ?? 0}%"></div>
  </div>
  <span class="text-xs font-mono text-text-muted whitespace-nowrap">
    {#if pct == null}
      -
    {:else}
      {label || `${pct.toFixed(0)}%`}
    {/if}
  </span>
</div>
