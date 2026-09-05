<!--
  项目表格组件
  使用 Element Plus 的 el-table 展示项目，支持多选、搜索、双击打开文件夹
-->
<template>
  <div class="project-table">
    <!-- 搜索框 -->
    <div class="table-toolbar">
      <el-input
        v-model="searchQuery"
        placeholder="搜索项目..."
        :prefix-icon="Search"
        clearable
        class="search-input"
      />
    </div>

    <!-- 项目列表 -->
    <div class="table-wrapper">
      <el-table
        ref="tableRef"
        :data="filteredProjects"
        height="100%"
        row-key="id"
        @selection-change="handleSelectionChange"
        @row-dblclick="openRepo"
      >
        <el-table-column type="selection" width="45" />

        <el-table-column label="名称" min-width="180" show-overflow-tooltip>
          <template #default="{ row }">
            <span class="project-name">{{ row.name }}</span>
            <el-icon v-if="row.is_favorite" class="favorite-icon" :size="14">
              <Star />
            </el-icon>
          </template>
        </el-table-column>

        <el-table-column prop="path" label="路径" min-width="220" show-overflow-tooltip />

        <el-table-column label="分支" width="140" show-overflow-tooltip>
          <template #default="{ row }">
            {{ statusFor(row.id)?.branch || '-' }}
          </template>
        </el-table-column>

        <el-table-column label="状态" width="100" align="center">
          <template #default="{ row }">
            <StatusBadge :status="statusFor(row.id)" />
          </template>
        </el-table-column>

        <el-table-column label="变更" min-width="200" align="center">
          <template #default="{ row }">
            <StatusChanges :status="statusFor(row.id)" />
          </template>
        </el-table-column>

        <el-table-column label="操作" width="150" align="center" fixed="right">
          <template #default="{ row }">
            <el-button
              text
              size="small"
              :icon="Monitor"
              title="打开工作区"
              @click.stop="enterWorkspace(row)"
            />
            <el-dropdown trigger="click" @command="(command) => handleRowCommand(command, row)">
              <el-button text size="small" :icon="MoreFilled" title="更多操作" @click.stop />
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item command="workspace" :icon="Monitor">打开工作区</el-dropdown-item>
                  <el-dropdown-item command="open" :icon="FolderOpened">打开文件夹</el-dropdown-item>
                  <el-dropdown-item command="favorite" :icon="Star">
                    {{ row.is_favorite ? '取消收藏' : '设为收藏' }}
                  </el-dropdown-item>
                  <el-dropdown-item command="history" :icon="Collection">Git 记录</el-dropdown-item>

                  <el-dropdown-item command="__title" divided disabled>移动到分组</el-dropdown-item>
                  <el-dropdown-item v-if="userGroups.length === 0" disabled>
                    暂无分组，请先添加
                  </el-dropdown-item>
                  <el-dropdown-item
                    v-for="group in userGroups"
                    :key="group.id"
                    :command="`move:${group.id}`"
                  >
                    <span class="group-dot" :style="{ backgroundColor: group.color }"></span>
                    <span>{{ group.name }}</span>
                  </el-dropdown-item>
                  <el-dropdown-item v-if="row.group_id" command="move:none" divided>
                    移出分组
                  </el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
          </template>
        </el-table-column>

        <template #empty>
          <el-empty description="暂无项目" :image-size="80" />
        </template>
      </el-table>
    </div>

    <!-- 底部批量操作栏 -->
    <div v-if="hasSelection" class="batch-bar">
      <span class="batch-count">已选择 {{ selectedProjectIds.size }} 个</span>
      <div class="batch-actions">
        <el-dropdown trigger="click" @command="handleBatchMove">
          <el-button size="small" :icon="Folder" :disabled="userGroups.length === 0">
            移动到分组
          </el-button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item v-for="group in userGroups" :key="group.id" :command="group.id">
                <span class="group-dot" :style="{ backgroundColor: group.color }"></span>
                <span>{{ group.name }}</span>
              </el-dropdown-item>
              <el-dropdown-item command="__none__" divided>移出分组</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>

        <el-button
          size="small"
          :icon="RefreshRight"
          :loading="operationStore.isQueueRunning"
          @click="batchFetch"
        >
          获取
        </el-button>
        <el-button
          size="small"
          type="primary"
          :icon="Download"
          :loading="operationStore.isQueueRunning"
          @click="batchPull"
        >
          拉取
        </el-button>
        <el-button
          size="small"
          :icon="Upload"
          :loading="operationStore.isQueueRunning"
          @click="batchPush"
        >
          推送
        </el-button>
        <el-button size="small" text @click="clearSelection">取消选择</el-button>
      </div>
    </div>

    <GitHistoryModal v-model="showHistory" :project="historyProject" />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, watch } from 'vue';
