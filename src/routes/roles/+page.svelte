<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("Roles");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">Roles Placeholder</h2>
</div>
