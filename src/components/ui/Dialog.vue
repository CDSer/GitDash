<template>
  <DialogRoot :open="modelValue" @update:open="(v: boolean) => emit('update:modelValue', v)">
    <DialogPortal>
      <DialogOverlay
        class="ui-dialog-overlay data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:animate-in data-[state=open]:fade-in-0"
      />
      <DialogContent
        class="ui-dialog-content data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:animate-in data-[state=open]:fade-in-0"
        :style="width ? { maxWidth: width } : undefined"
      >
        <DialogTitle v-if="title" class="ui-dialog-title">{{ title }}</DialogTitle>
        <DialogDescription class="sr-only" />
        <DialogClose class="ui-dialog-close" aria-label="关闭">
          <X :size="16" />
        </DialogClose>
        <div class="ui-dialog-body"><slot /></div>
        <div v-if="$slots.footer" class="ui-dialog-footer"><slot name="footer" /></div>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>

<script setup lang="ts">
import {
  DialogRoot,
  DialogPortal,
  DialogOverlay,
  DialogContent,
  DialogTitle,
  DialogDescription,
  DialogClose,
} from 'reka-ui';
import { X } from 'lucide-vue-next';

withDefaults(
  defineProps<{
    modelValue: boolean;
    title?: string;
    width?: string;
  }>(),
  { title: '', width: '520px' },
);

const emit = defineEmits<{ (e: 'update:modelValue', v: boolean): void }>();
</script>

<style scoped>
.ui-dialog-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  background-color: rgba(0, 0, 0, 0.45);
  padding: 24px;
}
.ui-dialog-content {
  position: fixed;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
  z-index: 1001;
  display: flex;
  flex-direction: column;
  width: calc(100% - 32px);
  max-height: 86vh;
  border-radius: 12px;
  background-color: var(--popover);
  color: var(--popover-foreground);
  border: 1px solid var(--border);
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.3);
  overflow: hidden;
}
.ui-dialog-title {
  margin: 0;
  padding: 14px 16px 12px 16px;
  font-size: 15px;
  font-weight: 600;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}
.ui-dialog-close {
  position: absolute;
  top: 12px;
  right: 12px;
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
</style>
