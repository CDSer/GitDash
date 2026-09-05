<!--
  操作队列面板组件
  显示批量操作的进度和结果（右下角浮层）
-->
<template>
  <div v-if="showPanel" class="operation-queue">
    <div class="queue-header">
      <span class="queue-title">操作队列</span>
      <div class="header-actions">
        <Button variant="ghost" size="sm" @click="clearCompleted">清除已完成</Button>
        <Button variant="ghost" size="icon" title="关闭" @click="showPanel = false">
          <X :size="16" />
        </Button>
      </div>
    </div>

    <Empty v-if="tasks.length === 0" description="暂无任务" />

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

        <div v-if="task.status === 'running'" class="task-progress">
          <div class="task-progress-bar" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { X } from 'lucide-vue-next';
import { useOperationStore } from '../../stores/operationStore';
import StatusIcon from './StatusIcon.vue';
import Button from '../ui/Button.vue';
import Empty from '../ui/Empty.vue';

const operationStore = useOperationStore();

const tasks = computed(() => operationStore.tasks);
const showPanel = computed({
  get: () => operationStore.showPanel,
  set: (value) => (operationStore.showPanel = value),
});

function clearCompleted() {
  operationStore.clearCompleted();
}

function operationText(operation: string) {
  switch (operation) {
    case 'pull':
      return '拉取';
    case 'push':
      return '推送';
    case 'fetch':
      return '获取';
    default:
      return operation;
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
  border-radius: 10px;
  border: 1px solid var(--border);
  background-color: var(--popover);
  color: var(--popover-foreground);
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.25);
  overflow: hidden;
}
.queue-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  border-bottom: 1px solid var(--border);
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
  overflow-y: auto;
  padding: 4px 12px 10px;
}
.task-item + .task-item {
  border-top: 1px solid var(--border);
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
}
.task-op {
  font-size: 12px;
  color: var(--muted-foreground);
}
.task-message {
  margin-top: 2px;
  font-size: 12px;
  color: var(--muted-foreground);
  word-break: break-all;
}
.task-progress {
  margin-top: 6px;
  height: 3px;
  border-radius: 9999px;
  background-color: var(--muted);
  overflow: hidden;
}
.task-progress-bar {
  height: 100%;
  width: 40%;
  border-radius: 9999px;
  background-color: var(--primary);
  animation: task-indeterminate 1.4s ease-in-out infinite;
}
@keyframes task-indeterminate {
  0% {
    transform: translateX(-100%);
  }
  100% {
    transform: translateX(350%);
  }
}
</style>
