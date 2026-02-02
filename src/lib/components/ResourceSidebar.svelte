<script lang="ts">
  import {
    Layers,
    Settings as SettingsIcon,
    Box,
    FileText,
    HardDrive,
    Network,
    Anchor,
    Database,
    Cpu,
    Activity,
    Shield,
  } from "lucide-svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import SidebarGroup from "$lib/components/ui/SidebarGroup.svelte";
  import type { Cluster } from "$lib/stores/clusters.svelte";

  let { cluster, namespaces, activeNamespace, onNamespaceChange } = $props<{
    cluster: Cluster;
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

  const clusterId = $derived(cluster.id);
</script>

<aside class="flex flex-col h-full w-64 bg-bg-sidebar border-r border-border-main text-text-main">
  <!-- Cluster Info Area -->
  <div class="p-4 border-b border-border-subtle space-y-3">
    <div class="flex items-center gap-2 px-1">
      {#if cluster.icon}
        {#if cluster.icon.startsWith("http") || cluster.icon.startsWith("data:")}
          <img src={cluster.icon} alt={cluster.name} class="w-6 h-6 rounded object-contain" />
        {:else}
          <span class="text-xl">{cluster.icon}</span>
        {/if}
      {:else}
        <div class="w-6 h-6 rounded bg-primary/20 flex items-center justify-center text-xs font-bold">
          {cluster.name.charAt(0).toUpperCase()}
        </div>
      {/if}
      <span class="font-bold text-lg truncate">{cluster.name}</span>
    </div>

    <!-- Cluster Settings Link -->
    <a
      href="/cluster/{clusterId}/settings"
      class="flex items-center gap-2 px-3 py-2 rounded-md hover:bg-bg-main text-sm text-text-muted hover:text-text-main transition-colors"
    >
      <SettingsIcon size={16} />
      <span>Cluster Settings</span>
    </a>

    <!-- Namespace Dropdown -->
    <div>
      <label
        for="namespace-select"
        class="text-xs font-semibold text-text-muted px-1 uppercase mb-1 block"
        >Namespace</label
      >
      <Select
        id="namespace-select"
        options={["all", ...namespaces]}
        value={activeNamespace}
        onselect={onNamespaceChange}
        placeholder="Namespace"
      />
    </div>
  </div>

  <!-- Navigation Links -->
  <nav class="flex-1 overflow-y-auto py-4 px-2 space-y-1">
    <a
      href="/cluster/{clusterId}"
      class="flex items-center gap-3 px-3 py-2 rounded-md hover:bg-bg-popover text-sm group"
    >
      <Cpu size={18} class="group-hover:text-primary transition-colors" />
      <span>Nodes</span>
    </a>

    <SidebarGroup title="Workloads" icon={Box} bind:open={groups.workloads}>
      <a
        href="/cluster/{clusterId}/workloads"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm">Overview</a
      >
      <a
        href="/cluster/{clusterId}/pods"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm">Pods</a
      >
      <a
        href="/cluster/{clusterId}/deployments"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm">Deployments</a
      >
      <a
        href="/cluster/{clusterId}/statefulsets"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm">StatefulSets</a
      >
      <a
        href="/cluster/{clusterId}/daemonsets"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm">DaemonSets</a
      >
      <a
        href="/cluster/{clusterId}/replicasets"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm">ReplicaSets</a
      >
      <a
        href="/cluster/{clusterId}/jobs"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm">Jobs</a
      >
      <a
        href="/cluster/{clusterId}/cronjobs"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm">CronJobs</a
      >
    </SidebarGroup>

    <SidebarGroup title="Configuration" icon={FileText} bind:open={groups.config}>
      <a
        href="/cluster/{clusterId}/config-maps"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm">ConfigMaps</a
      >
      <a
        href="/cluster/{clusterId}/secrets"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm">Secrets</a
      >
      <a
        href="/cluster/{clusterId}/resource-quotas"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm">Resource Quotas</a
      >
      <a
        href="/cluster/{clusterId}/limit-ranges"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm">Limit Ranges</a
      >
      <a
        href="/cluster/{clusterId}/hpa"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm"
        >HPA</a
      >
      <a
        href="/cluster/{clusterId}/pdb"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm"
        >Pod Disruption Budgets</a
      >
    </SidebarGroup>

    <SidebarGroup title="Network" icon={Network} bind:open={groups.network}>
      <a
        href="/cluster/{clusterId}/services"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm">Services</a
      >
      <a
        href="/cluster/{clusterId}/endpoints"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm">Endpoints</a
      >
      <a
        href="/cluster/{clusterId}/ingresses"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm">Ingresses</a
      >
      <a
        href="/cluster/{clusterId}/network-policies"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm">Network Policies</a
      >
    </SidebarGroup>

    <SidebarGroup title="Storage" icon={HardDrive} bind:open={groups.storage}>
      <a
        href="/cluster/{clusterId}/pvc"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm"
        >Persistent Volume Claims</a
      >
      <a
        href="/cluster/{clusterId}/pv"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm"
        >Persistent Volumes</a
      >
      <a
        href="/cluster/{clusterId}/storage-classes"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm">Storage Classes</a
      >
    </SidebarGroup>

    <SidebarGroup title="Access Control" icon={Shield} bind:open={groups.access}>
      <a
        href="/cluster/{clusterId}/service-accounts"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm">Service Accounts</a
      >
      <a
        href="/cluster/{clusterId}/roles"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm">Roles</a
      >
      <a
        href="/cluster/{clusterId}/role-bindings"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm">Role Bindings</a
      >
      <a
        href="/cluster/{clusterId}/cluster-roles"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm">Cluster Roles</a
      >
      <a
        href="/cluster/{clusterId}/cluster-role-bindings"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm">Cluster Role Bindings</a
      >
    </SidebarGroup>

    <a
      href="/cluster/{clusterId}/namespaces"
      class="flex items-center gap-3 px-3 py-2 rounded-md hover:bg-bg-popover text-sm group"
    >
      <Layers size={18} class="group-hover:text-primary transition-colors" />
      <span>Namespaces</span>
    </a>

    <a
      href="/cluster/{clusterId}/events"
      class="flex items-center gap-3 px-3 py-2 rounded-md hover:bg-bg-popover text-sm group"
    >
      <Activity size={18} class="group-hover:text-primary transition-colors" />
      <span>Events</span>
    </a>

    <SidebarGroup title="Helm" icon={Anchor} bind:open={groups.helm}>
      <a
        href="/cluster/{clusterId}/helm/releases"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm">Releases</a
      >
      <a
        href="/cluster/{clusterId}/helm/charts"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm">Charts</a
      >
    </SidebarGroup>

    <SidebarGroup title="Custom Resources" icon={Database} bind:open={groups.custom}>
      <a
        href="/cluster/{clusterId}/crd"
        class="block px-3 py-1.5 rounded-md hover:bg-bg-popover text-sm"
        >CRDs</a
      >
    </SidebarGroup>
  </nav>

  <!-- Settings Link -->
  <div class="p-4 border-t border-border-subtle">
    <a
      href="/settings"
      class="flex items-center gap-3 px-3 py-2 rounded-md hover:bg-bg-popover text-sm group"
    >
      <SettingsIcon size={18} class="group-hover:text-primary transition-colors" />
      <span>App Settings</span>
    </a>
  </div>
</aside>
