<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("Deployments");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">Deployments Placeholder</h2>
</div>
