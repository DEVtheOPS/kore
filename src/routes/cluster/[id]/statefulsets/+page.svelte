<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("StatefulSets");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">StatefulSets Placeholder</h2>
</div>
