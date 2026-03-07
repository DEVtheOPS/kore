<script lang="ts">
  import "./layout.css";
  import IconSidebar from "$lib/components/IconSidebar.svelte";
  import ClusterImportModal from "$lib/components/ClusterImportModal.svelte";
  import LockScreen from "$lib/components/LockScreen.svelte";
  import { lockStore } from "$lib/stores/lock.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { onMount } from "svelte";

  let { children } = $props();

  let importModalOpen = $state(false);

  onMount(async () => {
    await settingsStore.load();
    await lockStore.init();
  });

  $effect(() => {
    if (typeof document !== "undefined") {
      const root = document.documentElement;
      root.classList.remove("rusty", "rusty-light", "dracula", "alucard", "kore", "kore-light");
      root.classList.add(settingsStore.value.theme);
    }
  });

  function openImportModal() {
    importModalOpen = true;
  }

  function closeImportModal() {
    importModalOpen = false;
  }
</script>

<div class="flex h-screen w-screen bg-bg-main text-text-main overflow-hidden">
  <!-- Icon Sidebar -->
  <IconSidebar onAddCluster={openImportModal} />

  <!-- Content Area (filled by nested layouts/pages) -->
  <div class="flex-1 overflow-hidden">
    {@render children()}
  </div>
</div>

{#if lockStore.initializing}
  <div class="fixed inset-0 z-[100] flex items-center justify-center bg-bg-main text-text-main">
    <div class="text-sm text-text-muted">Preparing secure workspace...</div>
  </div>
{:else if lockStore.locked}
  <LockScreen />
{/if}

<!-- Import Modal -->
<ClusterImportModal bind:isOpen={importModalOpen} onClose={closeImportModal} />
