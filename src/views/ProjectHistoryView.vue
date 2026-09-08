<!--
  Git 记录页面（由原 GitHistoryModal 弹窗改版而来）
  左侧分支列表 / 中间提交列表 / 右侧提交详情
  点击左侧项目行或项目列表「Git 记录」进入本页
-->
<template>
  <div class="flex h-full flex-col overflow-hidden">
    <header
      class="flex h-12 shrink-0 items-center justify-between border-b border-border bg-card px-4"
    >
      <div class="flex min-w-0 items-center gap-2">
        <Button variant="ghost" size="sm" @click="goBack">
          <ArrowLeft :size="14" /> 返回
        </Button>
        <Divider direction="vertical" />
        <GitCommitHorizontal :size="15" class="text-muted-foreground" />
        <span class="font-semibold">{{ project?.name ?? 'Git 记录' }}</span>
        <span v-if="project" class="truncate text-xs text-muted-foreground">
          {{ project.path }}
        </span>
      </div>
      <div class="flex items-center gap-2">
        <Button variant="ghost" size="sm" title="刷新" @click="reload">
          <RefreshCw :size="14" />
        </Button>
      </div>
    </header>

    <div v-if="project" class="flex min-h-0 flex-1">
      <!-- 左侧分支 -->
      <aside class="history-aside">
        <div class="aside-title">分支</div>
        <div class="branch-list">
          <template v-if="localBranches.length">
            <div class="branch-group-title">本地</div>
            <button
              v-for="branch in localBranches"
              :key="branch.name"
              type="button"
              :class="['branch-item', selectedBranch === branch.display_name ? 'branch-item--active' : '']"
              @click="selectBranch(branch.display_name)"
            >
              <span class="branch-name" :title="branch.display_name">{{ branch.display_name }}</span>
              <Tag v-if="branch.is_current" variant="success">当前</Tag>
            </button>
          </template>
          <template v-if="remoteBranches.length">
            <div class="branch-group-title">远程</div>
            <button
              v-for="branch in remoteBranches"
              :key="branch.name"
              type="button"
              :class="['branch-item', selectedBranch === branch.display_name ? 'branch-item--active' : '']"
              @click="selectBranch(branch.display_name)"
            >
              <span class="branch-name" :title="branch.display_name">{{ branch.display_name }}</span>
            </button>
          </template>
          <Empty v-if="!branches.length" description="暂无分支" />
        </div>
        <div
          class="history-resizer history-resizer--right"
          title="拖动调整宽度"
          @pointerdown="startBranchResize"
        />
      </aside>

      <!-- 中间提交列表 -->
      <main class="history-main">
        <div v-if="loadingCommits" class="flex h-full items-center justify-center">
          <Spinner size="lg" />
        </div>
        <template v-else>
          <div class="commit-scroll">
            <div class="commit-table">
              <div class="commit-head commit-cols">
                <div>ID</div>
                <div>提交信息</div>
                <div>作者</div>
                <div>时间</div>
              </div>
              <div class="commit-list">
                <div
                  v-for="commit in commits"
                  :key="commit.id"
                  :class="['commit-row commit-cols', selectedCommit?.id === commit.id ? 'commit-row--active' : '']"
                  @click="handleCommitChange(commit)"
                >
                  <div class="commit-id font-mono">{{ commit.short_id }}</div>
                  <div class="commit-msg" :title="commit.message">{{ firstLine(commit.message) }}</div>
                  <div class="commit-author truncate">{{ commit.author }}</div>
                  <div class="commit-date">{{ formatDate(commit.date) }}</div>
                </div>
                <Empty v-if="!commits.length" description="暂无提交记录" />
              </div>
            </div>
          </div>
        </template>
      </main>

      <!-- 右侧详情 -->
      <aside class="history-aside detail-aside">
        <div class="aside-title">提交详情</div>
        <div v-if="detail" class="detail-scroll">
          <div class="detail-section">
            <div class="detail-label">提交信息</div>
            <div class="detail-message">{{ detail.message }}</div>
          </div>
          <div v-if="detail.body" class="detail-section">
            <div class="detail-label">详细说明</div>
            <pre class="detail-body">{{ detail.body }}</pre>
          </div>
          <div class="detail-section">
            <div class="detail-label">作者</div>
            <div class="detail-value">{{ detail.author }} &lt;{{ detail.email }}&gt;</div>
          </div>
          <div class="detail-section">
            <div class="detail-label">时间</div>
            <div class="detail-value">{{ formatDate(detail.date) }}</div>
          </div>
          <div class="detail-section">
            <div class="detail-label">Commit</div>
            <div class="detail-hash" :title="detail.id">{{ detail.id }}</div>
            <Button v-if="remoteUrl" size="sm" variant="ghost" class="detail-remote" @click="openRemoteCommit">
              <ExternalLink :size="12" /> 在远程查看
            </Button>
          </div>
          <div v-if="detail.parents.length" class="detail-section">
            <div class="detail-label">父提交</div>
            <div class="detail-parents">
              <div v-for="p in detail.parents" :key="p" class="detail-hash" :title="p">{{ p }}</div>
            </div>
          </div>
          <div class="detail-section">
            <div class="detail-label">改动文件</div>
            <div v-if="!detail.files.length" class="detail-empty">无文件改动</div>
            <div
              v-for="f in detail.files"
              :key="f.path"
              class="detail-file detail-file--click"
              :title="f.original_path && f.original_path !== f.path ? f.original_path + ' → ' + f.path : f.path"
              @click="openFileDiff(f)"
            >
              <Tag :variant="statusVariant(f.status)" class="file-status">{{ f.status }}</Tag>
              <span class="file-path" :title="f.path">{{
                f.original_path && f.original_path !== f.path ? f.original_path + ' → ' + f.path : f.path
              }}</span>
              <span v-if="f.added || f.removed" class="file-count">+{{ f.added }} -{{ f.removed }}</span>
            </div>
          </div>
        </div>
        <Empty v-else description="选择一条提交查看详情" />
        <div
          class="history-resizer history-resizer--left"
          title="拖动调整宽度"
          @pointerdown="startDetailResize"
        />
      </aside>
    </div>
    <div v-else class="flex min-h-0 flex-1 items-center justify-center">
      <Empty description="项目不存在或已被移除" />
    </div>

    <Dialog v-model="fileDiffVisible" :title="`文件改动 · ${fileDiffTitle}`" width="720px">
      <div class="file-diff-box">
        <DiffViewer :patch="fileDiffPatch" :is-binary="fileDiffBinary" empty-text="该文件无文本差异" />
      </div>
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { open } from '@tauri-apps/plugin-shell';
import { ArrowLeft, ExternalLink, GitCommitHorizontal, RefreshCw } from 'lucide-vue-next';
import type { Branch, Commit, CommitDetail, CommitFile } from '../types';
import {
  getBranches,
  getCommits,
  getCommitDetail,
  gitRemoteUrl,
  gitCommitFileDiff,
} from '../lib/tauriApi';
import { useAppStore } from '../stores/appStore';
import Dialog from '../components/ui/Dialog.vue';
import Tag from '../components/ui/Tag.vue';
import Button from '../components/ui/Button.vue';
import Spinner from '../components/ui/Spinner.vue';
import Empty from '../components/ui/Empty.vue';
import Divider from '../components/ui/Divider.vue';
import DiffViewer from '../components/Git/DiffViewer.vue';
import { toast } from '../lib/toast';

