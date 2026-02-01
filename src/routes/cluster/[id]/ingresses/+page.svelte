<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("Ingresses");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">Ingresses Placeholder</h2>
</div>
