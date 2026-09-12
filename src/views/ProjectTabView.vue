<!--
  单项目标签内容壳（Fork 风格）
  顶栏：项目名 + 模式切换（工作区 / 变更 / 历史）+ 快捷操作
  主体：按模式嵌入 Workspace / SourceControl / History
-->
<template>
  <div v-if="project" class="project-tab flex h-full min-h-0 flex-col overflow-hidden">
    <header class="tab-header">
      <div class="tab-header-left">
        <GitBranch :size="15" class="shrink-0 text-muted-foreground" />
        <span class="tab-project-name" :title="project.path">{{ project.name }}</span>
        <span v-if="status?.is_detached" class="tab-path">分离 HEAD</span>
        <span v-else class="tab-path">{{ status?.branch || project.path }}</span>
        <span v-if="status && (status.ahead || status.behind)" class="tab-ab">
          <span v-if="status.ahead" class="text-[color:var(--sys-green)]">↑{{ status.ahead }}</span>
          <span v-if="status.behind" class="text-[color:var(--sys-orange)]">↓{{ status.behind }}</span>
        </span>
      </div>

      <div class="tab-modes" role="tablist" aria-label="视图模式">
        <button
          v-for="m in modes"
          :key="m.value"
          type="button"
          role="tab"
          class="mode-btn"
          :class="{ 'mode-btn--active': mode === m.value }"
          :aria-selected="mode === m.value"
          @click="setMode(m.value)"
        >
          <component :is="m.icon" :size="14" />
          {{ m.label }}
          <span v-if="m.value === 'changes' && changeBadge" class="mode-badge">
            {{ changeBadge }}
          </span>
        </button>
      </div>

      <div class="tab-header-right">
        <Button size="sm" variant="outline" title="刷新" @click="refreshMode">
          <RefreshCw :size="14" />
        </Button>
        <Button
          size="sm"
          variant="outline"
          :disabled="opBusy"
          title="获取远端更新（不合并）"
          @click="onFetch"
        >
          <CloudDownload :size="14" /> 获取
        </Button>
        <Button
          size="sm"
          variant="primary"
          :disabled="opBusy || !!status?.in_progress"
          title="拉取并合并当前分支"
          @click="onPull"
        >
          <Download :size="14" /> 拉取
        </Button>
        <Button
          size="sm"
          variant="outline"
          :disabled="opBusy || !!status?.in_progress || !canPush"
          :title="pushTitle"
          @click="onPush"
        >
          <Upload :size="14" /> 推送
        </Button>
        <Button
          size="sm"
          variant="outline"
          :disabled="opBusy || !!status?.in_progress"
          title="暂存全部并提交请在变更页操作"
          @click="setMode('changes')"
        >
          变更
        </Button>
      </div>
    </header>

    <div class="tab-body">
      <WorkspacePanel
        v-if="mode === 'workspace'"
        :key="`ws-${project.id}`"
        :project-id="project.id"
        embedded
      />
      <SourceControlPanel
        v-else-if="mode === 'changes'"
        :key="`sc-${project.id}`"
        :project-id="project.id"
      />
      <HistoryPanel
        v-else-if="mode === 'history'"
        :key="`hi-${project.id}-${historyRefreshKey}`"
        :project-id="project.id"
        embedded
      />
      <TerminalPanel
        v-else
        :key="`te-${project.id}`"
        :project-id="project.id"
      />
    </div>
  </div>
  <div v-else class="flex h-full items-center justify-center text-sm text-muted-foreground">
    项目不存在或已被移除
  </div>
</template>

<script setup lang="ts">
import { computed, defineAsyncComponent, ref } from 'vue';
import { GitBranch, RefreshCw, CloudDownload, Download, Upload, FolderTree, GitCommitHorizontal, GitMerge, SquareTerminal } from 'lucide-vue-next';
import type { ProjectTabMode } from '../stores/tabStore';
import { useAppStore } from '../stores/appStore';
import { useTabStore } from '../stores/tabStore';
import { useProjectStatus } from '../composables/useProjectStatus';
import { useOperationStore } from '../stores/operationStore';
import { toast } from '../lib/toast';
import Button from '../components/ui/Button.vue';

const WorkspacePanel = defineAsyncComponent(() => import('./WorkspaceView.vue'));
const HistoryPanel = defineAsyncComponent(() => import('./ProjectHistoryView.vue'));
const SourceControlPanel = defineAsyncComponent(
  () => import('../components/Modals/SourceControlPanel.vue'),
);
const TerminalPanel = defineAsyncComponent(
  () => import('../components/Terminal/TerminalView.vue'),
);

const props = defineProps<{ projectId: string }>();

const appStore = useAppStore();
const tabStore = useTabStore();
const operationStore = useOperationStore();
const { getStatus } = useProjectStatus();

const project = computed(
  () => appStore.projects.find((p) => p.id === props.projectId) ?? null,
);
const status = computed(() => appStore.statuses.get(props.projectId) ?? null);

const mode = computed<ProjectTabMode>(
  () => tabStore.tabs.find((t) => t.projectId === props.projectId)?.mode ?? 'history',
);

