<template>
  <div class="ui-empty">
    <div class="ui-empty-icon">
      <slot name="icon">
        <Inbox :size="32" />
      </slot>
    </div>
    <p class="ui-empty-text">{{ description }}</p>
    <slot />
  </div>
</template>

<script setup lang="ts">
import { Inbox } from 'lucide-vue-next';

withDefaults(
  defineProps<{
    description?: string;
  }>(),
  { description: '暂无数据' },
);
</script>

<style scoped>
.ui-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 32px 16px;
  color: var(--muted-foreground);
}
.ui-empty-icon {
  opacity: 0.6;
}
.ui-empty-text {
  margin: 0;
  font-size: 13px;
}
</style>
