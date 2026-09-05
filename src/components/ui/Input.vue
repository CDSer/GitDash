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
    'h-8 w-full rounded-md border border-input bg-transparent px-3 text-[13px] text-foreground',
    'placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring',
    'disabled:cursor-not-allowed disabled:opacity-50',
    props.class,
  ),
);
</script>
