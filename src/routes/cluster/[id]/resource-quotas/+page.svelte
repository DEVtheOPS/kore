<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("Resource Quotas");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">Resource Quotas Placeholder</h2>
</div>
