// 项目状态 Composable
// 负责获取和轮询项目状态

import { ref, onUnmounted } from 'vue';
import { getProjectStatus } from '../lib/tauriApi';
import type { ProjectStatus } from '../types';
import { useAppStore } from '../stores/appStore';

export function useProjectStatus() {
  const appStore = useAppStore();
  const lastFetched = ref<Map<string, number>>(new Map());
  let pollInterval: ReturnType<typeof setInterval> | null = null;

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
   * 开始自动轮询
   * @param interval 轮询间隔（毫秒）
   */
  function startPolling(interval: number = 30000) {
    if (pollInterval) return;

    pollInterval = setInterval(async () => {
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

  // 组件卸载时停止轮询
  onUnmounted(() => {
    stopPolling();
  });

  return {
    getStatus,
    startPolling,
    stopPolling,
    lastFetched,
  };
}
