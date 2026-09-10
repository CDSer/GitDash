// 应用主状态管理 (Pinia Store)
// 管理项目列表、分组、选择状态、搜索等核心状态

import { defineStore } from 'pinia';
import { ref, computed, watch } from 'vue';
import type { Project, Group, Settings, ProjectStatus } from '../types';
import {
  getConfig,
  addProject as addProjectApi,
  scanProjects as scanProjectsApi,
  batchImportProjects as batchImportProjectsApi,
  removeProject as removeProjectApi,
  removeProjects as removeProjectsApi,
  updateProjects as updateProjectsApi,
  addGroup as addGroupApi,
  removeGroup as removeGroupApi,
  updateGroups as updateGroupsApi,
  updateSettings as updateSettingsApi,
} from '../lib/tauriApi';

export const useAppStore = defineStore('app', () => {
  // ========== State ==========
  const projects = ref<Project[]>([]);
  const groups = ref<Group[]>([]);
  const settings = ref<Settings>({
    git_path: null,
    // 0 = 关闭自动 fetch；默认 10 分钟
    auto_fetch_interval: 600,
    max_concurrent_git: 3,
    theme: 'system',
    global_shortcut: 'CommandOrControl+Shift+G',
    scan_blacklist: [
      '.git',
      'node_modules',
      'target',
      'dist',
      'build',
      '.next',
      '.nuxt',
      'vendor',
      '__pycache__',
      '.venv',
      'venv',
      '.idea',
      '.vscode',
    ],
  });
  const statuses = ref<Map<string, ProjectStatus>>(new Map());
  const selectedProjectIds = ref<Set<string>>(new Set());
  const activeGroupId = ref<string | null>('all');
  const searchQuery = ref('');

  // ========== Getters ==========
  
  /**
   * 过滤后的项目列表（按分组 + 搜索）
   */
  const filteredProjects = computed(() => {
    let result = projects.value;

    // 按分组过滤
    // 'all' / null 表示全部；untagged 是前端系统分组，不对应项目上的 group_id
    if (activeGroupId.value && activeGroupId.value !== 'all') {
      if (activeGroupId.value === 'untagged') {
        result = result.filter(p => !p.group_id);
      } else {
        result = result.filter(p => p.group_id === activeGroupId.value);
      }
    }

    // 按搜索关键词过滤
    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase();
      result = result.filter(p =>
        p.name.toLowerCase().includes(query) ||
        p.path.toLowerCase().includes(query)
      );
    }

    return result;
  });

  /**
   * 已选中的项目列表
   */
  const selectedProjects = computed(() => {
    return projects.value.filter(p => selectedProjectIds.value.has(p.id));
  });

  /**
   * 是否有选中项
   */
  const hasSelection = computed(() => selectedProjectIds.value.size > 0);

  /**
   * 所有分组（含系统分组「全部 / 未分组」，固定在最前）
   */
  const allGroups = computed(() => {
    const system: Group[] = [
      { id: 'all', name: '全部', color: '#3b82f6', sort_order: -2 },
      { id: 'untagged', name: '未分组', color: '#9ca3af', sort_order: -1 },
    ];
    return [...system, ...[...groups.value].sort((a, b) => a.sort_order - b.sort_order)];
  });

  // ========== Actions ==========

  /**
   * 加载配置文件
   */
  async function loadConfig() {
    try {
      const config = await getConfig();
      projects.value = config.projects;
      groups.value = config.groups;
      settings.value = config.settings;
    } catch (error) {
      console.error('加载配置失败：', error);
    }
  }

  /**
   * 添加项目
   * @param path Git 仓库路径
   * @param groupId 分组 ID，为空则加入未分组
   */
  async function addProject(path: string, groupId?: string | null) {
    try {
      const project = await addProjectApi(path, groupId);
      projects.value.push(project);
      return project;
    } catch (error) {
      console.error('添加项目失败：', error);
      throw error;
    }
  }

  /**
   * 扫描目录中的 Git 仓库候选
   * @param basePath Workspace 根目录
   * @param options 扫描选项
   */
  async function scanProjects(basePath: string, options?: { max_depth?: number }) {
    try {
      return await scanProjectsApi(basePath, options);
    } catch (error) {
      console.error('扫描目录失败：', error);
      throw error;
    }
  }

  /**
   * 批量导入 Git 项目
   * @param paths 用户勾选的仓库路径列表
   * @param groupId 分组 ID，为空则加入未分组
   */
  async function batchImportProjects(paths: string[], groupId?: string | null) {
    try {
      const result = await batchImportProjectsApi(paths, groupId);
      if (result.added.length > 0) {
        projects.value.push(...result.added);
      }
      return result;
    } catch (error) {
      console.error('批量导入失败：', error);
      throw error;
    }
  }

  /**
   * 删除项目
   * @param projectId 项目 ID
   */
  async function removeProject(projectId: string) {
    try {
      await removeProjectApi(projectId);
      projects.value = projects.value.filter(p => p.id !== projectId);
      selectedProjectIds.value.delete(projectId);
      statuses.value.delete(projectId);
    } catch (error) {
      console.error('删除项目失败：', error);
    }
  }

  /**
   * 批量删除项目
   * @param projectIds 项目 ID 列表
   */
  async function removeProjects(projectIds: string[]) {
    if (projectIds.length === 0) return;

    try {
      await removeProjectsApi(projectIds);
      const idSet = new Set(projectIds);
      projects.value = projects.value.filter(p => !idSet.has(p.id));
      projectIds.forEach(id => {
        selectedProjectIds.value.delete(id);
        statuses.value.delete(id);
      });
    } catch (error) {
      console.error('批量删除项目失败：', error);
      throw error;
    }
  }

  /**
   * 批量移动项目到指定分组（只保存一次配置）
   * @param projectIds 项目 ID 列表
   * @param groupId 分组 ID，传 null 表示移出分组
   */
  function moveProjectsToGroup(projectIds: string[], groupId: string | null) {
    const idSet = new Set(projectIds);
    let changed = false;

    projects.value.forEach(p => {
      if (idSet.has(p.id) && p.group_id !== groupId) {
        p.group_id = groupId;
        changed = true;
      }
    });

    if (changed) {
      updateProjectsApi(projects.value).catch(err => {
        console.error('保存分组变更失败：', err);
      });
    }
  }

  /**
   * 移动单个项目到指定分组
   * @param projectId 项目 ID
   * @param groupId 分组 ID，传 null 表示移出分组
   */
  function moveToGroup(projectId: string, groupId: string | null) {
    moveProjectsToGroup([projectId], groupId);
  }

  /**
   * 添加分组
   * @param name 分组名称
   * @param color 分组颜色
   */
  async function addGroup(name: string, color: string) {
    try {
      const group = await addGroupApi(name, color);
      groups.value.push(group);
      return group;
    } catch (error) {
      console.error('添加分组失败：', error);
      throw error;
    }
  }

  /**
   * 重排分组顺序（将 sort_order 重新按顺序编号并触发持久化）
   * @param orderedIds 排序后的分组 ID 列表（应包含所有用户分组）
   */
  function reorderGroups(orderedIds: string[]) {
    const byId = new Map(groups.value.map((g) => [g.id, g] as const));
    const ordered: Group[] = [];
    for (const id of orderedIds) {
      const group = byId.get(id);
      if (group) {
        ordered.push(group);
        byId.delete(id);
      }
    }
    // 兜底：未包含在内的分组追加到末尾
    ordered.push(...byId.values());
    ordered.forEach((g, index) => {
      g.sort_order = index;
    });
    groups.value = ordered;
  }

  /**
   * 删除分组（该分组下的项目会变为未分组）
   * @param groupId 分组 ID
   */
  async function removeGroup(groupId: string) {
    try {
      await removeGroupApi(groupId);
      groups.value = groups.value.filter(g => g.id !== groupId);
      projects.value.forEach(p => {
        if (p.group_id === groupId) {
          p.group_id = null;
        }
      });
      if (activeGroupId.value === groupId) {
        activeGroupId.value = 'all';
      }
    } catch (error) {
      console.error('删除分组失败：', error);
      throw error;
    }
  }

  /**
   * 重命名分组（可同时修改颜色）
   * 改本地数组即可，groups 的 deep watch 会自动持久化
   * @param groupId 分组 ID
   * @param name 新名称
   * @param color 新颜色（可选）
   */
  function renameGroup(groupId: string, name: string, color?: string) {
    const group = groups.value.find(g => g.id === groupId);
    if (!group) {
      throw new Error('分组不存在或已被删除');
    }
    group.name = name;
    if (color) {
      group.color = color;
    }
  }

  /**
   * 更新项目状态
   * @param projectId 项目 ID
   * @param status 状态信息
   */
  function updateStatus(projectId: string, status: ProjectStatus) {
    statuses.value.set(projectId, status);
  }

  /**
   * 保存应用设置（调用后端 update_settings 命令并落盘）
   * 空字符串的 git_path 会被转成 null 再发送
   */
  async function saveSettings() {
    const payload: Settings = { ...settings.value };
    if (payload.git_path === '') {
      payload.git_path = null;
    }
    await updateSettingsApi(payload);
  }

  /**
   * 切换项目选中状态
   * @param projectId 项目 ID
   */
  function toggleSelect(projectId: string) {
    if (selectedProjectIds.value.has(projectId)) {
      selectedProjectIds.value.delete(projectId);
    } else {
      selectedProjectIds.value.add(projectId);
    }
  }

  /**
   * 全选
   */
  function selectAll() {
    selectedProjectIds.value = new Set(projects.value.map(p => p.id));
  }

  /**
   * 取消全选
   */
  function clearSelection() {
    selectedProjectIds.value.clear();
  }

  /**
   * 范围选择（Shift + 点击）
   * @param startId 起始项目 ID
   * @param endId 结束项目 ID
   */
  function selectRange(startId: string, endId: string) {
    const startIndex = projects.value.findIndex(p => p.id === startId);
    const endIndex = projects.value.findIndex(p => p.id === endId);
    
    if (startIndex === -1 || endIndex === -1) return;

    const [min, max] = startIndex < endIndex ? [startIndex, endIndex] : [endIndex, startIndex];
    selectedProjectIds.value = new Set(
      projects.value.slice(min, max + 1).map(p => p.id)
    );
  }

  // ========== Watchers ==========
  
  // 项目列表变化时自动保存到配置
  watch(projects, (newProjects) => {
    updateProjectsApi(newProjects);
  }, { deep: true });

  // 分组列表变化时自动保存到配置
  watch(groups, (newGroups) => {
    updateGroupsApi(newGroups);
  }, { deep: true });

  return {
    // State
    projects,
    groups,
    settings,
    statuses,
    selectedProjectIds,
    activeGroupId,
    searchQuery,
    // Getters
    filteredProjects,
    selectedProjects,
    hasSelection,
    allGroups,
    // Actions
    loadConfig,
    addProject,
    scanProjects,
    batchImportProjects,
    removeProject,
    removeProjects,
    addGroup,
    removeGroup,
    renameGroup,
    reorderGroups,
    moveToGroup,
    moveProjectsToGroup,
    updateStatus,
    saveSettings,
    toggleSelect,
    selectAll,
    clearSelection,
    selectRange,
  };
});
