<script lang="ts">
  import WorkloadList from "$lib/components/WorkloadList.svelte";
</script>

<WorkloadList
  title="Namespaces"
  listCommand="cluster_list_namespaces_detailed"
  deleteCommand="cluster_delete_namespace"
/>
