<template>
  <span :class="cn('ui-tag', `ui-tag--${variant}`)">
    <slot />
  </span>
</template>

<script setup lang="ts">
import { cn } from '../../lib/utils';

type TagVariant = 'default' | 'success' | 'danger' | 'warning' | 'info' | 'primary';

withDefaults(
  defineProps<{
    variant?: TagVariant;
    class?: string;
  }>(),
  { variant: 'default' },
);
</script>

<style scoped>
.ui-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  height: 18px;
  padding: 0 7px;
  font-size: 11px;
  line-height: 1;
  border-radius: 4px;
  font-weight: 500;
  white-space: nowrap;
}
.ui-tag--default {
  background-color: var(--muted);
  color: var(--muted-foreground);
}
.ui-tag--primary {
  background-color: color-mix(in oklab, var(--primary) 18%, transparent);
  color: var(--primary);
}
.ui-tag--success {
  background-color: color-mix(in oklab, oklch(0.7 0.18 150) 18%, transparent);
  color: oklch(0.6 0.16 150);
}
.ui-tag--danger {
  background-color: color-mix(in oklab, var(--destructive) 18%, transparent);
  color: var(--destructive);
}
.ui-tag--warning {
  background-color: color-mix(in oklab, oklch(0.8 0.16 85) 20%, transparent);
  color: oklch(0.7 0.15 75);
}
.ui-tag--info {
  background-color: color-mix(in oklab, var(--primary) 14%, transparent);
  color: var(--muted-foreground);
}
</style>
