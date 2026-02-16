<script lang="ts">
  import WorkloadList from "$lib/components/WorkloadList.svelte";
</script>

<WorkloadList title="CRDs" listCommand="cluster_list_crds" deleteCommand="cluster_delete_crd" />
