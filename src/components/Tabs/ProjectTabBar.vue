<!--
  标签栏（Fork / 浏览器风格）
  横向标签：项目列表系统标签 + 仓库标签；右侧 + 打开项目
  中键关闭；右键菜单：切换模式 / 关闭
-->
<template>
  <div class="tab-bar" role="tablist">
    <div class="tab-strip">
      <!-- 项目列表系统标签（置首，可关闭） -->
      <div
        v-if="hasProjectsTab"
        class="tab-item tab-item--system"
        :class="{ 'tab-item--active': isProjectsActive }"
        title="项目列表"
        role="tab"
        :aria-selected="isProjectsActive"
        @click="tabStore.setActive(PROJECTS_TAB_ID)"
        @auxclick.middle.prevent="tabStore.closeTab(PROJECTS_TAB_ID)"
      >
        <span class="tab-dot tab-dot--none" />
        <span class="tab-label tab-label--system">项目列表</span>
        <button
          type="button"
          class="tab-close"
          title="关闭标签"
          @click.stop="tabStore.closeTab(PROJECTS_TAB_ID)"
        >
          <X :size="12" />
        </button>
      </div>

      <ContextMenu v-for="tab in projectTabs" :key="tab.projectId">
        <template #trigger>
          <div
            class="tab-item"
            :class="{ 'tab-item--active': tab.projectId === activeProjectId }"
            :title="projectPath(tab.projectId)"
            role="tab"
            :aria-selected="tab.projectId === activeProjectId"
            @click="tabStore.setActive(tab.projectId)"
            @auxclick.middle.prevent="tabStore.closeTab(tab.projectId)"
          >
            <span class="tab-dot" :class="`tab-dot--${dotTone(tab.projectId)}`" />
            <span class="tab-label">{{ projectName(tab.projectId) }}</span>
            <button
              type="button"
              class="tab-close"
              title="关闭标签"
              @click.stop="tabStore.closeTab(tab.projectId)"
            >
              <X :size="12" />
            </button>
          </div>
        </template>
        <ContextMenuItem @click="tabStore.openProject(tab.projectId, 'workspace')">
          在工作区打开
        </ContextMenuItem>
        <ContextMenuItem @click="tabStore.openProject(tab.projectId, 'changes')">
          打开变更
        </ContextMenuItem>
        <ContextMenuItem @click="tabStore.openProject(tab.projectId, 'history')">
          打开历史
        </ContextMenuItem>
        <ContextMenuDivider />
        <ContextMenuItem @click="tabStore.closeTab(tab.projectId)">关闭标签</ContextMenuItem>
        <ContextMenuItem @click="tabStore.closeOtherTabs(tab.projectId)">
          关闭其他
        </ContextMenuItem>
        <ContextMenuItem @click="closeAll">关闭全部</ContextMenuItem>
      </ContextMenu>

      <button
        type="button"
        class="tab-add"
        title="打开项目标签"
        @click="$emit('open-picker')"
      >
        <Plus :size="14" />
      </button>
    </div>

    <div class="tab-actions">
      <slot name="actions" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Plus, X } from 'lucide-vue-next';
import { useAppStore } from '../../stores/appStore';
import { useTabStore, PROJECTS_TAB_ID } from '../../stores/tabStore';
import ContextMenu from '../ui/ContextMenu.vue';
import ContextMenuItem from '../ui/ContextMenuItem.vue';
import ContextMenuDivider from '../ui/Divider.vue';

defineEmits<{ (e: 'open-picker'): void }>();

const appStore = useAppStore();
const tabStore = useTabStore();

const projectTabs = computed(() =>
  tabStore.tabs.filter((t) => t.projectId !== PROJECTS_TAB_ID),
);
const hasProjectsTab = computed(() =>
  tabStore.tabs.some((t) => t.projectId === PROJECTS_TAB_ID),
);
const isProjectsActive = computed(
  () => tabStore.activeProjectId === PROJECTS_TAB_ID,
);
const activeProjectId = computed(() => tabStore.activeProjectId);

function projectName(projectId: string): string {
  return appStore.projects.find((p) => p.id === projectId)?.name ?? '未知项目';
}

function projectPath(projectId: string): string {
  return appStore.projects.find((p) => p.id === projectId)?.path ?? '';
}

function dotTone(projectId: string): string {
  const s = appStore.statuses.get(projectId);
  if (!s) return 'none';
  if (s.error || s.conflict_count > 0) return 'danger';
  if (s.in_progress || !s.is_clean) return 'warn';
  if (s.ahead || s.behind) return 'info';
  return 'ok';
}

function closeAll() {
  // 关闭全部时若只想回到项目列表，可保留系统标签；默认全关回主页
  tabStore.closeAllTabs();
}
</script>

<style scoped>
.tab-bar {
  display: flex;
  align-items: stretch;
  height: 36px;
  flex-shrink: 0;
  border-bottom: 1px solid var(--border);
  background-color: var(--card);
}
.tab-strip {
  display: flex;
  align-items: stretch;
  flex: 1;
  min-width: 0;
  overflow-x: auto;
  overflow-y: hidden;
  scrollbar-width: thin;
}
.tab-strip :deep(.ui-dropdown-panel) {
  min-width: 160px;
}
.tab-item {
  display: flex;
  align-items: center;
  gap: 6px;
  max-width: 180px;
  min-width: 100px;
  height: 100%;
  padding: 0 8px 0 12px;
  border-right: 1px solid var(--border);
  cursor: pointer;
  color: var(--muted-foreground);
  font-size: 12px;
  user-select: none;
  white-space: nowrap;
  transition: background-color 0.12s ease, color 0.12s ease;
}
.tab-item:hover {
  background-color: var(--accent);
  color: var(--accent-foreground);
}
.tab-item--active {
  background-color: var(--background);
  color: var(--foreground);
  box-shadow: inset 0 -2px 0 0 var(--primary);
}
.tab-item--system {
  min-width: 96px;
}
.tab-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
  background-color: transparent;
}
.tab-dot--ok {
  background-color: oklch(0.65 0.16 150);
}
.tab-dot--warn {
  background-color: oklch(0.75 0.16 85);
}
.tab-dot--danger {
  background-color: var(--destructive);
}
.tab-dot--info {
  background-color: var(--primary);
}
.tab-dot--none {
  background-color: var(--muted-foreground);
  opacity: 0.35;
}
.tab-label {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
}
.tab-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  border-radius: 4px;
  flex-shrink: 0;
  color: inherit;
  opacity: 0.5;
}
.tab-close:hover {
  background-color: var(--muted);
  opacity: 1;
}
.tab-item:hover .tab-close {
  opacity: 0.8;
}
.tab-add {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  flex-shrink: 0;
  color: var(--muted-foreground);
}
.tab-add:hover {
  background-color: var(--accent);
  color: var(--accent-foreground);
}
.tab-actions {
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 0 8px;
  flex-shrink: 0;
}
</style>
