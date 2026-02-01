<script lang="ts">
  import type { Snippet } from 'svelte';
  import { ChevronDown, ChevronRight } from 'lucide-svelte';

  interface Props {
    title: string;
    icon?: any;
    open?: boolean;
    children: Snippet;
  }

  let { title, icon: Icon, open = $bindable(false), children }: Props = $props();

  function toggle() {
    open = !open;
  }
</script>

<div>
  <button 
    class="flex items-center w-full gap-3 px-3 py-2 rounded-md hover:bg-bg-popover text-sm group text-text-muted hover:text-text-main transition-colors select-none"
    onclick={toggle}
  >
    {#if Icon}
      <Icon size={18} class="group-hover:text-primary transition-colors" />
    {/if}
    <span class="flex-1 text-left font-medium">{title}</span>
    {#if open}
      <ChevronDown size={14} />
    {:else}
      <ChevronRight size={14} />
    {/if}
  </button>

  {#if open}
    <div class="ml-4 pl-3 border-l border-border-subtle mt-1 space-y-1">
      {@render children()}
    </div>
  {/if}
</div>
