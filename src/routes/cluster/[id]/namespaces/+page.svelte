<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import WorkloadList from "$lib/components/WorkloadList.svelte";
  import PromptModal from "$lib/components/ui/PromptModal.svelte";
  import { activeClusterStore } from "$lib/stores/activeCluster.svelte";

  let showCreate = $state(false);

  // RFC 1123 DNS label — mirrors the backend validation so users get instant feedback.
  function validateNamespaceName(name: string): string | null {
    if (!name) return "Namespace name is required.";
    if (name.length > 63) return "Namespace name must be 63 characters or fewer.";
    if (!/^[a-z0-9]([a-z0-9-]*[a-z0-9])?$/.test(name)) {
      return "Use lowercase letters, digits, and '-' only; must start and end with a letter or digit.";
    }
    return null;
  }

  async function createNamespace(name: string, reload: () => Promise<void>) {
    await invoke("cluster_create_namespace", {
      clusterId: activeClusterStore.clusterId,
      name,
    });
    await Promise.all([reload(), activeClusterStore.fetchNamespaces()]);
  }
</script>

<WorkloadList
  title="Namespaces"
  listCommand="cluster_list_namespaces_detailed"
  deleteCommand="cluster_delete_namespace"
  createLabel="Create Namespace"
  onCreate={() => (showCreate = true)}
  onDeleted={() => activeClusterStore.fetchNamespaces()}
>
  {#snippet children({ reload })}
    <PromptModal
      bind:open={showCreate}
      title="Create Namespace"
      label="Namespace name"
      placeholder="my-namespace"
      confirmLabel="Create"
      validate={validateNamespaceName}
      onConfirm={(name) => createNamespace(name, reload)}
    />
  {/snippet}
</WorkloadList>
