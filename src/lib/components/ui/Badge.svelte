<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    variant?: 'success' | 'warning' | 'error' | 'info' | 'neutral';
    class?: string;
    children?: Snippet;
  }

  let { 
    variant = 'neutral', 
    class: className = '', 
    children 
  }: Props = $props();

  const variants = {
    success: 'bg-success/20 text-success border-success/30',
    warning: 'bg-warning/20 text-warning border-warning/30',
    error: 'bg-error/20 text-error border-error/30',
    info: 'bg-info/20 text-info border-info/30',
    neutral: 'bg-bg-panel text-text-muted border-border-main',
  };
</script>

<span
  class="
    inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold border
    {variants[variant]} {className}
  "
>
  {@render children?.()}
</span>