const props = defineProps<{ projectId: string }>();

const appStore = useAppStore();
const router = useRouter();

const project = computed(() => appStore.projects.find((p) => p.id === props.projectId) ?? null);

const branches = ref<Branch[]>([]);
const selectedBranch = ref('');
const commits = ref<Commit[]>([]);
const selectedCommit = ref<Commit | null>(null);
const detail = ref<CommitDetail | null>(null);
const loadingCommits = ref(false);
const remoteUrl = ref<string | null>(null);
const fileDiffVisible = ref(false);
const fileDiffPatch = ref('');
const fileDiffBinary = ref(false);
const fileDiffTitle = ref('');

const localBranches = computed(() => branches.value.filter((b) => b.is_local));
const remoteBranches = computed(() => branches.value.filter((b) => b.is_remote));

// 观察项目实体：路由切换或 store 加载完成后（项目从无到有）都会重新加载
watch(
  () => project.value,
  async (p) => {
    branches.value = [];
    commits.value = [];
    selectedBranch.value = '';
    selectedCommit.value = null;
    detail.value = null;
    remoteUrl.value = null;
    if (!p) return;
    // 清空分支后 loadBranches 会重新赋值，确保触发提交列表加载
    await Promise.all([loadBranches(), loadRemoteUrl()]);
  },
  { immediate: true },
);

