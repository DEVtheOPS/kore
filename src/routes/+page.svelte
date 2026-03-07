<script lang="ts">
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import { confirm } from '@tauri-apps/plugin-dialog';
  import { ExternalLink, Pin, PinOff, Settings as SettingsIcon, Trash2, Users as UsersIcon } from 'lucide-svelte';

  import DataTable from '$lib/components/ui/DataTable.svelte';
  import type { Column } from '$lib/components/ui/DataTable.svelte';
  import type { MenuItem } from '$lib/components/ui/Menu.svelte';
  import { clustersStore, type Cluster } from '$lib/stores/clusters.svelte';
  import { contextsStore, type ContextRecord } from '$lib/stores/contexts.svelte';
  import { usersStore, type UserRecord } from '$lib/stores/users.svelte';

  let clusterSearch = $state('');
  let userSearch = $state('');
  let contextSearch = $state('');

  const clusterColumns: Column[] = [
    { id: 'display_name', label: 'Name', sortable: true },
    { id: 'name', label: 'Kubeconfig Name', sortable: true },
    { id: 'server', label: 'Server', sortable: true },
    { id: 'context_count', label: 'Contexts', sortable: true },
    { id: 'tags', label: 'Tags' },
  ];

  const userColumns: Column[] = [
    { id: 'display_name', label: 'Name', sortable: true },
    { id: 'name', label: 'Kubeconfig Name', sortable: true },
    { id: 'context_count', label: 'Contexts', sortable: true },
    { id: 'updated_at', label: 'Updated', sortable: true },
  ];

  const contextColumns: Column[] = [
    { id: 'icon', label: 'Icon' },
    { id: 'display_name', label: 'Name', sortable: true },
    { id: 'cluster_display_name', label: 'Cluster', sortable: true },
    { id: 'user_display_name', label: 'User', sortable: true },
    { id: 'namespace', label: 'Namespace', sortable: true },
    { id: 'last_accessed', label: 'Last Accessed', sortable: true },
  ];

  onMount(async () => {
    await Promise.all([clustersStore.load(), usersStore.load(), contextsStore.load()]);
  });

  const filteredClusters = $derived.by(() =>
    clustersStore.clusters.filter((cluster) => {
      const search = clusterSearch.toLowerCase();
      return [cluster.display_name, cluster.name, cluster.server ?? '', cluster.description ?? '']
        .some((value) => value.toLowerCase().includes(search));
    }),
  );

  const filteredUsers = $derived.by(() =>
    usersStore.users.filter((user) => {
      const search = userSearch.toLowerCase();
      return [user.display_name, user.name].some((value) => value.toLowerCase().includes(search));
    }),
  );

  const filteredContexts = $derived.by(() =>
    contextsStore.contexts.filter((context) => {
      const search = contextSearch.toLowerCase();
      return [
        context.display_name,
        context.cluster_display_name,
        context.user_display_name,
        context.name,
        context.namespace ?? '',
      ].some((value) => value.toLowerCase().includes(search));
    }),
  );

  function formatTimestamp(timestamp: number): string {
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 30) return `${diffDays}d ago`;
    return date.toLocaleDateString();
  }

  function getColorForString(str: string): string {
    let hash = 0;
    for (let i = 0; i < str.length; i += 1) {
      hash = str.charCodeAt(i) + ((hash << 5) - hash);
    }
    const hue = hash % 360;
    return `hsl(${hue}, 60%, 50%)`;
  }

  async function getClusterActions(cluster: Cluster): Promise<MenuItem[]> {
    return [
      {
        label: 'Settings',
        action: () => goto(`/cluster/${cluster.id}/settings`),
      },
      {
        label: 'Delete',
        action: async () => {
          const confirmed = await confirm(
            `Delete cluster "${cluster.display_name}" and all of its contexts?`,
            { title: 'Delete Cluster', kind: 'warning' },
          );

          if (confirmed) {
            await clustersStore.remove(cluster.id);
            await contextsStore.load();
          }
        },
      },
    ];
  }

  function clusterActions(cluster: Cluster): MenuItem[] {
    return [
      { label: 'Settings', action: () => goto(`/cluster/${cluster.id}/settings`) },
      {
        label: 'Delete',
        action: async () => {
          const confirmed = await confirm(
            `Delete cluster "${cluster.display_name}" and all of its contexts?`,
            { title: 'Delete Cluster', kind: 'warning' },
          );
          if (confirmed) {
            await clustersStore.remove(cluster.id);
            await contextsStore.load();
          }
        },
      },
    ];
  }

  function userActions(user: UserRecord): MenuItem[] {
    return [
      {
        label: 'Delete',
        action: async () => {
          try {
            await usersStore.remove(user.id, false);
          } catch (error) {
            const message = error instanceof Error ? error.message : String(error);
            const confirmed = await confirm(
              `${message}\n\nDelete the user and all linked contexts instead?`,
              { title: 'Delete User', kind: 'warning' },
            );
            if (confirmed) {
              await usersStore.remove(user.id, true);
              await contextsStore.load();
            }
          }
        },
      },
    ];
  }

  function contextActions(context: ContextRecord): MenuItem[] {
    return [
      {
        label: 'Open',
        action: () => goto(`/context/${context.id}`),
      },
      {
        label: context.is_pinned ? 'Unpin from Sidebar' : 'Pin to Sidebar',
        action: async () => {
          await contextsStore.togglePinned(context.id);
        },
      },
      {
        label: 'Settings',
        action: () => goto(`/context/${context.id}/settings`),
      },
      {
        label: 'Delete',
        action: async () => {
          const confirmed = await confirm(
            `Delete context "${context.display_name}"?`,
            { title: 'Delete Context', kind: 'warning' },
          );
          if (confirmed) {
            await contextsStore.remove(context.id);
          }
        },
      },
    ];
  }
