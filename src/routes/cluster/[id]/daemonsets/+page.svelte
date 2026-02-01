<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("DaemonSets");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">DaemonSets Placeholder</h2>
</div>