// ===== 面板宽度拖拽调整（与分组侧边栏一致的交互） =====
const BRANCH_WIDTH_KEY = 'gitdash:history-branch-width';
const DETAIL_WIDTH_KEY = 'gitdash:history-detail-width';
const MIN_PANEL_WIDTH = 40;
// 上限不再限制面板拖宽：实际受窗口宽度约束，中栏表格可横向滚动不会被挤坏
const MAX_PANEL_WIDTH = 2000;
const BRANCH_DEFAULT = 200;
const DETAIL_DEFAULT = 260;

function restorePanelWidth(cssVar: string, storageKey: string, defaultWidth: number) {
  let width = defaultWidth;
  const saved = localStorage.getItem(storageKey);
  if (saved) {
    const parsed = parseInt(saved, 10);
    if (!isNaN(parsed)) {
      width = Math.max(MIN_PANEL_WIDTH, Math.min(MAX_PANEL_WIDTH, parsed));
    }
  }
  document.documentElement.style.setProperty(cssVar, `${width}px`);
}

onMounted(() => {
  restorePanelWidth('--history-branch-width', BRANCH_WIDTH_KEY, BRANCH_DEFAULT);
  restorePanelWidth('--history-detail-width', DETAIL_WIDTH_KEY, DETAIL_DEFAULT);
});

// dir=1：向右拖变宽；dir=-1：向右拖变窄
function beginResize(cssVar: string, storageKey: string, dir: 1 | -1, defaultWidth: number) {
  return (e: PointerEvent) => {
    const target = e.currentTarget as HTMLElement;
    target.setPointerCapture(e.pointerId);

    const current = parseFloat(getComputedStyle(document.documentElement).getPropertyValue(cssVar));
    const startWidth = Number.isFinite(current) && current > 0 ? current : defaultWidth;
    const startX = e.clientX;
    let rafId: number | null = null;
    let latestWidth = startWidth;

    // 拖拽期间禁止文本选中，避免拖动时选中页面文字
    document.body.classList.add('select-none');
    document.body.style.userSelect = 'none';

    function onMove(ev: PointerEvent) {
      const delta = (ev.clientX - startX) * dir;
      const newWidth = Math.max(MIN_PANEL_WIDTH, Math.min(MAX_PANEL_WIDTH, startWidth + delta));
      latestWidth = newWidth;
      if (rafId === null) {
        rafId = requestAnimationFrame(() => {
          rafId = null;
          document.documentElement.style.setProperty(cssVar, `${latestWidth}px`);
        });
      }
    }

    function onUp(ev: PointerEvent) {
      if (rafId !== null) {
        cancelAnimationFrame(rafId);
        rafId = null;
      }
      target.releasePointerCapture(ev.pointerId);
      window.removeEventListener('pointermove', onMove);
      window.removeEventListener('pointerup', onUp);
      document.documentElement.style.setProperty(cssVar, `${latestWidth}px`);
      document.body.classList.remove('select-none');
      document.body.style.userSelect = '';
      localStorage.setItem(storageKey, String(latestWidth));
    }

    window.addEventListener('pointermove', onMove);
    window.addEventListener('pointerup', onUp);
  };
}

