<script lang="ts">
 import WorkloadList from "$lib/components/WorkloadList.svelte";
</script>
<WorkloadList title="ReplicaSets" listCommand="cluster_list_replicasets" deleteCommand="cluster_delete_replicaset" />
