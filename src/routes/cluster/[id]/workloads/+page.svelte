<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { headerStore } from "$lib/stores/header.svelte";
  import { activeClusterStore } from "$lib/stores/activeCluster.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import Button from "$lib/components/ui/Button.svelte";

  interface WorkloadSummary {
    id: string;
    name: string;
    namespace: string;
    status: string;
  }

  type WorkloadKey =
    | "deployments"
    | "statefulsets"
    | "daemonsets"
    | "replicasets"
    | "jobs"
    | "cronjobs";

  const sections: { key: WorkloadKey; title: string; route: string; command: string }[] = [
    { key: "deployments", title: "Deployments", route: "deployments", command: "cluster_list_deployments" },
    { key: "statefulsets", title: "StatefulSets", route: "statefulsets", command: "cluster_list_statefulsets" },
    { key: "daemonsets", title: "DaemonSets", route: "daemonsets", command: "cluster_list_daemonsets" },
    { key: "replicasets", title: "ReplicaSets", route: "replicasets", command: "cluster_list_replicasets" },
    { key: "jobs", title: "Jobs", route: "jobs", command: "cluster_list_jobs" },
    { key: "cronjobs", title: "CronJobs", route: "cronjobs", command: "cluster_list_cronjobs" },
  ];

  let loading = $state(false);
  let error = $state<string | null>(null);
  let counts = $state<Record<WorkloadKey, number>>({
    deployments: 0,
    statefulsets: 0,
    daemonsets: 0,
    replicasets: 0,
    jobs: 0,
    cronjobs: 0,
  });
  let recent = $state<Record<WorkloadKey, WorkloadSummary[]>>({
    deployments: [],
    statefulsets: [],
    daemonsets: [],
    replicasets: [],
    jobs: [],
    cronjobs: [],
  });

  $effect(() => {
    headerStore.setTitle("Workloads Overview");
  });

  $effect(() => {
    if (activeClusterStore.clusterId) {
      loadData();
    }
  });

  async function loadData() {
    if (!activeClusterStore.clusterId) return;
    loading = true;
    error = null;

    try {
      const namespace = activeClusterStore.activeNamespace === "all" ? null : activeClusterStore.activeNamespace;
      const responses = await Promise.all(
        sections.map((section) =>
          invoke<WorkloadSummary[]>(section.command, {
            clusterId: activeClusterStore.clusterId,
            namespace,
          })
        )
      );

      for (let i = 0; i < sections.length; i++) {
        const key = sections[i].key;
        const list = responses[i];
        counts[key] = list.length;
        recent[key] = list.slice(0, 5);
      }
    } catch (e) {
      console.error("Failed to load workload overview", e);
      error = "Failed to load workloads overview.";
    } finally {
      loading = false;
    }
  }
</script>

<div class="space-y-6">
  {#if error}
    <div class="p-3 bg-error/10 text-error rounded-md border border-error/20 flex items-center justify-between gap-3">
      <span>{error}</span>
      <Button variant="outline" size="sm" onclick={loadData}>Retry</Button>
    </div>
  {/if}

  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
    {#each sections as section (section.key)}
      <Card class="p-4">
        <div class="flex items-center justify-between mb-2">
          <h3 class="font-semibold">{section.title}</h3>
          <a href={`/cluster/${activeClusterStore.clusterId}/${section.route}`} class="text-primary text-sm hover:underline">
            Open
          </a>
        </div>
        <div class="text-3xl font-bold text-primary mb-3">{loading ? "..." : counts[section.key]}</div>
        <div class="space-y-1">
          {#if recent[section.key].length > 0}
            {#each recent[section.key] as item (item.id)}
              <div class="text-xs text-text-muted truncate" title={`${item.namespace}/${item.name}`}>
                {item.namespace}/{item.name}
              </div>
            {/each}
          {:else}
            <div class="text-xs text-text-muted">No resources found</div>
          {/if}
        </div>
      </Card>
    {/each}
  </div>
</div>
