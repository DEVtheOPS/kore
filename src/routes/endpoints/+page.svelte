<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("Endpoints");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">Endpoints Placeholder</h2>
</div>
