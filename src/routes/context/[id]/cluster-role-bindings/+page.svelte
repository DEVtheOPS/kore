<script lang="ts">
  import WorkloadList from "$lib/components/WorkloadList.svelte";
</script>

<WorkloadList
  title="ClusterRoleBindings"
  listCommand="cluster_list_cluster_role_bindings"
  deleteCommand="cluster_delete_cluster_role_binding"
/>
