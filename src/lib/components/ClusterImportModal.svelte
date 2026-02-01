<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import Button from "$lib/components/ui/Button.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import { X, FileText, Folder, Loader2 } from "lucide-svelte";
  import { clustersStore } from "$lib/stores/clusters.svelte";

  let { isOpen = $bindable(), onClose } = $props<{
    isOpen: boolean;
    onClose: () => void;
  }>();

  let activeTab = $state<"file" | "folder">("file");
  let loading = $state(false);
  let error = $state<string | null>(null);

  interface DiscoveredContext {
    context_name: string;
    cluster_name: string;
    file_path: string;
    display_name: string;
    icon: string;
  }

  let discoveredContexts = $state<DiscoveredContext[]>([]);
  let selectedContexts = $state<Set<string>>(new Set());

  async function handleImportFile() {
    loading = true;
    error = null;
    discoveredContexts = [];
    selectedContexts = new Set();

    try {
      const selected = await open({
        multiple: false,
        title: "Select Kubeconfig File",
      });

      if (!selected) {
        loading = false;
        return;
      }

      const contexts = await invoke<DiscoveredContext[]>("import_discover_file", {
        path: selected,
      });

      discoveredContexts = contexts.map((ctx) => ({
        ...ctx,
        display_name: ctx.context_name,
        icon: "🌐",
      }));

      // Select all by default
      contexts.forEach((ctx) => selectedContexts.add(ctx.context_name));
    } catch (e) {
      error = `Failed to import file: ${e}`;
      console.error(e);
    } finally {
      loading = false;
    }
  }

  async function handleImportFolder() {
    loading = true;
    error = null;
    discoveredContexts = [];
    selectedContexts = new Set();

    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Folder with Kubeconfig Files",
      });

      if (!selected) {
        loading = false;
        return;
      }

      const contexts = await invoke<DiscoveredContext[]>("import_discover_folder", {
        path: selected,
      });

      discoveredContexts = contexts.map((ctx) => ({
        ...ctx,
        display_name: ctx.context_name,
        icon: "🌐",
      }));

      // Select all by default
      contexts.forEach((ctx) => selectedContexts.add(ctx.context_name));
    } catch (e) {
      error = `Failed to import folder: ${e}`;
      console.error(e);
    } finally {
      loading = false;
    }
  }

  function toggleContext(contextName: string) {
    if (selectedContexts.has(contextName)) {
      selectedContexts.delete(contextName);
    } else {
      selectedContexts.add(contextName);
    }
    selectedContexts = new Set(selectedContexts); // Trigger reactivity
  }

  function updateDisplayName(contextName: string, newName: string) {
    const ctx = discoveredContexts.find((c) => c.context_name === contextName);
    if (ctx) {
      ctx.display_name = newName;
    }
  }

  function updateIcon(contextName: string, newIcon: string) {
    const ctx = discoveredContexts.find((c) => c.context_name === contextName);
    if (ctx) {
      ctx.icon = newIcon;
    }
  }

  async function handleImportSelected() {
    loading = true;
    error = null;

    try {
      const toImport = discoveredContexts.filter((ctx) =>
        selectedContexts.has(ctx.context_name)
      );

      for (const ctx of toImport) {
        await invoke("import_add_cluster", {
          name: ctx.display_name,
          contextName: ctx.context_name,
          filePath: ctx.file_path,
          icon: ctx.icon !== "🌐" ? ctx.icon : null,
        });
      }

      await clustersStore.load();
      onClose();
    } catch (e) {
      error = `Failed to import clusters: ${e}`;
      console.error(e);
    } finally {
      loading = false;
    }
  }

  function handleClose() {
    discoveredContexts = [];
    selectedContexts = new Set();
    error = null;
    onClose();
  }
</script>

