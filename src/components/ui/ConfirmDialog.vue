<template>
  <Dialog :model-value="modelValue" :title="title" width="400px" @update:model-value="emit('update:modelValue', $event)">
    <p class="text-[13px] leading-relaxed text-muted-foreground">{{ message }}</p>
    <template #footer>
      <Button variant="ghost" @click="emit('cancel')">{{ cancelText }}</Button>
      <Button :variant="danger ? 'destructive' : 'primary'" @click="emit('confirm')">
        {{ confirmText }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import Dialog from './Dialog.vue';
import Button from './Button.vue';

withDefaults(
  defineProps<{
    modelValue: boolean;
    title?: string;
    message?: string;
    confirmText?: string;
    cancelText?: string;
    danger?: boolean;
  }>(),
  {
    title: '确认',
    message: '',
    confirmText: '确定',
    cancelText: '取消',
    danger: false,
  },
);

const emit = defineEmits<{
  (e: 'update:modelValue', v: boolean): void;
  (e: 'confirm'): void;
  (e: 'cancel'): void;
}>();
</script>
