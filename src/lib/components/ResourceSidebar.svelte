<script lang="ts">
  import {
    Activity,
    Anchor,
    Box,
    Cpu,
    Database,
    FileText,
    HardDrive,
    LayoutDashboard,
    Network,
    Settings as SettingsIcon,
    Shield,
  } from 'lucide-svelte';

  import Select from '$lib/components/ui/Select.svelte';
  import SidebarGroup from '$lib/components/ui/SidebarGroup.svelte';
  import type { ContextRecord } from '$lib/stores/contexts.svelte';

  let { context, namespaces, activeNamespace, onNamespaceChange } = $props<{
    context: ContextRecord;
    namespaces: string[];
    activeNamespace: string;
    onNamespaceChange: (ns: string) => void;
  }>();

  let groups = $state({
    workloads: true,
    config: true,
    network: false,
    storage: false,
    access: false,
    helm: false,
    custom: false,
  });

  const contextId = $derived(context.id);

  function getColorForString(str: string): string {
    let hash = 0;
    for (let i = 0; i < str.length; i += 1) {
      hash = str.charCodeAt(i) + ((hash << 5) - hash);
    }
    const hue = hash % 360;
    return `hsl(${hue}, 60%, 50%)`;
  }
</script>

<aside class="flex h-full w-64 flex-col border-r border-border-main bg-bg-sidebar text-text-main">
  <div class="space-y-3 border-b border-border-subtle p-4">
    <div class="flex items-center gap-3 px-1">
      {#if context.icon}
        {#if context.icon.startsWith('http') || context.icon.startsWith('data:')}
          <img
            src={context.icon}
            alt={context.display_name}
            class="h-8 w-8 rounded-full border-2 object-contain"
            style:border-color={context.icon_ring_color || 'var(--color-primary)'}
          />
        {:else}
          <div class="flex h-8 w-8 items-center justify-center rounded-full border-2 text-xl" style:border-color={context.icon_ring_color || 'var(--color-primary)'}>
            {context.icon}
          </div>
        {/if}
      {:else}
        <div
          class="flex h-8 w-8 items-center justify-center rounded-full border-2 text-xs font-bold text-white"
          style="background-color: {getColorForString(context.display_name)}; border-color: {context.icon_ring_color || 'var(--color-primary)'};"
        >
          {context.display_name.charAt(0).toUpperCase()}
        </div>
      {/if}

      <div class="min-w-0">
        <div class="truncate font-bold text-lg">{context.display_name}</div>
        <div class="truncate text-xs text-text-muted">{context.cluster_display_name}</div>
      </div>
    </div>

    <a
      href="/context/{contextId}/settings"
      class="flex items-center gap-2 rounded-md px-3 py-2 text-sm text-text-muted transition-colors hover:bg-bg-main hover:text-text-main"
    >
      <SettingsIcon size={16} />
      <span>Context Settings</span>
    </a>

    <div>
      <label for="namespace-select" class="mb-1 block px-1 text-xs font-semibold uppercase text-text-muted">Namespace</label>
      <Select
        id="namespace-select"
        options={["all", ...namespaces]}
        value={activeNamespace}
        onselect={onNamespaceChange}
        placeholder="Namespace"
      />
    </div>
  </div>

  <nav class="flex-1 space-y-1 overflow-y-auto px-2 py-4">
    <a href="/context/{contextId}/dashboard" class="group flex items-center gap-3 rounded-md px-3 py-2 text-sm hover:bg-bg-popover">
      <LayoutDashboard size={18} class="transition-colors group-hover:text-primary" />
      <span>Dashboard</span>
    </a>

    <a href="/context/{contextId}/nodes" class="group flex items-center gap-3 rounded-md px-3 py-2 text-sm hover:bg-bg-popover">
      <Cpu size={18} class="transition-colors group-hover:text-primary" />
      <span>Nodes</span>
    </a>

    <SidebarGroup title="Workloads" icon={Box} bind:open={groups.workloads}>
      <a href="/context/{contextId}/workloads" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">Overview</a>
      <a href="/context/{contextId}/pods" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">Pods</a>
      <a href="/context/{contextId}/deployments" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">Deployments</a>
      <a href="/context/{contextId}/statefulsets" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">StatefulSets</a>
      <a href="/context/{contextId}/daemonsets" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">DaemonSets</a>
      <a href="/context/{contextId}/replicasets" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">ReplicaSets</a>
      <a href="/context/{contextId}/jobs" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">Jobs</a>
      <a href="/context/{contextId}/cronjobs" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">CronJobs</a>
    </SidebarGroup>

    <SidebarGroup title="Configuration" icon={FileText} bind:open={groups.config}>
      <a href="/context/{contextId}/config-maps" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">ConfigMaps</a>
      <a href="/context/{contextId}/secrets" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">Secrets</a>
      <a href="/context/{contextId}/resource-quotas" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">Resource Quotas</a>
      <a href="/context/{contextId}/limit-ranges" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">Limit Ranges</a>
      <a href="/context/{contextId}/hpa" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">Horizontal Pod Autoscalers</a>
      <a href="/context/{contextId}/pdb" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">Pod Disruption Budgets</a>
    </SidebarGroup>

    <SidebarGroup title="Network" icon={Network} bind:open={groups.network}>
      <a href="/context/{contextId}/services" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">Services</a>
      <a href="/context/{contextId}/endpoints" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">Endpoints</a>
      <a href="/context/{contextId}/ingresses" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">Ingresses</a>
      <a href="/context/{contextId}/network-policies" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">Network Policies</a>
    </SidebarGroup>

    <SidebarGroup title="Storage" icon={HardDrive} bind:open={groups.storage}>
      <a href="/context/{contextId}/pvc" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">PersistentVolumeClaims</a>
      <a href="/context/{contextId}/pv" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">PersistentVolumes</a>
      <a href="/context/{contextId}/storage-classes" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">Storage Classes</a>
    </SidebarGroup>

    <SidebarGroup title="Access Control" icon={Shield} bind:open={groups.access}>
      <a href="/context/{contextId}/service-accounts" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">Service Accounts</a>
      <a href="/context/{contextId}/roles" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">Roles</a>
      <a href="/context/{contextId}/role-bindings" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">Role Bindings</a>
      <a href="/context/{contextId}/cluster-roles" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">Cluster Roles</a>
      <a href="/context/{contextId}/cluster-role-bindings" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">Cluster Role Bindings</a>
    </SidebarGroup>

    <a href="/context/{contextId}/namespaces" class="group flex items-center gap-3 rounded-md px-3 py-2 text-sm hover:bg-bg-popover">
      <Database size={18} class="transition-colors group-hover:text-primary" />
      <span>Namespaces</span>
    </a>

    <a href="/context/{contextId}/events" class="group flex items-center gap-3 rounded-md px-3 py-2 text-sm hover:bg-bg-popover">
      <Activity size={18} class="transition-colors group-hover:text-primary" />
      <span>Events</span>
    </a>

    <SidebarGroup title="Helm" icon={Anchor} bind:open={groups.helm}>
      <a href="/context/{contextId}/helm/releases" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">Releases</a>
      <a href="/context/{contextId}/helm/charts" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">Charts</a>
    </SidebarGroup>

    <SidebarGroup title="Custom Resources" icon={Database} bind:open={groups.custom}>
      <a href="/context/{contextId}/crd" class="block rounded-md px-3 py-1.5 text-sm hover:bg-bg-popover">CRDs</a>
    </SidebarGroup>
  </nav>
</aside>
