<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("Helm Charts");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">Helm Charts Placeholder</h2>
</div>
