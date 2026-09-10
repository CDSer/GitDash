<template>
  <Tag v-if="!status" variant="info">未知</Tag>
  <Tag v-else-if="status.error" variant="danger" :title="status.error">错误</Tag>
  <Tag
    v-else-if="status.conflict_count > 0"
    variant="danger"
    :title="`存在 ${status.conflict_count} 个冲突文件`"
  >
    冲突
  </Tag>
  <Tag
    v-else-if="status.in_progress"
    variant="warning"
    :title="`进行中：${status.in_progress.kind}`"
  >
    {{ status.in_progress.kind === 'merge' ? '合并中' : status.in_progress.kind }}
  </Tag>
  <Tag v-else-if="status.is_clean" variant="success">干净</Tag>
  <Tag v-else variant="warning">有变更</Tag>
</template>

<script setup lang="ts">
import type { ProjectStatus } from '../../types';
import Tag from '../ui/Tag.vue';

defineProps<{
  status: ProjectStatus | null;
}>();
</script>
