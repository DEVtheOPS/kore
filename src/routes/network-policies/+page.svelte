<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("Network Policies");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">Network Policies Placeholder</h2>
</div>