</script>

<div class="flex h-full w-full flex-col overflow-hidden bg-bg-panel">
  <div class="border-b border-border-subtle bg-bg-main p-6">
    <h1 class="text-2xl font-bold">Configuration Vault</h1>
    <p class="mt-1 text-text-muted">Manage encrypted clusters, users, and contexts from one place.</p>
  </div>

  <div class="flex-1 space-y-8 overflow-auto p-6">
    <section class="space-y-3">
      <div>
        <h2 class="text-lg font-semibold">Clusters</h2>
        <p class="text-sm text-text-muted">Server definitions stored in the encrypted vault.</p>
      </div>

      <DataTable
        data={filteredClusters}
        columns={clusterColumns}
        bind:search={clusterSearch}
        loading={clustersStore.loading}
        onRefresh={() => clustersStore.load()}
        storageKey="overview-clusters"
        actions={clusterActions}
      >
        {#snippet children({ row: cluster, column })}
          {#if column.id === 'display_name'}
            <span class="font-medium">{cluster.display_name}</span>
          {:else if column.id === 'name'}
            <span class="font-mono text-sm text-text-muted">{cluster.name}</span>
          {:else if column.id === 'server'}
            <span class="text-sm text-text-muted">{cluster.server || '-'}</span>
          {:else if column.id === 'context_count'}
            <span>{cluster.context_count}</span>
          {:else if column.id === 'tags'}
            <div class="flex flex-wrap gap-1">
              {#each clustersStore.getTags(cluster) as tag}
                <span class="rounded border border-border-main px-2 py-0.5 text-xs">{tag}</span>
              {/each}
            </div>
          {/if}
        {/snippet}
      </DataTable>
    </section>

    <section class="space-y-3">
      <div>
        <h2 class="text-lg font-semibold">Users</h2>
        <p class="text-sm text-text-muted">Credential sets that can be reused across contexts.</p>
      </div>

      <DataTable
        data={filteredUsers}
        columns={userColumns}
        bind:search={userSearch}
        loading={usersStore.loading}
        onRefresh={() => usersStore.load()}
        storageKey="overview-users"
        actions={userActions}
      >
        {#snippet children({ row: user, column })}
          {#if column.id === 'display_name'}
            <div class="flex items-center gap-2">
              <UsersIcon size={14} class="text-primary" />
              <span class="font-medium">{user.display_name}</span>
            </div>
          {:else if column.id === 'name'}
            <span class="font-mono text-sm text-text-muted">{user.name}</span>
          {:else if column.id === 'context_count'}
            <span>{user.context_count}</span>
          {:else if column.id === 'updated_at'}
            <span class="text-sm text-text-muted">{formatTimestamp(user.updated_at)}</span>
          {/if}
        {/snippet}
      </DataTable>
    </section>

    <section class="space-y-3">
      <div>
        <h2 class="text-lg font-semibold">Contexts</h2>
        <p class="text-sm text-text-muted">Navigation targets combining cluster, user, namespace, and visual metadata.</p>
      </div>

      <DataTable
        data={filteredContexts}
        columns={contextColumns}
        bind:search={contextSearch}
        loading={contextsStore.loading}
        onRefresh={() => contextsStore.load()}
        storageKey="overview-contexts"
        actions={contextActions}
        onRowClick={(context) => goto(`/context/${context.id}`)}
      >
        {#snippet children({ row: context, column })}
          {#if column.id === 'icon'}
            <div class="flex items-center justify-center">
              <div
                class="flex h-9 w-9 items-center justify-center overflow-hidden rounded-full border-2 bg-bg-main"
                style:border-color={context.icon_ring_color || 'var(--color-primary)'}
              >
                {#if context.icon}
                  {#if context.icon.startsWith('http') || context.icon.startsWith('data:')}
                    <img
                      src={context.icon}
                      alt={context.display_name}
                      class="h-7 w-7 object-contain"
                    />
                  {:else}
                    <span class="text-xl leading-none">{context.icon}</span>
                  {/if}
                {:else}
                  <div
                    class="flex h-full w-full items-center justify-center text-sm font-bold text-white"
                    style:background-color={getColorForString(context.display_name)}
                  >
                    {context.display_name.charAt(0).toUpperCase()}
                  </div>
                {/if}
              </div>
            </div>
          {:else if column.id === 'display_name'}
            <div class="flex items-center gap-2">
              <span class="font-medium">{context.display_name}</span>
              {#if context.is_pinned}
                <Pin size={14} class="text-primary" />
              {/if}
            </div>
          {:else if column.id === 'cluster_display_name'}
            <span>{context.cluster_display_name}</span>
          {:else if column.id === 'user_display_name'}
            <span>{context.user_display_name}</span>
          {:else if column.id === 'namespace'}
            <span class="text-sm text-text-muted">{context.namespace || '-'}</span>
          {:else if column.id === 'last_accessed'}
            <span class="text-sm text-text-muted">{formatTimestamp(context.last_accessed)}</span>
          {/if}
        {/snippet}
      </DataTable>
    </section>
  </div>
</div>
