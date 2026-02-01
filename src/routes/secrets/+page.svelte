<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("Secrets");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">Secrets Placeholder</h2>
</div>
