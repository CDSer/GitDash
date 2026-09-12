<template>
  <button
    :class="cn(buttonClass, props.class)"
    :disabled="disabled"
    :type="type"
    @click="(e: MouseEvent) => emit('click', e)"
  >
    <slot />
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { cn } from '../../lib/utils';

type Variant = 'primary' | 'default' | 'outline' | 'ghost' | 'destructive';
type Size = 'sm' | 'md' | 'lg' | 'icon';

const props = withDefaults(
  defineProps<{
    variant?: Variant;
    size?: Size;
    disabled?: boolean;
    type?: 'button' | 'submit' | 'reset';
    class?: string;
  }>(),
  { variant: 'default', size: 'md', disabled: false, type: 'button' },
);

const emit = defineEmits<{ (e: 'click', ev: MouseEvent): void }>();

const base =
  'inline-flex items-center justify-center gap-1.5 whitespace-nowrap rounded-[6px] font-medium ' +
  'transition-colors duration-150 ease-[cubic-bezier(0.25,0.1,0.25,1)] ' +
  'focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-primary/28 ' +
  'disabled:pointer-events-none disabled:opacity-40 select-none cursor-default active:scale-[0.98]';

const variants: Record<Variant, string> = {
  primary: 'bg-primary text-primary-foreground hover:bg-primary/90 shadow-[0_0.5px_1px_rgba(0,0,0,0.08)]',
  default:
    'bg-secondary text-secondary-foreground hover:bg-secondary/70 shadow-[0_0.5px_1px_rgba(0,0,0,0.06)]',
  outline:
    'border-[0.5px] border-border bg-card/60 text-foreground hover:bg-accent hover:text-accent-foreground',
  ghost: 'bg-transparent text-foreground hover:bg-accent hover:text-accent-foreground',
  destructive:
    'bg-destructive text-destructive-foreground hover:bg-destructive/90 shadow-[0_0.5px_1px_rgba(0,0,0,0.08)]',
};

const sizes: Record<Size, string> = {
  sm: 'h-7 px-2.5 text-xs',
  md: 'h-8 px-3 text-[13px]',
  lg: 'h-9 px-4 text-sm',
  icon: 'h-8 w-8 p-0',
};

const buttonClass = computed(() => cn(base, variants[props.variant], sizes[props.size]));
</script>
