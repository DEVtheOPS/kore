<script lang="ts">
 import WorkloadList from "$lib/components/WorkloadList.svelte";
</script>
<WorkloadList title="Deployments" listCommand="cluster_list_deployments" deleteCommand="cluster_delete_deployment" />
