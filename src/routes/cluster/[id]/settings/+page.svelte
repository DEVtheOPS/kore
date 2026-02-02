<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { confirm, open } from "@tauri-apps/plugin-dialog";
  import { headerStore } from "$lib/stores/header.svelte";
  import { clustersStore, type Cluster } from "$lib/stores/clusters.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import { Trash2, Save, Upload, Image as ImageIcon } from "lucide-svelte";

  const clusterId = $derived($page.params.id);
  
  let cluster = $state<Cluster | null>(null);
  let loading = $state(true);
  let saving = $state(false);

  // Form fields
  let name = $state("");
  let icon = $state("");
  let description = $state("");
  let tags = $state<string[]>([]);
  let tagInput = $state("");

  $effect(() => {
    headerStore.setTitle("Cluster Settings");
  });

  $effect(() => {
    if (clusterId) {
      loadCluster();
    }
  });

  async function loadCluster() {
    if (!clusterId) {
      loading = false;
      return;
    }

    loading = true;
    try {
      cluster = await clustersStore.get(clusterId);
      if (cluster) {
        name = cluster.name;
        icon = cluster.icon || "";
        description = cluster.description || "";
        tags = clustersStore.getTags(cluster);
      }
    } catch (e) {
      console.error("Failed to load cluster", e);
    } finally {
      loading = false;
    }
  }

  async function handleSave() {
    if (!cluster) return;

    saving = true;
    try {
      await clustersStore.update(cluster.id, {
        name: name || undefined,
        icon: icon || null,
        description: description || null,
        tags: tags.length > 0 ? tags : [],
      });

      // Reload cluster to get updated data
      await loadCluster();
    } catch (e) {
      console.error("Failed to update cluster", e);
    } finally {
      saving = false;
    }
  }

  async function handleDelete() {
    if (!cluster) return;

    const confirmed = await confirm(
      `Are you sure you want to delete cluster "${cluster.name}"? This will remove the cluster configuration permanently.`,
      { title: "Delete Cluster", kind: "warning" }
    );

    if (confirmed) {
      try {
        await clustersStore.remove(cluster.id);
        goto("/");
      } catch (e) {
        console.error("Failed to delete cluster", e);
      }
    }
  }

  function handleAddTag() {
    if (tagInput.trim() && !tags.includes(tagInput.trim())) {
      tags = [...tags, tagInput.trim()];
      tagInput = "";
    }
  }

  function handleRemoveTag(tag: string) {
    tags = tags.filter((t) => t !== tag);
  }

  function handleTagInputKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      handleAddTag();
    }
  }

  async function handleIconFileSelect() {
    try {
      const selected = await open({
        multiple: false,
        title: "Select Cluster Icon",
        filters: [{
          name: "Images",
          extensions: ["png", "jpg", "jpeg", "gif", "webp", "bmp", "ico"]
        }]
      });

      if (!selected) return;

      // Process the image (resize and convert to base64 PNG)
      const processedIcon = await invoke<string>("process_icon_file", {
        path: selected
      });

      icon = processedIcon;
    } catch (e) {
      console.error("Failed to process icon:", e);
    }
  }
</script>

