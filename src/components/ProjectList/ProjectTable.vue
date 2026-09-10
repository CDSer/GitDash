<!--
  项目表格组件
  自绘表格展示项目，支持多选、搜索、双击打开文件夹
-->
<template>
  <div class="flex h-full flex-col">
    <!-- 搜索框 + 冲突汇总条 -->
    <div class="border-b border-border p-3">
      <div class="flex flex-wrap items-center gap-3">
        <div class="relative max-w-[320px] flex-1">
          <Search
            :size="14"
            class="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground"
          />
          <Input v-model="searchQuery" placeholder="搜索项目..." :style="{ paddingLeft: '2rem' }" />
        </div>
        <button
          v-if="conflictProjects.length"
          type="button"
          class="conflict-banner"
          @click="openFirstConflict"
        >
          {{ conflictProjects.length }} 个仓库存在冲突 · 点击处理
        </button>
        <button
          v-if="inProgressProjects.length"
          type="button"
          class="progress-banner"
          @click="openSourceControl(inProgressProjects[0])"
        >
          {{ inProgressProjects.length }} 个仓库有进行中的合并/变基
        </button>
      </div>
    </div>

    <!-- 项目表格 -->
    <div class="min-h-0 flex-1 overflow-auto">
      <div class="min-w-[860px]">
        <!-- 表头 -->
        <div class="grid items-center border-b border-border bg-card px-3 py-2 text-xs font-medium text-muted-foreground"
          style="grid-template-columns: 36px minmax(160px, 1.4fr) minmax(180px, 1.6fr) 120px 84px 200px 130px"
        >
          <div class="flex justify-center">
            <input
              type="checkbox"
              class="row-check"
              :checked="allSelected"
              :indeterminate.prop="someSelected && !allSelected"
              @change="toggleSelectAll"
            />
          </div>
          <div>名称</div>
          <div>路径</div>
          <div>分支</div>
          <div class="text-center">状态</div>
          <div class="text-center">变更</div>
          <div class="text-right pr-1">操作</div>
        </div>

        <!-- 行 -->
        <div
          v-for="row in filteredProjects"
          :key="row.id"
          :data-project-row-id="row.id"
          class="group grid items-center border-b border-border px-3 py-2 text-[13px] hover:bg-accent/60"
          style="grid-template-columns: 36px minmax(160px, 1.4fr) minmax(180px, 1.6fr) 120px 84px 200px 130px"
          @dblclick="openRepo(row)"
        >
          <div class="flex justify-center">
            <input
              type="checkbox"
              class="row-check"
              :checked="selectedProjectIds.has(row.id)"
              @click.stop="appStore.toggleSelect(row.id)"
            />
          </div>
          <div class="flex min-w-0 select-none items-center gap-1.5">
            <span class="truncate font-medium">{{ row.name }}</span>
          </div>
          <div class="truncate text-muted-foreground" :title="row.path">{{ row.path }}</div>
          <div class="truncate text-muted-foreground">
            {{ statusFor(row.id)?.branch || '-' }}
          </div>
          <div class="flex justify-center">
            <StatusBadge :status="statusFor(row.id)" />
          </div>
          <div class="flex justify-center">
            <StatusChanges
              :status="statusFor(row.id)"
              @open-conflicts="openSourceControl(row)"
            />
          </div>
          <div class="flex items-center justify-end gap-1 pr-1">
            <Button variant="ghost" size="icon" title="打开工作区" @click.stop="enterWorkspace(row)">
              <Monitor :size="15" />
            </Button>
            <Dropdown>
              <template #trigger>
                <Button variant="ghost" size="icon" title="更多操作" @click.stop>
                  <MoreHorizontal :size="15" />
                </Button>
              </template>
              <DropdownItem :icon="Monitor" @click="enterWorkspace(row)">打开工作区</DropdownItem>
              <DropdownItem :icon="FolderOpen" @click="openRepo(row)">打开文件夹</DropdownItem>
              <DropdownItem :icon="GitCommitHorizontal" @click="openHistory(row)">Git 记录</DropdownItem>
              <DropdownItem :icon="GitBranch" @click="openSourceControl(row)">源码控制</DropdownItem>
              <div class="my-1 h-px bg-border" />
              <div class="px-2 py-1 text-xs text-muted-foreground">移动到分组</div>
              <DropdownItem
                v-if="userGroups.length === 0"
                disabled
              >
                暂无分组，请先添加
              </DropdownItem>
              <DropdownItem
                v-for="g in userGroups"
                :key="g.id"
                @click="moveTo(row, g.id)"
              >
                <span class="group-dot" :style="{ backgroundColor: g.color }" />
                {{ g.name }}
              </DropdownItem>
              <DropdownItem v-if="row.group_id" danger @click="moveTo(row, null)">
                移出分组
              </DropdownItem>
              <div class="my-1 h-px bg-border" />
              <DropdownItem danger :icon="Trash2" @click="openRemove(row)">
                取消管理项目
              </DropdownItem>
            </Dropdown>
          </div>
        </div>

        <Empty v-if="!filteredProjects.length" description="暂无项目" />
      </div>
    </div>

    <!-- 底部批量操作栏 -->
    <div
      v-if="hasSelection"
      class="flex h-11 shrink-0 items-center justify-between border-t border-border bg-card px-4"
    >
      <span class="text-[13px] text-muted-foreground">已选择 {{ selectedProjectIds.size }} 个</span>
      <div class="flex items-center gap-2">
        <Dropdown>
          <template #trigger>
            <Button size="sm">
              <Folder :size="14" /> 移动到分组
            </Button>
          </template>
          <DropdownItem v-for="g in userGroups" :key="g.id" @click="batchMove(g.id)">
            <span class="group-dot" :style="{ backgroundColor: g.color }" />
            {{ g.name }}
          </DropdownItem>
          <DropdownItem danger @click="batchMove(null)">移出分组</DropdownItem>
        </Dropdown>

        <Button size="sm" :disabled="operationStore.isQueueRunning" @click="batchFetch">
          <RefreshCw :size="14" /> 获取
        </Button>
        <Button size="sm" variant="primary" :disabled="operationStore.isQueueRunning" @click="batchPull">
          <Download :size="14" /> 拉取
        </Button>
        <Button size="sm" :disabled="operationStore.isQueueRunning" @click="batchPush">
          <Upload :size="14" /> 推送
        </Button>
        <Button size="sm" variant="ghost" danger @click="openBatchRemove">
          <Trash2 :size="14" /> 取消管理
        </Button>
        <Button size="sm" variant="ghost" @click="clearSelection">取消选择</Button>
      </div>
    </div>

    <SourceControlModal v-model="showSourceControl" :project="sourceControlProject" />
    <ConfirmDialog
      v-model="confirmOpen"
      title="取消管理项目"
      :message="confirmMessage"
      confirm-text="取消管理"
      danger
      @confirm="doRemove"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, watch, defineAsyncComponent } from 'vue';
