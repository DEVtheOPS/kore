<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { EditorView, basicSetup } from 'codemirror';
  import { yaml } from '@codemirror/lang-yaml';
  import { EditorState } from '@codemirror/state';
  import { HighlightStyle, syntaxHighlighting } from '@codemirror/language';
  import { tags } from '@lezer/highlight';
  import { settingsStore } from '$lib/stores/settings.svelte';

  let {
    value = $bindable(''),
    readonly = false,
    placeholder = '',
  }: {
    value: string;
    readonly?: boolean;
    placeholder?: string;
  } = $props();

  let editorElement: HTMLDivElement;
  let editorView: EditorView | null = null;
  let codeTheme = $derived(settingsStore.effectiveCodeTheme);

  onMount(() => {
    // Custom syntax highlighting theme that uses CSS variables
    const customHighlightStyle = HighlightStyle.define([
      { tag: tags.keyword, color: 'var(--color-primary)' },
      { tag: tags.propertyName, color: 'var(--color-accent)' },
      { tag: tags.string, color: 'var(--color-success)' },
      { tag: tags.number, color: 'var(--color-warning)' },
      { tag: tags.bool, color: 'var(--color-info)' },
      { tag: tags.null, color: 'var(--text-muted)' },
      { tag: tags.comment, color: 'var(--text-muted)', fontStyle: 'italic' },
      { tag: tags.operator, color: 'var(--text-main)' },
      { tag: tags.punctuation, color: 'var(--text-muted)' },
      { tag: tags.bracket, color: 'var(--text-main)' },
      { tag: tags.variableName, color: 'var(--text-main)' },
    ]);

    // Create the editor state
    const startState = EditorState.create({
      doc: value,
      extensions: [
        basicSetup,
        yaml(),
        syntaxHighlighting(customHighlightStyle),
        EditorView.editable.of(!readonly),
        EditorView.lineWrapping,
        EditorView.updateListener.of((update) => {
          if (update.docChanged && !readonly) {
            value = update.state.doc.toString();
          }
        }),
        EditorView.theme({
          '&': {
            height: '100%',
            fontSize: '12px',
            backgroundColor: 'var(--bg-main)',
            color: 'var(--text-main)',
          },
          '.cm-scroller': {
            overflow: 'auto',
            fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace',
          },
          '.cm-content': {
            padding: '8px 0',
            caretColor: 'var(--color-primary)',
          },
          '.cm-line': {
            padding: '0 4px',
          },
          '.cm-cursor, .cm-dropCursor': {
            borderLeftColor: 'var(--color-primary)',
          },
          '&.cm-focused .cm-selectionBackground, ::selection': {
            backgroundColor: 'var(--color-primary)',
            opacity: '0.3',
          },
          '.cm-activeLine': {
            backgroundColor: 'var(--bg-panel)',
          },
          '.cm-gutters': {
            backgroundColor: 'var(--bg-sidebar)',
            color: 'var(--text-muted)',
            border: 'none',
          },
          '.cm-activeLineGutter': {
            backgroundColor: 'var(--bg-panel)',
          },
        }),
      ],
    });

    // Create the editor view
    editorView = new EditorView({
      state: startState,
      parent: editorElement,
    });
  });

  onDestroy(() => {
    if (editorView) {
      editorView.destroy();
    }
  });

  // Update editor content when value changes externally
  $effect(() => {
    if (editorView && value !== editorView.state.doc.toString()) {
      editorView.dispatch({
        changes: {
          from: 0,
          to: editorView.state.doc.length,
          insert: value,
        },
      });
    }
  });
</script>

<div class="h-full w-full {codeTheme}">
  <div bind:this={editorElement} class="h-full w-full border border-border-main rounded-md bg-bg-main overflow-hidden"></div>
</div>
