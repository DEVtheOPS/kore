<script lang="ts">
  import WorkloadList from "$lib/components/WorkloadList.svelte";
  import StatefulSetDetailDrawer from "$lib/components/StatefulSetDetailDrawer.svelte";

  let showDrawer = $state(false);
  let selected = $state<{ name: string; namespace: string }>({ name: "", namespace: "" });

  function openDetails(row: { name: string; namespace: string }) {
    selected = { name: row.name, namespace: row.namespace };
    showDrawer = true;
  }
</script>

<WorkloadList
  title="StatefulSets"
  listCommand="cluster_list_statefulsets"
  deleteCommand="cluster_delete_statefulset"
  onRowClick={openDetails}
>
  {#snippet children({ reload })}
    <StatefulSetDetailDrawer
      bind:open={showDrawer}
      bind:name={selected.name}
      bind:namespace={selected.namespace}
      onDeleted={() => reload()}
    />
  {/snippet}
</WorkloadList>
