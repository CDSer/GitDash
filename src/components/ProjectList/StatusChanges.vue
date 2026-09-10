<template>
  <span v-if="!status" class="text-muted-foreground">-</span>
  <div v-else-if="status.conflict_count > 0" class="flex items-center justify-center">
    <button
      type="button"
      class="status-conflict-btn"
      title="点击查看并解决冲突"
      @click.stop="emit('open-conflicts')"
    >
      冲突 {{ status.conflict_count }}
    </button>
  </div>
  <div v-else class="flex items-center justify-center gap-1">
    <Tag v-if="status.in_progress" variant="warning" :title="`进行中：${status.in_progress.kind}`">
      {{ status.in_progress.kind === 'merge' ? '合并中' : status.in_progress.kind }}
    </Tag>
    <Tag v-if="status.staged > 0" variant="primary" :title="`${status.staged} 个已暂存`">
      暂存 {{ status.staged }}
    </Tag>
    <Tag v-if="status.modified > 0" variant="warning" :title="`${status.modified} 个已修改`">
      修改 {{ status.modified }}
    </Tag>
    <Tag v-if="status.untracked > 0" variant="info" :title="`${status.untracked} 个未跟踪`">
      未跟踪 {{ status.untracked }}
    </Tag>
  </div>
</template>

<script setup lang="ts">
import type { ProjectStatus } from '../../types';
import Tag from '../ui/Tag.vue';

defineProps<{
  status: ProjectStatus | null;
}>();

const emit = defineEmits<{
  (e: 'open-conflicts'): void;
}>();
</script>

<style scoped>
.status-conflict-btn {
  border: none;
  cursor: pointer;
  height: 18px;
  padding: 0 7px;
  font-size: 11px;
  line-height: 1;
  border-radius: 4px;
  font-weight: 600;
  background-color: color-mix(in oklab, #dc2626 18%, transparent);
  color: #dc2626;
}
.status-conflict-btn:hover {
  background-color: color-mix(in oklab, #dc2626 28%, transparent);
}
</style>
