<!--
  状态图标组件
  显示任务状态（pending / running / success / error）
-->
<template>
  <span :title="tooltipText" class="status-icon" :class="`status-icon--${status}`">
    <LoaderCircle v-if="status === 'running'" :size="16" class="spin" />
    <Clock v-else-if="status === 'pending'" :size="16" />
    <CircleCheck v-else-if="status === 'success'" :size="16" />
    <CircleX v-else :size="16" />
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Clock, LoaderCircle, CircleCheck, CircleX } from 'lucide-vue-next';

const props = defineProps<{
  status: 'pending' | 'running' | 'success' | 'error';
}>();

const tooltipText = computed(() => {
  switch (props.status) {
    case 'running':
      return '运行中';
    case 'success':
      return '成功';
    case 'error':
      return '失败';
    default:
      return '等待中';
  }
});
</script>

<style scoped>
.status-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
.status-icon--running {
  color: var(--primary);
}
.status-icon--success {
  color: oklch(0.7 0.18 150);
}
.status-icon--error {
  color: var(--destructive);
}
.status-icon--pending {
  color: var(--muted-foreground);
}
.spin {
  animation: status-spin 0.9s linear infinite;
}
@keyframes status-spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