/** 推送成功后 bump，强制历史面板重新拉取（刷新未推送标记） */
const historyRefreshKey = ref(0);

const modes = [
  { value: 'changes' as const, label: '变更', icon: GitMerge },
  { value: 'history' as const, label: '历史', icon: GitCommitHorizontal },
  { value: 'workspace' as const, label: '工作区', icon: FolderTree },
  { value: 'terminal' as const, label: '终端', icon: SquareTerminal },
];

const changeBadge = computed(() => {
  const s = status.value;
  if (!s) return 0;
  return (s.conflict_count || 0) + (s.modified || 0) + (s.staged || 0) + (s.untracked || 0);
});

const opBusy = computed(() => operationStore.isQueueRunning);
const canPush = computed(() => (status.value?.ahead ?? 0) > 0);
const pushTitle = computed(() => {
  if (status.value?.in_progress) return '有进行中的合并/变基，无法推送';
  if (!canPush.value) return '没有需要推送的提交';
  return `推送 ${status.value?.ahead ?? 0} 个提交到远程`;
});

function setMode(m: ProjectTabMode) {
  tabStore.setMode(props.projectId, m);
}

async function refreshMode() {
  await getStatus(props.projectId, true);
  // 子面板通过 key 不变时靠 repo:changed；这里再触发一次即可
  toast.info('已刷新状态');
}

function latestTaskMessage(projectId: string): string | null {
  const list = operationStore.tasks.filter((t) => t.projectId === projectId);
  const last = list[list.length - 1];
  if (!last || last.status !== 'error') return null;
  return last.message || '操作失败';
}

async function onFetch() {
  if (opBusy.value) return;
  try {
    await operationStore.batchFetch([props.projectId]);
    await getStatus(props.projectId, true);
    const errMsg = latestTaskMessage(props.projectId);
    if (errMsg) {
      toast.error(`获取失败：${errMsg}`);
    } else {
      toast.success('获取完成');
    }
  } catch {
    toast.error('获取失败');
  }
}

async function onPull() {
  if (opBusy.value || status.value?.in_progress) return;
  try {
    await operationStore.batchPull([props.projectId]);
    await getStatus(props.projectId, true);
    historyRefreshKey.value += 1;
    const errMsg = latestTaskMessage(props.projectId);
    if (errMsg) {
      toast.error(`拉取失败：${errMsg}`);
    } else {
      toast.success('拉取完成');
    }
  } catch {
    toast.error('拉取失败');
  }
}

async function onPush() {
  if (opBusy.value || !canPush.value || status.value?.in_progress) return;
  try {
    await operationStore.batchPush([props.projectId]);
    await getStatus(props.projectId, true);
    const errMsg = latestTaskMessage(props.projectId);
    if (errMsg) {
      toast.error(`推送失败：${errMsg}`);
    } else {
      toast.success('推送完成');
      // 推送成功后刷新历史列表的 is_pushed 标记
      historyRefreshKey.value += 1;
    }
  } catch {
    toast.error('推送失败');
  }
}
</script>

<style scoped>
.project-tab {
  background-color: var(--background);
}
.tab-header {
  display: flex;
  align-items: center;
  gap: 12px;
  height: 48px;
  flex-shrink: 0;
  padding: 0 16px;
  box-shadow: var(--glass-specular), inset 0 -1px 0 0 var(--separator);
  background-color: var(--toolbar-bg);
  backdrop-filter: saturate(var(--glass-saturate)) blur(var(--glass-blur));
  -webkit-backdrop-filter: saturate(var(--glass-saturate)) blur(var(--glass-blur));
}
.tab-header-left {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  flex: 1;
}
.tab-project-name {
  font-size: 13px;
  font-weight: 600;
  letter-spacing: -0.015em;
  white-space: nowrap;
}
.tab-path {
  font-size: 11px;
  color: var(--muted-foreground);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.tab-ab {
  display: flex;
  gap: 4px;
  font-size: 11px;
  flex-shrink: 0;
}
.tab-modes {
  display: flex;
  align-items: center;
  gap: 2px;
  flex-shrink: 0;
  padding: 2px;
  border-radius: 8px;
  background-color: var(--muted);
}
.mode-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  height: 28px;
  padding: 0 10px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 500;
  color: var(--muted-foreground);
  cursor: default;
  transition: background-color 0.15s var(--ease-out), color 0.15s var(--ease-out);
}
.mode-btn:hover {
  color: var(--foreground);
}
.mode-btn--active {
  background-color: var(--card);
  color: var(--foreground);
  box-shadow: 0 0.5px 1.5px rgba(0, 0, 0, 0.08), 0 1px 2px rgba(0, 0, 0, 0.04);
}
.mode-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 16px;
  height: 16px;
  padding: 0 4px;
  border-radius: 999px;
  font-size: 10px;
  background-color: color-mix(in oklab, var(--primary) 18%, transparent);
  color: var(--primary);
}
.tab-header-right {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.tab-body {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  padding-bottom: 30px;
}
</style>
