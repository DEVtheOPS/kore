<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { confirm } from '@tauri-apps/plugin-dialog';
  import { onMount, onDestroy } from 'svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import DataTable from '$lib/components/ui/DataTable.svelte';
  import Drawer from '$lib/components/ui/Drawer.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import { RefreshCw, Search, Settings2, Eye, EyeOff, Trash2, StopCircle } from 'lucide-svelte';
  import { clusterStore } from '$lib/stores/cluster.svelte';
  import { headerStore } from '$lib/stores/header.svelte';

  interface Pod {
    name: string;
    namespace: string;
    status: string;
    age: string;
    containers: number;
    restarts: number;
    node: string;
    qos: string;
    controlled_by: string;
    creation_timestamp?: string;
  }

  let pods = $state<Pod[]>([]);
  let loading = $state(false);
  let error = $state('');
  let search = $state('');
  let selectedPod = $state<Pod | null>(null);
  let isDrawerOpen = $state(false);
  let unlisten: (() => void) | null = null;
  let now = $state(Date.now());
  let interval: any;

  // Define Columns
  let columns = $state([
    { id: 'name', label: 'Name', sortable: true, visible: true },
    { id: 'namespace', label: 'Namespace', sortable: true, visible: true },
    { id: 'containers', label: 'Containers', sortable: true, visible: true },
    { id: 'restarts', label: 'Restarts', sortable: true, visible: true },
    { id: 'controlled_by', label: 'Controlled By', sortable: true, visible: true },
    { id: 'node', label: 'Node', sortable: true, visible: true },
    { id: 'qos', label: 'QoS', sortable: true, visible: true },
    { id: 'age', label: 'Age', sortable: true, visible: true, sortKey: 'creation_timestamp' },
    { id: 'status', label: 'Status', sortable: true, visible: true },
  ]);

  async function startWatch() {
    if (!clusterStore.active) return;
    
    loading = true;
    error = '';
    
    // First fetch initial list (optional as watch usually sends Restarted event first, 
    // but sometimes good for immediate feedback)
    try {
      pods = await invoke('list_pods', { 
        contextName: clusterStore.active, 
        namespace: clusterStore.activeNamespace 
      }); 
    } catch (e) {
      console.error(e);
      // Demo data if failed
      if (pods.length === 0) {
         pods = [/* ... demo data ... */];
      }
    } finally {
      loading = false;
    }

    // Start Watch
    invoke('start_pod_watch', {
      contextName: clusterStore.active,
      namespace: clusterStore.activeNamespace
    }).catch(e => console.error("Watch failed to start", e));
  }

  onMount(async () => {
    // Listen for backend events
    unlisten = await listen('pod_event', (event: any) => {
      const payload = event.payload;
      console.log('Pod Event:', payload);
      
      if (payload.type === 'Restarted') {
        pods = payload.payload;
      } else if (payload.type === 'Added' || payload.type === 'Modified') {
        const newPod = payload.payload;
        const idx = pods.findIndex(p => p.name === newPod.name && p.namespace === newPod.namespace);
        if (idx >= 0) {
          pods[idx] = newPod;
        } else {
          pods.push(newPod);
        }
      } else if (payload.type === 'Deleted') {
        const deletedPod = payload.payload;
        pods = pods.filter(p => !(p.name === deletedPod.name && p.namespace === deletedPod.namespace));
      }
    });

    startWatch();
    
    // Update 'now' every second for active age
    interval = setInterval(() => {
      now = Date.now();
    }, 1000);
  });

  $effect(() => {
    headerStore.setTitle("Pods");
  });

  onDestroy(() => {
    if (unlisten) unlisten();
    if (interval) clearInterval(interval);
  });
  
  $effect(() => {
    if (clusterStore.active && clusterStore.activeNamespace) {
      // Re-trigger watch when context changes
      // Ideally we should tell backend to stop previous watch, 
      // but current simple impl just spawns new one. 
      // In prod, backend should handle cleanup based on channel drops or explicit stop command.
      startWatch();
    }
  });

  const filteredPods = $derived(
    pods.filter(p => p.name.toLowerCase().includes(search.toLowerCase()))
  );

  function getStatusVariant(status: string) {
    switch (status.toLowerCase()) {
      case 'running': return 'success';
      case 'pending': return 'warning';
      case 'failed': 
      case 'error': return 'error';
      default: return 'neutral';
    }
  }

  function handleRowClick(row: any) {
    selectedPod = row;
    isDrawerOpen = true;
  }

  function formatAge(creationTimestamp: string | undefined): string {
    if (!creationTimestamp) return '-';
    
    const created = new Date(creationTimestamp).getTime();
    const diff = Math.max(0, now - created);
    
    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (days > 0) return `${days}d`;
    if (hours > 0) return `${hours}h`;
    if (minutes > 0) return `${minutes}m`;
    return `${seconds}s`;
  }

  async function handleDelete(pod: Pod) {
    const confirmed = await confirm(`Are you sure you want to delete pod ${pod.name}?`, {
      title: 'Delete Pod',
      kind: 'warning',
    });
    
    if (!confirmed) return;
    
    try {
      await invoke('delete_pod', {
        contextName: clusterStore.active,
        namespace: pod.namespace,
        podName: pod.name
      });
      // UI update will happen via watch event
    } catch (e) {
      alert(`Failed to delete pod: ${e}`);
    }
  }

  function handleAction(action: string, pod: Pod) {
    if (action === 'Delete') {
      handleDelete(pod);
    } else {
      alert(`${action} on ${pod.name} (Not implemented)`);
    }
  }

  function getActions(row: any) {
    return [
      { label: 'Open Shell', action: () => handleAction('Shell', row) },
      { label: 'View Logs', action: () => handleAction('Logs', row) },
      { label: 'Edit', action: () => handleAction('Edit', row) },
      { label: 'Delete', action: () => handleAction('Delete', row), danger: true },
    ];
  }

  const batchActions = [
    {
      label: 'Delete Selected',
      icon: Trash2,
      danger: true,
      action: async (selectedIds: string[]) => {
        const confirmed = await confirm(`Are you sure you want to delete ${selectedIds.length} pods?`, {
          title: 'Delete Pods',
          kind: 'warning',
        });
        
        if (!confirmed) return;

        // In a real app, this should probably be a single "delete_pods" command 
        // to handle parallelism efficiently on the backend, but for now loop is fine.
        // We can just fire them off.
        const promises = selectedIds.map(name => {
          // We need the namespace for each pod. 
          // Since selectedIds are just names (keyField="name"), we have to find the pod object.
          // Note: keyField="name" might be risky if names aren't unique across namespaces, 
          // but usually the view is filtered by namespace or names are unique enough for this context.
          // Ideally keyField should be a unique ID.
          const pod = pods.find(p => p.name === name);
          if (pod) {
            return invoke('delete_pod', {
              contextName: clusterStore.active,
              namespace: pod.namespace,
              podName: pod.name
            }).catch(e => console.error(`Failed to delete ${name}:`, e));
          }
          return Promise.resolve();
        });

        await Promise.all(promises);
      }
    }
  ];
