<template>
  <button type="button" :class="['ui-dropdown-item', { 'is-disabled': disabled, 'is-danger': danger }]" :disabled="disabled" @click="(e: MouseEvent) => emit('click', e)">
    <component :is="icon" v-if="icon" :size="14" class="ui-dropdown-item-icon" />
    <span class="truncate"><slot /></span>
  </button>
</template>

<script setup lang="ts">
import type { Component } from 'vue';

defineProps<{
  icon?: Component;
  disabled?: boolean;
  danger?: boolean;
  divided?: boolean;
}>();

const emit = defineEmits<{ (e: 'click', ev: MouseEvent): void }>();
</script>

<style scoped>
.ui-dropdown-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 7px 8px;
  font-size: 13px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: inherit;
  text-align: left;
  cursor: pointer;
}
.ui-dropdown-item:hover:not(.is-disabled) {
  background-color: var(--accent);
  color: var(--accent-foreground);
}
.ui-dropdown-item.is-disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
.ui-dropdown-item.is-danger {
  color: var(--destructive);
}
.ui-dropdown-item.is-danger:hover:not(.is-disabled) {
  background-color: color-mix(in oklab, var(--destructive) 16%, transparent);
  color: var(--destructive);
}
.ui-dropdown-item-icon {
  flex-shrink: 0;
}
</style>