import {
  Search,
  Folder,
  FolderOpen,
  RefreshCw,
  Download,
  MoreHorizontal,
  Upload,
  GitCommitHorizontal,
  Monitor,
  GitBranch,
  Trash2,
} from 'lucide-vue-next';
import type { Project, ProjectStatus } from '../../types';
import { useAppStore } from '../../stores/appStore';
import { useOperationStore } from '../../stores/operationStore';
import { useTabStore } from '../../stores/tabStore';
import { openRepoFolder } from '../../lib/tauriApi';
import StatusBadge from './StatusBadge.vue';
import StatusChanges from './StatusChanges.vue';
const SourceControlModal = defineAsyncComponent(
  () => import('../Modals/SourceControlModal.vue'),
);
const ConfirmDialog = defineAsyncComponent(
  () => import('../ui/ConfirmDialog.vue'),
);
import { useProjectStatus } from '../../composables/useProjectStatus';
import { toast } from '../../lib/toast';
import Button from '../ui/Button.vue';
import Input from '../ui/Input.vue';
import Dropdown from '../ui/Dropdown.vue';
import DropdownItem from '../ui/DropdownItem.vue';
import Empty from '../ui/Empty.vue';

const appStore = useAppStore();
const operationStore = useOperationStore();
const tabStore = useTabStore();
const { getStatus } = useProjectStatus();

const showSourceControl = ref(false);
const sourceControlProject = ref<Project | null>(null);

const confirmOpen = ref(false);
const pendingRemoveId = ref<string | null>(null);
const batchRemoveMode = ref(false);

const statusFor = (projectId: string): ProjectStatus | null =>
  appStore.statuses.get(projectId) ?? null;

const searchQuery = computed({
  get: () => appStore.searchQuery,
  set: (value) => (appStore.searchQuery = value),
});

const selectedProjectIds = computed(() => appStore.selectedProjectIds);
const hasSelection = computed(() => appStore.hasSelection);
const filteredProjects = computed(() => appStore.filteredProjects);
const userGroups = computed(() => appStore.groups);

const allSelected = computed(
  () =>
    filteredProjects.value.length > 0 &&
    filteredProjects.value.every((p) => selectedProjectIds.value.has(p.id)),
);
const someSelected = computed(() => selectedProjectIds.value.size > 0);

