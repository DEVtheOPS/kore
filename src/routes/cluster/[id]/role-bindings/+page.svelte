<script lang="ts">
  import WorkloadList from "$lib/components/WorkloadList.svelte";
</script>

<WorkloadList
  title="RoleBindings"
  listCommand="cluster_list_role_bindings"
  deleteCommand="cluster_delete_role_binding"
/>