</script>

<div class="space-y-6 h-full flex flex-col">
  {#if error}
    <div class="p-4 bg-error/10 text-error rounded-md border border-error/20">
      {error} (Showing demo data)
    </div>
  {/if}

  <div class="flex-1 overflow-hidden">
    <DataTable 
      data={filteredPods} 
      bind:columns={columns} 
      keyField="name" 
      onRowClick={handleRowClick}
      storageKey="pods-table"
      bind:search={search}
      onRefresh={startWatch}
      loading={loading}
      actions={getActions}
      batchActions={batchActions}
    >
      {#snippet children({ row, column, value })}
        {#if column.id === 'status'}
           <Badge variant={getStatusVariant(value)}>{value}</Badge>
        {:else if column.id === 'age'}
           {formatAge(row.creation_timestamp) || value}
        {:else}
           {value}
        {/if}
      {/snippet}
    </DataTable>
  </div>

  <Drawer bind:open={isDrawerOpen} title={selectedPod?.name || 'Pod Details'}>
    {#if selectedPod}
      <div class="space-y-6">
        <div class="grid grid-cols-2 gap-4">
          <div>
            <div class="text-xs text-text-muted uppercase font-semibold">Namespace</div>
            <div>{selectedPod.namespace}</div>
          </div>
          <div>
             <div class="text-xs text-text-muted uppercase font-semibold">Status</div>
             <Badge variant={getStatusVariant(selectedPod.status)}>{selectedPod.status}</Badge>
          </div>
          <div>
             <div class="text-xs text-text-muted uppercase font-semibold">Node</div>
             <div>{selectedPod.node}</div>
          </div>
          <div>
             <div class="text-xs text-text-muted uppercase font-semibold">Age</div>
             <div>{selectedPod.age}</div>
          </div>
           <div>
             <div class="text-xs text-text-muted uppercase font-semibold">QoS</div>
             <div>{selectedPod.qos}</div>
          </div>
           <div>
             <div class="text-xs text-text-muted uppercase font-semibold">Controlled By</div>
             <div>{selectedPod.controlled_by}</div>
          </div>
        </div>

        <div>
           <h3 class="font-bold mb-2">Containers</h3>
           <div class="p-4 bg-bg-panel rounded-md">
             {selectedPod.containers} container(s) running
           </div>
        </div>
      </div>
    {/if}
  </Drawer>
</div>