const startBranchResize = beginResize('--history-branch-width', BRANCH_WIDTH_KEY, 1, BRANCH_DEFAULT);
const startDetailResize = beginResize('--history-detail-width', DETAIL_WIDTH_KEY, -1, DETAIL_DEFAULT);

function goBack() {
  if (window.history.length > 1) {
    router.back();
  } else {
    router.push({ name: 'projects' });
  }
}

async function reload() {
  if (!project.value) return;
  if (selectedBranch.value) {
    await loadCommits(selectedBranch.value);
  } else {
    await loadBranches();
  }
  await loadRemoteUrl();
}

async function loadRemoteUrl() {
  if (!project.value) return;
  try {
    remoteUrl.value = await gitRemoteUrl(project.value.id);
  } catch {
    remoteUrl.value = null;
  }
}

function openRemoteCommit() {
  if (remoteUrl.value && detail.value) {
    open(`${remoteUrl.value.replace(/\.git$/, '')}/commit/${detail.value.id}`);
  }
}

async function openFileDiff(f: CommitFile) {
  if (!project.value || !detail.value) return;
  fileDiffTitle.value = f.original_path && f.original_path !== f.path
    ? `${f.original_path} → ${f.path}`
    : f.path;
  try {
    const res = await gitCommitFileDiff(
      project.value.id,
      detail.value.id,
      f.path,
      f.original_path ?? undefined,
    );
    fileDiffBinary.value = res.is_binary;
    fileDiffPatch.value = res.fallback_patch || res.modified_content || '';
    fileDiffVisible.value = true;
  } catch (e) {
    console.error('加载文件 diff 失败：', e);
    toast.error('加载文件 diff 失败');
  }
}

watch(selectedBranch, async (branch) => {
  if (!branch) return;
  selectedCommit.value = null;
  detail.value = null;
  await loadCommits(branch);
});

async function loadBranches() {
  if (!project.value) return;
  try {
    branches.value = await getBranches(project.value.id);
    const current = branches.value.find((b) => b.is_current);
    selectedBranch.value = current?.display_name || branches.value[0]?.display_name || '';
  } catch (error) {
    console.error('加载分支失败：', error);
    toast.error('加载分支失败');
  }
}

async function loadCommits(branch: string) {
  if (!project.value) return;
  loadingCommits.value = true;
  try {
    commits.value = await getCommits(project.value.id, branch, 100);
  } catch (error) {
    console.error('加载提交记录失败：', error);
    toast.error('加载提交记录失败');
  } finally {
    loadingCommits.value = false;
  }
}

function selectBranch(branch: string) {
  selectedBranch.value = branch;
}

function handleCommitChange(commit: Commit) {
  selectedCommit.value = commit;
  loadCommitDetail(commit.id);
}

async function loadCommitDetail(commitId: string) {
  if (!project.value) return;
  try {
    detail.value = await getCommitDetail(project.value.id, commitId);
  } catch (error) {
    console.error('加载提交详情失败：', error);
    toast.error('加载提交详情失败');
  }
}

function firstLine(msg: string): string {
  return msg.split('\n')[0] || '(无说明)';
}

function pad(n: number): string {
  return n < 10 ? '0' + n : '' + n;
}

