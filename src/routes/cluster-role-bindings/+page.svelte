<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("Cluster Role Bindings");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">Cluster Role Bindings Placeholder</h2>
</div>
