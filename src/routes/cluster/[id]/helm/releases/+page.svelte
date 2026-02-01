<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("Helm Releases");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">Helm Releases Placeholder</h2>
</div>
