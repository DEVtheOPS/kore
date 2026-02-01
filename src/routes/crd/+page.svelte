<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("CRD");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">CRD Placeholder</h2>
</div>
