<template>
  <div :class="['ui-alert', `ui-alert--${variant}`]">
    <component :is="icon" v-if="icon" :size="16" class="ui-alert-icon" />
    <span class="ui-alert-title">{{ title }}</span>
  </div>
</template>

<script setup lang="ts">
import { AlertCircle, CheckCircle2, Info, TriangleAlert } from 'lucide-vue-next';

type AlertVariant = 'error' | 'success' | 'warning' | 'info';

const props = withDefaults(
  defineProps<{
    variant?: AlertVariant;
    title?: string;
  }>(),
  { variant: 'info' },
);

const icon = {
  error: AlertCircle,
  success: CheckCircle2,
  warning: TriangleAlert,
  info: Info,
}[props.variant];
</script>

<style scoped>
.ui-alert {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: 8px;
  font-size: 12px;
  border: 1px solid transparent;
}
.ui-alert-icon {
  flex-shrink: 0;
}
.ui-alert--error {
  background-color: color-mix(in oklab, var(--destructive) 14%, transparent);
  color: var(--destructive);
  border-color: color-mix(in oklab, var(--destructive) 30%, transparent);
}
.ui-alert--success {
  background-color: color-mix(in oklab, oklch(0.7 0.18 150) 14%, transparent);
  color: oklch(0.6 0.16 150);
  border-color: color-mix(in oklab, oklch(0.7 0.18 150) 30%, transparent);
}
.ui-alert--warning {
  background-color: color-mix(in oklab, oklch(0.8 0.16 85) 16%, transparent);
  color: oklch(0.7 0.15 75);
  border-color: color-mix(in oklab, oklch(0.8 0.16 85) 32%, transparent);
}
.ui-alert--info {
  background-color: color-mix(in oklab, var(--primary) 12%, transparent);
  color: var(--foreground);
  border-color: color-mix(in oklab, var(--primary) 28%, transparent);
}
</style>
