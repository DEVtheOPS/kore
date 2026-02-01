<script lang="ts">
  import "./layout.css";
  import TitleBar from "$lib/components/TitleBar.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import BottomDrawer from "$lib/components/BottomDrawer.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { headerStore } from "$lib/stores/header.svelte";

  let { children } = $props();

  $effect(() => {
    if (typeof document !== "undefined") {
      const root = document.documentElement;
      root.classList.remove("rusty", "rusty-light", "dracula", "alucard", "kore", "kore-light");
      root.classList.add(settingsStore.value.theme);
    }
  });
</script>

<div class="flex flex-col h-screen w-screen bg-bg-main text-text-main overflow-hidden">
  <!-- Custom Title Bar -->
  <TitleBar />

  <!-- Main Layout -->
  <div class="flex flex-1 overflow-hidden">
    <!-- Sidebar is fixed to ensure z-index priority and no layout shifting -->
    <div class="fixed inset-y-0 left-0 w-64 z-50">
      <Sidebar />
    </div>

    <!-- Main content has left margin to accommodate fixed sidebar -->
    <main class="flex-1 flex flex-col h-full overflow-hidden ml-64">
    <!-- Top Bar -->
    <header class="h-14 border-b border-border-subtle flex items-center justify-between px-6 bg-bg-main">
      <h2 class="font-semibold text-lg">{headerStore.title}</h2>
    </header>

    <!-- Content Area -->
    <div class="flex-1 overflow-auto p-6 bg-bg-panel">
      {@render children()}
    </div>

    <!-- Global Bottom Drawer -->
    <BottomDrawer />
  </main>
  </div>
</div>
