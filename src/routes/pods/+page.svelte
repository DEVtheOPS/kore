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

  interface ContainerPort {
    name?: string;
    container_port: number;
    host_port?: number;
    protocol: string;
  }

  interface EnvVar {
    name: string;
    value?: string;
    value_from?: string;
  }

  interface VolumeMount {
    name: string;
    mount_path: string;
    sub_path?: string;
    read_only: boolean;
  }

  interface ProbeInfo {
    probe_type: string;
    handler_type: string;
    details: string;
    initial_delay_seconds: number;
    period_seconds: number;
    timeout_seconds: number;
    success_threshold: number;
    failure_threshold: number;
  }

  interface ContainerInfo {
    name: string;
    image: string;
    image_pull_policy: string;
    ready: boolean;
    restart_count: number;
    state: string;
    cpu_request?: string;
    cpu_limit?: string;
    memory_request?: string;
    memory_limit?: string;
    ports: ContainerPort[];
    env: EnvVar[];
    volume_mounts: VolumeMount[];
    probes: ProbeInfo[];
  }

  interface VolumeInfo {
    name: string;
    volume_type: string;
  }

  interface PodCondition {
    condition_type: string;
    status: string;
    reason?: string;
    message?: string;
    last_transition_time?: string;
  }

  interface PodEventInfo {
    event_type: string;
    reason: string;
    message: string;
    count: number;
    first_timestamp?: string;
    last_timestamp?: string;
    source: string;
  }

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
    labels: Record<string, string>;
    annotations: Record<string, string>;
    pod_ip: string;
    host_ip: string;
    service_account: string;
    priority_class: string;
    container_details: ContainerInfo[];
    volumes: VolumeInfo[];
    conditions: PodCondition[];
  }

  let pods = $state<Pod[]>([]);
  let loading = $state(false);
  let error = $state('');
  let search = $state('');
  let selectedPod = $state<Pod | null>(null);
  let podEvents = $state<PodEventInfo[]>([]);
  let loadingEvents = $state(false);
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

  async function handleRowClick(row: any) {
    selectedPod = row;
    isDrawerOpen = true;
    
    // Load events for this pod
    loadingEvents = true;
    podEvents = [];
    try {
      podEvents = await invoke<PodEventInfo[]>('get_pod_events', {
        contextName: clusterStore.active,
        namespace: row.namespace,
        podName: row.name,
      });
    } catch (e) {
      console.error('Failed to load pod events:', e);
    } finally {
      loadingEvents = false;
    }
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
        <!-- Overview Section -->
        <div class="space-y-4">
          <h3 class="text-sm font-bold uppercase text-text-muted border-b border-border pb-2">Overview</h3>
          <div class="grid grid-cols-2 gap-4">
            <div>
              <div class="text-xs text-text-muted uppercase font-semibold mb-1">Namespace</div>
              <div class="text-sm">{selectedPod.namespace}</div>
            </div>
            <div>
              <div class="text-xs text-text-muted uppercase font-semibold mb-1">Status</div>
              <Badge variant={getStatusVariant(selectedPod.status)}>{selectedPod.status}</Badge>
            </div>
            <div>
              <div class="text-xs text-text-muted uppercase font-semibold mb-1">Node</div>
              <div class="text-sm">{selectedPod.node || '-'}</div>
            </div>
            <div>
              <div class="text-xs text-text-muted uppercase font-semibold mb-1">Age</div>
              <div class="text-sm">{selectedPod.age}</div>
            </div>
            <div>
              <div class="text-xs text-text-muted uppercase font-semibold mb-1">QoS Class</div>
              <div class="text-sm">{selectedPod.qos || '-'}</div>
            </div>
            <div>
              <div class="text-xs text-text-muted uppercase font-semibold mb-1">Controlled By</div>
              <div class="text-sm">{selectedPod.controlled_by}</div>
            </div>
            <div>
              <div class="text-xs text-text-muted uppercase font-semibold mb-1">Pod IP</div>
              <div class="text-sm font-mono">{selectedPod.pod_ip}</div>
            </div>
            <div>
              <div class="text-xs text-text-muted uppercase font-semibold mb-1">Host IP</div>
              <div class="text-sm font-mono">{selectedPod.host_ip}</div>
            </div>
            <div>
              <div class="text-xs text-text-muted uppercase font-semibold mb-1">Service Account</div>
              <div class="text-sm">{selectedPod.service_account}</div>
            </div>
            <div>
              <div class="text-xs text-text-muted uppercase font-semibold mb-1">Priority Class</div>
              <div class="text-sm">{selectedPod.priority_class}</div>
            </div>
          </div>
        </div>

        <!-- Containers Section -->
        <div class="space-y-4">
          <h3 class="text-sm font-bold uppercase text-text-muted border-b border-border pb-2">
            Containers ({selectedPod.container_details?.length || 0})
          </h3>
          {#if selectedPod.container_details && selectedPod.container_details.length > 0}
            <div class="space-y-3">
              {#each selectedPod.container_details as container}
                <div class="p-4 bg-bg-panel rounded-md border border-border space-y-2">
                  <div class="flex items-center justify-between">
                    <div class="font-semibold">{container.name}</div>
                    <Badge variant={container.ready ? 'success' : 'warning'}>
                      {container.ready ? 'Ready' : 'Not Ready'}
                    </Badge>
                  </div>
                  <div class="text-xs text-text-muted space-y-1">
                    <div><span class="font-semibold">Image:</span> {container.image}</div>
                    <div><span class="font-semibold">Pull Policy:</span> {container.image_pull_policy}</div>
                    <div><span class="font-semibold">State:</span> {container.state}</div>
                    <div><span class="font-semibold">Restarts:</span> {container.restart_count}</div>
                  </div>

                  <!-- Ports -->
                  {#if container.ports && container.ports.length > 0}
                    <div class="mt-2 pt-2 border-t border-border/50">
                      <div class="text-xs text-text-muted font-semibold mb-2">Ports</div>
                      <div class="space-y-1">
                        {#each container.ports as port}
                          <div class="text-xs font-mono bg-bg-main/50 p-2 rounded">
                            {#if port.name}<span class="text-text-muted">{port.name}:</span> {/if}
                            {port.container_port}
                            {#if port.host_port} → {port.host_port}{/if}
                            <span class="text-text-muted">/{port.protocol}</span>
                          </div>
                        {/each}
                      </div>
                    </div>
                  {/if}

                  <!-- Environment Variables -->
                  {#if container.env && container.env.length > 0}
                    <div class="mt-2 pt-2 border-t border-border/50">
                      <div class="text-xs text-text-muted font-semibold mb-2">Environment ({container.env.length})</div>
                      <div class="space-y-1 max-h-40 overflow-y-auto">
                        {#each container.env as envVar}
                          <div class="text-xs font-mono bg-bg-main/50 p-2 rounded break-all">
                            <span class="text-text-muted font-semibold">{envVar.name}:</span>
                            {#if envVar.value}
                              {envVar.value}
                            {:else if envVar.value_from}
                              <span class="text-text-muted italic">{envVar.value_from}</span>
                            {:else}
                              <span class="text-text-muted">-</span>
                            {/if}
                          </div>
                        {/each}
                      </div>
                    </div>
                  {/if}

                  <!-- Volume Mounts -->
                  {#if container.volume_mounts && container.volume_mounts.length > 0}
                    <div class="mt-2 pt-2 border-t border-border/50">
                      <div class="text-xs text-text-muted font-semibold mb-2">Mounts ({container.volume_mounts.length})</div>
                      <div class="space-y-1 max-h-40 overflow-y-auto">
                        {#each container.volume_mounts as mount}
                          <div class="text-xs font-mono bg-bg-main/50 p-2 rounded">
                            <div><span class="text-text-muted">Volume:</span> {mount.name}</div>
                            <div><span class="text-text-muted">Path:</span> {mount.mount_path}</div>
                            {#if mount.sub_path}
                              <div><span class="text-text-muted">SubPath:</span> {mount.sub_path}</div>
                            {/if}
                            <div>
                              <Badge variant={mount.read_only ? 'neutral' : 'success'}>
                                {mount.read_only ? 'Read-Only' : 'Read-Write'}
                              </Badge>
                            </div>
                          </div>
                        {/each}
                      </div>
                    </div>
                  {/if}

                  <!-- Probes -->
                  {#if container.probes && container.probes.length > 0}
                    <div class="mt-2 pt-2 border-t border-border/50">
                      <div class="text-xs text-text-muted font-semibold mb-2">Probes</div>
                      <div class="space-y-2">
                        {#each container.probes as probe}
                          <div class="text-xs bg-bg-main/50 p-2 rounded">
                            <div class="font-semibold capitalize mb-1">{probe.probe_type}</div>
                            <div class="text-text-muted">
                              <span class="font-semibold">{probe.handler_type}:</span> {probe.details}
                            </div>
                            <div class="grid grid-cols-2 gap-1 mt-1 text-text-muted">
                              <div>Delay: {probe.initial_delay_seconds}s</div>
                              <div>Period: {probe.period_seconds}s</div>
                              <div>Timeout: {probe.timeout_seconds}s</div>
                              <div>Threshold: {probe.success_threshold}/{probe.failure_threshold}</div>
                            </div>
                          </div>
                        {/each}
                      </div>
                    </div>
                  {/if}

                  <!-- Resources -->
                  {#if container.cpu_request || container.cpu_limit || container.memory_request || container.memory_limit}
                    <div class="mt-2 pt-2 border-t border-border/50">
                      <div class="text-xs text-text-muted font-semibold mb-1">Resources</div>
                      <div class="grid grid-cols-2 gap-2 text-xs">
                        <div>
                          <span class="text-text-muted">CPU Request:</span> {container.cpu_request || '-'}
                        </div>
                        <div>
                          <span class="text-text-muted">CPU Limit:</span> {container.cpu_limit || '-'}
                        </div>
                        <div>
                          <span class="text-text-muted">Memory Request:</span> {container.memory_request || '-'}
                        </div>
                        <div>
                          <span class="text-text-muted">Memory Limit:</span> {container.memory_limit || '-'}
                        </div>
                      </div>
                    </div>
                  {/if}
                </div>
              {/each}
            </div>
          {:else}
            <div class="text-sm text-text-muted">No container details available</div>
          {/if}
        </div>

        <!-- Conditions Section -->
        {#if selectedPod.conditions && selectedPod.conditions.length > 0}
          <div class="space-y-4">
            <h3 class="text-sm font-bold uppercase text-text-muted border-b border-border pb-2">Conditions</h3>
            <div class="space-y-2">
              {#each selectedPod.conditions as condition}
                <div class="p-3 bg-bg-panel rounded-md border border-border">
                  <div class="flex items-center justify-between mb-1">
                    <div class="font-semibold text-sm">{condition.condition_type}</div>
                    <Badge variant={condition.status === 'True' ? 'success' : 'neutral'}>
                      {condition.status}
                    </Badge>
                  </div>
                  {#if condition.reason}
                    <div class="text-xs text-text-muted"><span class="font-semibold">Reason:</span> {condition.reason}</div>
                  {/if}
                  {#if condition.message}
                    <div class="text-xs text-text-muted mt-1">{condition.message}</div>
                  {/if}
                </div>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Volumes Section -->
        {#if selectedPod.volumes && selectedPod.volumes.length > 0}
          <div class="space-y-4">
            <h3 class="text-sm font-bold uppercase text-text-muted border-b border-border pb-2">
              Volumes ({selectedPod.volumes.length})
            </h3>
            <div class="space-y-2">
              {#each selectedPod.volumes as volume}
                <div class="p-3 bg-bg-panel rounded-md border border-border flex items-center justify-between">
                  <div class="font-semibold text-sm">{volume.name}</div>
                  <Badge variant="neutral">{volume.volume_type}</Badge>
                </div>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Labels Section -->
        {#if selectedPod.labels && Object.keys(selectedPod.labels).length > 0}
          <div class="space-y-4">
            <h3 class="text-sm font-bold uppercase text-text-muted border-b border-border pb-2">
              Labels ({Object.keys(selectedPod.labels).length})
            </h3>
            <div class="space-y-1">
              {#each Object.entries(selectedPod.labels) as [key, value]}
                <div class="p-2 bg-bg-panel rounded-md border border-border text-xs font-mono">
                  <span class="text-text-muted">{key}:</span> {value}
                </div>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Annotations Section -->
        {#if selectedPod.annotations && Object.keys(selectedPod.annotations).length > 0}
          <div class="space-y-4">
            <h3 class="text-sm font-bold uppercase text-text-muted border-b border-border pb-2">
              Annotations ({Object.keys(selectedPod.annotations).length})
            </h3>
            <div class="space-y-1 max-h-64 overflow-y-auto">
              {#each Object.entries(selectedPod.annotations) as [key, value]}
                <div class="p-2 bg-bg-panel rounded-md border border-border text-xs font-mono break-all">
                  <div class="text-text-muted font-semibold mb-1">{key}</div>
                  <div class="text-text">{value}</div>
                </div>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Events Section -->
        <div class="space-y-4">
          <h3 class="text-sm font-bold uppercase text-text-muted border-b border-border pb-2">
            Events {#if podEvents.length > 0}({podEvents.length}){/if}
          </h3>
          {#if loadingEvents}
            <div class="text-sm text-text-muted text-center py-4">Loading events...</div>
          {:else if podEvents.length > 0}
            <div class="space-y-2 max-h-96 overflow-y-auto">
              {#each podEvents as event}
                <div class="p-3 bg-bg-panel rounded-md border border-border">
                  <div class="flex items-start justify-between gap-2 mb-2">
                    <div class="flex items-center gap-2">
                      <Badge variant={event.event_type === 'Warning' ? 'error' : 'neutral'}>
                        {event.event_type}
                      </Badge>
                      <span class="text-sm font-semibold">{event.reason}</span>
                    </div>
                    {#if event.count > 1}
                      <Badge variant="neutral">{event.count}x</Badge>
                    {/if}
                  </div>
                  <div class="text-xs text-text mb-2">{event.message}</div>
                  <div class="flex items-center justify-between text-xs text-text-muted">
                    <div>Source: {event.source}</div>
                    {#if event.last_timestamp}
                      <div>{new Date(event.last_timestamp).toLocaleString()}</div>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          {:else}
            <div class="text-sm text-text-muted text-center py-4">No events found</div>
          {/if}
        </div>
      </div>
    {/if}
  </Drawer>
</div>
