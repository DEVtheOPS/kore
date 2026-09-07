<script lang="ts">
  import Drawer from '$lib/components/ui/Drawer.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import CodeEditor from '$lib/components/ui/CodeEditor.svelte';
  import YamlDisplay from '$lib/components/ui/YamlDisplay.svelte';
  import WorkloadUsage from '$lib/components/WorkloadUsage.svelte';
  import { Edit, RefreshCw, Trash2, Save } from 'lucide-svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { confirm } from '@tauri-apps/plugin-dialog';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { activeClusterStore } from '$lib/stores/activeCluster.svelte';
  import yaml from 'js-yaml';

  interface StatefulSetCondition {
    condition_type: string;
    status: string;
    reason?: string;
    message?: string;
    last_transition_time?: string;
  }

  interface StatefulSetDetails {
    name: string;
    namespace: string;
    uid: string;
    created_at: string;
    labels: Record<string, string>;
    annotations: Record<string, string>;
    replicas_desired: number;
    replicas_current: number;
    replicas_ready: number;
    replicas_updated: number;
    replicas_available: number;
    update_strategy_type: string;
    pod_management_policy: string;
    service_name: string;
    selector: Record<string, string>;
    conditions: StatefulSetCondition[];
    images: string[];
  }

  interface StatefulSetPodInfo {
    name: string;
    namespace: string;
    status: string;
    age: string;
    ready: string;
    restarts: number;
    node: string;
    pod_ip: string;
  }

  interface K8sEventInfo {
    event_type: string;
    reason: string;
    message: string;
    count: number;
    first_timestamp?: string;
    last_timestamp?: string;
    source: string;
  }

  let {
    open = $bindable(false),
    name = $bindable(''),
    namespace = $bindable(''),
    onDeleted,
  }: {
    open: boolean;
    name: string;
    namespace: string;
    /** Called after the statefulset has been deleted so the parent can refresh its list. */
    onDeleted?: (name: string, namespace: string) => void;
  } = $props();

  let details = $state<StatefulSetDetails | null>(null);
  let pods = $state<StatefulSetPodInfo[]>([]);
  let events = $state<K8sEventInfo[]>([]);
  let loading = $state(false);
  let deleting = $state(false);
  let error = $state<string | null>(null);

  // YAML editor drawer state
  let showYamlDrawer = $state(false);
  let yamlContent = $state('');
  let loadingYaml = $state(false);
  let applyingYaml = $state(false);

  $effect(() => {
    if (open && name && namespace) {
      loadDetails();
    }
  });

  async function loadDetails() {
    const clusterId = activeClusterStore.clusterId;
    if (!clusterId) return;

    loading = true;
    error = null;
    try {
      const [detailsData, podsData, eventsData] = await Promise.all([
        invoke<StatefulSetDetails>('cluster_get_statefulset_details', { clusterId, namespace, name }),
        invoke<StatefulSetPodInfo[]>('cluster_get_statefulset_pods', {
          clusterId,
          namespace,
          statefulsetName: name,
        }),
        invoke<K8sEventInfo[]>('cluster_get_statefulset_events', {
          clusterId,
          namespace,
          statefulsetName: name,
        }),
      ]);
      details = detailsData;
      pods = podsData;
      events = eventsData;
    } catch (e) {
      console.error('Failed to load statefulset details:', e);
      error = `Failed to load StatefulSet details: ${e}`;
    } finally {
      loading = false;
    }
  }

  async function handleEdit() {
    if (!activeClusterStore.clusterId) return;

    loadingYaml = true;
    showYamlDrawer = true;
    try {
      yamlContent = await invoke<string>('cluster_get_resource_yaml', {
        clusterId: activeClusterStore.clusterId,
        kind: 'statefulset',
        name,
        namespace,
      });
    } catch (e) {
      console.error('Failed to load yaml', e);
      error = `Failed to load YAML: ${e}`;
      showYamlDrawer = false;
    } finally {
      loadingYaml = false;
    }
  }

  async function applyYamlChanges() {
    if (!activeClusterStore.clusterId || !yamlContent) return;

    const confirmed = await confirm('Apply YAML changes to the cluster?', {
      title: 'Apply Resource YAML',
      kind: 'warning',
    });
    if (!confirmed) return;

    applyingYaml = true;
    try {
      await invoke('cluster_apply_resource_yaml', {
        clusterId: activeClusterStore.clusterId,
        yaml: yamlContent,
      });
      showYamlDrawer = false;
      await loadDetails();
    } catch (e) {
      console.error('Failed to apply yaml', e);
      error = `Failed to apply YAML: ${e}`;
    } finally {
      applyingYaml = false;
    }
  }

  async function handleDelete() {
    if (!activeClusterStore.clusterId || !name || deleting) return;

    const confirmed = await confirm(
      `Are you sure you want to delete statefulset ${name}? Its pods will be terminated (persistent volume claims are kept).`,
      { title: 'Delete StatefulSet', kind: 'warning' }
    );
    if (!confirmed) return;

    deleting = true;
    error = null;
    try {
      await invoke('cluster_delete_statefulset', {
        clusterId: activeClusterStore.clusterId,
        namespace,
        name,
      });
      open = false;
      onDeleted?.(name, namespace);
    } catch (e) {
      console.error('Failed to delete statefulset', e);
      error = `Failed to delete StatefulSet: ${e}`;
    } finally {
      deleting = false;
    }
  }

  function getConditionVariant(status: string): 'success' | 'warning' | 'error' | 'info' | 'neutral' {
    if (status === 'True') return 'success';
    if (status === 'False') return 'error';
    return 'neutral';
  }

  function formatAnnotationValue(value: string): { formatted: string; isYaml: boolean } {
    try {
      const parsed = JSON.parse(value);
      return { formatted: yaml.dump(parsed, { indent: 2, lineWidth: -1 }), isYaml: true };
    } catch {
      return { formatted: value, isYaml: false };
    }
  }

  function handlePodClick(pod: StatefulSetPodInfo) {
    const clusterId = $page.params.id;
    goto(`/cluster/${clusterId}/pods?pod=${encodeURIComponent(pod.name)}&namespace=${encodeURIComponent(pod.namespace)}`);
  }