import { Search, Star, Folder, FolderOpened, RefreshRight, Download, MoreFilled, Upload, Collection, Monitor } from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';
import type { Project, ProjectStatus } from '../../types';
import { useAppStore } from '../../stores/appStore';
import { useOperationStore } from '../../stores/operationStore';
import { openRepoFolder } from '../../lib/tauriApi';
import StatusBadge from './StatusBadge.vue';
import StatusChanges from './StatusChanges.vue';
import GitHistoryModal from '../Modals/GitHistoryModal.vue';
import { useProjectStatus } from '../../composables/useProjectStatus';

const appStore = useAppStore();
const operationStore = useOperationStore();
const { getStatus } = useProjectStatus();

const tableRef = ref();
const showHistory = ref(false);
const historyProject = ref<Project | null>(null);

const statusFor = (projectId: string): ProjectStatus | null => appStore.statuses.get(projectId) ?? null;

const searchQuery = computed({
  get: () => appStore.searchQuery,
  set: (value) => appStore.searchQuery = value
});

const selectedProjectIds = computed(() => appStore.selectedProjectIds);
const hasSelection = computed(() => appStore.hasSelection);
const filteredProjects = computed(() => appStore.filteredProjects);

// 用户自建分组（不含「全部 / 收藏 / 未分组」三个系统分组）
const userGroups = computed(() => appStore.groups);

function handleSelectionChange(rows: Project[]) {
  appStore.selectedProjectIds = new Set(rows.map(r => r.id));
}

function clearSelection() {
  tableRef.value?.clearSelection();
  appStore.clearSelection();
}

// el-table 的行数据类型是内部的 DefaultRow，这里做一次断言取出项目 ID
function openRepo(row: unknown) {
  const project = row as Project;
  openRepoFolder(project.id);
}

function groupName(groupId: string | null) {
  if (!groupId) return '未分组';
  return userGroups.value.find(g => g.id === groupId)?.name ?? '未分组';
}

/**
 * 单行操作菜单
 * command 约定：open / favorite / history / move:<分组ID> / move:none
 */
function enterWorkspace(row: unknown) {
  appStore.openWorkspace(row as Project);
}

function handleRowCommand(command: string, row: unknown) {
  const project = row as Project;

  if (command === 'workspace') {
    appStore.openWorkspace(project);
    return;
  }

  if (command === 'open') {
    openRepo(project);
    return;
  }

  if (command === 'favorite') {
    appStore.toggleFavorite(project.id);
    return;
  }

  if (command === 'history') {
    historyProject.value = project;
    showHistory.value = true;
    return;
  }

  if (command.startsWith('move:')) {
    const value = command.slice('move:'.length);
    const groupId = value === 'none' ? null : value;
    appStore.moveToGroup(project.id, groupId);
    ElMessage.success(`已将「${project.name}」移动到「${groupName(groupId)}」`);
  }
}

/** 批量移动选中的项目到指定分组 */
function handleBatchMove(command: string) {
  const ids = Array.from(selectedProjectIds.value);
  if (ids.length === 0) return;

  const groupId = command === '__none__' ? null : command;
  appStore.moveProjectsToGroup(ids, groupId);
  ElMessage.success(`已移动 ${ids.length} 个项目到「${groupName(groupId)}」`);
}

async function batchFetch() {
  await operationStore.batchFetch(Array.from(selectedProjectIds.value));
}

async function batchPull() {
  await operationStore.batchPull(Array.from(selectedProjectIds.value));
}

async function batchPush() {
  await operationStore.batchPush(Array.from(selectedProjectIds.value));
}

// 项目列表加载完成后拉取各仓库状态（配置是异步加载的，需要监听变化）
watch(
  () => appStore.projects.length,
  () => {
    appStore.projects.forEach(project => {
      getStatus(project.id, true);
    });
  }
);

onMounted(() => {
  appStore.projects.forEach(project => {
    getStatus(project.id, true);
  });
});
</script>

<style scoped>
.project-table {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.table-toolbar {
  padding: 12px 16px;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.search-input {
  max-width: 320px;
}

.table-wrapper {
  flex: 1;
  min-height: 0;
  padding: 0 16px;
}

.project-name {
  margin-right: 4px;
}

.favorite-icon {
  color: var(--el-color-warning);
  vertical-align: -2px;
}

.group-dot {
  display: inline-block;
  width: 8px;
  height: 8px;
  margin-right: 6px;
  border-radius: 50%;
  vertical-align: middle;
}

.batch-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 44px;
  padding: 0 16px;
  border-top: 1px solid var(--el-border-color);
  background-color: var(--el-bg-color-overlay);
}

.batch-count {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

.batch-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}
</style>
