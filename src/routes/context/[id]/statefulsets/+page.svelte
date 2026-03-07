<script lang="ts">
 import WorkloadList from "$lib/components/WorkloadList.svelte";
</script>
<WorkloadList title="StatefulSets" listCommand="cluster_list_statefulsets" deleteCommand="cluster_delete_statefulset" />
