<script lang="ts">
 import WorkloadList from "$lib/components/WorkloadList.svelte";
</script>
<WorkloadList title="DaemonSets" listCommand="cluster_list_daemonsets" deleteCommand="cluster_delete_daemonset" />
