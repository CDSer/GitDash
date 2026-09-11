<!--
  全局顶栏：横跨侧栏 + 内容区的最顶部，与 macOS 红绿灯同一排
  - 中间 .toolbar-drag 作为窗口拖拽区（不整条 header 拖拽，避免抢走按钮点击）
  - macOS：测量按钮中心 → 后端移动红绿灯 → 用 reserved_inset 设 paddingLeft
  - 非 macOS：右侧自绘窗口控制按钮（mousedown 直接触发，Windows 无边框更可靠）
-->
<template>
  <header
    ref="toolbarEl"
    class="global-toolbar"
    :style="toolbarStyle"
  >
    <div class="toolbar-left">
      <button type="button" class="toolbar-btn" title="添加项目" @click="$emit('add-project')">
        <Plus :size="15" />
        <span ref="firstActionLabelEl">添加项目</span>
      </button>
      <button type="button" class="toolbar-btn" title="批量导入" @click="$emit('batch-import')">
        <FolderPlus :size="15" />
        <span>批量导入</span>
      </button>
    </div>

    <!-- 拖拽区：空白可拖窗口；左右控件不受影响 -->
    <div class="toolbar-drag" data-tauri-drag-region />

    <div class="toolbar-right">
      <span v-if="projectCount !== undefined && projectCount > 0" class="toolbar-meta">
        {{ projectCount }} 个项目
      </span>
      <button type="button" class="toolbar-icon-btn" title="设置" @click="$emit('settings')">
        <Settings :size="16" />
      </button>

      <!-- 非 macOS：自绘窗口控制按钮（Windows 已关闭系统标题栏） -->
      <template v-if="showControls">
        <span class="toolbar-divider" />
        <div class="window-controls">
          <button
            type="button"
            class="win-btn"
            title="最小化"
            @pointerdown.stop.prevent="onWinMinimize"
            @click.stop.prevent="onWinMinimize"
          >
            <Minus :size="12" :stroke-width="2" />
          </button>
          <button
            type="button"
            class="win-btn"
            :title="isMaximized ? '还原' : '最大化'"
            @pointerdown.stop.prevent="onWinMaximize"
            @click.stop.prevent="onWinMaximize"
          >
            <Copy v-if="isMaximized" :size="11" :stroke-width="2" />
            <Square v-else :size="11" :stroke-width="2" />
          </button>
          <button
            type="button"
            class="win-btn win-btn--close"
            title="关闭"
            @pointerdown.stop.prevent="onWinClose"
            @click.stop.prevent="onWinClose"
          >
            <X :size="13" :stroke-width="2" />
          </button>
        </div>
      </template>
    </div>
  </header>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { Plus, FolderPlus, Settings, Minus, Square, Copy, X } from 'lucide-vue-next';
import {
  macTrafficLightInsetPaddingForScale,
  shouldReserveMacTrafficLightInset,
  syncMacTrafficLightsToToolbar,
  useWindowControls,
} from '../../composables/useWindowControls';

const UI_SCALE = 1;

const {
  isMac,
  isDesktop,
  showControls,
  isFullscreen,
  isMaximized,
  minimize,
  toggleMaximize,
  close,
} = useWindowControls();

// mousedown 与 click 可能连续触发两次，做短防抖
let winActionLock = 0;
function runWinAction(fn: () => void | Promise<void>) {
  const now = performance.now();
  if (now - winActionLock < 200) return;
  winActionLock = now;
  void fn();
}
const onWinMinimize = () => runWinAction(minimize);
const onWinMaximize = () => runWinAction(toggleMaximize);
const onWinClose = () => runWinAction(close);

defineProps<{ projectCount?: number }>();
defineEmits<{
  (e: 'add-project'): void;
  (e: 'batch-import'): void;
  (e: 'settings'): void;
}>();

