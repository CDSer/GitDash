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
  border-radius: 5px;
  font-weight: 500;
  letter-spacing: -0.01em;
  white-space: nowrap;
}
.ui-tag--default {
  background-color: var(--muted);
  color: var(--muted-foreground);
}
.ui-tag--primary {
  background-color: color-mix(in srgb, var(--primary) 14%, transparent);
  color: var(--primary);
}
.ui-tag--success {
  background-color: color-mix(in srgb, var(--sys-green) 16%, transparent);
  color: var(--sys-green);
}
.ui-tag--danger {
  background-color: color-mix(in srgb, var(--destructive) 14%, transparent);
  color: var(--destructive);
}
.ui-tag--warning {
  background-color: color-mix(in srgb, var(--sys-orange) 16%, transparent);
  color: var(--sys-orange);
}
.ui-tag--info {
  background-color: color-mix(in srgb, var(--primary) 10%, transparent);
  color: var(--muted-foreground);
}
.dark .ui-tag--success {
  color: var(--sys-green);
}
.dark .ui-tag--warning {
  color: var(--sys-orange);
}
</style>
