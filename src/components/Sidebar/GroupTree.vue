<!--
  分组树组件
  显示所有分组（未分组 / 自定义分组）
  支持点击切换当前分组、右键重命名 / 删除
  支持分组折叠/展开，以及拖拽项目到分组修改分组
-->
<template>
  <div class="flex h-full flex-col">
    <div class="border-b border-border p-3">
      <div class="mb-3 text-sm font-semibold">GitDash</div>
      <Button variant="outline" class="w-full" @click="openAddGroup">
        <Plus :size="14" /> 添加分组
      </Button>
    </div>

    <div class="min-h-0 flex-1 overflow-y-auto p-1.5">
      <div
        v-for="group in allGroups"
        :key="group.id"
        :data-group-id="group.id"
        class="group-block"
        :class="{
          'group-block--active': activeGroupId === group.id,
        }"
        @request-expand="expandGroup(group.id)"
      >
        <div class="group-header select-none">
          <button
            class="toggle-btn"
            :class="{ 'toggle-btn--visible': canExpand(group.id) }"
            @click.stop="toggleGroup(group.id)"
          >
            <ChevronRight
              v-if="!isExpanded(group.id)"
              :size="14"
              class="transition-transform duration-150"
            />
            <ChevronDown
              v-else
              :size="14"
              class="transition-transform duration-150"
            />
          </button>

          <ContextMenu>
            <template #trigger>
              <span class="group-row-inner" @click="selectGroup(group.id)">
                <span class="group-dot" :style="{ backgroundColor: group.color }" />
                <span class="group-name">{{ group.name }}</span>
                <span class="group-count">{{ getGroupCount(group.id) }}</span>
              </span>
            </template>
            <template v-if="isUserGroup(group.id)">
              <ContextMenuItem :icon="Pencil" @click="startRename(group)">重命名</ContextMenuItem>
              <ContextMenuItem danger :icon="Trash2" @click="askDelete(group)">删除</ContextMenuItem>
              <Divider />
            </template>
            <ContextMenuItem
              danger
              :icon="FolderX"
              :disabled="getGroupCount(group.id) === 0"
              @click="askUnmanage(group)"
            >
              取消管理所有项目
            </ContextMenuItem>
          </ContextMenu>
        </div>

        <!-- 展开后的项目列表 -->
        <div
          v-if="isExpanded(group.id) && canShowProjects(group.id)"
          class="project-list"
        >
          <div
            v-for="project in projectsInGroup(group.id)"
            :key="project.id"
            :data-project-id="project.id"
            class="project-item select-none"
            :title="project.path"
            @pointerdown="drag.startDrag(project.id, $event)"
          >
            <span class="project-dot" />
            <span class="project-name">{{ project.name }}</span>
          </div>
          <div
            v-if="!projectsInGroup(group.id).length"
            class="project-empty"
          >
            暂无项目
          </div>
        </div>
      </div>
    </div>

    <AddGroupModal v-model="groupModalVisible" :group="editingGroup" @saved="handleGroupSaved" />
    <ConfirmDialog
      v-model="confirmOpen"
      title="删除分组"
      :message="confirmMessage"
      confirm-text="删除"
      danger
      @confirm="doDelete"
    />
    <ConfirmDialog
      v-model="unmanageOpen"
      title="取消管理项目"
      :message="unmanageMessage"
      confirm-text="取消管理"
      danger
      @confirm="doUnmanage"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, watch } from 'vue';
import { Plus, Pencil, Trash2, FolderX, ChevronRight, ChevronDown } from 'lucide-vue-next';
import { useAppStore } from '../../stores/appStore';
import { useDragProject } from '../../composables/useDragProject';
import type { Group, Project } from '../../types';
import { defineAsyncComponent } from 'vue';
import Button from '../ui/Button.vue';
import ContextMenu from '../ui/ContextMenu.vue';
import ContextMenuItem from '../ui/ContextMenuItem.vue';
import Divider from '../ui/Divider.vue';

const AddGroupModal = defineAsyncComponent(
  () => import('../Modals/AddGroupModal.vue'),
);
const ConfirmDialog = defineAsyncComponent(
  () => import('../ui/ConfirmDialog.vue'),
);
import { toast } from '../../lib/toast';

const appStore = useAppStore();
const drag = useDragProject();

const groupModalVisible = ref(false);
const editingGroup = ref<Group | null>(null);
const confirmOpen = ref(false);
const pendingDelete = ref<Group | null>(null);
const unmanageOpen = ref(false);
const pendingUnmanage = ref<Group | null>(null);
const expandedGroupIds = ref<Set<string>>(new Set());

const confirmMessage = computed(() =>
  pendingDelete.value
    ? `确定要删除分组「${pendingDelete.value.name}」吗？该分组下的项目会被移动到「未分组」。`
    : '',
);

const unmanageMessage = computed(() => {
  if (!pendingUnmanage.value) return '';
  const count = getGroupCount(pendingUnmanage.value.id);
  return `确定要取消管理分组「${pendingUnmanage.value.name}」中的 ${count} 个项目吗？项目文件不会被删除。`;
});

const allGroups = computed(() => appStore.allGroups);
const activeGroupId = computed(() => appStore.activeGroupId);

// 默认展开自定义分组和「未分组」
onMounted(() => {
  const ids = new Set<string>(['untagged']);
  appStore.groups.forEach((g) => ids.add(g.id));
  expandedGroupIds.value = ids;
});