const toolbarEl = ref<HTMLElement | null>(null);
const firstActionLabelEl = ref<HTMLElement | null>(null);
const measuredTrafficLightInset = ref<number | null>(null);

const shouldReserveTrafficLightInset = computed(() =>
  shouldReserveMacTrafficLightInset(isMac, isFullscreen.value, isDesktop),
);

const toolbarStyle = computed(() => {
  if (!shouldReserveTrafficLightInset.value) return undefined;
  return {
    paddingLeft: `${
      measuredTrafficLightInset.value ??
      parseInt(macTrafficLightInsetPaddingForScale(UI_SCALE), 10)
    }px`,
  };
});

let trafficLightSyncRaf = 0;
let resizeObserver: ResizeObserver | null = null;

function scheduleTrafficLightSync() {
  if (!shouldReserveTrafficLightInset.value) return;
  if (trafficLightSyncRaf) cancelAnimationFrame(trafficLightSyncRaf);
  trafficLightSyncRaf = requestAnimationFrame(() => {
    trafficLightSyncRaf = 0;
    void syncTrafficLightsToToolbar();
  });
}

async function syncTrafficLightsToToolbar() {
  if (!shouldReserveTrafficLightInset.value) return;
  const inset = await syncMacTrafficLightsToToolbar(
    toolbarEl.value,
    firstActionLabelEl.value,
    UI_SCALE,
  );
  if (inset != null) {
    measuredTrafficLightInset.value = inset;
  }
}

function handleWindowResize() {
  scheduleTrafficLightSync();
}

watch(shouldReserveTrafficLightInset, (should) => {
  if (!should) {
    measuredTrafficLightInset.value = null;
  } else {
    scheduleTrafficLightSync();
  }
});

onMounted(() => {
  resizeObserver = new ResizeObserver(() => scheduleTrafficLightSync());
  if (toolbarEl.value) resizeObserver.observe(toolbarEl.value);
  if (firstActionLabelEl.value) resizeObserver.observe(firstActionLabelEl.value);
  window.addEventListener('resize', handleWindowResize);
  scheduleTrafficLightSync();
  window.setTimeout(scheduleTrafficLightSync, 120);
});

onBeforeUnmount(() => {
  if (trafficLightSyncRaf) cancelAnimationFrame(trafficLightSyncRaf);
  resizeObserver?.disconnect();
  window.removeEventListener('resize', handleWindowResize);
});
</script>

<style scoped>
.global-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 40px;
  flex-shrink: 0;
  border-bottom: 1px solid var(--border);
  background: var(--card);
  user-select: none;
  /* 非 macOS / 全屏默认边距；macOS 非全屏时由 toolbarStyle 覆盖为实测 inset */
  padding: 0 8px;
}

.toolbar-left,
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.toolbar-drag {
  flex: 1;
  align-self: stretch;
  min-width: 24px;
  cursor: default;
}

.toolbar-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 32px;
  padding: 0 10px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--foreground);
  font-size: 13px;
  cursor: pointer;
  transition: background-color 0.12s;
}
.toolbar-btn:hover {
  background: var(--accent);
  color: var(--accent-foreground);
}

.toolbar-icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--muted-foreground);
  cursor: pointer;
  transition: background-color 0.12s, color 0.12s;
}
.toolbar-icon-btn:hover {
  background: var(--accent);
  color: var(--accent-foreground);
}

.toolbar-meta {
  margin-right: 8px;
  font-size: 12px;
  color: var(--muted-foreground);
}

.toolbar-divider {
  width: 1px;
  height: 20px;
  margin: 0 6px;
  background: var(--border);
  flex-shrink: 0;
}

.window-controls {
  display: flex;
  align-items: center;
  gap: 2px;
  padding-right: 0;
}

.win-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--foreground);
  cursor: default;
  transition: background-color 0.12s;
}
.win-btn:hover {
  background: var(--accent);
}
.win-btn--close:hover {
  background: #e81123;
  color: #fff;
}
</style>
