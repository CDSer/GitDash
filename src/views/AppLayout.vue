<!--
  应用布局组件
  包含侧边栏分组树、顶部工具栏、中间 RouterView 主体区域
  宽度通过 CSS 变量控制，避免拖拽时触发 Vue 重渲染
-->
<template>
  <div class="flex h-screen flex-col overflow-hidden bg-background text-foreground">
    <div class="flex min-h-0 flex-1">
      <aside
        ref="sidebarRef"
        class="relative flex shrink-0 flex-col overflow-hidden border-r border-border bg-card"
        :class="{ collapsed: sidebarCollapsed, resizing: isResizing }"
      >
        <div v-if="!sidebarCollapsed" class="flex h-full flex-col">
          <GroupTree />
        </div>

        <!-- 收起状态：窄条 + 展开按钮 -->
        <div
          v-else
          class="flex h-full flex-col items-center border-r border-border bg-card py-2"
        >
          <Button variant="ghost" size="icon" title="展开侧边栏" @click="toggleSidebar">
            <PanelLeftOpen :size="18" />
          </Button>
        </div>

        <!-- 顶部折叠按钮（展开时显示） -->
        <Button
          v-if="!sidebarCollapsed"
          variant="ghost"
          size="icon"
          class="absolute right-1 top-1 h-7 w-7 opacity-60 hover:opacity-100"
          title="收起侧边栏"
          @click="toggleSidebar"
        >
          <PanelLeftClose :size="16" />
        </Button>

        <!-- 拖拽调整宽度条 -->
        <div
          v-if="!sidebarCollapsed"
          class="resizer"
          @pointerdown="startResize"
        />
      </aside>

      <div class="flex min-w-0 flex-1 flex-col">
        <!-- 项目多标签栏（Fork 风格）：始终可见；无标签时也可从 + 打开 -->
        <ProjectTabBar @open-picker="showProjectPicker = true" />

        <!-- 工具栏：仅在无活动标签（主页/列表）时显示完整操作；有标签时 ProjectTabView 自带顶栏 -->
        <header
          v-if="!tabStore.activeTab"
          class="flex h-12 shrink-0 items-center justify-between border-b border-border bg-card px-4"
        >
          <div class="flex items-center gap-2">
            <Dropdown>
              <template #trigger>
                <Button variant="primary">
                  <Plus :size="14" /> 添加项目 <ChevronDown :size="14" />
                </Button>
              </template>
              <DropdownItem :icon="Plus" @click="showAddModal = true">
                单个添加
              </DropdownItem>
              <DropdownItem :icon="FolderPlus" @click="showBatchImportModal = true">
                批量导入
              </DropdownItem>
            </Dropdown>
            <Button variant="ghost" size="icon" title="设置" @click="showSettings = true">
              <Settings :size="16" />
            </Button>
          </div>
          <span class="text-xs text-muted-foreground">{{ projects.length }} 个项目</span>
        </header>
        <header
          v-else
          class="flex h-10 shrink-0 items-center justify-between border-b border-border bg-card px-3"
        >
          <div class="flex items-center gap-2 text-xs text-muted-foreground">
            <span>{{ projects.length }} 个项目</span>
            <span v-if="tabStore.tabs.length">· {{ tabStore.tabs.length }} 个标签已打开</span>
          </div>
          <div class="flex items-center gap-1">
            <Button variant="ghost" size="sm" title="设置" @click="showSettings = true">
              <Settings :size="14" />
            </Button>
            <Button variant="ghost" size="sm" @click="showAddModal = true">
              <Plus :size="14" /> 添加
            </Button>
          </div>
        </header>

        <main class="min-h-0 flex-1 overflow-hidden">
          <!-- 活动项目标签：嵌入工作区 / 变更 / 历史 -->
          <ProjectTabView
            v-if="tabStore.activeTab"
            :key="tabStore.activeTab.projectId"
            :project-id="tabStore.activeTab.projectId"
          />
          <RouterView v-else />
        </main>
      </div>
    </div>

    <OperationQueue />

    <AddProjectModal v-model="showAddModal" />
    <BatchImportModal v-model="showBatchImportModal" />
    <SettingsModal v-model="showSettings" />
    <ProjectPickerModal v-model="showProjectPicker" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, defineAsyncComponent } from 'vue';
import { Plus, Settings, ChevronDown, FolderPlus, PanelLeftOpen, PanelLeftClose } from 'lucide-vue-next';
import { useAppStore } from '../stores/appStore';
import { useProjectStatus } from '../composables/useProjectStatus';
import { useTabStore } from '../stores/tabStore';
import GroupTree from '../components/Sidebar/GroupTree.vue';
import ProjectTabBar from '../components/Tabs/ProjectTabBar.vue';
import OperationQueue from '../components/OperationPanel/OperationQueue.vue';
import Button from '../components/ui/Button.vue';
import Dropdown from '../components/ui/Dropdown.vue';
import DropdownItem from '../components/ui/DropdownItem.vue';

const ProjectTabView = defineAsyncComponent(
  () => import('./ProjectTabView.vue'),
);

