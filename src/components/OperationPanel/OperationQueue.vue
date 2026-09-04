<!--
  操作队列面板组件
  显示批量操作的进度和结果（右下角浮层）
-->
<template>
  <el-card v-if="showPanel" class="operation-queue" shadow="always" :body-style="{ padding: '8px 12px' }">
    <template #header>
      <div class="queue-header">
        <span class="queue-title">操作队列</span>
        <div class="header-actions">
          <el-button text size="small" @click="clearCompleted">清除已完成</el-button>
          <el-button text size="small" :icon="Close" @click="showPanel = false" />
        </div>
      </div>
    </template>

    <el-empty v-if="tasks.length === 0" description="暂无任务" :image-size="60" />

    <div v-else class="task-list">
      <div v-for="task in tasks" :key="task.id" class="task-item">
        <div class="task-main">
          <div class="task-info">
            <div class="task-name">{{ task.projectName }}</div>
            <div class="task-op">{{ operationText(task.operation) }}</div>
          </div>
          <StatusIcon :status="task.status" />
        </div>

        <div v-if="task.message" class="task-message">{{ task.message }}</div>

        <el-progress
          v-if="task.status === 'running'"
          class="task-progress"
          :percentage="100"
          :show-text="false"
          :stroke-width="3"
          indeterminate
          :duration="2"
        />
      </div>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Close } from '@element-plus/icons-vue';
import { useOperationStore } from '../../stores/operationStore';
import StatusIcon from './StatusIcon.vue';

const operationStore = useOperationStore();

const tasks = computed(() => operationStore.tasks);
const showPanel = computed({
  get: () => operationStore.showPanel,
  set: (value) => operationStore.showPanel = value
});

function clearCompleted() {
  operationStore.clearCompleted();
}

function operationText(operation: string) {
  switch (operation) {
    case 'pull': return '拉取';
    case 'push': return '推送';
    case 'fetch': return '获取';
    default: return operation;
  }
}
</script>

<style scoped>
.operation-queue {
  position: fixed;
  right: 16px;
  bottom: 16px;
  width: 340px;
  max-height: 320px;
  z-index: 2000;
  display: flex;
  flex-direction: column;
}

.operation-queue :deep(.el-card__header) {
  padding: 8px 12px;
}

.operation-queue :deep(.el-card__body) {
  overflow-y: auto;
}

.queue-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.queue-title {
  font-size: 14px;
  font-weight: 600;
}

.header-actions {
  display: flex;
  align-items: center;
}

.task-list {
  display: flex;
  flex-direction: column;
}

.task-item + .task-item {
  border-top: 1px solid var(--el-border-color-lighter);
}

.task-item {
  padding: 6px 0;
}

.task-main {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.task-name {
  font-size: 13px;
  color: var(--el-text-color-primary);
}

.task-op {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.task-message {
  margin-top: 2px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  word-break: break-all;
}

.task-progress {
  margin-top: 6px;
}
</style>