{#if loading}
  <div class="flex items-center justify-center h-full">
    <div class="text-text-muted">Loading cluster settings...</div>
  </div>
{:else if !cluster}
  <div class="flex items-center justify-center h-full">
    <div class="text-center space-y-2">
      <h2 class="text-xl font-semibold">Cluster Not Found</h2>
      <p class="text-text-muted">The cluster you're looking for doesn't exist.</p>
      <a href="/" class="text-primary hover:underline">Go to Overview</a>
    </div>
  </div>
{:else}
  <div class="max-w-3xl space-y-6">
    <!-- General Settings -->
    <Card>
      <div class="p-6 space-y-4">
        <h2 class="text-lg font-semibold">General Settings</h2>

        <!-- Name -->
        <div>
          <label for="name" class="block text-sm font-medium mb-2">Cluster Name</label>
          <Input
            id="name"
            bind:value={name}
            placeholder="My Kubernetes Cluster"
            class="w-full"
          />
        </div>

        <!-- Icon -->
        <div>
          <label for="icon" class="block text-sm font-medium mb-2">
            Icon
          </label>
          <div class="flex gap-2 items-start">
            <!-- Icon Preview -->
            <div class="flex items-center justify-center w-16 h-16 border border-border-main rounded bg-bg-panel overflow-hidden flex-shrink-0">
              {#if icon}
                {#if icon.startsWith("data:image") || icon.startsWith("http")}
                  <img src={icon} alt="Icon" class="w-full h-full object-contain" />
                {:else}
                  <span class="text-3xl">{icon}</span>
                {/if}
              {:else}
                <ImageIcon size={24} class="text-text-muted" />
              {/if}
            </div>
            
            <div class="flex-1 space-y-2">
              <!-- File Browse Button -->
              <Button onclick={handleIconFileSelect} variant="outline">
                <Upload size={16} />
                Upload Image
              </Button>
              
              <!-- Manual Input (for emoji or URL) -->
              <Input
                id="icon"
                bind:value={icon}
                placeholder="🌐 or paste image URL"
                class="w-full"
              />
              <p class="text-xs text-text-muted">
                Upload an image file (auto-resized to 512x512) or enter an emoji/URL
              </p>
            </div>
          </div>
        </div>

        <!-- Description -->
        <div>
          <label for="description" class="block text-sm font-medium mb-2">Description</label>
          <Input
            id="description"
            bind:value={description}
            placeholder="Production cluster in US East"
            class="w-full"
          />
        </div>

        <!-- Tags -->
        <div>
          <label for="tags" class="block text-sm font-medium mb-2">Tags</label>
          <div class="flex gap-2 mb-2">
            <Input
              id="tags"
              bind:value={tagInput}
              onkeydown={handleTagInputKeydown}
              placeholder="Add a tag..."
              class="flex-1"
            />
            <Button onclick={handleAddTag}>Add</Button>
          </div>
          <div class="flex flex-wrap gap-2">
            {#each tags as tag}
              <span
                class="inline-flex items-center gap-1 px-3 py-1 bg-bg-panel border border-border-main rounded"
              >
                {tag}
                <button
                  onclick={() => handleRemoveTag(tag)}
                  class="text-text-muted hover:text-text-main"
                >
                  ×
                </button>
              </span>
            {/each}
          </div>
        </div>

        <!-- Context Info (read-only) -->
        <div class="pt-4 border-t border-border-subtle">
          <h3 class="text-sm font-medium mb-2 text-text-muted">Configuration</h3>
          <div class="space-y-1 text-sm">
            <div class="flex justify-between">
              <span class="text-text-muted">Context Name:</span>
              <span class="font-mono">{cluster.context_name}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-text-muted">Cluster ID:</span>
              <span class="font-mono text-xs">{cluster.id}</span>
            </div>
          </div>
        </div>

        <!-- Save Button -->
        <div class="pt-4">
          <Button onclick={handleSave} disabled={saving}>
            <Save size={16} />
            {saving ? "Saving..." : "Save Changes"}
          </Button>
        </div>
      </div>
    </Card>

    <!-- Advanced Settings -->
    <Card>
      <div class="p-6 space-y-4">
        <h2 class="text-lg font-semibold">Advanced Settings</h2>

        <div class="space-y-2">
          <h3 class="text-sm font-medium text-text-muted">Proxy Settings</h3>
          <p class="text-sm text-text-muted">Coming soon</p>
        </div>

        <div class="space-y-2">
          <h3 class="text-sm font-medium text-text-muted">Terminal Settings</h3>
          <p class="text-sm text-text-muted">Coming soon</p>
        </div>
      </div>
    </Card>

    <!-- Danger Zone -->
    <Card>
      <div class="p-6 space-y-4">
        <h2 class="text-lg font-semibold text-red-400">Danger Zone</h2>

        <div class="flex items-start justify-between gap-4 p-4 border border-red-500/20 rounded bg-red-500/5">
          <div>
            <h3 class="font-medium text-red-400">Delete Cluster</h3>
            <p class="text-sm text-text-muted mt-1">
              Permanently remove this cluster and its configuration. This action cannot be undone.
            </p>
          </div>
          <Button variant="outline" onclick={handleDelete}>
            <Trash2 size={16} />
            Delete
          </Button>
        </div>
      </div>
    </Card>
  </div>
{/if}