const AddProjectModal = defineAsyncComponent(
  () => import('../components/Modals/AddProjectModal.vue'),
);
const BatchImportModal = defineAsyncComponent(
  () => import('../components/Modals/BatchImportModal.vue'),
);
const SettingsModal = defineAsyncComponent(
  () => import('../components/Modals/SettingsModal.vue'),
);
const ProjectPickerModal = defineAsyncComponent(
  () => import('../components/Modals/ProjectPickerModal.vue'),
);

const appStore = useAppStore();
const tabStore = useTabStore();
const { getStatus, startPolling, stopPolling, watchRepoChanges } = useProjectStatus();

const showAddModal = ref(false);
const showBatchImportModal = ref(false);
const showSettings = ref(false);
const showProjectPicker = ref(false);

const projects = computed(() => appStore.projects);

const SIDEBAR_WIDTH_KEY = 'gitdash:sidebar-width';
const SIDEBAR_COLLAPSED_KEY = 'gitdash:sidebar-collapsed';
const MIN_WIDTH = 160;
const MAX_WIDTH = 420;
const DEFAULT_WIDTH = 220;

const sidebarCollapsed = ref(false);
const isResizing = ref(false);
const sidebarRef = ref<HTMLElement | null>(null);

function setSidebarWidth(width: number) {
  document.documentElement.style.setProperty('--sidebar-width', `${width}px`);
}

onMounted(async () => {
  await appStore.loadConfig();

  // 配置加载后：清理已删除项目的标签，并确保打开的标签状态可用
  tabStore.removeProjectTabs(
    appStore.projects.length === 0
      ? tabStore.openProjectIds.slice()
      : tabStore.openProjectIds.filter((id) => !appStore.projects.some((p) => p.id === id)),
  );

  // 实时状态：监听后端 repo:changed + 低频轮询兜底
  await watchRepoChanges();
  startPolling(60_000);
  appStore.projects.forEach((project) => {
    void getStatus(project.id, false);
  });

  let width = DEFAULT_WIDTH;
  const savedWidth = localStorage.getItem(SIDEBAR_WIDTH_KEY);
  if (savedWidth) {
    const parsed = parseInt(savedWidth, 10);
    if (!isNaN(parsed)) {
      width = Math.max(MIN_WIDTH, Math.min(MAX_WIDTH, parsed));
    }
  }
  setSidebarWidth(width);

  const savedCollapsed = localStorage.getItem(SIDEBAR_COLLAPSED_KEY);
  if (savedCollapsed) {
    sidebarCollapsed.value = savedCollapsed === 'true';
  }
});

onUnmounted(() => {
  stopPolling();
});

function toggleSidebar() {
  sidebarCollapsed.value = !sidebarCollapsed.value;
  localStorage.setItem(SIDEBAR_COLLAPSED_KEY, String(sidebarCollapsed.value));
}

function startResize(e: PointerEvent) {
  if (sidebarCollapsed.value) return;

  const target = e.currentTarget as HTMLElement;
  target.setPointerCapture(e.pointerId);

  const rootStyle = getComputedStyle(document.documentElement);
  const currentWidth = parseInt(rootStyle.getPropertyValue('--sidebar-width') || `${DEFAULT_WIDTH}`, 10);
  const startX = e.clientX;
  const startWidth = Number.isFinite(currentWidth) ? currentWidth : DEFAULT_WIDTH;
  let rafId: number | null = null;
  let latestWidth = startWidth;

  isResizing.value = true;

  // 拖拽期间禁止文本选中，避免拖动时选中页面文字
  document.body.classList.add('select-none');
  document.body.style.userSelect = 'none';

  function onPointerMove(ev: PointerEvent) {
    const delta = ev.clientX - startX;
    const newWidth = Math.max(MIN_WIDTH, Math.min(MAX_WIDTH, startWidth + delta));
    latestWidth = newWidth;

    if (rafId === null) {
      rafId = requestAnimationFrame(() => {
        rafId = null;
        setSidebarWidth(latestWidth);
      });
    }
  }

  function onPointerUp(ev: PointerEvent) {
    if (rafId !== null) {
      cancelAnimationFrame(rafId);
      rafId = null;
    }

    target.releasePointerCapture(ev.pointerId);
    window.removeEventListener('pointermove', onPointerMove);
    window.removeEventListener('pointerup', onPointerUp);

    isResizing.value = false;
    setSidebarWidth(latestWidth);
    document.body.classList.remove('select-none');
    document.body.style.userSelect = '';
    localStorage.setItem(SIDEBAR_WIDTH_KEY, String(latestWidth));
  }

  window.addEventListener('pointermove', onPointerMove);
  window.addEventListener('pointerup', onPointerUp);
}
</script>

<style scoped>
aside {
  width: var(--sidebar-width, 220px);
  transition: width 0.2s ease-out;
}
aside.resizing {
  transition: none;
}
aside.collapsed {
  width: 44px;
}
.resizer {
  position: absolute;
  right: 0;
  top: 0;
  bottom: 0;
  width: 4px;
  cursor: col-resize;
  background-color: transparent;
  transition: background-color 0.15s ease;
  z-index: 10;
}
.resizer:hover,
.resizer:active {
  background-color: var(--primary);
}
</style>
