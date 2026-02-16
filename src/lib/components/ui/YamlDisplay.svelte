<script lang="ts">
  import { onMount } from 'svelte';
  import hljs from 'highlight.js/lib/core';
  import yaml from 'highlight.js/lib/languages/yaml';
  import { settingsStore } from '$lib/stores/settings.svelte';

  let { code }: { code: string } = $props();

  let codeElement: HTMLElement;
  let highlighted = $state('');
  let codeTheme = $derived(settingsStore.effectiveCodeTheme);

  // Register YAML language once
  hljs.registerLanguage('yaml', yaml);

  // Highlight the code whenever it changes
  $effect(() => {
    if (code) {
      try {
        highlighted = hljs.highlight(code, { language: 'yaml' }).value;
      } catch (e) {
        console.error('Failed to highlight YAML:', e);
        highlighted = code;
      }
    }
  });
</script>

<div class={codeTheme}>
  <pre class="yaml-display"><code>{@html highlighted}</code></pre>
</div>

<style>
  pre {
    font-size: 0.75rem;
    overflow-x: auto;
    background-color: var(--bg-main);
    border-radius: 0.25rem;
    padding: 0.5rem;
    margin: 0;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  }

  code {
    background: transparent;
    color: var(--text-main);
  }

  /* Theme-aware syntax highlighting using CSS variables */
  :global(.yaml-display .hljs-attr) {
    color: var(--color-accent);
  }

  :global(.yaml-display .hljs-string) {
    color: var(--color-success);
  }

  :global(.yaml-display .hljs-number) {
    color: var(--color-warning);
  }

  :global(.yaml-display .hljs-literal),
  :global(.yaml-display .hljs-built_in) {
    color: var(--color-info);
  }

  :global(.yaml-display .hljs-comment) {
    color: var(--text-muted);
    font-style: italic;
  }

  :global(.yaml-display .hljs-meta) {
    color: var(--color-primary);
  }

  :global(.yaml-display .hljs-bullet),
  :global(.yaml-display .hljs-punctuation) {
    color: var(--text-muted);
  }

  :global(.yaml-display .hljs-type),
  :global(.yaml-display .hljs-title) {
    color: var(--color-primary);
  }

  :global(.yaml-display .hljs-keyword) {
    color: var(--color-primary);
  }
</style>
