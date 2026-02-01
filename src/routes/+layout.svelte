<script lang="ts">
  import "./layout.css";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import BottomDrawer from "$lib/components/BottomDrawer.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";

  let { children } = $props();

  $effect(() => {
    if (typeof document !== "undefined") {
      const root = document.documentElement;
      root.classList.remove("rusty", "rusty-light", "dracula", "alucard", "kore", "kore-light");
      root.classList.add(settingsStore.value.theme);
    }
  });
</script>

<div class="flex h-screen w-screen bg-bg-main text-text-main overflow-hidden">
  <!-- Sidebar -->
  <Sidebar />

  <!-- Main content area -->
  <main class="flex-1 flex flex-col h-full overflow-hidden">
    <!-- Content Area -->
    <div class="flex-1 overflow-auto p-6 bg-bg-panel">
      {@render children()}
    </div>

    <!-- Global Bottom Drawer -->
    <BottomDrawer />
  </main>
</div>
