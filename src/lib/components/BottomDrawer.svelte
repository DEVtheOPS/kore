<script lang="ts">
  import { X, ChevronDown, ChevronUp } from 'lucide-svelte';
  import { bottomDrawerStore } from '$lib/stores/bottomDrawer.svelte';
  import LogsTab from './tabs/LogsTab.svelte';

  let drawerHeight = $state(400);
  let isResizing = $state(false);
  let startY = $state(0);
  let startHeight = $state(0);

  function startResize(e: MouseEvent) {
    isResizing = true;
    startY = e.clientY;
    startHeight = drawerHeight;
    e.preventDefault();
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isResizing) return;
    const deltaY = startY - e.clientY;
    drawerHeight = Math.max(200, Math.min(window.innerHeight - 100, startHeight + deltaY));
  }

  function handleMouseUp() {
    isResizing = false;
  }

  $effect(() => {
    if (isResizing) {
      window.addEventListener('mousemove', handleMouseMove);
      window.addEventListener('mouseup', handleMouseUp);
      return () => {
        window.removeEventListener('mousemove', handleMouseMove);
        window.removeEventListener('mouseup', handleMouseUp);
      };
    }
  });
</script>

{#if bottomDrawerStore.open}
  <div 
    class="fixed bottom-0 left-0 right-0 bg-bg-main border-t border-border z-50 flex flex-col"
    style="height: {drawerHeight}px;"
  >
    <!-- Resize Handle -->
    <div 
      class="h-1 bg-border hover:bg-primary cursor-ns-resize transition-colors"
      onmousedown={startResize}
      role="separator"
      aria-orientation="horizontal"
    ></div>

    <!-- Tabs Bar -->
    <div class="flex items-center justify-between border-b border-border bg-bg-panel px-2">
      <div class="flex items-center gap-1 overflow-x-auto flex-1">
        {#each bottomDrawerStore.tabs as tab}
          <div
            class="px-3 py-2 text-sm flex items-center gap-2 hover:bg-bg-main transition-colors cursor-pointer {bottomDrawerStore.activeTabId === tab.id ? 'bg-bg-main border-b-2 border-primary' : ''}"
            onclick={() => bottomDrawerStore.setActiveTab(tab.id)}
            role="tab"
            tabindex="0"
          >
            <span>{tab.title}</span>
            <button
              class="hover:bg-error/20 rounded p-0.5"
              onclick={(e) => {
                e.stopPropagation();
                bottomDrawerStore.closeTab(tab.id);
              }}
            >
              <X size={14} />
            </button>
          </div>
        {/each}
      </div>
      
      <div class="flex items-center gap-2 ml-2">
        <button
          class="p-1 hover:bg-bg-main rounded transition-colors"
          onclick={() => bottomDrawerStore.toggle()}
          title={bottomDrawerStore.open ? 'Minimize' : 'Maximize'}
        >
          {#if bottomDrawerStore.open}
            <ChevronDown size={18} />
          {:else}
            <ChevronUp size={18} />
          {/if}
        </button>
      </div>
    </div>

    <!-- Tab Content -->
    <div class="flex-1 overflow-hidden">
      {#if bottomDrawerStore.activeTab}
        {#if bottomDrawerStore.activeTab.type === 'logs'}
          <LogsTab data={bottomDrawerStore.activeTab.data} />
        {:else if bottomDrawerStore.activeTab.type === 'edit'}
          <div class="p-4">Edit functionality coming soon...</div>
        {:else}
          <div class="p-4">Unknown tab type</div>
        {/if}
      {/if}
    </div>
  </div>
{/if}

<style>
  button {
    user-select: none;
  }
</style>
