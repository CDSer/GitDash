<!--
  分组树组件
  显示所有分组（全部 / 收藏 / 未分组 / 自定义分组）
  支持点击切换当前分组、右键重命名 / 删除
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
        :class="[
          'group-row',
          activeGroupId === group.id ? 'group-row--active' : '',
        ]"
        @click="selectGroup(group.id)"
      >
        <Dropdown v-if="isUserGroup(group.id)" trigger="contextmenu">
          <template #trigger>
            <span class="group-row-inner">
              <span class="group-dot" :style="{ backgroundColor: group.color }" />
              <span class="group-name">{{ group.name }}</span>
              <span class="group-count">{{ getGroupCount(group.id) }}</span>
            </span>
          </template>
          <DropdownItem :icon="Pencil" @click="startRename(group)">重命名</DropdownItem>
          <DropdownItem danger :icon="Trash2" @click="askDelete(group)">删除</DropdownItem>
        </Dropdown>

        <span v-else class="group-row-inner">
          <span class="group-dot" :style="{ backgroundColor: group.color }" />
          <span class="group-name">{{ group.name }}</span>
          <span class="group-count">{{ getGroupCount(group.id) }}</span>
        </span>
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
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { Plus, Pencil, Trash2 } from 'lucide-vue-next';
import { useAppStore } from '../../stores/appStore';
import type { Group } from '../../types';
import AddGroupModal from '../Modals/AddGroupModal.vue';
import Button from '../ui/Button.vue';
import Dropdown from '../ui/Dropdown.vue';
import DropdownItem from '../ui/DropdownItem.vue';
import ConfirmDialog from '../ui/ConfirmDialog.vue';
import { toast } from '../../lib/toast';

const appStore = useAppStore();

const groupModalVisible = ref(false);
const editingGroup = ref<Group | null>(null);
const confirmOpen = ref(false);
const pendingDelete = ref<Group | null>(null);

const confirmMessage = computed(() =>
  pendingDelete.value
    ? `确定要删除分组「${pendingDelete.value.name}」吗？该分组下的项目会被移动到「未分组」。`
    : '',
);

const allGroups = computed(() => appStore.allGroups);
const activeGroupId = computed(() => appStore.activeGroupId);

function selectGroup(groupId: string) {
  appStore.activeGroupId = groupId;
}

function getGroupCount(groupId: string) {
  if (groupId === 'all') return appStore.projects.length;
  if (groupId === 'favorites') return appStore.projects.filter((p) => p.is_favorite).length;
  if (groupId === 'untagged') return appStore.projects.filter((p) => !p.group_id).length;
  return appStore.projects.filter((p) => p.group_id === groupId).length;
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
  return !['all', 'favorites', 'untagged'].includes(groupId);
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
  }
}
</script>

<style scoped>
.group-row {
  border-radius: 6px;
  cursor: pointer;
  color: var(--muted-foreground);
}
.group-row:hover {
  background-color: var(--accent);
  color: var(--accent-foreground);
}
.group-row--active {
  background-color: color-mix(in oklab, var(--primary) 16%, transparent);
  color: var(--primary);
}
.group-row-inner {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 7px 10px;
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
</style>
