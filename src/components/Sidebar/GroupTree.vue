<!--
  分组树组件
  显示所有分组（全部 / 未分组 / 自定义分组）
  支持点击切换当前分组、右键重命名 / 删除
  支持分组折叠/展开，以及按住分组标题拖拽排序（全部 / 未分组固定在最前）
-->
<template>
  <div class="flex h-full flex-col">
    <div class="border-b border-border p-3">
      <div class="mb-3 text-sm font-semibold">GitDash</div>
      <Button
        variant="ghost"
        class="mb-1.5 w-full justify-start"
        :class="isHomeRoute ? 'nav-active' : ''"
        @click="goHome"
      >
        <Home :size="14" /> 主页
      </Button>
      <Button
        variant="ghost"
        class="mb-1.5 w-full justify-start"
        :class="isProjectsRoute ? 'nav-active' : ''"
        @click="goProjects"
      >
        <List :size="14" /> 项目
      </Button>
      <Button variant="outline" class="w-full" @click="openAddGroup">
        <Plus :size="14" /> 添加分组
      </Button>
    </div>

    <div ref="scrollContainer" class="min-h-0 flex-1 overflow-y-auto p-1.5">
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
        <div
          class="group-header select-none"
          :data-group-block-id="isUserGroup(group.id) ? group.id : undefined"
          :class="{
            'group-header--sorting': sortDragId === group.id,
            'group-header--sort-target-before':
              sortTarget?.groupId === group.id && sortTarget?.side === 'before',
            'group-header--sort-target-after':
              sortTarget?.groupId === group.id && sortTarget?.side === 'after',
          }"
          :title="isUserGroup(group.id) ? '拖动可排序分组' : undefined"
          @pointerdown="startGroupSort(group, $event)"
          @click="onGroupRowClick(group)"
        >
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

          <ContextMenu v-if="group.id !== 'all'">
            <template #trigger>
              <span class="group-row-inner">
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
          <span v-else class="group-row-inner">
            <span class="group-dot" :style="{ backgroundColor: group.color }" />
            <span class="group-name">{{ group.name }}</span>
            <span class="group-count">{{ getGroupCount(group.id) }}</span>
          </span>
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
            @click="openProjectHistory(project)"
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
import { computed, ref, onMounted, watch, onBeforeUnmount } from 'vue';
import { Plus, Pencil, Trash2, FolderX, ChevronRight, ChevronDown, Home, List } from 'lucide-vue-next';
import { useAppStore } from '../../stores/appStore';
import { useDragProject } from '../../composables/useDragProject';
import { useRoute, useRouter } from 'vue-router';
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
const route = useRoute();
const router = useRouter();

const groupModalVisible = ref(false);
const editingGroup = ref<Group | null>(null);
const confirmOpen = ref(false);
const pendingDelete = ref<Group | null>(null);
const unmanageOpen = ref(false);
const pendingUnmanage = ref<Group | null>(null);
const expandedGroupIds = ref<Set<string>>(new Set());
const scrollContainer = ref<HTMLElement | null>(null);

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
  // 从主页点分组时切到项目列表，保证过滤结果可见
  if (route.name !== 'projects') {
    router.push({ name: 'projects' });
  }
}

const isHomeRoute = computed(() => route.name === 'home');
const isProjectsRoute = computed(() => route.name === 'projects');

function goHome() {
  router.push({ name: 'home' });
}

function goProjects() {
  appStore.activeGroupId = 'all';
  router.push({ name: 'projects' });
}

let suppressRowClickUntil = 0;

// 点击整行：可展开的分组展开/折叠，同时保持选中切换；空分组退化为仅选中
function onGroupRowClick(group: Group) {
  if (Date.now() < suppressRowClickUntil) return;
  if (canExpand(group.id)) {
    toggleGroup(group.id);
  }
  selectGroup(group.id);
}

function getGroupCount(groupId: string) {
  if (groupId === 'all') return appStore.projects.length;
  if (groupId === 'untagged') return appStore.projects.filter((p) => !p.group_id).length;
  return appStore.projects.filter((p) => p.group_id === groupId).length;
}

function canExpand(groupId: string) {
  if (groupId === 'all') return appStore.projects.length > 0;
  return groupId !== 'untagged' || appStore.projects.some((p) => !p.group_id);
}

function canShowProjects(groupId: string) {
  return canExpand(groupId);
}

