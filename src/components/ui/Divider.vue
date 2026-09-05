<template>
  <div
    :class="direction === 'vertical' ? 'ui-divider ui-divider--v' : 'ui-divider ui-divider--h'"
    role="separator"
  />
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    direction?: 'horizontal' | 'vertical';
  }>(),
  { direction: 'horizontal' },
);
</script>

<style scoped>
.ui-divider--h {
  width: 100%;
  height: 1px;
  background-color: var(--border);
}
.ui-divider--v {
  width: 1px;
  align-self: stretch;
  min-height: 16px;
  background-color: var(--border);
}
</style>
