<template>
  <input
    :value="modelValue"
    :type="type"
    :placeholder="placeholder"
    :disabled="disabled"
    :class="cn(inputClass, props.class)"
    @input="(e: Event) => emit('update:modelValue', (e.target as HTMLInputElement).value)"
    @keydown.enter="(e: KeyboardEvent) => emit('enter', (e.target as HTMLInputElement).value)"
  />
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { cn } from '../../lib/utils';

const props = withDefaults(
  defineProps<{
    modelValue?: string;
    type?: string;
    placeholder?: string;
    disabled?: boolean;
    class?: string;
  }>(),
  { type: 'text', modelValue: '' },
);

const emit = defineEmits<{
  (e: 'update:modelValue', v: string): void;
  (e: 'enter', v: string): void;
}>();

const inputClass = computed(() =>
  cn(
    'h-8 w-full rounded-[6px] border-[0.5px] border-input bg-card/70 px-3 text-[13px] text-foreground',
    'placeholder:text-muted-foreground',
    'transition-[border-color,box-shadow] duration-150 ease-[cubic-bezier(0.25,0.1,0.25,1)]',
    'hover:border-input/70',
    'focus-visible:outline-none focus-visible:border-ring focus-visible:ring-[3.5px] focus-visible:ring-primary/28',
    'disabled:cursor-not-allowed disabled:opacity-40',
    props.class,
  ),
);
</script>
