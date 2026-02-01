<script lang="ts">
  import type { Snippet } from "svelte";
  import { ChevronDown, Check } from "lucide-svelte";
  import Button from "./Button.svelte";

  interface Props {
    value?: string;
    options: string[];
    onselect?: (value: string) => void;
    placeholder?: string;
    class?: string;
    id?: string;
    align?: "left" | "center" | "right";
  }

  let {
    value = $bindable(""),
    options,
    onselect,
    placeholder = "Select...",
    class: className = "",
    id,
    align = "left",
  }: Props = $props();

  const alignClass = {
    left: "text-left justify-start",
    center: "text-center justify-center",
    right: "text-right justify-end",
  };

  let isOpen = $state(false);
  let containerRef: HTMLElement;

  function toggle() {
    isOpen = !isOpen;
  }

  function select(opt: string) {
    value = opt;
    onselect?.(opt);
    isOpen = false;
  }

  function handleClickOutside(event: MouseEvent) {
    if (isOpen && containerRef && !containerRef.contains(event.target as Node)) {
      isOpen = false;
    }
  }
</script>

<svelte:window onclick={handleClickOutside} />

<div class="relative inline-block w-full {className}" bind:this={containerRef}>
  <Button variant="secondary" class="w-full flex items-center {alignClass[align]}" onclick={toggle} {id}>
    <span class="truncate flex-1 {align === 'left' ? 'text-left' : ''}">{value || placeholder}</span>
    <ChevronDown size={16} class="ml-2 opacity-50 flex-shrink-0" />
  </Button>

  {#if isOpen}
    <div
      class="absolute z-50 mt-1 max-h-60 w-full overflow-auto rounded-md border border-border-subtle bg-bg-popover shadow-lg"
    >
      <div class="p-1">
        {#each options as option}
          <button
            class="flex w-full items-center justify-between rounded-sm px-2 py-1.5 text-sm hover:bg-bg-panel text-text-main"
            onclick={() => select(option)}
          >
            <span class="truncate">{option}</span>
            {#if value === option}
              <Check size={14} class="text-primary" />
            {/if}
          </button>
        {/each}
        {#if options.length === 0}
          <div class="px-2 py-1.5 text-sm text-text-muted">No options</div>
        {/if}
      </div>
    </div>
  {/if}
</div>
