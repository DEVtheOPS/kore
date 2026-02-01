<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("Namespaces");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">Namespaces Placeholder</h2>
</div>
