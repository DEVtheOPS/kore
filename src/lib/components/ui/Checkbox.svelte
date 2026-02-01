<script lang="ts">
  interface Props {
    checked?: boolean;
    indeterminate?: boolean;
    onchange?: (checked: boolean) => void;
    disabled?: boolean;
    class?: string;
  }

  let { 
    checked = $bindable(false), 
    indeterminate = $bindable(false),
    onchange,
    disabled = false,
    class: className = '' 
  }: Props = $props();

  function handleChange(e: Event) {
    const target = e.target as HTMLInputElement;
    checked = target.checked;
    onchange?.(checked);
  }
</script>

<input 
  type="checkbox" 
  bind:checked 
  bind:indeterminate
  {disabled}
  onchange={handleChange}
  class="
    appearance-none h-4 w-4 rounded-sm border border-border-main bg-bg-card 
    checked:bg-primary checked:border-primary checked:text-text-inverse
    focus:outline-none focus:ring-2 focus:ring-primary/20
    disabled:opacity-50 disabled:cursor-not-allowed
    cursor-pointer transition-colors relative
    {className}
  "
/>

<style>
  input[type="checkbox"]:checked::after {
    content: "";
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 10px;
    height: 10px;
    background-image: url("data:image/svg+xml,%3csvg viewBox='0 0 16 16' fill='white' xmlns='http://www.w3.org/2000/svg'%3e%3cpath d='M12.207 4.793a1 1 0 010 1.414l-5 5a1 1 0 01-1.414 0l-2-2a1 1 0 011.414-1.414L6.5 9.086l4.293-4.293a1 1 0 011.414 0z'/%3e%3c/svg%3e");
    background-size: cover;
    background-repeat: no-repeat;
  }

  input[type="checkbox"]:indeterminate::after {
    content: "";
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 8px;
    height: 2px;
    background-color: currentColor; 
  }
  
  /* Use the semantic text color for the indeterminate dash if generic styling fails */
  input[type="checkbox"]:indeterminate {
    background-color: var(--color-primary);
    border-color: var(--color-primary);
  }
  input[type="checkbox"]:indeterminate::after {
    background-color: white;
  }
</style>
