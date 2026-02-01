<script lang="ts">
  import { Minus, Square, X } from "lucide-svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  const appWindow = getCurrentWindow();

  async function minimize() {
    await appWindow.minimize();
  }

  async function toggleMaximize() {
    await appWindow.toggleMaximize();
  }

  async function close() {
    await appWindow.close();
  }
</script>

<!-- Title bar with drag region -->
<div
  data-tauri-drag-region
  class="h-8 bg-bg-sidebar border-b border-border flex items-center justify-between px-3 select-none"
>
  <!-- App title (draggable area) -->
  <div data-tauri-drag-region class="flex items-center gap-2 flex-1">
    <img src="/kore.svg" alt="Kore" class="w-4 h-4" />
    <span class="text-sm text-text-muted">Kore</span>
  </div>

  <!-- Window controls (not draggable) -->
  <div class="flex items-center gap-1">
    <button
      onclick={minimize}
      class="p-1 hover:bg-bg-main rounded transition-colors"
      title="Minimize"
    >
      <Minus size={14} />
    </button>
    <button
      onclick={toggleMaximize}
      class="p-1 hover:bg-bg-main rounded transition-colors"
      title="Maximize"
    >
      <Square size={14} />
    </button>
    <button
      onclick={close}
      class="p-1 hover:bg-red-500 hover:text-white rounded transition-colors"
      title="Close"
    >
      <X size={14} />
    </button>
  </div>
</div>
