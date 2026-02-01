<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    variant?: 'primary' | 'secondary' | 'ghost' | 'outline';
    size?: 'sm' | 'md' | 'lg';
    class?: string;
    children?: Snippet;
    onclick?: (e: MouseEvent) => void;
    type?: 'button' | 'submit' | 'reset';
    disabled?: boolean;
    [key: string]: any;
  }

  let { 
    variant = 'primary', 
    size = 'md', 
    class: className = '', 
    children, 
    onclick, 
    type = 'button',
    disabled = false,
    ...rest 
  }: Props = $props();

  const variants = {
    primary: 'bg-primary text-text-inverse hover:bg-primary-hover border-transparent',
    secondary: 'bg-bg-panel text-text-main hover:bg-bg-card border-border-main',
    ghost: 'bg-transparent text-text-main hover:bg-bg-card border-transparent',
    outline: 'bg-transparent text-text-main border-border-main hover:border-primary hover:text-primary',
  };

  const sizes = {
    sm: 'px-3 py-1.5 text-xs',
    md: 'px-4 py-2 text-sm',
    lg: 'px-6 py-3 text-base',
  };
</script>

<button
  {type}
  class="
    inline-flex items-center justify-center rounded-button font-medium transition-colors
    disabled:opacity-50 disabled:cursor-not-allowed
    {variants[variant]} {sizes[size]} {className}
  "
  {onclick}
  {disabled}
  {...rest}
>
  {@render children?.()}
</button>
