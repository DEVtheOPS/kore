<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("Services");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">Services Placeholder</h2>
</div>
