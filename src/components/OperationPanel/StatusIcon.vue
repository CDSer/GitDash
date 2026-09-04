<!--
  状态图标组件
  显示任务状态（pending / running / success / error）
-->
<template>
  <el-tooltip :content="tooltipText" placement="top">
    <el-icon :size="16" :color="iconColor">
      <Clock v-if="status === 'pending'" />
      <Loading v-else-if="status === 'running'" />
      <CircleCheckFilled v-else-if="status === 'success'" />
      <CircleCloseFilled v-else />
    </el-icon>
  </el-tooltip>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Clock, Loading, CircleCheckFilled, CircleCloseFilled } from '@element-plus/icons-vue';

const props = defineProps<{
  status: 'pending' | 'running' | 'success' | 'error';
}>();

const iconColor = computed(() => {
  switch (props.status) {
    case 'running':
      return 'var(--el-color-primary)';
    case 'success':
      return 'var(--el-color-success)';
    case 'error':
      return 'var(--el-color-danger)';
    default:
      return 'var(--el-text-color-secondary)';
  }
});

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
