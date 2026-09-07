<script lang="ts">
  import { X } from 'lucide-svelte';
  import Button from './Button.svelte';

  let {
    open = $bindable(false),
    title,
    label = 'Name',
    placeholder = '',
    confirmLabel = 'Create',
    initialValue = '',
    validate,
    onConfirm,
  }: {
    open: boolean;
    title: string;
    label?: string;
    placeholder?: string;
    confirmLabel?: string;
    initialValue?: string;
    /** Return an error message to block submission, or null when valid. */
    validate?: (value: string) => string | null;
    /** Resolve to complete; throw/reject to show the error inline and keep the modal open. */
    onConfirm: (value: string) => Promise<void> | void;
  } = $props();

  // Unique ids so multiple instances can coexist on a page
  const uid = Math.random().toString(36).slice(2, 8);
  const titleId = `prompt-modal-title-${uid}`;
  const inputId = `prompt-modal-input-${uid}`;

  let value = $state('');
  let error = $state<string | null>(null);
  let submitting = $state(false);
  let inputEl = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (open) {
      value = initialValue;
      error = null;
      submitting = false;
      // Focus the input once rendered
      queueMicrotask(() => inputEl?.focus());
    }
  });

  function close() {
    if (submitting) return;
    open = false;
  }

  async function submit() {
    const trimmed = value.trim();
    const validationError = validate?.(trimmed) ?? (trimmed ? null : `${label} is required.`);
    if (validationError) {
      error = validationError;
      return;
    }

    submitting = true;
    error = null;
    try {
      await onConfirm(trimmed);
      open = false;
    } catch (e) {
      error = String(e);
    } finally {
      submitting = false;
    }
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="fixed inset-0 bg-black/50 z-50 flex items-center justify-center"
    onclick={close}
    onkeydown={(e) => e.key === 'Escape' && close()}
    role="presentation"
  >
    <div
      class="bg-bg-main border border-border-subtle rounded-lg shadow-xl w-full max-w-md flex flex-col"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => {
        if (e.key === 'Escape') close();
        e.stopPropagation();
      }}
      role="dialog"
      aria-modal="true"
      aria-labelledby={titleId}
      tabindex="-1"
    >
      <div class="flex items-center justify-between px-5 py-4 border-b border-border-subtle">
        <h2 id={titleId} class="font-semibold">{title}</h2>
        <button
          class="p-1 rounded hover:bg-bg-panel text-text-muted hover:text-text-main transition-colors"
          onclick={close}
          aria-label="Close"
        >
          <X size={18} />
        </button>
      </div>

      <form
        class="px-5 py-4 space-y-3"
        onsubmit={(e) => {
          e.preventDefault();
          submit();
        }}
      >
        <label class="block text-sm text-text-muted" for={inputId}>{label}</label>
        <input
          id={inputId}
          bind:this={inputEl}
          bind:value
          {placeholder}
          disabled={submitting}
          class="w-full px-3 py-2 bg-bg-panel border border-border-main rounded-md text-sm text-text-main focus:outline-none focus:border-primary disabled:opacity-60"
        />
        {#if error}
          <p class="text-sm text-error">{error}</p>
        {/if}

        <div class="flex items-center justify-end gap-2 pt-2">
          <Button variant="outline" type="button" onclick={close} disabled={submitting}>Cancel</Button>
          <Button type="submit" disabled={submitting}>
            {submitting ? 'Working...' : confirmLabel}
          </Button>
        </div>
      </form>
    </div>
  </div>
{/if}
