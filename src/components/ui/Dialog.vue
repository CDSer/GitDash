<template>
  <Teleport to="body">
    <div v-if="modelValue" class="ui-dialog-overlay" @mousedown.self="close">
      <div class="ui-dialog-panel" :style="{ width: width }" role="dialog" aria-modal="true">
        <div v-if="title || $slots.header" class="ui-dialog-header">
          <slot name="header">
            <h3 class="ui-dialog-title">{{ title }}</h3>
          </slot>
          <button class="ui-dialog-close" type="button" title="关闭" @click="close">
            <X :size="16" />
          </button>
        </div>
        <div class="ui-dialog-body">
          <slot />
        </div>
        <div v-if="$slots.footer" class="ui-dialog-footer">
          <slot name="footer" />
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { X } from 'lucide-vue-next';

withDefaults(
  defineProps<{
    modelValue: boolean;
    title?: string;
    width?: string;
  }>(),
  { width: '520px' },
);

const emit = defineEmits<{ (e: 'update:modelValue', v: boolean): void }>();

function close() {
  emit('update:modelValue', false);
}
</script>

<style scoped>
.ui-dialog-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: rgba(0, 0, 0, 0.45);
  padding: 24px;
}
.ui-dialog-panel {
  display: flex;
  flex-direction: column;
  max-width: 92vw;
  max-height: 86vh;
  border-radius: 12px;
  background-color: var(--popover);
  color: var(--popover-foreground);
  border: 1px solid var(--border);
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.3);
  overflow: hidden;
  animation: ui-dialog-in 160ms cubic-bezier(0.16, 1, 0.3, 1);
}
.ui-dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 14px 16px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}
.ui-dialog-title {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
}
.ui-dialog-close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--muted-foreground);
  cursor: pointer;
}
.ui-dialog-close:hover {
  background-color: var(--accent);
  color: var(--accent-foreground);
}
.ui-dialog-body {
  padding: 16px;
  overflow: auto;
  min-height: 0;
}
.ui-dialog-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}
@keyframes ui-dialog-in {
  from {
    opacity: 0;
    transform: translateY(8px) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}
</style>
