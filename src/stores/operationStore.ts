// 操作队列状态管理 (Pinia Store)
// 管理批量 Pull/Fetch/Push 的队列、进度、取消与失败重试
// 同时驱动顶部进度日志 toast（实时过程信息）

import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import type { OperationTask, OperationEvent } from '../types';
import {
  batchPull as batchPullApi,
  batchFetch as batchFetchApi,
  batchPush as batchPushApi,
} from '../lib/tauriApi';
import { beginProgressToast } from '../lib/toast';
import { useAppStore } from './appStore';

type BatchOp = 'pull' | 'push' | 'fetch';

const OP_LABEL: Record<BatchOp, string> = {
  pull: '拉取',
  push: '推送',
  fetch: '获取',
};

const OP_GIT: Record<BatchOp, string> = {
  pull: 'git pull --no-edit',
  push: 'git push',
  fetch: 'git fetch --prune --all',
};

function extractLogLines(result: { stdout: string; stderr: string; success: boolean }): string[] {
  const lines: string[] = [];
  const pushBlock = (label: string, text: string) => {
    const trimmed = text.trim();
    if (!trimmed) return;
    lines.push(`—— ${label} ——`);
    for (const line of trimmed.split('\n')) {
      if (line.trim()) lines.push(line.replace(/\s+$/, ''));
    }
  };
  pushBlock('stdout', result.stdout);
  pushBlock('stderr', result.stderr);
  return lines;
}

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

  function projectNameOf(projectId: string, fallback = projectId): string {
    const appStore = useAppStore();
    return appStore.projects.find((p) => p.id === projectId)?.name ?? fallback;
  }

  async function runBatch(
    projectIds: string[],
    operation: BatchOp,
    options?: {
      rebase?: boolean;
      forceWithLease?: boolean;
      tags?: boolean;
    }
  ) {
    if (projectIds.length === 0) return;

    const appStore = useAppStore();
    const projectMap = new Map(appStore.projects.map((p) => [p.id, p]));
    const label = OP_LABEL[operation];

    projectIds.forEach((id) => {
      const project = projectMap.get(id);
      if (project) {
        createTask(id, project.name, operation);
      }
    });

    lastOperation.value = operation;
    showPanel.value = true;
    isQueueRunning.value = true;

    const progress = beginProgressToast(
      `${label} ${projectIds.length} 个项目`,
      `开始${label} · 执行 ${OP_GIT[operation]} · 并发受 Git 限制`,
    );
    projectIds.forEach((id) => {
      const name = projectMap.get(id)?.name ?? id;
      progress.log(`排队：${name}`);
    });

    let unlisten: (() => void) | null = null;
    try {
      unlisten = await listen<OperationEvent>('git:progress', (event) => {
        const { project_id, status, message } = event.payload;
        if (!tasks.value.some((t) => t.projectId === project_id)) return;

        const name = projectNameOf(project_id);
        if (message) {
          progress.log(message);
        } else if (status === 'running') {
          progress.log(`${name}：执行中…`);
        }

        updateTaskByProject(project_id, {
          status: status as OperationTask['status'],
          message: message || undefined,
        });
      });

      progress.log(`调用后端 batch_${operation}…`);
      let results;
      if (operation === 'pull') {
        results = await batchPullApi(projectIds, options?.rebase ?? false);
      } else if (operation === 'push') {
        results = await batchPushApi(
          projectIds,
          options?.forceWithLease ?? false,
          options?.tags ?? false,
        );
      } else {
        results = await batchFetchApi(projectIds);
      }
      progress.log(`后端返回 ${results.length} 条结果`);

      results.forEach((r) => {
        const name = projectNameOf(r.project_id);
        const detailLines = extractLogLines(r);
        detailLines.forEach((line) => progress.log(`[${name}] ${line}`));

        updateTaskByProject(r.project_id, {
          status: r.success ? 'success' : 'error',
          message: r.success ? undefined : r.stderr || undefined,
        });
      });

      projectIds.forEach((id) => {
        const t = tasks.value.find((x) => x.projectId === id);
        if (t && t.status === 'pending') {
          progress.log(`[${projectNameOf(id)}] 未收到执行结果`);
          updateTask(t.id, { status: 'error', message: '未收到执行结果' });
        }
      });

      const okCount = results.filter((r) => r.success).length;
      const failCount = results.filter((r) => !r.success).length;
      const allOk = results.length > 0 && failCount === 0;
      const summary = failCount === 0 && results.length > 0
        ? `${label}完成：成功 ${okCount}/${results.length}`
        : `${label}结束：成功 ${okCount}，失败 ${failCount}`;
      progress.log(summary);
      progress.done(allOk, summary);
    } catch (error) {
      console.error('批量操作失败：', error);
      const msg = String(error);
      progress.log(`异常：${msg}`);
      projectIds.forEach((id) => {
        updateTaskByProject(id, { status: 'error', message: msg });
      });
      progress.done(false, `${label}失败：${msg}`);
    } finally {
      if (unlisten) {
        unlisten();
      }
      isQueueRunning.value = false;
      // 全部执行完后自动收起操作队列面板（留短暂时间看一眼结果）
      setTimeout(() => {
        if (!isQueueRunning.value) {
          showPanel.value = false;
        }
      }, 1200);
    }
  }

  async function batchPull(projectIds: string[], options?: { rebase?: boolean }) {
    await runBatch(projectIds, 'pull', options);
  }

  async function batchFetch(projectIds: string[]) {
    await runBatch(projectIds, 'fetch');
  }

  async function batchPush(
    projectIds: string[],
    options?: { forceWithLease?: boolean; tags?: boolean }
  ) {
    await runBatch(projectIds, 'push', options);
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
