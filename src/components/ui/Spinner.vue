<template>
  <span :class="['ui-spinner', `ui-spinner--${size}`]" :style="{ color: color || 'var(--muted-foreground)' }" />
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    size?: 'sm' | 'md' | 'lg';
    color?: string;
  }>(),
  { size: 'md' },
);
</script>

<style scoped>
.ui-spinner {
  display: inline-block;
  border-radius: 9999px;
  border-style: solid;
  border-color: currentColor;
  border-right-color: transparent;
  animation: ui-spin 0.7s linear infinite;
  vertical-align: middle;
}
.ui-spinner--sm {
  width: 14px;
  height: 14px;
  border-width: 2px;
}
.ui-spinner--md {
  width: 18px;
  height: 18px;
  border-width: 2px;
}
.ui-spinner--lg {
  width: 26px;
  height: 26px;
  border-width: 3px;
}
@keyframes ui-spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
