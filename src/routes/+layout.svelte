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

{#if lockStore.initializing}
  <!-- Initializing overlay — shown before lock state is determined.
       Children are NOT mounted yet, preventing any data load before auth. -->
  <div class="fixed inset-0 z-[100] flex items-center justify-center bg-bg-main text-text-main">
    <div class="text-sm text-text-muted">Preparing secure workspace...</div>
  </div>
{:else if lockStore.locked}
  <!-- Lock screen replaces the entire UI — children are unmounted.
       This prevents all IPC data commands from firing while locked,
       and cannot be bypassed by manipulating frontend state alone. -->
  <LockScreen />
{:else}
  <div class="flex h-screen w-screen bg-bg-main text-text-main overflow-hidden">
    <!-- Icon Sidebar -->
    <IconSidebar onAddCluster={openImportModal} />

    <!-- Content Area (filled by nested layouts/pages) -->
    <div class="flex-1 overflow-hidden">
      {@render children()}
    </div>
  </div>
{/if}

<!-- Import Modal -->
<ClusterImportModal bind:isOpen={importModalOpen} onClose={closeImportModal} />
