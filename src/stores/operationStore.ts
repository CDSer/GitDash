// 操作队列状态管理 (Pinia Store)
// 管理批量 Pull/Fetch/Push 的队列、进度、取消与失败重试

import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import type { OperationTask, OperationEvent } from '../types';
import {
  batchPull as batchPullApi,
  batchFetch as batchFetchApi,
  batchPush as batchPushApi,
} from '../lib/tauriApi';
import { useAppStore } from './appStore';

type BatchOp = 'pull' | 'push' | 'fetch';

export const useOperationStore = defineStore('operation', () => {
  const tasks = ref<OperationTask[]>([]);
  const isQueueRunning = ref(false);
  const showPanel = ref(false);
  const lastOperation = ref<BatchOp | null>(null);

  const failedTasks = computed(() => tasks.value.filter((t) => t.status === 'error'));
  const successCount = computed(() => tasks.value.filter((t) => t.status === 'success').length);
  const errorCount = computed(() => tasks.value.filter((t) => t.status === 'error').length);

  function createTask(
    projectId: string,
    projectName: string,
    operation: BatchOp
  ): OperationTask {
    const task: OperationTask = {
      id: crypto.randomUUID(),
      projectId,
      projectName,
      operation,
      status: 'pending',
      createdAt: Date.now(),
    };
    tasks.value.push(task);
    return task;
  }

  function updateTask(taskId: string, updates: Partial<OperationTask>) {
    const task = tasks.value.find((t) => t.id === taskId);
    if (task) {
      Object.assign(task, updates);
    }
  }

  function updateTaskByProject(projectId: string, updates: Partial<OperationTask>) {
    // 优先更新该项目最近一条未完成/失败任务
    const list = tasks.value.filter((t) => t.projectId === projectId);
    const target =
      list.find((t) => t.status === 'pending' || t.status === 'running') ??
      list[list.length - 1];
    if (target) {
      Object.assign(target, updates);
    }
  }

  const apiFor = {
    pull: batchPullApi,
    fetch: batchFetchApi,
    push: batchPushApi,
  } as const;

  async function runBatch(projectIds: string[], operation: BatchOp) {
    if (projectIds.length === 0) return;

    const appStore = useAppStore();
    const projectMap = new Map(appStore.projects.map((p) => [p.id, p]));

    projectIds.forEach((id) => {
      const project = projectMap.get(id);
      if (project) {
        createTask(id, project.name, operation);
      }
    });

    lastOperation.value = operation;
    showPanel.value = true;
    isQueueRunning.value = true;

    let unlisten: (() => void) | null = null;
    try {
      unlisten = await listen<OperationEvent>('git:progress', (event) => {
        const { project_id, status, message } = event.payload;
        if (tasks.value.some((t) => t.projectId === project_id)) {
          updateTaskByProject(project_id, {
            status: status as OperationTask['status'],
            message: message || undefined,
          });
        }
      });

      const results = await apiFor[operation](projectIds);

      results.forEach((r) => {
        updateTaskByProject(r.project_id, {
          status: r.success ? 'success' : 'error',
          message: r.success ? undefined : r.stderr,
        });
      });

      projectIds.forEach((id) => {
        const t = tasks.value.find((x) => x.projectId === id);
        if (t && t.status === 'pending') {
          updateTask(t.id, { status: 'error', message: '未收到执行结果' });
        }
      });
    } catch (error) {
      console.error('批量操作失败：', error);
      projectIds.forEach((id) => {
        updateTaskByProject(id, { status: 'error', message: String(error) });
      });
    } finally {
      if (unlisten) {
        unlisten();
      }
      isQueueRunning.value = false;
    }
  }

  async function batchPull(projectIds: string[]) {
    await runBatch(projectIds, 'pull');
  }

  async function batchFetch(projectIds: string[]) {
    await runBatch(projectIds, 'fetch');
  }

  async function batchPush(projectIds: string[]) {
    await runBatch(projectIds, 'push');
  }

  /** 取消：未开始的任务标为已取消；已执行的仍等待后端结束 */
  function cancelQueue() {
    if (!isQueueRunning.value) return;
    tasks.value.forEach((t) => {
      if (t.status === 'pending') {
        updateTask(t.id, { status: 'error', message: '已取消' });
      }
    });
  }

  /** 重试最近一批中的失败任务 */
  async function retryFailed() {
    if (isQueueRunning.value || !lastOperation.value) return;
    const ids = failedTasks.value
      .filter((t) => t.operation === lastOperation.value)
      .map((t) => t.projectId);
    if (!ids.length) return;

    // 移除旧行再重跑，避免同一项目多条历史
    tasks.value = tasks.value.filter(
      (t) => !(t.status === 'error' && ids.includes(t.projectId)),
    );
    await runBatch(ids, lastOperation.value);
  }

  function clearCompleted() {
    tasks.value = tasks.value.filter((t) => t.status === 'pending' || t.status === 'running');
  }

  return {
    tasks,
    isQueueRunning,
    showPanel,
    lastOperation,
    failedTasks,
    successCount,
    errorCount,
    batchPull,
    batchFetch,
    batchPush,
    cancelQueue,
    retryFailed,
    clearCompleted,
  };
});
