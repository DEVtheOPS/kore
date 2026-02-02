<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { confirm } from "@tauri-apps/plugin-dialog";
  import { headerStore } from "$lib/stores/header.svelte";
  import { activeClusterStore } from "$lib/stores/activeCluster.svelte";
  import DataTable, { type Column } from "$lib/components/ui/DataTable.svelte";
  import type { MenuItem } from "$lib/components/ui/Menu.svelte";
  import { Trash2, Eye } from "lucide-svelte";
  import Drawer from "$lib/components/ui/Drawer.svelte";

  let { title, listCommand, deleteCommand } = $props<{
    title: string;
    listCommand: string;
    deleteCommand: string;
  }>();

  let data = $state<any[]>([]);
  let loading = $state(false);
  let search = $state("");
  
  // Detail Drawer state
  let showDrawer = $state(false);
  let selectedItem = $state<any>(null);

  const columns: Column[] = [
    { id: "name", label: "Name", sortable: true },
    { id: "namespace", label: "Namespace", sortable: true },
    { id: "status", label: "Status", sortable: true },
    { id: "images", label: "Images", sortable: true },
    { id: "age", label: "Age", sortable: true, sortKey: "created_at" },
  ];

  $effect(() => {
    headerStore.setTitle(title);
  });

  $effect(() => {
    if (activeClusterStore.clusterId) {
      loadData();
    }
  });

  async function loadData() {
    loading = true;
    try {
      data = await invoke(listCommand, {
        clusterId: activeClusterStore.clusterId,
        namespace: activeClusterStore.activeNamespace === "all" ? null : activeClusterStore.activeNamespace,
      });
    } catch (e) {
      console.error(`Failed to load ${title}`, e);
    } finally {
      loading = false;
    }
  }

  function handleRowClick(row: any) {
    selectedItem = row;
    showDrawer = true;
  }

  async function handleBatchDelete(selectedIds: any[]) {
    const itemsToDelete = data.filter((item) => selectedIds.includes(item.id));
    
    const confirmed = await confirm(
      `Are you sure you want to delete ${itemsToDelete.length} ${title}?`,
      { title: `Delete ${title}`, kind: "warning" }
    );

    if (confirmed) {
      let successCount = 0;
      for (const item of itemsToDelete) {
        try {
          await invoke(deleteCommand, {
            clusterId: activeClusterStore.clusterId,
            namespace: item.namespace,
            name: item.name,
          });
          successCount++;
        } catch (e) {
          console.error(`Failed to delete ${item.name}`, e);
        }
      }
      if (successCount > 0) {
        loadData();
      }
    }
  }

  function getActions(row: any): MenuItem[] {
    return [
      {
        label: "View Details",
        action: () => {
            selectedItem = row;
            showDrawer = true;
        },
        icon: Eye,
      },
      {
        label: "Delete",
        action: async () => {
          const confirmed = await confirm(
            `Are you sure you want to delete ${row.name}?`,
            { title: `Delete ${title.slice(0, -1)}`, kind: "warning" }
          );

          if (confirmed) {
            try {
              await invoke(deleteCommand, {
                clusterId: activeClusterStore.clusterId,
                namespace: row.namespace,
                name: row.name,
              });
              loadData();
            } catch (e) {
              console.error("Failed to delete", e);
            }
          }
        },
        icon: Trash2,
        danger: true,
      },
    ];
  }
</script>

<div class="h-full">
    <DataTable
        {data}
        {columns}
        bind:search
        {loading}
        onRefresh={loadData}
        actions={getActions}
        onRowClick={handleRowClick}
        batchActions={[
            {
                label: "Delete",
                icon: Trash2,
                danger: true,
                action: handleBatchDelete
            }
        ]}
        storageKey={`workload-${title.toLowerCase()}`}
    >
        {#snippet children({ row, column, value })}
            {#if column.id === "images"}
                <div class="flex flex-col gap-1">
                    {#if Array.isArray(value)}
                        {#each value.slice(0, 2) as img}
                            <span class="text-xs font-mono bg-bg-panel px-1 rounded truncate max-w-[200px]" title={img}>
                                {img.split('/').pop()}
                            </span>
                        {/each}
                        {#if value.length > 2}
                            <span class="text-xs text-text-muted">+{(value).length - 2} more</span>
                        {/if}
                    {/if}
                </div>
            {:else if column.id === "status"}
                 <span class="font-medium font-mono">{value}</span>
            {:else}
                {value}
            {/if}
        {/snippet}
    </DataTable>

    <Drawer bind:open={showDrawer} title={selectedItem?.name || "Details"}>
        <div class="p-4 space-y-4">
            <h3 class="font-bold">Details</h3>
            {#if selectedItem}
                <div class="grid grid-cols-2 gap-4 text-sm">
                    <div>
                        <span class="text-text-muted">Namespace:</span>
                        <br/>
                        <span class="font-mono">{selectedItem.namespace}</span>
                    </div>
                    <div>
                        <span class="text-text-muted">Age:</span>
                        <br/>
                        <span>{selectedItem.age}</span>
                    </div>
                    <div>
                        <span class="text-text-muted">Status:</span>
                        <br/>
                        <span class="font-mono">{selectedItem.status}</span>
                    </div>
                    <div>
                        <span class="text-text-muted">ID:</span>
                        <br/>
                        <span class="font-mono text-xs truncate block">{selectedItem.id}</span>
                    </div>
                </div>

                <div>
                    <span class="text-text-muted">Images:</span>
                    <ul class="list-disc list-inside font-mono text-xs mt-1">
                        {#each selectedItem.images as img}
                            <li>{img}</li>
                        {/each}
                    </ul>
                </div>

                <div>
                    <span class="text-text-muted">Labels:</span>
                    <div class="flex flex-wrap gap-1 mt-1">
                        {#each Object.entries(selectedItem.labels) as [k, v]}
                            <span class="px-2 py-0.5 bg-bg-main border border-border-main rounded text-xs font-mono">
                                {k}: {v}
                            </span>
                        {/each}
                    </div>
                </div>
            {/if}
        </div>
    </Drawer>
</div>
