<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("ReplicaSets");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">ReplicaSets Placeholder</h2>
</div>
