<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("Events");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">Events Placeholder</h2>
</div>
