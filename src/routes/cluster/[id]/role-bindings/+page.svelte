<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("Role Bindings");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">Role Bindings Placeholder</h2>
</div>