function formatDate(ts: number) {
  // 后端返回 unix 秒
  const d = new Date(ts * 1000);
  return `${d.getFullYear()}/${pad(d.getMonth() + 1)}/${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

function statusVariant(status: string): 'success' | 'danger' | 'warning' | 'info' {
  switch (status.charAt(0)) {
    case 'A':
      return 'success';
    case 'D':
      return 'danger';
    case 'M':
      return 'warning';
    default:
      return 'info';
  }
}
</script>

<style scoped>
.history-aside {
  position: relative;
  display: flex;
  flex-direction: column;
  width: var(--history-branch-width, 200px);
  flex-shrink: 0;
  background-color: var(--muted);
  border-right: 1px solid var(--border);
}
.detail-aside {
  border-right: none;
  border-left: 1px solid var(--border);
  width: var(--history-detail-width, 260px);
}
.history-resizer {
  position: absolute;
  top: 0;
  bottom: 0;
  width: 4px;
  cursor: col-resize;
  background-color: transparent;
  transition: background-color 0.15s ease;
  z-index: 10;
}
.history-resizer:hover,
.history-resizer:active {
  background-color: var(--primary);
}
.history-resizer--right {
  right: 0;
}
.history-resizer--left {
  left: 0;
}
.aside-title {
  padding: 10px 12px;
  font-size: 13px;
  font-weight: 600;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}
.branch-list {
  flex: 1;
  overflow-y: auto;
  padding: 4px;
}
.branch-group-title {
  padding: 6px 8px 2px;
  font-size: 11px;
  color: var(--muted-foreground);
}
.branch-item {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  padding: 6px 8px;
  font-size: 13px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--foreground);
  text-align: left;
  cursor: pointer;
}
.branch-item:hover {
  background-color: var(--accent);
}
.branch-item--active {
  background-color: color-mix(in oklab, var(--primary) 18%, transparent);
  color: var(--primary);
}
.branch-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.history-main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}
.commit-scroll {
  flex: 1;
  min-height: 0;
  overflow-x: auto;
  display: flex;
  flex-direction: column;
}
.commit-table {
  flex: 1;
  min-height: 0;
  min-width: 640px;
  display: flex;
  flex-direction: column;
}
.commit-cols {
  grid-template-columns: 80px minmax(240px, 1fr) 110px 150px;
}
.commit-head {
  display: grid;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  font-size: 12px;
  font-weight: 600;
  color: var(--muted-foreground);
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
  width: 100%;
}
.commit-list {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}
.commit-row {
  display: grid;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  font-size: 13px;
  border-bottom: 1px solid var(--border);
  cursor: pointer;
  width: 100%;
}
.commit-row:hover {
  background-color: var(--accent);
}
.commit-row--active {
  background-color: color-mix(in oklab, var(--primary) 14%, transparent);
}
.commit-id {
  color: var(--muted-foreground);
}
.commit-msg {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.commit-author {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--muted-foreground);
}
.commit-date {
  color: var(--muted-foreground);
  font-variant-numeric: tabular-nums;
}
.detail-scroll {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
  padding: 12px;
}
.detail-section {
  margin-bottom: 16px;
}
.detail-section:last-child {
  margin-bottom: 0;
}
.detail-label {
  font-size: 12px;
  color: var(--muted-foreground);
  margin-bottom: 6px;
}
.detail-message {
  font-size: 13px;
  font-weight: 600;
  word-break: break-word;
}
.detail-value {
  font-size: 13px;
  word-break: break-word;
}
.detail-hash {
  font-size: 12px;
  font-family: var(--font-mono, monospace);
  word-break: break-all;
}
.detail-body {
  font-size: 12px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-word;
  margin: 0;
  font-family: inherit;
}
.detail-empty {
  font-size: 12px;
  color: var(--muted-foreground);
  padding: 8px 0;
}
.detail-file {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 0;
  font-size: 12px;
}
.file-status {
  flex-shrink: 0;
  min-width: 36px;
  justify-content: center;
}
.file-path {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.detail-file--click {
  cursor: pointer;
  border-radius: 4px;
  padding: 4px 4px;
}
.detail-file--click:hover {
  background-color: var(--accent);
}
.file-count {
  flex-shrink: 0;
  font-size: 11px;
  font-variant-numeric: tabular-nums;
  color: var(--muted-foreground);
}
.detail-remote {
  margin-top: 6px;
}
.file-diff-box {
  height: 520px;
}
</style>
