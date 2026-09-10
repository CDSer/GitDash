// 项目标签页状态
// 管理已打开的仓库标签（Fork 风格多标签），持久化到 localStorage

import { defineStore } from 'pinia';
import { computed, ref, watch } from 'vue';

export type ProjectTabMode = 'workspace' | 'changes' | 'history';

export interface ProjectTab {
  projectId: string;
  mode: ProjectTabMode;
}

const STORAGE_KEY = 'gitdash:project-tabs';

function loadTabs(): ProjectTab[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const arr = JSON.parse(raw);
    if (!Array.isArray(arr)) return [];
    return arr
      .filter((t): t is ProjectTab =>
        !!t && typeof t.projectId === 'string' &&
        (t.mode === 'workspace' || t.mode === 'changes' || t.mode === 'history')
      )
      .map((t) => ({ projectId: t.projectId, mode: t.mode }));
  } catch {
    return [];
  }
}

function loadActive(): string | null {
  return localStorage.getItem(`${STORAGE_KEY}:active`);
}

export const useTabStore = defineStore('tabs', () => {
  const tabs = ref<ProjectTab[]>(loadTabs());
  const activeProjectId = ref<string | null>(loadActive() ?? tabs.value[0]?.projectId ?? null);

  const activeTab = computed(
    () => tabs.value.find((t) => t.projectId === activeProjectId.value) ?? null,
  );

  // 已打开的项目 ID 列表
  const openProjectIds = computed(() => tabs.value.map((t) => t.projectId));

  function persist() {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(tabs.value));
    if (activeProjectId.value) {
      localStorage.setItem(`${STORAGE_KEY}:active`, activeProjectId.value);
    } else {
      localStorage.removeItem(`${STORAGE_KEY}:active`);
    }
  }

  function openProject(projectId: string, mode: ProjectTabMode = 'history') {
    if (!projectId) return;
    const existing = tabs.value.find((t) => t.projectId === projectId);
    if (existing) {
      if (mode && existing.mode !== mode) {
        existing.mode = mode;
      }
    } else {
      tabs.value.push({ projectId, mode });
    }
    activeProjectId.value = projectId;
    persist();
  }

  function closeTab(projectId: string) {
    const idx = tabs.value.findIndex((t) => t.projectId === projectId);
    if (idx === -1) return;
    tabs.value.splice(idx, 1);
    if (activeProjectId.value === projectId) {
      const next = tabs.value[Math.min(idx, tabs.value.length - 1)];
      activeProjectId.value = next?.projectId ?? null;
    }
    persist();
  }

  function closeOtherTabs(projectId: string) {
    tabs.value = tabs.value.filter((t) => t.projectId === projectId);
    activeProjectId.value = projectId;
    persist();
  }

  function closeAllTabs() {
    tabs.value = [];
    activeProjectId.value = null;
    persist();
  }

  function setActive(projectId: string | null) {
    if (projectId === null) {
      activeProjectId.value = null;
      persist();
      return;
    }
    if (!tabs.value.some((t) => t.projectId === projectId)) return;
    activeProjectId.value = projectId;
    persist();
  }

  function setMode(projectId: string, mode: ProjectTabMode) {
    const t = tabs.value.find((x) => x.projectId === projectId);
    if (!t) return;
    t.mode = mode;
    if (activeProjectId.value !== projectId) {
      activeProjectId.value = projectId;
    }
    persist();
  }

  /** 项目被移除时清理标签 */
  function removeProjectTabs(projectIds: string[]) {
    const idSet = new Set(projectIds);
    tabs.value = tabs.value.filter((t) => !idSet.has(t.projectId));
    if (activeProjectId.value && idSet.has(activeProjectId.value)) {
      activeProjectId.value = tabs.value[0]?.projectId ?? null;
    }
    persist();
  }

  // store 内部变化自动落盘（与显式 persist 双保险）
  watch(tabs, persist, { deep: true });

  return {
    tabs,
    activeProjectId,
    activeTab,
    openProjectIds,
    openProject,
    closeTab,
    closeOtherTabs,
    closeAllTabs,
    setActive,
    setMode,
    removeProjectTabs,
  };
});