const conflictProjects = computed(() =>
  appStore.projects.filter((p) => (appStore.statuses.get(p.id)?.conflict_count ?? 0) > 0),
);
const inProgressProjects = computed(() =>
  appStore.projects.filter((p) => !!appStore.statuses.get(p.id)?.in_progress),
);

function openFirstConflict() {
  const p = conflictProjects.value[0];
  if (p) openSourceControl(p);
}

function toggleSelectAll() {
  if (allSelected.value) {
    appStore.clearSelection();
  } else {
    appStore.selectAll();
  }
}

function clearSelection() {
  appStore.clearSelection();
}

function openRepo(project: Project) {
  openRepoFolder(project.id);
}

function groupName(groupId: string | null) {
  if (!groupId) return '未分组';
  return userGroups.value.find((g) => g.id === groupId)?.name ?? '未分组';
}

function enterWorkspace(project: Project) {
  tabStore.openProject(project.id, 'workspace');
}

function openHistory(project: Project) {
  tabStore.openProject(project.id, 'history');
}

function openSourceControl(project: Project) {
  sourceControlProject.value = project;
  showSourceControl.value = true;
}

const confirmMessage = computed(() => {
  if (batchRemoveMode.value) {
    const n = selectedProjectIds.value.size;
    return `确定要取消管理选中的 ${n} 个项目吗？该操作不会删除本地文件，仅从 GitDash 中移除。`;
  }
  const name =
    pendingRemoveId.value
      ? appStore.projects.find((p) => p.id === pendingRemoveId.value)?.name ?? ''
      : '';
  return `确定要取消管理项目「${name}」吗？该操作不会删除本地文件，仅从 GitDash 中移除。`;
});

function openRemove(row: Project) {
  pendingRemoveId.value = row.id;
  batchRemoveMode.value = false;
  confirmOpen.value = true;
}

function openBatchRemove() {
  if (selectedProjectIds.value.size === 0) return;
  batchRemoveMode.value = false;
  pendingRemoveId.value = null;
  confirmOpen.value = true;
}

async function doRemove() {
  if (batchRemoveMode.value) {
    const ids = [...selectedProjectIds.value];
    for (const id of ids) {
      try {
        await appStore.removeProject(id);
      } catch {
        /* 单个失败不阻断其余 */
      }
    }
    clearSelection();
    toast.success(`已取消管理 ${ids.length} 个项目`);
  } else if (pendingRemoveId.value) {
    const name =
      appStore.projects.find((p) => p.id === pendingRemoveId.value)?.name ?? '';
    try {
      await appStore.removeProject(pendingRemoveId.value);
      toast.success(`已取消管理「${name}」`);
    } catch {
      toast.error('取消管理失败');
    }
  }
  confirmOpen.value = false;
  pendingRemoveId.value = null;
  batchRemoveMode.value = false;
}

function moveTo(project: Project, groupId: string | null) {
  appStore.moveToGroup(project.id, groupId);
  toast.success(`已将「${project.name}」移动到「${groupName(groupId)}」`);
}

function batchMove(groupId: string | null) {
  const ids = Array.from(selectedProjectIds.value);
  if (!ids.length) return;
  appStore.moveProjectsToGroup(ids, groupId);
  toast.success(`已移动 ${ids.length} 个项目到「${groupName(groupId)}」`);
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

watch(
  () => appStore.projects.length,
  () => {
    appStore.projects.forEach((project) => getStatus(project.id, true));
  },
);

onMounted(() => {
  appStore.projects.forEach((project) => getStatus(project.id, true));
});
</script>

<style scoped>
.row-check {
  width: 15px;
  height: 15px;
  accent-color: var(--primary);
  cursor: pointer;
}
.group-dot {
  display: inline-block;
  width: 8px;
  height: 8px;
  margin-right: 6px;
  border-radius: 50%;
  vertical-align: middle;
}
.conflict-banner,
.progress-banner {
  border: none;
  cursor: pointer;
  font-size: 12px;
  font-weight: 600;
  padding: 4px 10px;
  border-radius: 6px;
  white-space: nowrap;
}
.conflict-banner {
  background: color-mix(in oklab, #dc2626 16%, transparent);
  color: #dc2626;
}
.conflict-banner:hover {
  background: color-mix(in oklab, #dc2626 26%, transparent);
}
.progress-banner {
  background: color-mix(in oklab, #d97706 16%, transparent);
  color: #b45309;
}
.progress-banner:hover {
  background: color-mix(in oklab, #d97706 26%, transparent);
}
</style>
