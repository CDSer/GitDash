<!--
  分组树组件
  显示所有分组（全部 / 收藏 / 未分组 / 自定义分组）
  支持点击切换当前分组、添加分组
-->
<template>
  <div class="group-tree">
    <div class="tree-header">
      <div class="brand">GitDash</div>
      <el-button type="primary" plain :icon="Plus" class="add-btn" @click="openAddGroup">
        添加分组
      </el-button>
    </div>

    <el-menu
      class="group-menu"
      :default-active="activeGroupId ?? ''"
      @select="selectGroup"
    >
      <el-menu-item v-for="group in allGroups" :key="group.id" :index="group.id">
        <el-dropdown
          v-if="isUserGroup(group.id)"
          trigger="contextmenu"
          placement="bottom-start"
          @command="(cmd) => handleGroupCommand(cmd, group)"
        >
          <span class="menu-item-content">
            <span class="group-dot" :style="{ backgroundColor: group.color }"></span>
            <span class="group-name">{{ group.name }}</span>
            <span class="group-count">{{ getGroupCount(group.id) }}</span>
          </span>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item :icon="EditPen" command="rename">重命名</el-dropdown-item>
              <el-dropdown-item :icon="Delete" command="delete" divided>删除</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>

        <template v-else>
          <span class="group-dot" :style="{ backgroundColor: group.color }"></span>
          <span class="group-name">{{ group.name }}</span>
          <span class="group-count">{{ getGroupCount(group.id) }}</span>
        </template>
      </el-menu-item>
    </el-menu>

    <AddGroupModal v-model="groupModalVisible" :group="editingGroup" @saved="handleGroupSaved" />
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { Plus, EditPen, Delete } from '@element-plus/icons-vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { useAppStore } from '../../stores/appStore';
import type { Group } from '../../types';
import AddGroupModal from '../Modals/AddGroupModal.vue';

const appStore = useAppStore();

const groupModalVisible = ref(false);
const editingGroup = ref<Group | null>(null);

const allGroups = computed(() => appStore.allGroups);
const activeGroupId = computed(() => appStore.activeGroupId);

function selectGroup(groupId: string) {
  appStore.activeGroupId = groupId;
}

function getGroupCount(groupId: string) {
  if (groupId === 'all') {
    return appStore.projects.length;
  } else if (groupId === 'favorites') {
    return appStore.projects.filter(p => p.is_favorite).length;
  } else if (groupId === 'untagged') {
    return appStore.projects.filter(p => !p.group_id).length;
  } else {
    return appStore.projects.filter(p => p.group_id === groupId).length;
  }
}

/** 打开「添加分组」弹窗 */
function openAddGroup() {
  editingGroup.value = null;
  groupModalVisible.value = true;
}

/** 打开「重命名分组」弹窗 */
function startRename(group: Group) {
  editingGroup.value = group;
  groupModalVisible.value = true;
}

function handleGroupSaved() {
  groupModalVisible.value = false;
}

/** 「全部 / 收藏 / 未分组」是系统虚拟分组，不可删除/重命名 */
function isUserGroup(groupId: string) {
  return !['all', 'favorites', 'untagged'].includes(groupId);
}

/** 右键菜单命令分发 */
function handleGroupCommand(command: unknown, group: Group) {
  if (command === 'rename') {
    startRename(group);
  } else if (command === 'delete') {
    handleDeleteGroup(group);
  }
}

/** 删除自建分组（该分组下的项目会变为未分组） */
async function handleDeleteGroup(group: Group) {
  const count = getGroupCount(group.id);

  try {
    await ElMessageBox.confirm(
      `确定删除分组「${group.name}」吗？该分组下的 ${count} 个项目会变为「未分组」，此操作不可撤销。`,
      '删除分组',
      { type: 'warning', confirmButtonText: '删除', cancelButtonText: '取消', closeOnClickModal: false }
    );
  } catch {
    return; // 用户点了取消
  }

  try {
    await appStore.removeGroup(group.id);
    ElMessage.success(`已删除分组「${group.name}」`);
  } catch (error) {
    console.error('删除分组失败：', error);
    ElMessage.error('删除分组失败');
  }
}
</script>

<style scoped>
.group-tree {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.tree-header {
  padding: 12px;
  border-bottom: 1px solid var(--el-border-color);
}

.brand {
  margin-bottom: 12px;
  font-size: 14px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.add-btn {
  width: 100%;
}

.group-menu {
  flex: 1;
  overflow-y: auto;
  border-right: none;
  background-color: transparent;
}

.group-menu :deep(.el-menu-item) {
  height: 36px;
  line-height: 36px;
  display: flex;
  align-items: center;
  gap: 8px;
  padding-right: 12px !important;
}

.group-menu :deep(.el-dropdown) {
  display: block;
  width: 100%;
}

.menu-item-content {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
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
}

.group-count {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
</style>
