<script lang="ts">
  import Card from '$lib/components/ui/Card.svelte';
  import Select from '$lib/components/ui/Select.svelte';
  import { Palette } from 'lucide-svelte';
  import { settingsStore, type Theme } from '$lib/stores/settings.svelte';

  const themeOptions: Theme[] = ['kore', 'kore-light', 'rusty', 'rusty-light', 'dracula', 'alucard'];
</script>

<div class="max-w-3xl space-y-6">
  <div class="space-y-2">
    <h1 class="text-2xl font-bold">Settings</h1>
    <p class="text-text-muted">
      Manage your application preferences and settings.
    </p>
  </div>

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
        <div>
          <label for="theme-select" class="block text-sm font-medium mb-2">Theme</label>
          <div class="w-64">
            <Select 
              id="theme-select"
              options={themeOptions}
              value={settingsStore.value.theme}
              onselect={(val) => settingsStore.setTheme(val as Theme)}
              placeholder="Select Theme"
            />
          </div>
        </div>
      </div>
    </div>
  </Card>
</div>
