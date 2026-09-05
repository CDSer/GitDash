<template>
  <span v-if="!status" class="text-muted-foreground">-</span>
  <Tag v-else-if="status.is_clean" variant="success">无变更</Tag>
  <div v-else class="flex items-center justify-center gap-1">
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
</script>
