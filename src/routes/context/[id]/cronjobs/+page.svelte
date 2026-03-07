<script lang="ts">
 import WorkloadList from "$lib/components/WorkloadList.svelte";
</script>
<WorkloadList title="CronJobs" listCommand="cluster_list_cronjobs" deleteCommand="cluster_delete_cronjob" />
