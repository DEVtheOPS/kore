<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { copyFile } from '@tauri-apps/plugin-fs';
  import Button from '$lib/components/ui/Button.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import Select from '$lib/components/ui/Select.svelte';
  import { Plus, Trash2, FileText, Palette } from 'lucide-svelte';
  import { settingsStore, type Theme } from '$lib/stores/settings.svelte';
  import { headerStore } from '$lib/stores/header.svelte';
  
  let configs = $state<string[]>([]);

  $effect(() => {
    headerStore.setTitle("Cluster Management");
  });

  // Function to load config list (mock for now, would list files in ~/.rustylens/configs)
  // Since we haven't made a command to list the files specifically, we can infer from list_contexts 
  // or add a new command. For now, let's just show the "Import" button functionality.
  
  async function importKubeconfig() {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'Kubeconfig',
          extensions: ['yaml', 'yml', 'conf', 'config']
        }]
      });
      
      if (selected && typeof selected === 'string') {
        try {
          const importedName = await invoke('import_kubeconfig', { path: selected });
          alert(`Successfully imported config: ${importedName}`);
          // Ideally we would refresh the list of imported configs here
        } catch (e) {
          console.error(e);
          alert(`Failed to import config: ${e}`);
        }
      }
    } catch (err) {
      console.error(err);
    }
  }
</script>

<div class="space-y-6">
  <div class="flex items-center justify-between">
    <div></div>
    <Button onclick={importKubeconfig}>
      <Plus size={16} class="mr-2" />
      Add Cluster Config
    </Button>
  </div>

  <div class="grid gap-4">
    <Card class="p-6">
      <div class="flex items-start gap-4">
        <div class="p-3 bg-bg-popover rounded-full">
          <Palette size={24} class="text-primary" />
        </div>
        <div class="flex-1">
          <h3 class="font-bold text-lg mb-1">Appearance</h3>
          <p class="text-text-muted text-sm mb-4">
            Customize the look and feel of the application.
          </p>
          <div class="w-64">
            <Select 
              options={['rusty', 'rusty-light', 'dracula', 'alucard']}
              value={settingsStore.value.theme}
              onselect={(val) => settingsStore.setTheme(val as Theme)}
              placeholder="Select Theme"
            />
          </div>
        </div>
      </div>
    </Card>

    <Card class="p-6">
      <div class="flex items-start gap-4">
        <div class="p-3 bg-bg-popover rounded-full">
          <FileText size={24} class="text-primary" />
        </div>
        <div class="flex-1">
          <h3 class="font-bold text-lg mb-1">Default Config</h3>
          <p class="text-text-muted text-sm mb-4">
            Loading from ~/.kube/config
          </p>
          <div class="flex items-center gap-2">
            <span class="text-xs bg-bg-main px-2 py-1 rounded border border-border-subtle">
              System Default
            </span>
          </div>
        </div>
      </div>
    </Card>
    
    <!-- List imported configs here -->
  </div>
</div>
