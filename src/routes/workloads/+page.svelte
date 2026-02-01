<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("Workloads Overview");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">Workloads Overview Placeholder</h2>
</div>
