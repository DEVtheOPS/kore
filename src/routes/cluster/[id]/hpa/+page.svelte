<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("HPA");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">HPA Placeholder</h2>
</div>
