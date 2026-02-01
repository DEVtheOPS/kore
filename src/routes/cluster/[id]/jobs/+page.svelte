<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("Jobs");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">Jobs Placeholder</h2>
</div>
