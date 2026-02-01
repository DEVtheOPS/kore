<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("Cluster Roles");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">Cluster Roles Placeholder</h2>
</div>
