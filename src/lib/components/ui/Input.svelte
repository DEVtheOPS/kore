<script lang="ts">
  interface Props {
    value?: string;
    placeholder?: string;
    type?: string;
    class?: string;
    id?: string;
    onkeydown?: (e: KeyboardEvent) => void;
    oninput?: (e: Event) => void;
  }

  let { 
    value = $bindable(''), 
    placeholder = '', 
    type = 'text',
    class: className = '',
    id,
    onkeydown,
    oninput,
    ...rest
  }: Props = $props();
</script>

<input
  {type}
  {id}
  bind:value
  {placeholder}
  {oninput}
  {onkeydown}
  class="
    flex w-full rounded-input border border-border-main bg-bg-panel px-3 py-2 text-sm text-text-main 
    placeholder:text-text-muted 
    focus:outline-none focus:ring-1 focus:ring-primary focus:border-primary
    disabled:cursor-not-allowed disabled:opacity-50
    {className}
  "
  {...rest}
/>
