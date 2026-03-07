<script lang="ts">
 import WorkloadList from "$lib/components/WorkloadList.svelte";
</script>
<WorkloadList title="Jobs" listCommand="cluster_list_jobs" deleteCommand="cluster_delete_job" />
