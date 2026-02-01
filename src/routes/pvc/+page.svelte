<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("Persistent Volume Claims");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">Persistent Volume Claims Placeholder</h2>
</div>
