<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("Limit Ranges");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">Limit Ranges Placeholder</h2>
</div>
