<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("ConfigMaps");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">ConfigMaps Placeholder</h2>
</div>
