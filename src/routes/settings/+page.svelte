<script lang="ts">
  import Card from '$lib/components/ui/Card.svelte';
  import Select from '$lib/components/ui/Select.svelte';
  import { Palette, Code } from 'lucide-svelte';
  import { settingsStore, type Theme, type CodeTheme } from '$lib/stores/settings.svelte';

  const themeOptions: Theme[] = ['kore', 'kore-light', 'rusty', 'rusty-light', 'dracula', 'alucard'];
  const codeThemeOptions: CodeTheme[] = ['same-as-app', 'kore', 'kore-light', 'rusty', 'rusty-light', 'dracula', 'alucard'];
</script>

<div class="max-w-3xl space-y-6 p-6">
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
        <div class="space-y-4">
          <div>
            <label for="theme-select" class="block text-sm font-medium mb-2">App Theme</label>
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

          <div>
            <label for="code-theme-select" class="block text-sm font-medium mb-2">Code Editor Theme</label>
            <p class="text-text-muted text-xs mb-2">
              Choose a different theme for code blocks and YAML editors
            </p>
            <div class="w-64">
              <Select
                id="code-theme-select"
                options={codeThemeOptions}
                value={settingsStore.value.codeTheme}
                onselect={(val) => settingsStore.setCodeTheme(val as CodeTheme)}
                placeholder="Select Code Theme"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  </Card>
</div>
