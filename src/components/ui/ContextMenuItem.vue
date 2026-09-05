<template>
  <ContextMenuItem
    :class="cn('ui-dropdown-item', danger && 'is-danger', disabled && 'is-disabled')"
    :disabled="disabled"
    @select="emit('click')"
  >
    <component :is="icon" v-if="icon" :size="14" class="ui-dropdown-item-icon" />
    <span class="truncate"><slot /></span>
  </ContextMenuItem>
</template>

<script setup lang="ts">
import type { Component } from 'vue';
import { ContextMenuItem } from 'reka-ui';
import { cn } from '../../lib/utils';

defineProps<{
  icon?: Component;
  disabled?: boolean;
  danger?: boolean;
}>();

const emit = defineEmits<{ (e: 'click'): void }>();
</script>
