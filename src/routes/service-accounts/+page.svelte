<script lang="ts">
  import { headerStore } from '$lib/stores/header.svelte';

  $effect(() => {
    headerStore.setTitle("Service Accounts");
  });
</script>

<div class="p-4">
  <h2 class="text-xl">Service Accounts Placeholder</h2>
</div>
