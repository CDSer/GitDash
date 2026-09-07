// 操作队列状态管理 (Pinia Store)
// 管理批量 Pull/Fetch 操作的队列和进度

import { defineStore } from 'pinia';
import { ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import type { OperationTask, OperationEvent } from '../types';
import { batchPull as batchPullApi, batchFetch as batchFetchApi, batchPush as batchPushApi } from '../lib/tauriApi';
import { useAppStore } from './appStore';

export const useOperationStore = defineStore('operation', () => {
  // ========== State ==========
  const tasks = ref<OperationTask[]>([]);
  const isQueueRunning = ref(false);
  const showPanel = ref(false);

  // ========== Helper Functions ==========

  /**
   * 创建操作任务
   */
  function createTask(
    projectId: string,
    projectName: string,
    operation: 'pull' | 'push' | 'fetch'
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

  /**
   * 更新任务状态
   */
  function updateTask(taskId: string, updates: Partial<OperationTask>) {
    const task = tasks.value.find(t => t.id === taskId);
    if (task) {
      Object.assign(task, updates);
    }
  }

  // ========== Actions ==========

  /**
   * 批量 Pull 操作
   * @param projectIds 项目 ID 列表
   */
  async function batchPull(projectIds: string[]) {
    const appStore = useAppStore();
    const projectMap = new Map(appStore.projects.map(p => [p.id, p]));

    // 创建任务列表
    const taskIds: string[] = [];
    projectIds.forEach(id => {
      const project = projectMap.get(id);
      if (project) {
        const task = createTask(id, project.name, 'pull');
        taskIds.push(task.id);
      }
    });

    showPanel.value = true;
    isQueueRunning.value = true;

    try {
      // 监听进度事件
      const unlisten = await listen<OperationEvent>('git:progress', (event) => {
        const task = tasks.value.find(t => t.id === event.payload.task_id);
        if (task) {
          updateTask(event.payload.task_id, {
            status: event.payload.status as OperationTask['status'],
            message: event.payload.message || undefined,
          });
        }
      });

      // 执行 Pull
      const results = await batchPullApi(projectIds);

      // 以命令真实返回结果覆盖任务状态
      results.forEach((r, i) => {
        const tid = taskIds[i];
        if (tid) {
          updateTask(tid, {
            status: r.success ? 'success' : 'error',
            message: r.success ? undefined : r.stderr,
          });
        }
      });

      setTimeout(() => {
        unlisten();
        isQueueRunning.value = false;
      }, 1000);
    } catch (error) {
      console.error('批量拉取失败：', error);
      taskIds.forEach(id => {
        updateTask(id, { status: 'error', message: String(error) });
      });
      isQueueRunning.value = false;
    }
  }

  /**
   * 批量 Fetch 操作
   * @param projectIds 项目 ID 列表
   */
  async function batchFetch(projectIds: string[]) {
    const appStore = useAppStore();
    const projectMap = new Map(appStore.projects.map(p => [p.id, p]));

    // 创建任务列表
    const taskIds: string[] = [];
    projectIds.forEach(id => {
      const project = projectMap.get(id);
      if (project) {
        const task = createTask(id, project.name, 'fetch');
        taskIds.push(task.id);
      }
    });

    showPanel.value = true;
    isQueueRunning.value = true;

    try {
      const results = await batchFetchApi(projectIds);

      results.forEach((r, i) => {
        const tid = taskIds[i];
        if (tid) {
          updateTask(tid, {
            status: r.success ? 'success' : 'error',
            message: r.success ? undefined : r.stderr,
          });
        }
      });

      isQueueRunning.value = false;
    } catch (error) {
      console.error('批量获取失败：', error);
      taskIds.forEach(id => {
        updateTask(id, { status: 'error', message: String(error) });
      });
      isQueueRunning.value = false;
    }
  }

  /**
   * 批量 Push 操作
   * @param projectIds 项目 ID 列表
   */
  async function batchPush(projectIds: string[]) {
    const appStore = useAppStore();
    const projectMap = new Map(appStore.projects.map(p => [p.id, p]));

    // 创建任务列表
    const taskIds: string[] = [];
    projectIds.forEach(id => {
      const project = projectMap.get(id);
      if (project) {
        const task = createTask(id, project.name, 'push');
        taskIds.push(task.id);
      }
    });

    showPanel.value = true;
    isQueueRunning.value = true;

    try {
      // 监听进度事件
      const unlisten = await listen<OperationEvent>('git:progress', (event) => {
        const task = tasks.value.find(t => t.id === event.payload.task_id);
        if (task) {
          updateTask(event.payload.task_id, {
            status: event.payload.status as OperationTask['status'],
            message: event.payload.message || undefined,
          });
        }
      });

      // 执行 Push
      const results = await batchPushApi(projectIds);

      // 以命令真实返回结果覆盖任务状态
      results.forEach((r, i) => {
        const tid = taskIds[i];
        if (tid) {
          updateTask(tid, {
            status: r.success ? 'success' : 'error',
            message: r.success ? undefined : r.stderr,
          });
        }
      });

      setTimeout(() => {
        unlisten();
        isQueueRunning.value = false;
      }, 1000);
    } catch (error) {
      console.error('批量推送失败：', error);
      taskIds.forEach(id => {
        updateTask(id, { status: 'error', message: String(error) });
      });
      isQueueRunning.value = false;
    }
  }

  /**
   * 清除已完成的任务
   */
  function clearCompleted() {
    tasks.value = tasks.value.filter(t => t.status === 'pending' || t.status === 'running');
  }

  return {
    tasks,
    isQueueRunning,
    showPanel,
    batchPull,
    batchFetch,
    batchPush,
    clearCompleted,
  };
});
