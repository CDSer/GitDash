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
  letter-spacing: -0.01em;
}
.ui-alert-icon {
  flex-shrink: 0;
}
.ui-alert--error {
  background-color: color-mix(in srgb, var(--destructive) 10%, transparent);
  color: var(--destructive);
}
.ui-alert--success {
  background-color: color-mix(in srgb, var(--sys-green) 12%, transparent);
  color: color-mix(in srgb, var(--sys-green) 80%, var(--foreground));
}
.ui-alert--warning {
  background-color: color-mix(in srgb, var(--sys-orange) 12%, transparent);
  color: color-mix(in srgb, var(--sys-orange) 75%, var(--foreground));
}
.ui-alert--info {
  background-color: color-mix(in srgb, var(--primary) 8%, transparent);
  color: var(--foreground);
}
</style>
