<!--
  变更数显示组件
  显示 staged / modified / untracked 文件数量
-->
<template>
  <span v-if="!status" class="no-data">-</span>
  <el-tag v-else-if="status.is_clean" size="small" type="success" effect="plain">
    无变更
  </el-tag>
  <div v-else class="changes">
    <el-tooltip v-if="status.staged > 0" :content="`${status.staged} 个已暂存`" placement="top">
      <el-tag size="small" type="primary" effect="plain">暂存 {{ status.staged }}</el-tag>
    </el-tooltip>
    <el-tooltip v-if="status.modified > 0" :content="`${status.modified} 个已修改`" placement="top">
      <el-tag size="small" type="warning" effect="plain">修改 {{ status.modified }}</el-tag>
    </el-tooltip>
    <el-tooltip v-if="status.untracked > 0" :content="`${status.untracked} 个未跟踪`" placement="top">
      <el-tag size="small" type="info" effect="plain">未跟踪 {{ status.untracked }}</el-tag>
    </el-tooltip>
  </div>
</template>

<script setup lang="ts">
import type { ProjectStatus } from '../../types';

defineProps<{
  status: ProjectStatus | null;
}>();
</script>

<style scoped>
.no-data {
  color: var(--el-text-color-placeholder);
}

.changes {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
}
</style>
