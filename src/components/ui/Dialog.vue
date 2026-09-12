<template>
  <DialogRoot :open="modelValue" @update:open="(v: boolean) => emit('update:modelValue', v)">
    <DialogPortal>
      <DialogOverlay
        class="ui-dialog-overlay data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:animate-in data-[state=open]:fade-in-0"
      />
      <DialogContent
        :class="[
          'ui-dialog-content',
          sizeClass,
          'data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:slide-in-from-bottom-2',
        ]"
        :style="contentStyle"
      >
        <DialogTitle v-if="title" class="ui-dialog-title">{{ title }}</DialogTitle>
        <DialogDescription class="sr-only" />
        <DialogClose class="ui-dialog-close" aria-label="关闭">
          <X :size="14" />
        </DialogClose>
        <div class="ui-dialog-body" :class="{ 'ui-dialog-body--flush': flush }">
          <slot />
        </div>
        <div v-if="$slots.footer" class="ui-dialog-footer"><slot name="footer" /></div>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>

<script setup lang="ts">
import { computed } from 'vue';
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

const props = withDefaults(
  defineProps<{
    modelValue: boolean;
    title?: string;
    /** 兼容旧用法的固定宽度，例如 520px */
    width?: string;
    /** 预设尺寸；xl 适合 diff 等大内容 */
    size?: 'md' | 'lg' | 'xl';
    /** 自定义高度，例如 90vh */
    height?: string;
    /** 去掉 body 内边距，让内容铺满 */
    flush?: boolean;
  }>(),
  { title: '', width: '', size: 'md', height: '', flush: false },
);

const emit = defineEmits<{ (e: 'update:modelValue', v: boolean): void }>();

const sizeClass = computed(() => {
  if (props.height || props.width) return '';
  if (props.size === 'xl') return 'ui-dialog-content--xl';
  if (props.size === 'lg') return 'ui-dialog-content--lg';
  return 'ui-dialog-content--md';
});

const contentStyle = computed(() => {
  const style: Record<string, string> = {};
  if (props.height) style.maxHeight = props.height;
  if (props.width) style.maxWidth = props.width;
  return style;
});
</script>

<style scoped>
.ui-dialog-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  background-color: rgba(0, 0, 0, 0.22);
  backdrop-filter: blur(8px) saturate(150%);
  -webkit-backdrop-filter: blur(8px) saturate(150%);
  padding: 24px;
}
.dark .ui-dialog-overlay {
  background-color: rgba(0, 0, 0, 0.4);
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
  background-color: var(--popover-glass);
  backdrop-filter: saturate(var(--glass-saturate)) blur(var(--glass-popover-blur));
  -webkit-backdrop-filter: saturate(var(--glass-saturate)) blur(var(--glass-popover-blur));
  color: var(--popover-foreground);
  border: 0.5px solid transparent;
  box-shadow: var(--glass-float-edge), var(--shadow-lg);
  overflow: hidden;
}
.ui-dialog-content--md {
  max-width: 520px;
}
.ui-dialog-content--lg {
  max-width: 900px;
}
.ui-dialog-content--xl {
  width: calc(100vw - 48px);
  max-width: 1400px;
  max-height: 92vh;
  height: 92vh;
}
.ui-dialog-title {
  margin: 0;
  padding: 14px 16px 12px 16px;
  font-family: var(--font-display);
  font-size: 15px;
  font-weight: 600;
  letter-spacing: -0.015em;
  flex-shrink: 0;
  padding-right: 40px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.ui-dialog-close {
  position: absolute;
  top: 11px;
  right: 11px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: var(--muted-foreground);
  cursor: default;
  transition: background-color 0.15s var(--ease-out), color 0.15s var(--ease-out);
}
.ui-dialog-close:hover {
  background-color: var(--accent);
  color: var(--foreground);
}
.ui-dialog-body {
  padding: 4px 16px 16px;
  overflow: auto;
  min-height: 0;
  flex: 1;
}
.ui-dialog-body--flush {
  padding: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
.ui-dialog-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 16px 14px;
  flex-shrink: 0;
  background: color-mix(in srgb, var(--muted) 50%, transparent);
}
</style>
