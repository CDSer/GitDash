// 项目状态 Composable
// 负责获取状态、轮询，以及响应后端 repo:changed 事件做防抖刷新

import { ref, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { getProjectStatus } from '../lib/tauriApi';
import type { ProjectStatus } from '../types';
import { useAppStore } from '../stores/appStore';

/** 单仓库状态变更防抖窗口（毫秒） */
const REPO_CHANGE_DEBOUNCE_MS = 400;

export function useProjectStatus() {
  const appStore = useAppStore();
  const lastFetched = ref<Map<string, number>>(new Map());
  let pollInterval: ReturnType<typeof setInterval> | null = null;
  let unlistenRepo: UnlistenFn | null = null;
  const debounceTimers = new Map<string, ReturnType<typeof setTimeout>>();

  /**
   * 获取项目状态
   * @param projectId 项目 ID
   * @param force 是否强制刷新
   */
  async function getStatus(projectId: string, force: boolean = false): Promise<ProjectStatus | null> {
    try {
      const status = await getProjectStatus(projectId, force);
      appStore.updateStatus(projectId, status);
      if (status.is_clean) {
        lastFetched.value.set(projectId, Date.now());
      }
      return status;
    } catch (error) {
      console.error(`获取项目 ${projectId} 状态失败：`, error);
      return null;
    }
  }

  /**
   * 防抖刷新单个仓库状态（监听 repo:changed 时使用）
   */
  function scheduleStatusRefresh(projectId: string) {
    const existing = debounceTimers.get(projectId);
    if (existing) {
      clearTimeout(existing);
    }
    debounceTimers.set(
      projectId,
      setTimeout(() => {
        debounceTimers.delete(projectId);
        void getStatus(projectId, true);
      }, REPO_CHANGE_DEBOUNCE_MS),
    );
  }

  /**
   * 订阅后端仓库变化事件（git 操作 / 文件监听触发）
   * 返回是否成功挂载（重复调用会先卸载旧监听）
   */
  async function watchRepoChanges(): Promise<void> {
    if (unlistenRepo) {
      unlistenRepo();
      unlistenRepo = null;
    }
    unlistenRepo = await listen<string>('repo:changed', (event) => {
      const projectId = event.payload;
      if (projectId) {
        scheduleStatusRefresh(projectId);
      }
    });
  }

  /**
   * 开始自动轮询（兜底；主路径是 repo:changed）
   * @param interval 轮询间隔（毫秒）
   */
  function startPolling(interval: number = 30000) {
    if (pollInterval) return;

    pollInterval = setInterval(async () => {
      // 跳过正在批量操作的场景，避免和 force 刷新抢信号量
      for (const project of appStore.projects) {
        await getStatus(project.id, false);
      }
    }, interval);
  }

  /**
   * 停止轮询
   */
  function stopPolling() {
    if (pollInterval) {
      clearInterval(pollInterval);
      pollInterval = null;
    }
  }

  function teardown() {
    stopPolling();
    if (unlistenRepo) {
      unlistenRepo();
      unlistenRepo = null;
    }
    for (const t of debounceTimers.values()) {
      clearTimeout(t);
    }
    debounceTimers.clear();
  }

  // 组件卸载时停止轮询与监听
  onUnmounted(() => {
    teardown();
  });

  return {
    getStatus,
    startPolling,
    stopPolling,
    watchRepoChanges,
    scheduleStatusRefresh,
    lastFetched,
  };
}
