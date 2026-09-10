// 操作队列状态管理 (Pinia Store)
// 管理批量 Pull/Fetch 操作的队列和进度

import { defineStore } from 'pinia';
import { ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import type { OperationTask, OperationEvent, ProjectGitResult } from '../types';
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

  /**
   * 按 projectId 更新任务（进度事件与后端结果均带 project_id）
   */
  function updateTaskByProject(projectId: string, updates: Partial<OperationTask>) {
    const task = tasks.value.find(t => t.projectId === projectId);
    if (task) {
      Object.assign(task, updates);
    }
  }

  /**
   * 统一批量操作流程：建任务 → 监听进度 → 并行结果回写
   */
  async function runBatch(
    projectIds: string[],
    operation: 'pull' | 'push' | 'fetch',
    api: (ids: string[]) => Promise<ProjectGitResult[]>
  ) {
    if (projectIds.length === 0) return;

    const appStore = useAppStore();
    const projectMap = new Map(appStore.projects.map(p => [p.id, p]));

    // 创建任务列表（按 project_id 匹配）
    projectIds.forEach(id => {
      const project = projectMap.get(id);
      if (project) {
        createTask(id, project.name, operation);
      }
    });

    showPanel.value = true;
    isQueueRunning.value = true;

    let unlisten: (() => void) | null = null;
    try {
      // 监听进度事件：用 project_id 对齐任务行
      unlisten = await listen<OperationEvent>('git:progress', (event) => {
        const { project_id, status, message } = event.payload;
        if (tasks.value.some(t => t.projectId === project_id)) {
          updateTaskByProject(project_id, {
            status: status as OperationTask['status'],
            message: message || undefined,
          });
        }
      });

      const results = await api(projectIds);

      // 以 project_id 回写真实结果（后端返回顺序与传入顺序无关）
      results.forEach(r => {
        updateTaskByProject(r.project_id, {
          status: r.success ? 'success' : 'error',
          message: r.success ? undefined : r.stderr,
        });
      });

      // 未返回的 id 标记失败，避免一直 pending
      projectIds.forEach(id => {
        const t = tasks.value.find(x => x.projectId === id);
        if (t && t.status === 'pending') {
          updateTask(t.id, { status: 'error', message: '未收到执行结果' });
        }
      });

      isQueueRunning.value = false;
    } catch (error) {
      console.error('批量操作失败：', error);
      projectIds.forEach(id => {
        updateTaskByProject(id, { status: 'error', message: String(error) });
      });
      isQueueRunning.value = false;
    } finally {
      if (unlisten) {
        unlisten();
      }
    }
  }

  // ========== Actions ==========

  /**
   * 批量 Pull 操作
   */
  async function batchPull(projectIds: string[]) {
    await runBatch(projectIds, 'pull', batchPullApi);
  }

  /**
   * 批量 Fetch 操作
   */
  async function batchFetch(projectIds: string[]) {
    await runBatch(projectIds, 'fetch', batchFetchApi);
  }

  /**
   * 批量 Push 操作
   */
  async function batchPush(projectIds: string[]) {
    await runBatch(projectIds, 'push', batchPushApi);
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
