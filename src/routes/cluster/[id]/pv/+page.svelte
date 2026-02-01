<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("Persistent Volumes");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">Persistent Volumes Placeholder</h2>
</div>
