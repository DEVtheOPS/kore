<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    class?: string;
    children?: Snippet;
    hover?: boolean;
    padding?: string;
  }

  let { 
    class: className = '', 
    children, 
    hover = false,
    padding = 'p-4'
  }: Props = $props();
</script>

<div
  class="
    bg-bg-card border border-border-subtle rounded-card shadow-sm
    {hover ? 'hover:border-border-main transition-colors' : ''}
    {padding} {className}
  "
>
  {@render children?.()}
</div>