function isExpanded(groupId: string) {
  return expandedGroupIds.value.has(groupId);
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

function expandGroup(groupId: string) {
  if (!canExpand(groupId)) return;
  const next = new Set(expandedGroupIds.value);
  next.add(groupId);
  expandedGroupIds.value = next;
}

// ========== 分组拖拽排序 ==========
// 按住分组标题行拖动调整顺序；系统分组（全部 / 未分组）固定在最前，不参与排序
const SORT_DRAG_THRESHOLD = 5;
const sortDragId = ref<string | null>(null);
const sortTarget = ref<{ groupId: string; side: 'before' | 'after' } | null>(null);
let sortCandidate: { id: string; x: number; y: number } | null = null;
let sortScrollTimer: number | null = null;
let sortLastClientY = 0;

function startGroupSort(group: Group, event: PointerEvent) {
  if (event.button !== 0 || !isUserGroup(group.id)) return;
  if (sortCandidate) return;
  sortCandidate = { id: group.id, x: event.clientX, y: event.clientY };
  sortLastClientY = event.clientY;
  window.addEventListener('pointermove', onGroupSortMove);
  window.addEventListener('pointerup', onGroupSortEnd);
  window.addEventListener('pointercancel', onGroupSortEnd);
}

function onGroupSortMove(event: PointerEvent) {
  if (!sortCandidate) return;
  sortLastClientY = event.clientY;
  if (!sortDragId.value) {
    const dx = event.clientX - sortCandidate.x;
    const dy = event.clientY - sortCandidate.y;
    if (Math.hypot(dx, dy) < SORT_DRAG_THRESHOLD) return;
    sortDragId.value = sortCandidate.id;
    startSortAutoScroll();
  }
  updateSortTarget(event.clientY);
}

function updateSortTarget(clientY: number) {
  const dragId = sortDragId.value;
  if (!dragId) {
    sortTarget.value = null;
    return;
  }
  const headers = Array.from(
    document.querySelectorAll<HTMLElement>('[data-group-block-id]'),
  )
    .map((el) => {
      const rect = el.getBoundingClientRect();
      return { id: el.dataset.groupBlockId as string, mid: rect.top + rect.height / 2 };
    })
    .filter((h) => h.id !== dragId);

  let target: { groupId: string; side: 'before' | 'after' } | null = null;
  for (const h of headers) {
    if (clientY < h.mid) {
      target = { groupId: h.id, side: 'before' };
      break;
    }
  }
  if (!target && headers.length > 0) {
    target = { groupId: headers[headers.length - 1].id, side: 'after' };
  }
  sortTarget.value = target;
}

// 拖到列表容器上/下边缘时自动滚动（指针停在边缘也会持续滚动）
function startSortAutoScroll() {
  if (sortScrollTimer !== null) return;
  sortScrollTimer = window.setInterval(() => {
    const el = scrollContainer.value;
    if (!el || !sortDragId.value) return;
    const rect = el.getBoundingClientRect();
    const margin = 28;
    let scrollBy = 0;
    if (sortLastClientY < rect.top + margin) {
      scrollBy = -12;
    } else if (sortLastClientY > rect.bottom - margin) {
      scrollBy = 12;
    }
    if (scrollBy !== 0) {
      const before = el.scrollTop;
      el.scrollTop += scrollBy;
      if (el.scrollTop !== before) {
        updateSortTarget(sortLastClientY);
      }
    }
  }, 60);
}

function stopSortAutoScroll() {
  if (sortScrollTimer !== null) {
    window.clearInterval(sortScrollTimer);
    sortScrollTimer = null;
  }
}

function onGroupSortEnd() {
  window.removeEventListener('pointermove', onGroupSortMove);
  window.removeEventListener('pointerup', onGroupSortEnd);
  window.removeEventListener('pointercancel', onGroupSortEnd);
  stopSortAutoScroll();
  if (!sortCandidate) return;
  const dragId = sortCandidate.id;
  sortCandidate = null;

  const target = sortTarget.value;
  const didDrag = sortDragId.value !== null;
  if (didDrag && target && target.groupId !== dragId) {
    // 以当前 sort_order 顺序为准，将被拖分组移到目标位置
    const ordered = [...appStore.groups]
      .sort((a, b) => a.sort_order - b.sort_order)
      .map((g) => g.id);
    const from = ordered.indexOf(dragId);
    if (from !== -1) {
      ordered.splice(from, 1);
      const base = ordered.indexOf(target.groupId);
      const to = base === -1 ? ordered.length : target.side === 'after' ? base + 1 : base;
      ordered.splice(Math.min(to, ordered.length), 0, dragId);
      appStore.reorderGroups(ordered);
    }
  }

  // 拖拽结束后短时间内忽略松手位置触发的行点击，避免误折叠
  if (didDrag) {
    suppressRowClickUntil = Date.now() + 300;
  }

  sortDragId.value = null;
  sortTarget.value = null;
}

onBeforeUnmount(() => {
  if (sortCandidate) {
    sortCandidate = null;
    sortDragId.value = null;
    sortTarget.value = null;
    stopSortAutoScroll();
    window.removeEventListener('pointermove', onGroupSortMove);
    window.removeEventListener('pointerup', onGroupSortEnd);
    window.removeEventListener('pointercancel', onGroupSortEnd);
  }
});

function projectsInGroup(groupId: string): Project[] {
  if (groupId === 'all') return appStore.projects;
  if (groupId === 'untagged') {
    return appStore.projects.filter((p) => !p.group_id);
  }
  return appStore.projects.filter((p) => p.group_id === groupId);
}

// 点击左侧项目行进入 Git 记录页（拖拽后松手瞬间忽略，避免误跳转）
function openProjectHistory(project: Project) {
  if (drag.wasDraggingRecently()) return;
  router.push({ name: 'history', params: { projectId: project.id } });
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
  return groupId !== 'untagged' && groupId !== 'all';
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
/* 拖拽排序中的分组标题 */
.group-header--sorting {
  opacity: 0.55;
  background-color: var(--accent);
  color: var(--accent-foreground);
}
/* 排序插入位置指示线 */
.group-header--sort-target-before {
  box-shadow: inset 0 2px 0 0 var(--primary);
}
.group-header--sort-target-after {
  box-shadow: inset 0 -2px 0 0 var(--primary);
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
.nav-active {
  background-color: color-mix(in oklab, var(--primary) 16%, transparent);
  color: var(--primary);
}
</style>
