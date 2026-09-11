// 项目标签页状态
// 管理已打开的仓库标签（Fork 风格多标签）+ 项目列表系统标签，持久化到 localStorage

import { defineStore } from 'pinia';
import { computed, ref, watch } from 'vue';
import { terminalClose } from '../lib/tauriApi';

export type ProjectTabMode = 'workspace' | 'changes' | 'history' | 'terminal';

/** 项目列表系统标签的固定 id */
export const PROJECTS_TAB_ID = '__projects__';

/** 设置系统标签的固定 id */
export const SETTINGS_TAB_ID = '__settings__';

export interface ProjectTab {
  /**
   * 项目 ID；PROJECTS_TAB_ID 表示「项目列表」系统标签，SETTINGS_TAB_ID 表示「设置」系统标签
   */
  projectId: string;
  /** 仅项目标签有；系统标签无 mode */
  mode?: ProjectTabMode;
}

const STORAGE_KEY = 'gitdash:project-tabs';

function isProjectsTabId(id: string): boolean {
  return id === PROJECTS_TAB_ID;
}

function isSettingsTabId(id: string): boolean {
  return id === SETTINGS_TAB_ID;
}

function isSystemTabId(id: string): boolean {
  return isProjectsTabId(id) || isSettingsTabId(id);
}

function loadTabs(): ProjectTab[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const arr = JSON.parse(raw);
    if (!Array.isArray(arr)) return [];
    return arr
      .filter((t): t is ProjectTab => {
        if (!t || typeof t.projectId !== 'string') return false;
        if (isSystemTabId(t.projectId)) return true;
        return (
          t.mode === 'workspace' ||
          t.mode === 'changes' ||
          t.mode === 'history' ||
          t.mode === 'terminal'
        );
      })
      .map((t) => {
        if (isProjectsTabId(t.projectId)) {
          return { projectId: PROJECTS_TAB_ID };
        }
        if (isSettingsTabId(t.projectId)) {
          return { projectId: SETTINGS_TAB_ID };
        }
        return { projectId: t.projectId, mode: t.mode as ProjectTabMode };
      });
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

  /** 当前活动标签；无活动时为 null（主页） */
  const activeTab = computed(
    () => tabs.value.find((t) => t.projectId === activeProjectId.value) ?? null,
  );

  /** 视图：主页 | 项目列表标签 | 设置标签 | 项目标签 */
  const viewKind = computed<'home' | 'projects' | 'settings' | 'project'>(() => {
    if (!activeProjectId.value) return 'home';
    if (isProjectsTabId(activeProjectId.value)) return 'projects';
    if (isSettingsTabId(activeProjectId.value) && tabs.value.some((t) => isSettingsTabId(t.projectId))) {
      return 'settings';
    }
    if (tabs.value.some((t) => t.projectId === activeProjectId.value)) return 'project';
    return 'home';
  });

  /** 已打开的项目 ID 列表（不含系统标签） */
  const openProjectIds = computed(() =>
    tabs.value.map((t) => t.projectId).filter((id) => !isSystemTabId(id)),
  );

  function persist() {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(tabs.value));
    if (activeProjectId.value) {
      localStorage.setItem(`${STORAGE_KEY}:active`, activeProjectId.value);
    } else {
      localStorage.removeItem(`${STORAGE_KEY}:active`);
    }
  }

  function openProject(projectId: string, mode: ProjectTabMode = 'history') {
    if (!projectId || isProjectsTabId(projectId)) return;
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

  /** 打开「项目列表」系统标签 */
  function openProjectsTab() {
    if (!tabs.value.some((t) => isProjectsTabId(t.projectId))) {
      tabs.value.unshift({ projectId: PROJECTS_TAB_ID });
    }
    activeProjectId.value = PROJECTS_TAB_ID;
    persist();
  }

  /** 打开「设置」系统标签（紧跟项目列表系统标签之后） */
  function openSettingsTab() {
    if (!tabs.value.some((t) => isSettingsTabId(t.projectId))) {
      const projectsIdx = tabs.value.findIndex((t) => isProjectsTabId(t.projectId));
      const at = projectsIdx === -1 ? tabs.value.length : projectsIdx + 1;
      tabs.value.splice(at, 0, { projectId: SETTINGS_TAB_ID });
    }
    activeProjectId.value = SETTINGS_TAB_ID;
    persist();
  }

  function closeTab(projectId: string) {
    const idx = tabs.value.findIndex((t) => t.projectId === projectId);
    if (idx === -1) return;
    tabs.value.splice(idx, 1);
    if (!isSystemTabId(projectId)) {
      void terminalClose(projectId).catch(() => {});
    }
    if (activeProjectId.value === projectId) {
      const next = tabs.value[Math.min(idx, tabs.value.length - 1)];
      activeProjectId.value = next?.projectId ?? null;
    }
    persist();
  }

  function closeOtherTabs(projectId: string) {
    for (const t of tabs.value) {
      if (t.projectId !== projectId && !isSystemTabId(t.projectId)) {
        void terminalClose(t.projectId).catch(() => {});
      }
    }
    tabs.value = tabs.value.filter((t) => t.projectId === projectId);
    activeProjectId.value = projectId;
    persist();
  }

  function closeAllTabs() {
    for (const t of tabs.value) {
      if (!isSystemTabId(t.projectId)) {
        void terminalClose(t.projectId).catch(() => {});
      }
    }
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
    if (isSystemTabId(projectId)) return;
    const t = tabs.value.find((x) => x.projectId === projectId);
    if (!t) return;
    t.mode = mode;
    if (activeProjectId.value !== projectId) {
      activeProjectId.value = projectId;
    }
    persist();
  }

  /** 项目被移除时清理标签（不影响系统标签） */
  function removeProjectTabs(projectIds: string[]) {
    const idSet = new Set(projectIds.filter((id) => !isSystemTabId(id)));
    for (const id of idSet) {
      void terminalClose(id).catch(() => {});
    }
    tabs.value = tabs.value.filter((t) => !idSet.has(t.projectId));
    if (activeProjectId.value && idSet.has(activeProjectId.value)) {
      activeProjectId.value = tabs.value[0]?.projectId ?? null;
    }
    persist();
  }

  watch(tabs, persist, { deep: true });

  return {
    tabs,
    activeProjectId,
    activeTab,
    viewKind,
    openProjectIds,
    openProject,
    openProjectsTab,
    openSettingsTab,
    closeTab,
    closeOtherTabs,
    closeAllTabs,
    setActive,
    setMode,
    removeProjectTabs,
  };
});