// 新增分组时自动展开
watch(
  () => appStore.groups,
  (newGroups) => {
    newGroups.forEach((g) => {
      if (!expandedGroupIds.value.has(g.id)) {
        expandedGroupIds.value.add(g.id);
      }
    });
  },
  { deep: true },
);

function selectGroup(groupId: string) {
  appStore.activeGroupId = groupId;
}

function getGroupCount(groupId: string) {
  if (groupId === 'untagged') return appStore.projects.filter((p) => !p.group_id).length;
  return appStore.projects.filter((p) => p.group_id === groupId).length;
}

function canExpand(groupId: string) {
  return groupId !== 'untagged' || appStore.projects.some((p) => !p.group_id);
}

function canShowProjects(groupId: string) {
  return canExpand(groupId);
}

function isExpanded(groupId: string) {
  return expandedGroupIds.value.has(groupId);
}

function expandGroup(groupId: string) {
  if (!canExpand(groupId)) return;
  const next = new Set(expandedGroupIds.value);
  next.add(groupId);
  expandedGroupIds.value = next;
}

function toggleGroup(groupId: string) {
  if (!canExpand(groupId)) return;
  const next = new Set(expandedGroupIds.value);
  if (next.has(groupId)) {
    next.delete(groupId);
  } else {
    next.add(groupId);
  }
  expandedGroupIds.value = next;
}

function projectsInGroup(groupId: string): Project[] {
  if (groupId === 'untagged') {
    return appStore.projects.filter((p) => !p.group_id);
  }
  return appStore.projects.filter((p) => p.group_id === groupId);
}

function openAddGroup() {
  editingGroup.value = null;
  groupModalVisible.value = true;
}

function startRename(group: Group) {
  editingGroup.value = group;
  groupModalVisible.value = true;
}

function handleGroupSaved() {
  groupModalVisible.value = false;
}

function isUserGroup(groupId: string) {
  return groupId !== 'untagged';
}

function askDelete(group: Group) {
  pendingDelete.value = group;
  confirmOpen.value = true;
}

async function doDelete() {
  const group = pendingDelete.value;
  if (!group) return;
  try {
    await appStore.removeGroup(group.id);
    toast.success(`已删除分组「${group.name}」`);
  } catch (error) {
    console.error('删除分组失败：', error);
    toast.error('删除分组失败');
  } finally {
    pendingDelete.value = null;
    confirmOpen.value = false;
  }
}

function askUnmanage(group: Group) {
  if (getGroupCount(group.id) === 0) return;
  pendingUnmanage.value = group;
  unmanageOpen.value = true;
}

async function doUnmanage() {
  const group = pendingUnmanage.value;
  if (!group) return;

  const projectIds = projectsInGroup(group.id).map((p) => p.id);
  if (projectIds.length === 0) {
    pendingUnmanage.value = null;
    unmanageOpen.value = false;
    return;
  }

  try {
    await appStore.removeProjects(projectIds);
    toast.success(`已取消管理 ${projectIds.length} 个项目`);
  } catch (error) {
    console.error('取消管理项目失败：', error);
    toast.error('取消管理项目失败');
  } finally {
    pendingUnmanage.value = null;
    unmanageOpen.value = false;
  }
}
</script>

<style scoped>
.group-block {
  border-radius: 6px;
  margin-bottom: 2px;
  transition: background-color 0.15s ease;
}
.group-block--active > .group-header {
  background-color: color-mix(in oklab, var(--primary) 16%, transparent);
  color: var(--primary);
}
.group-block--drop-over {
  background-color: color-mix(in oklab, var(--primary) 10%, transparent);
  outline: 2px dashed var(--primary);
  outline-offset: -2px;
}
.group-block--drop-over > .group-header {
  color: var(--primary);
}
.group-header {
  display: flex;
  align-items: center;
  border-radius: 6px;
  cursor: pointer;
  color: var(--muted-foreground);
}
.group-header:hover {
  background-color: var(--accent);
  color: var(--accent-foreground);
}
.toggle-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 28px;
  flex-shrink: 0;
  border-radius: 4px;
  opacity: 0;
  pointer-events: none;
  transition: opacity 0.15s ease;
}
.toggle-btn--visible {
  opacity: 0.6;
  pointer-events: auto;
}
.toggle-btn--visible:hover {
  opacity: 1;
  background-color: color-mix(in oklab, var(--foreground) 10%, transparent);
}
.group-row-inner {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
  padding: 7px 10px 7px 2px;
  outline: none;
}
.group-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}
.group-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 13px;
}
.group-count {
  font-size: 12px;
  opacity: 0.7;
}
.project-list {
  padding-left: 18px;
  padding-bottom: 2px;
}
.project-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 10px;
  border-radius: 5px;
  cursor: grab;
  color: var(--muted-foreground);
  font-size: 12px;
  transition: background-color 0.15s ease;
}
.project-item:hover {
  background-color: var(--accent);
  color: var(--accent-foreground);
}
.project-item:active {
  cursor: grabbing;
}
.project-dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background-color: currentColor;
  opacity: 0.5;
  flex-shrink: 0;
}
.project-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.project-empty {
  padding: 6px 10px;
  font-size: 12px;
  color: var(--muted-foreground);
  opacity: 0.6;
}
</style>