</script>

<Drawer bind:open title="StatefulSet: {name}" width="w-[800px]">
  {#snippet headerActions()}
    <button
      class="p-1.5 hover:bg-bg-panel rounded-md text-text-muted hover:text-text-main transition-colors"
      onclick={loadDetails}
      title="Refresh"
    >
      <RefreshCw size={18} />
    </button>
    <button
      class="p-1.5 hover:bg-bg-panel rounded-md text-text-muted hover:text-text-main transition-colors"
      onclick={handleEdit}
      title="Edit"
    >
      <Edit size={18} />
    </button>
    <button
      class="p-1.5 hover:bg-error/10 rounded-md text-text-muted hover:text-error transition-colors disabled:opacity-50"
      onclick={handleDelete}
      disabled={deleting}
      title="Delete"
    >
      <Trash2 size={18} />
    </button>
  {/snippet}

  {#if error}
    <div class="mx-4 mt-4 p-3 bg-error/10 text-error rounded-md border border-error/20 text-sm flex items-center justify-between gap-3">
      <span>{error}</span>
      <button class="underline" onclick={() => (error = null)}>Dismiss</button>
    </div>
  {/if}

  {#if loading && !details}
    <div class="flex items-center justify-center py-8">
      <div class="text-text-muted">Loading StatefulSet details...</div>
    </div>
  {:else if details}
    <div class="space-y-6">
      <WorkloadUsage namespace={details.namespace} selector={details.selector} />

      <div class="space-y-4">
        <h3 class="text-sm font-bold uppercase text-text-muted border-b border-border pb-2">Details</h3>
        <div class="grid grid-cols-2 gap-4">
          <div>
            <div class="text-xs text-text-muted uppercase font-semibold mb-1">Created</div>
            <div class="text-sm">{new Date(details.created_at).toLocaleString()}</div>
          </div>
          <div>
            <div class="text-xs text-text-muted uppercase font-semibold mb-1">Name</div>
            <div class="text-sm font-mono">{details.name}</div>
          </div>
          <div>
            <div class="text-xs text-text-muted uppercase font-semibold mb-1">Namespace</div>
            <div class="text-sm">{details.namespace}</div>
          </div>
          <div>
            <div class="text-xs text-text-muted uppercase font-semibold mb-1">Replicas</div>
            <div class="text-sm">
              {details.replicas_desired} desired, {details.replicas_current} current,
              {details.replicas_ready} ready, {details.replicas_updated} updated,
              {details.replicas_available} available
            </div>
          </div>
          <div>
            <div class="text-xs text-text-muted uppercase font-semibold mb-1">Update Strategy</div>
            <div class="text-sm">{details.update_strategy_type}</div>
          </div>
          <div>
            <div class="text-xs text-text-muted uppercase font-semibold mb-1">Pod Management</div>
            <div class="text-sm">{details.pod_management_policy}</div>
          </div>
          <div>
            <div class="text-xs text-text-muted uppercase font-semibold mb-1">Service</div>
            <div class="text-sm font-mono">{details.service_name || '-'}</div>
          </div>
        </div>
      </div>

      {#if details.images.length > 0}
        <div class="space-y-4">
          <h3 class="text-sm font-bold uppercase text-text-muted border-b border-border pb-2">Images</h3>
          <ul class="space-y-1">
            {#each details.images as image (image)}
              <li class="text-xs font-mono break-all">{image}</li>
            {/each}
          </ul>
        </div>
      {/if}

      {#if Object.keys(details.labels).length > 0}
        <div class="space-y-4">
          <h3 class="text-sm font-bold uppercase text-text-muted border-b border-border pb-2">
            Labels ({Object.keys(details.labels).length})
          </h3>
          <div class="flex flex-wrap gap-2">
            {#each Object.entries(details.labels) as [key, value] (key)}
              <Badge variant="neutral"><span class="font-mono text-xs">{key}={value}</span></Badge>
            {/each}
          </div>
        </div>
      {/if}

      {#if Object.keys(details.annotations).length > 0}
        <div class="space-y-4">
          <h3 class="text-sm font-bold uppercase text-text-muted border-b border-border pb-2">
            Annotations ({Object.keys(details.annotations).length})
          </h3>
          <div class="space-y-2 max-h-[500px] overflow-y-auto">
            {#each Object.entries(details.annotations) as [key, value] (key)}
              {@const { formatted, isYaml } = formatAnnotationValue(value)}
              <div class="p-3 bg-bg-panel rounded-md">
                <div class="text-text-muted font-semibold mb-2 text-xs">{key}</div>
                {#if isYaml}
                  <YamlDisplay code={formatted} />
                {:else}
                  <div class="text-xs font-mono break-all text-text">{formatted}</div>
                {/if}
              </div>
            {/each}
          </div>
        </div>
      {/if}

      {#if Object.keys(details.selector).length > 0}
        <div class="space-y-4">
          <h3 class="text-sm font-bold uppercase text-text-muted border-b border-border pb-2">Selector</h3>
          <div class="flex flex-wrap gap-2">
            {#each Object.entries(details.selector) as [key, value] (key)}
              <Badge variant="info"><span class="font-mono text-xs">{key}={value}</span></Badge>
            {/each}
          </div>
        </div>
      {/if}

      {#if details.conditions.length > 0}
        <div class="space-y-4">
          <h3 class="text-sm font-bold uppercase text-text-muted border-b border-border pb-2">Conditions</h3>
          <div class="flex flex-wrap gap-2">
            {#each details.conditions as condition (condition.condition_type)}
              <Badge variant={getConditionVariant(condition.status)}>
                {condition.condition_type}: {condition.status}
              </Badge>
            {/each}
          </div>
        </div>
      {/if}

      <div class="space-y-4">
        <h3 class="text-sm font-bold uppercase text-text-muted border-b border-border pb-2">Pods ({pods.length})</h3>
        {#if pods.length > 0}
          <div class="overflow-x-auto">
            <table class="w-full text-sm">
              <thead class="text-xs text-text-muted uppercase border-b border-border">
                <tr>
                  <th class="text-left py-2 px-3">Name</th>
                  <th class="text-left py-2 px-3">Ready</th>
                  <th class="text-left py-2 px-3">Status</th>
                  <th class="text-left py-2 px-3">Restarts</th>
                  <th class="text-left py-2 px-3">Node</th>
                  <th class="text-left py-2 px-3">Age</th>
                </tr>
              </thead>
              <tbody>
                {#each pods as pod (pod.name)}
                  <tr
                    class="border-b border-border/50 hover:bg-bg-panel/50 cursor-pointer transition-colors"
                    onclick={() => handlePodClick(pod)}
                    role="button"
                    tabindex="0"
                    onkeydown={(e) => e.key === 'Enter' && handlePodClick(pod)}
                  >
                    <td class="py-2 px-3 font-mono text-xs">{pod.name}</td>
                    <td class="py-2 px-3">{pod.ready}</td>
                    <td class="py-2 px-3">
                      <Badge variant={pod.status === 'Running' ? 'success' : 'warning'}>{pod.status}</Badge>
                    </td>
                    <td class="py-2 px-3">{pod.restarts}</td>
                    <td class="py-2 px-3 text-xs">{pod.node}</td>
                    <td class="py-2 px-3">{pod.age}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {:else}
          <div class="text-sm text-text-muted text-center py-4">No pods found</div>
        {/if}
      </div>

      <div class="space-y-4">
        <h3 class="text-sm font-bold uppercase text-text-muted border-b border-border pb-2">
          Events {#if events.length > 0}({events.length}){/if}
        </h3>
        {#if events.length > 0}
          <div class="space-y-2 max-h-96 overflow-y-auto">
            {#each events as event, i (`${event.reason}-${event.last_timestamp}-${i}`)}
              <div class="p-3 bg-bg-panel rounded-md">
                <div class="flex items-start justify-between gap-2 mb-2">
                  <div class="flex items-center gap-2">
                    <Badge variant={event.event_type === 'Warning' ? 'error' : 'neutral'}>{event.event_type}</Badge>
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
  {:else}
    <div class="text-sm text-text-muted text-center py-8">No StatefulSet details available</div>
  {/if}
</Drawer>

<Drawer bind:open={showYamlDrawer} title="Edit YAML: {name}" width="w-[900px]">
  <div class="p-4 space-y-3 h-full flex flex-col">
    {#if loadingYaml}
      <div class="text-text-muted">Loading YAML...</div>
    {:else}
      <div class="flex-1 min-h-0">
        <CodeEditor bind:value={yamlContent} />
      </div>
      <div class="flex items-center justify-end gap-2">
        <Button variant="outline" onclick={() => (showYamlDrawer = false)}>Cancel</Button>
        <Button onclick={applyYamlChanges} disabled={applyingYaml || !yamlContent.trim()}>
          <Save size={16} />
          {applyingYaml ? 'Applying...' : 'Apply YAML'}
        </Button>
      </div>
    {/if}
  </div>
</Drawer>
