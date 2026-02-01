<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("Storage Classes");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">Storage Classes Placeholder</h2>
</div>
