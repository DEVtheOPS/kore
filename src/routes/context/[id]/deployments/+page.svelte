<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { confirm } from "@tauri-apps/plugin-dialog";
  import { headerStore } from "$lib/stores/header.svelte";
  import { activeClusterStore } from "$lib/stores/activeCluster.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import DataTable, { type Column } from "$lib/components/ui/DataTable.svelte";
  import Drawer from "$lib/components/ui/Drawer.svelte";
  import type { MenuItem } from "$lib/components/ui/Menu.svelte";
  import { Trash2, Eye, FilePenLine, Scaling, RotateCw, Save } from "lucide-svelte";
  import DeploymentDetailDrawer from "$lib/components/DeploymentDetailDrawer.svelte";

  let data = $state<any[]>([]);
  let loading = $state(false);
  let search = $state("");
  let error = $state<string | null>(null);

  // Detail Drawer state
  let showDrawer = $state(false);
  let selectedDeployment = $state({
    name: '',
    namespace: '',
  });
  let showYamlDrawer = $state(false);
  let yamlContent = $state("");
  let yamlTarget = $state<{ name: string; namespace: string } | null>(null);
  let loadingYaml = $state(false);
  let applyingYaml = $state(false);

  const columns: Column[] = [
    { id: "name", label: "Name", sortable: true },
    { id: "namespace", label: "Namespace", sortable: true },
    { id: "status", label: "Status", sortable: true },
    { id: "images", label: "Images", sortable: true },
    { id: "age", label: "Age", sortable: true, sortKey: "created_at" },
  ];

  $effect(() => {
    headerStore.setTitle("Deployments");
  });

  $effect(() => {
    if (activeClusterStore.clusterId) {
      loadData();
    }
  });

  async function loadData() {
    loading = true;
    error = null;
    try {
      data = await invoke("cluster_list_deployments", {
        clusterId: activeClusterStore.clusterId,
        namespace: activeClusterStore.activeNamespace === "all" ? null : activeClusterStore.activeNamespace,
      });
    } catch (e) {
      console.error("Failed to load deployments", e);
    } finally {
      loading = false;
    }
  }

  function handleRowClick(row: any) {
    selectedDeployment = {
      name: row.name,
      namespace: row.namespace,
    };
    showDrawer = true;
  }

  async function handleBatchDelete(selectedIds: any[]) {
    const itemsToDelete = data.filter((item) => selectedIds.includes(item.id));

    const confirmed = await confirm(
      `Are you sure you want to delete ${itemsToDelete.length} deployment(s)?`,
      { title: "Delete Deployments", kind: "warning" }
    );

    if (confirmed) {
      let successCount = 0;
      for (const item of itemsToDelete) {
        try {
          await invoke("cluster_delete_deployment", {
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

  async function handleEditYaml(row: any) {
    if (!activeClusterStore.clusterId) return;
    loadingYaml = true;
    showYamlDrawer = true;
    yamlTarget = { name: row.name, namespace: row.namespace };
    try {
      yamlContent = await invoke<string>("cluster_get_resource_yaml", {
        clusterId: activeClusterStore.clusterId,
        kind: "deployment",
        name: row.name,
        namespace: row.namespace,
      });
    } catch (e) {
      console.error("Failed to load deployment yaml", e);
      error = `Failed to load YAML for ${row.name}.`;
      showYamlDrawer = false;
    } finally {
      loadingYaml = false;
    }
  }

  async function handleApplyYaml() {
    if (!activeClusterStore.clusterId || !yamlContent.trim()) return;
    const confirmed = await confirm("Apply YAML changes to this deployment?", {
      title: "Apply Deployment YAML",
      kind: "warning",
    });
    if (!confirmed) return;

    applyingYaml = true;
    try {
      await invoke("cluster_apply_resource_yaml", {
        clusterId: activeClusterStore.clusterId,
        yaml: yamlContent,
      });
      showYamlDrawer = false;
      await loadData();
    } catch (e) {
      console.error("Failed to apply deployment yaml", e);
      error = `Failed to apply YAML: ${e}`;
    } finally {
      applyingYaml = false;
    }
  }

  async function handleScale(row: any) {
    const input = window.prompt(`Scale ${row.name} to how many replicas?`, "1");
    if (input === null) return;
    const replicas = Number.parseInt(input, 10);
    if (!Number.isInteger(replicas) || replicas < 0) {
      error = "Replica count must be a non-negative integer.";
      return;
    }

    try {
      await invoke("cluster_scale_workload", {
        clusterId: activeClusterStore.clusterId,
        kind: "deployment",
        namespace: row.namespace,
        name: row.name,
        replicas,
      });
      await loadData();
    } catch (e) {
      console.error("Failed to scale deployment", e);
      error = `Failed to scale ${row.name}.`;
    }
  }

  async function handleRestart(row: any) {
    const confirmed = await confirm(`Restart rollout for ${row.name}?`, {
      title: "Restart Deployment",
      kind: "warning",
    });
    if (!confirmed) return;
    try {
      await invoke("cluster_restart_workload", {
        clusterId: activeClusterStore.clusterId,
        kind: "deployment",
        namespace: row.namespace,
        name: row.name,
      });
      await loadData();
    } catch (e) {
      console.error("Failed to restart deployment", e);
      error = `Failed to restart ${row.name}.`;
    }
  }

  function getActions(row: any): MenuItem[] {
    return [
      {
        label: "View Details",
        action: () => {
          selectedDeployment = {
            name: row.name,
            namespace: row.namespace,
          };
          showDrawer = true;
        },
        icon: Eye,
      },
      {
        label: "Edit YAML",
        action: () => handleEditYaml(row),
        icon: FilePenLine,
      },
      {
        label: "Scale",
        action: () => handleScale(row),
        icon: Scaling,
      },
      {
        label: "Restart",
        action: () => handleRestart(row),
        icon: RotateCw,
      },
      {
        label: "Delete",
        action: async () => {
          const confirmed = await confirm(
            `Are you sure you want to delete ${row.name}?`,
            { title: "Delete Deployment", kind: "warning" }
          );

          if (confirmed) {
            try {
              await invoke("cluster_delete_deployment", {
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
  {#if error}
    <div class="mb-4 p-3 bg-error/10 text-error rounded-md border border-error/20 flex items-center justify-between gap-3">
      <span>{error}</span>
      <Button variant="outline" size="sm" onclick={loadData}>Retry</Button>
    </div>
  {/if}

  <DataTable
    {data}
    {columns}
    bind:search
    {loading}
    onRefresh={loadData}
    actions={getActions}
    emptyMessage="No deployments found."
    onRowClick={handleRowClick}
    batchActions={[
      {
        label: "Delete",
        icon: Trash2,
        danger: true,
        action: handleBatchDelete
      }
    ]}
    storageKey="workload-deployments"
  >
    {#snippet children({ row, column, value })}
      {#if column.id === "images"}
        <div class="flex flex-col gap-1">
          {#if Array.isArray(value)}
            {#each value.slice(0, 2) as img, i (`${img}-${i}`)}
              <span class="text-xs font-mono bg-bg-panel px-1 rounded truncate max-w-[200px]" title={img}>
                {img.split('/').pop()}
              </span>
            {/each}
            {#if value.length > 2}
              <span class="text-xs text-text-muted">+{value.length - 2} more</span>
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

  <DeploymentDetailDrawer
    bind:open={showDrawer}
    bind:deploymentName={selectedDeployment.name}
    bind:namespace={selectedDeployment.namespace}
  />

  <Drawer bind:open={showYamlDrawer} title={yamlTarget ? `Edit YAML: ${yamlTarget.name}` : "Edit YAML"} width="w-[900px]">
    <div class="p-4 space-y-3">
      {#if loadingYaml}
        <div class="text-text-muted">Loading YAML...</div>
      {:else}
        <textarea
          class="w-full h-[65vh] rounded-md border border-border-main bg-bg-main text-text-main font-mono text-xs p-3 resize-y"
          bind:value={yamlContent}
          spellcheck="false"
        ></textarea>
        <div class="flex items-center justify-end gap-2">
          <Button variant="outline" onclick={() => (showYamlDrawer = false)}>Cancel</Button>
          <Button onclick={handleApplyYaml} disabled={applyingYaml || !yamlContent.trim()}>
            <Save size={16} />
            {applyingYaml ? "Applying..." : "Apply YAML"}
          </Button>
        </div>
      {/if}
    </div>
  </Drawer>
</div>
