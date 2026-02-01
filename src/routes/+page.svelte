<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("Nodes");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">Nodes Placeholder</h2>
</div>
