<script lang="ts">
  import { EllipsisVertical } from 'lucide-svelte';
  import type { Snippet } from 'svelte';

  export interface MenuItem {
    label: string;
    action: () => void;
    danger?: boolean;
    icon?: any;
  }

  interface Props {
    items: MenuItem[];
    align?: "left" | "right";
  }

  let { items, align = "right" }: Props = $props();
  let isOpen = $state(false);
  let menuRef: HTMLElement | undefined = $state();
  let buttonRef: HTMLElement | undefined = $state();
  let pos = $state<{ top: number; left?: number; right?: number }>({ top: 0, right: 0 });

  function updatePosition() {
    if (buttonRef) {
      const rect = buttonRef.getBoundingClientRect();
      
      if (align === "left") {
        pos = {
          top: rect.bottom + 4,
          left: rect.left,
          right: undefined
        };
      } else {
        pos = {
          top: rect.bottom + 4,
          right: window.innerWidth - rect.right,
          left: undefined
        };
      }
    }
  }

  function toggle(e: MouseEvent) {
    e.stopPropagation(); // Prevent row click
    if (!isOpen) {
      updatePosition();
    }
    isOpen = !isOpen;
  }

  function handleSelect(item: MenuItem, e: MouseEvent) {
    e.stopPropagation();
    // Close first, then execute action to allow UI updates (like confirm dialogs) to render cleanly
    isOpen = false;
    // Use setTimeout to defer action slightly to ensure menu close is processed
    setTimeout(() => {
        item.action();
    }, 0);
  }

  function handleClickOutside(event: MouseEvent) {
    const target = event.target as Node;
    if (
      isOpen && 
      menuRef && !menuRef.contains(target) && 
      buttonRef && !buttonRef.contains(target)
    ) {
      isOpen = false;
    }
  }

  function handleScroll() {
    if (isOpen) isOpen = false;
  }
</script>

<svelte:window onclick={handleClickOutside} onscroll={handleScroll} onresize={handleScroll} />

<div class="inline-block">
  <button 
    bind:this={buttonRef}
    class="p-1 hover:bg-bg-popover rounded-md text-text-muted hover:text-text-main transition-colors"
    onclick={toggle}
  >
    <EllipsisVertical size={16} />
  </button>

  {#if isOpen}
    <div 
      bind:this={menuRef}
      class="fixed w-40 bg-bg-popover border border-border-subtle rounded-md shadow-lg z-[9999] py-1"
      style="top: {pos.top}px; {pos.left !== undefined ? `left: ${pos.left}px` : `right: ${pos.right}px`};"
    >
      {#each items as item}
        <button
          class="w-full text-left px-3 py-2 text-sm hover:bg-bg-panel flex items-center gap-2
            {item.danger ? 'text-error hover:bg-error/10' : 'text-text-main'}"
          onclick={(e) => handleSelect(item, e)}
        >
          {#if item.icon}
            <item.icon size={14} />
          {/if}
          {item.label}
        </button>
      {/each}
    </div>
  {/if}
</div>
