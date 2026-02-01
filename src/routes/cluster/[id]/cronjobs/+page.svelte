<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("CronJobs");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">CronJobs Placeholder</h2>
</div>