{#if isOpen}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 bg-black/50 z-50 flex items-center justify-center"
    onclick={handleClose}
  >
    <!-- Modal -->
    <div
      class="bg-bg-main rounded-lg shadow-xl w-full max-w-3xl max-h-[80vh] flex flex-col"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="flex items-center justify-between p-4 border-b border-border-main">
        <h2 class="text-lg font-semibold">Import Clusters</h2>
        <button
          onclick={handleClose}
          class="p-1 hover:bg-bg-panel rounded transition-colors"
        >
          <X size={20} />
        </button>
      </div>

      <!-- Tabs -->
      <div class="flex border-b border-border-main">
        <button
          onclick={() => {
            activeTab = "file";
            discoveredContexts = [];
            selectedContexts = new Set();
            error = null;
          }}
          class="px-4 py-2 font-medium transition-colors"
          class:border-b-2={activeTab === "file"}
          class:border-primary={activeTab === "file"}
          class:text-primary={activeTab === "file"}
          class:text-text-muted={activeTab !== "file"}
        >
          <div class="flex items-center gap-2">
            <FileText size={16} />
            Import from File
          </div>
        </button>
        <button
          onclick={() => {
            activeTab = "folder";
            discoveredContexts = [];
            selectedContexts = new Set();
            error = null;
          }}
          class="px-4 py-2 font-medium transition-colors"
          class:border-b-2={activeTab === "folder"}
          class:border-primary={activeTab === "folder"}
          class:text-primary={activeTab === "folder"}
          class:text-text-muted={activeTab !== "folder"}
        >
          <div class="flex items-center gap-2">
            <Folder size={16} />
            Import from Folder
          </div>
        </button>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto p-4">
        {#if activeTab === "file"}
          <div class="space-y-4">
            <p class="text-text-muted text-sm">
              Select a kubeconfig file to import. If it contains multiple contexts, you can
              choose which ones to import.
            </p>

            {#if discoveredContexts.length === 0}
              <Button onclick={handleImportFile} disabled={loading}>
                {#if loading}
                  <Loader2 size={16} class="animate-spin" />
                  Loading...
                {:else}
                  Select File
                {/if}
              </Button>
            {/if}
          </div>
        {:else}
          <div class="space-y-4">
            <p class="text-text-muted text-sm">
              Select a folder to scan for kubeconfig files. All discovered contexts will be
              listed below.
            </p>

            {#if discoveredContexts.length === 0}
              <Button onclick={handleImportFolder} disabled={loading}>
                {#if loading}
                  <Loader2 size={16} class="animate-spin" />
                  Scanning...
                {:else}
                  Select Folder
                {/if}
              </Button>
            {/if}
          </div>
        {/if}

        {#if error}
          <div class="mt-4 p-3 bg-red-500/10 border border-red-500/20 rounded text-red-400 text-sm">
            {error}
          </div>
        {/if}

        {#if discoveredContexts.length > 0}
          <div class="mt-6 space-y-3">
            <h3 class="font-semibold">
              Discovered Contexts ({selectedContexts.size} selected)
            </h3>

            {#each discoveredContexts as ctx (ctx.context_name)}
              <div
                class="p-3 border rounded transition-colors"
                class:border-primary={selectedContexts.has(ctx.context_name)}
                class:border-border-main={!selectedContexts.has(ctx.context_name)}
                style={selectedContexts.has(ctx.context_name) ? "background-color: hsl(var(--primary) / 0.05)" : ""}
              >
                <div class="flex items-start gap-3">
                  <input
                    type="checkbox"
                    checked={selectedContexts.has(ctx.context_name)}
                    onchange={() => toggleContext(ctx.context_name)}
                    class="mt-1"
                  />

                  <div class="flex-1 space-y-2">
                    <div class="flex items-center gap-2">
                      <Input
                        value={ctx.icon}
                        oninput={(e) => updateIcon(ctx.context_name, (e.currentTarget as HTMLInputElement).value)}
                        placeholder="Icon (emoji or URL)"
                        class="w-16 text-center"
                      />
                      <Input
                        value={ctx.display_name}
                        oninput={(e) => updateDisplayName(ctx.context_name, (e.currentTarget as HTMLInputElement).value)}
                        placeholder="Display name"
                        class="flex-1"
                      />
                    </div>

                    <div class="text-xs text-text-muted">
                      <div>Context: {ctx.context_name}</div>
                      <div>Cluster: {ctx.cluster_name}</div>
                      <div class="truncate">File: {ctx.file_path}</div>
                    </div>
                  </div>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Footer -->
      {#if discoveredContexts.length > 0}
        <div class="flex items-center justify-end gap-2 p-4 border-t border-border-main">
          <Button variant="outline" onclick={handleClose}>Cancel</Button>
          <Button
            onclick={handleImportSelected}
            disabled={loading || selectedContexts.size === 0}
          >
            {#if loading}
              <Loader2 size={16} class="animate-spin" />
              Importing...
            {:else}
              Import {selectedContexts.size} Cluster{selectedContexts.size !== 1 ? "s" : ""}
            {/if}
          </Button>
        </div>
      {/if}
    </div>
  </div>
{/if}
