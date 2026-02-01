<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("Pod Disruption Budgets");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">Pod Disruption Budgets Placeholder</h2>
</div>
