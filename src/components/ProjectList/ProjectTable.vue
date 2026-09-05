<!--
  项目表格组件
  自绘表格展示项目，支持多选、搜索、双击打开文件夹
-->
<template>
  <div class="flex h-full flex-col">
    <!-- 搜索框 -->
    <div class="border-b border-border p-3">
      <div class="relative max-w-[320px]">
        <Search
          :size="14"
          class="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground"
        />
        <Input v-model="searchQuery" placeholder="搜索项目..." :style="{ paddingLeft: '2rem' }" />
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
          class="group grid cursor-default items-center border-b border-border px-3 py-2 text-[13px] hover:bg-accent/60"
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
          <div class="flex min-w-0 items-center gap-1.5">
            <span class="truncate font-medium">{{ row.name }}</span>
            <Star v-if="row.is_favorite" :size="14" class="shrink-0 text-amber-500" fill="currentColor" />
          </div>
          <div class="truncate text-muted-foreground" :title="row.path">{{ row.path }}</div>
          <div class="truncate text-muted-foreground">
            {{ statusFor(row.id)?.branch || '-' }}
          </div>
          <div class="flex justify-center">
            <StatusBadge :status="statusFor(row.id)" />
          </div>
          <div class="flex justify-center">
            <StatusChanges :status="statusFor(row.id)" />
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
              <DropdownItem :icon="Star" @click="appStore.toggleFavorite(row.id)">
                {{ row.is_favorite ? '取消收藏' : '设为收藏' }}
              </DropdownItem>
              <DropdownItem :icon="GitCommitHorizontal" @click="openHistory(row)">Git 记录</DropdownItem>
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
        <Button size="sm" variant="ghost" @click="clearSelection">取消选择</Button>
      </div>
    </div>

    <GitHistoryModal v-model="showHistory" :project="historyProject" />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, watch } from 'vue';
import {
  Search,
  Star,
  Folder,
  FolderOpen,
  RefreshCw,
  Download,
  MoreHorizontal,
  Upload,
  GitCommitHorizontal,
  Monitor,
} from 'lucide-vue-next';
import type { Project, ProjectStatus } from '../../types';
import { useAppStore } from '../../stores/appStore';
import { useOperationStore } from '../../stores/operationStore';
import { openRepoFolder } from '../../lib/tauriApi';
import StatusBadge from './StatusBadge.vue';
import StatusChanges from './StatusChanges.vue';
import GitHistoryModal from '../Modals/GitHistoryModal.vue';
import { useProjectStatus } from '../../composables/useProjectStatus';
import { toast } from '../../lib/toast';
import Button from '../ui/Button.vue';
import Input from '../ui/Input.vue';
import Dropdown from '../ui/Dropdown.vue';
import DropdownItem from '../ui/DropdownItem.vue';
import Empty from '../ui/Empty.vue';

const appStore = useAppStore();
const operationStore = useOperationStore();
const { getStatus } = useProjectStatus();

const showHistory = ref(false);
const historyProject = ref<Project | null>(null);

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
  appStore.openWorkspace(project);
}

function openHistory(project: Project) {
  historyProject.value = project;
  showHistory.value = true;
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
</style>
