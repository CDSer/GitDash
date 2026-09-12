<!--
  源码控制嵌入面板（项目标签「变更」模式）
  与 SourceControlModal 同源逻辑，无 Dialog 外壳，铺满标签内容区
-->
<template>
  <div v-if="project" class="sc-panel">
    <div class="sc-header">
      <div class="sc-branch">
        <GitBranch :size="14" />
        <span class="sc-branch-name" :title="status?.branch">{{ branchLabel }}</span>
        <Tag v-if="status?.is_detached" variant="warning">detached</Tag>
        <Tag v-if="conflictCount > 0" variant="danger">冲突 {{ conflictCount }}</Tag>
      </div>
      <div v-if="status && (status.ahead || status.behind)" class="sc-ahead-behind">
        <span v-if="status.ahead" class="text-emerald-600">↑{{ status.ahead }}</span>
        <span v-if="status.behind" class="text-amber-600">↓{{ status.behind }}</span>
      </div>
      <div class="sc-spacer" />
      <select
        v-if="localBranches.length"
        class="sc-select"
        :value="status?.branch"
        :disabled="busy || !!inProgress"
        @change="onCheckout"
      >
        <option v-for="b in localBranches" :key="b.name" :value="b.name">
          {{ b.display_name }}{{ b.is_current ? ' (当前)' : '' }}
        </option>
      </select>
      <Button
        v-if="localBranches.length"
        size="sm"
        variant="ghost"
        :disabled="busy || !!inProgress"
        @click="showMergeModal = true"
      >
        合并…
      </Button>
      <Button size="sm" variant="ghost" :disabled="busy" @click="refresh">
        <RefreshCw :size="14" /> 刷新
      </Button>
    </div>

    <div v-if="inProgress" class="sc-banner">
      <div class="sc-banner-text">
        <strong>{{ inProgressTitle }}</strong>
        <span v-if="inProgress.head_message" class="sc-banner-msg">{{ inProgress.head_message }}</span>
        <span v-if="conflictCount > 0" class="sc-banner-conflict">
          还有 {{ conflictCount }} 个冲突未解决
        </span>
        <span v-else class="sc-banner-ok">冲突已全部解决，可继续完成操作</span>
      </div>
      <div class="sc-banner-actions">
        <Button
          size="sm"
          variant="primary"
          :disabled="busy || conflictCount > 0"
          @click="continueOperation"
        >
          {{ inProgress.kind === 'merge' ? '完成合并' : '继续操作' }}
        </Button>
        <Button size="sm" variant="ghost" :disabled="busy" @click="abortOperation">
          中止
        </Button>
      </div>
    </div>

    <div v-if="status?.error" class="sc-error">{{ status.error }}</div>

    <div class="sc-body">
      <div class="sc-left">
        <template v-if="conflictFiles.length">
          <div class="sc-section">
            <span class="sc-section-title sc-section-title--conflict">
              冲突（{{ conflictFiles.length }}）
            </span>
          </div>
          <div class="sc-file-list sc-file-list--conflicts">
            <div
              v-for="f in conflictFiles"
              :key="'c-' + f.path"
              :class="[
                'sc-file',
                'sc-file--conflict',
                selectedFile?.path === f.path ? 'sc-file--active' : '',
              ]"
              @click="selectFile(f)"
            >
              <Tag variant="danger" class="sc-file-status">!</Tag>
              <span class="sc-file-path" :title="f.path">{{ pathLabel(f) }}</span>
              <div class="sc-file-actions">
                <Button size="sm" variant="ghost" :disabled="busy" @click.stop="resolveSide(f, 'ours')">
                  我方
                </Button>
                <Button size="sm" variant="ghost" :disabled="busy" @click.stop="resolveSide(f, 'theirs')">
                  对方
                </Button>
                <Button size="sm" variant="ghost" :disabled="busy" @click.stop="openInEditor(f)">
                  编辑
                </Button>
                <Button size="sm" variant="primary" :disabled="busy" @click.stop="markResolved(f)">
                  已解决
                </Button>
              </div>
            </div>
          </div>
        </template>

        <template v-if="stagedFiles.length">
          <div class="sc-section">
            <span class="sc-section-title">已暂存</span>
            <Button size="sm" variant="ghost" :disabled="busy || !stagedFiles.length" @click="unstageAll">
              全部取消暂存
            </Button>
          </div>
          <div class="sc-file-list">
            <div
              v-for="f in stagedFiles"
              :key="'s-' + f.path"
              :class="['sc-file', selectedFile?.path === f.path ? 'sc-file--active' : '']"
              @click="selectFile(f)"
            >
              <Tag :variant="statusVariant(f)" class="sc-file-status">{{ statusLabel(f) }}</Tag>
              <span class="sc-file-path" :title="f.path">{{ pathLabel(f) }}</span>
              <div class="sc-file-actions">
                <Button size="sm" variant="ghost" :disabled="busy" @click.stop="unstageFile(f)">
                  取消暂存
                </Button>
              </div>
            </div>
          </div>
        </template>

        <div class="sc-section">
          <span class="sc-section-title">更改</span>
          <Button size="sm" variant="ghost" :disabled="busy || !unstagedFiles.length" @click="stageAll">
            全部暂存
          </Button>
        </div>
        <div class="sc-file-list">
          <div
            v-for="f in unstagedFiles"
            :key="'u-' + f.path"
            :class="['sc-file', selectedFile?.path === f.path ? 'sc-file--active' : '']"
            @click="selectFile(f)"
          >
            <Tag :variant="statusVariant(f)" class="sc-file-status">{{ statusLabel(f) }}</Tag>
            <span class="sc-file-path" :title="f.path">{{ pathLabel(f) }}</span>
            <div class="sc-file-actions">
              <Button size="sm" variant="ghost" :disabled="busy" @click.stop="stageFile(f)">暂存</Button>
              <Button size="sm" variant="ghost" :disabled="busy" @click.stop="discardFile(f)">丢弃</Button>
            </div>
          </div>
          <Empty
            v-if="!stagedFiles.length && !unstagedFiles.length && !conflictFiles.length"
            description="工作区干净"
          />
        </div>

        <div class="sc-commit">
          <textarea
            v-model="commitMessage"
            class="sc-textarea"
            :placeholder="commitPlaceholder"
            rows="3"
            :disabled="!!inProgress && inProgress.kind !== 'merge'"
          />
          <Button
            v-if="inProgress && inProgress.kind === 'merge' && conflictCount === 0"
            variant="primary"
            size="sm"
            :disabled="busy"
            @click="continueOperation"
          >
            完成合并
          </Button>
          <Button
            v-else-if="inProgress && conflictCount > 0"
            variant="primary"
            size="sm"
            disabled
          >
            请先解决冲突
          </Button>
          <Button v-else variant="primary" size="sm" :disabled="busy || !canCommit" @click="commit">
            提交
          </Button>
        </div>
      </div>

      <div class="sc-right">
        <ConflictViewer
          v-if="selectedFile?.is_conflict"
          :content="conflictContent"
          :loading="conflictLoading"
        />
        <DiffViewer
          v-else
          :original="diffOriginal"
          :modified="diffModified"
          :patch="diffPatch"
          :is-binary="diffIsBinary"
          :loading="diffLoading"
          :empty-text="diffEmpty"
        />
      </div>
    </div>

    <MergeBranchModal
      v-model="showMergeModal"
      :project="project"
      :branches="localBranches"
      :current-branch="status?.branch ?? ''"
      @merged="refresh"
    />
  </div>
  <div v-else class="sc-panel-empty">项目不存在或已被移除</div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import type {
  Branch,
  ChangedFile,
  ConflictFileContent,
  ConflictSide,
  InProgressOp,
  ProjectStatus,
} from '../../types';
import {
  getBranches,
  getProjectStatusForce,
  gitAbortOperation,
  gitCheckoutBranch,
  gitCommit,
  gitConflictFileContent,
  gitDiffContent,
  gitDiscard,
  gitMarkConflictResolved,
  gitMergeContinue,
  gitRemoteUrl,
  gitResolveConflict,
  gitStage,
  gitUnstage,
  readFile,
} from '../../lib/tauriApi';
import { useAppStore } from '../../stores/appStore';
import { useTabStore } from '../../stores/tabStore';
import Button from '../ui/Button.vue';
import Tag from '../ui/Tag.vue';
import Empty from '../ui/Empty.vue';
import DiffViewer from '../Git/DiffViewer.vue';
import ConflictViewer from '../Git/ConflictViewer.vue';
import MergeBranchModal from './MergeBranchModal.vue';
import { GitBranch, RefreshCw } from 'lucide-vue-next';
import { toast } from '../../lib/toast';

const props = defineProps<{ projectId: string }>();

const appStore = useAppStore();
const tabStore = useTabStore();
const project = computed(
  () => appStore.projects.find((p) => p.id === props.projectId) ?? null,
);

const status = ref<ProjectStatus | null>(null);
const branches = ref<Branch[]>([]);
const remoteUrl = ref<string | null>(null);
const busy = ref(false);
const selectedFile = ref<ChangedFile | null>(null);
const commitMessage = ref('');
const showMergeModal = ref(false);

const diffOriginal = ref('');
const diffModified = ref('');
const diffPatch = ref('');
const diffIsBinary = ref(false);
const diffLoading = ref(false);
const diffEmpty = ref('选择左侧文件查看改动');

const conflictContent = ref<ConflictFileContent | null>(null);
const conflictLoading = ref(false);

const localBranches = computed(() => branches.value.filter((b) => b.is_local && !b.is_detached));
const conflictFiles = computed(() =>
  (status.value?.changed_files ?? []).filter((f) => f.is_conflict),
);
const stagedFiles = computed(() =>
  (status.value?.changed_files ?? []).filter((f) => f.staged && !f.is_conflict),
);
const unstagedFiles = computed(() =>
  (status.value?.changed_files ?? []).filter((f) => !f.staged && !f.is_conflict),
);
const conflictCount = computed(() => status.value?.conflict_count ?? 0);
const inProgress = computed<InProgressOp | null>(() => status.value?.in_progress ?? null);

const branchLabel = computed(() => {
  if (!status.value) return '-';
  if (status.value.is_detached) return '分离 HEAD';
  return status.value.branch || '-';
});

const inProgressTitle = computed(() => {
  const op = inProgress.value;
  if (!op) return '';
  switch (op.kind) {
    case 'merge':
      return '合并进行中';
    case 'rebase':
      return '变基进行中';
    case 'cherry-pick':
      return 'Cherry-pick 进行中';
    case 'revert':
      return 'Revert 进行中';
    default:
      return `${op.kind} 进行中`;
  }
});

const canCommit = computed(() => commitMessage.value.trim().length > 0 && !inProgress.value);

const commitPlaceholder = computed(() => {
  if (inProgress.value?.kind === 'merge' && conflictCount.value === 0) {
    return '可留空完成合并，或填写合并提交信息…';
  }
  if (inProgress.value) {
    return '请先中止或完成当前操作';
  }
  return '提交信息（第一行作为摘要）...';
});

watch(
  () => props.projectId,
  async () => {
    selectedFile.value = null;
    status.value = null;
    await refresh();
  },
  { immediate: true },
);

onMounted(() => {
  void refresh();
});

async function refresh() {
  if (!project.value) return;
  busy.value = true;
  try {
    const [st, brs, url] = await Promise.all([
      getProjectStatusForce(project.value.id),
      getBranches(project.value.id),
      gitRemoteUrl(project.value.id).catch(() => null),
    ]);
    status.value = st;
    branches.value = brs;
    remoteUrl.value = url;
    if (selectedFile.value) {
      const next = st.changed_files.find((f) => f.path === selectedFile.value!.path) ?? null;
      selectedFile.value = next;
      if (next) await loadDiff(next);
    }
  } catch (e) {
    console.error('加载源码控制失败：', e);
    toast.error('加载源码控制失败');
  } finally {
    busy.value = false;
  }
}

function statusLabel(f: ChangedFile): string {
  if (f.is_conflict) return '!';
  const ch = f.staged ? f.index_status : f.worktree_status;
  switch (ch) {
    case 'A':
      return 'A';
    case 'D':
      return 'D';
    case 'M':
      return 'M';
    case 'R':
      return 'R';
    case 'C':
      return 'C';
    case 'U':
      return 'U';
    case 'T':
      return 'T';
    case '?':
      return '?';
    default:
      return ch.trim() || '·';
  }
}

function statusVariant(f: ChangedFile): 'success' | 'danger' | 'warning' | 'info' {
  if (f.is_conflict) return 'danger';
  const ch = f.staged ? f.index_status : f.worktree_status;
  switch (ch) {
    case 'A':
      return 'success';
    case 'D':
    case 'U':
    case '?':
      return 'danger';
    case 'M':
      return 'warning';
    default:
      return 'info';
  }
}

function pathLabel(f: ChangedFile): string {
  if (f.original_path && f.original_path !== f.path) {
    return `${f.original_path} → ${f.path}`;
  }
  return f.path;
}

const isUntracked = (f: ChangedFile) => f.index_status === '?' || f.worktree_status === '?';

async function selectFile(f: ChangedFile) {
  selectedFile.value = f;
  await loadDiff(f);
}

async function loadDiff(f: ChangedFile) {
  if (!project.value) return;
  if (f.is_conflict) {
    conflictContent.value = null;
    conflictLoading.value = true;
    diffOriginal.value = '';
    diffModified.value = '';
    diffPatch.value = '';
    try {
      conflictContent.value = await gitConflictFileContent(project.value.id, f.path);
    } catch (e) {
      console.error('加载冲突内容失败：', e);
      toast.error('加载冲突内容失败');
    } finally {
      conflictLoading.value = false;
    }
    return;
  }

  conflictContent.value = null;
  diffIsBinary.value = false;
  diffEmpty.value = `${f.path} 无改动`;
  diffLoading.value = true;
  diffOriginal.value = '';
  diffModified.value = '';
  diffPatch.value = '';
  try {
    if (isUntracked(f)) {
      const content = await readFile([project.value.path, f.path].join('/'));
      diffOriginal.value = '';
      diffModified.value = content;
      diffEmpty.value = '未跟踪的新文件';
    } else {
      const res = await gitDiffContent(
        project.value.id,
        f.path,
        !!f.staged,
        f.original_path || undefined,
      );
      diffOriginal.value = res.original_content;
      diffModified.value = res.modified_content;
      diffPatch.value = res.fallback_patch;
      diffIsBinary.value = res.is_binary;
    }
  } catch (e) {
    diffEmpty.value = '无法加载 diff';
    console.error('加载 diff 失败：', e);
  } finally {
    diffLoading.value = false;
  }
}

async function stageFile(f: ChangedFile) {
  if (!project.value) return;
  busy.value = true;
  try {
    await gitStage(project.value.id, [f.path]);
    await refresh();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '暂存失败');
  } finally {
    busy.value = false;
  }
}

async function unstageFile(f: ChangedFile) {
  if (!project.value) return;
  busy.value = true;
  try {
    await gitUnstage(project.value.id, [f.path]);
    await refresh();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '取消暂存失败');
  } finally {
    busy.value = false;
  }
}

async function discardFile(f: ChangedFile) {
  if (!project.value) return;
  if (!confirm(`确定丢弃「${f.path}」的改动？此操作不可撤销。`)) return;
  busy.value = true;
  try {
    await gitDiscard(project.value.id, [{ path: f.path, untracked: isUntracked(f) }]);
    await refresh();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '丢弃失败');
  } finally {
    busy.value = false;
  }
}

async function stageAll() {
  if (!project.value || !unstagedFiles.value.length) return;
  busy.value = true;
  try {
    await gitStage(project.value.id, unstagedFiles.value.map((f) => f.path));
    await refresh();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '批量暂存失败');
  } finally {
    busy.value = false;
  }
}

async function unstageAll() {
  if (!project.value || !stagedFiles.value.length) return;
  busy.value = true;
  try {
    await gitUnstage(project.value.id, stagedFiles.value.map((f) => f.path));
    await refresh();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '批量取消暂存失败');
  } finally {
    busy.value = false;
  }
}

async function resolveSide(f: ChangedFile, side: ConflictSide) {
  if (!project.value) return;
  busy.value = true;
  try {
    await gitResolveConflict(project.value.id, [f.path], side);
    toast.success(side === 'ours' ? '已采纳我方版本' : '已采纳对方版本');
    await refresh();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '解决冲突失败');
  } finally {
    busy.value = false;
  }
}

async function markResolved(f: ChangedFile) {
  if (!project.value) return;
  busy.value = true;
  try {
    await gitMarkConflictResolved(project.value.id, [f.path]);
    toast.success(`已标记解决 ${f.path}`);
    await refresh();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '标记失败');
  } finally {
    busy.value = false;
  }
}

function openInEditor(_f: ChangedFile) {
  if (!project.value) return;
  tabStore.openProject(project.value.id, 'workspace');
}

async function commit() {
  if (!project.value || !canCommit.value) return;
  busy.value = true;
  try {
    const res = await gitCommit(project.value.id, commitMessage.value.trim());
    toast.success(res.summary || '提交成功');
    commitMessage.value = '';
    await refresh();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '提交失败');
  } finally {
    busy.value = false;
  }
}

async function continueOperation() {
  if (!project.value) return;
  busy.value = true;
  try {
    const msg = commitMessage.value.trim();
    const res = await gitMergeContinue(project.value.id, msg || undefined);
    toast.success(res.summary || '操作已完成');
    commitMessage.value = '';
    await refresh();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '继续操作失败');
  } finally {
    busy.value = false;
  }
}

async function abortOperation() {
  if (!project.value) return;
  if (!confirm('确定中止当前 merge / rebase 操作？')) return;
  busy.value = true;
  try {
    await gitAbortOperation(project.value.id);
    toast.success('已中止');
    await refresh();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '中止失败');
  } finally {
    busy.value = false;
  }
}

async function onCheckout(e: Event) {
  if (!project.value) return;
  const branch = (e.target as HTMLSelectElement).value;
  if (!branch || branch === status.value?.branch) return;
  busy.value = true;
  try {
    await gitCheckoutBranch(project.value.id, branch);
    toast.success(`已切换到 ${branch}`);
    await refresh();
  } catch (err) {
    toast.error(typeof err === 'string' ? err : '切换分支失败');
  } finally {
    busy.value = false;
  }
}
</script>

<style scoped>
.sc-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  overflow: hidden;
}
.sc-panel-empty {
  display: flex;
  height: 100%;
  align-items: center;
  justify-content: center;
  color: var(--muted-foreground);
  font-size: 13px;
}
.sc-header {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  padding: 8px 12px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}
.sc-branch {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
}
.sc-branch-name {
  font-weight: 500;
}
.sc-ahead-behind {
  display: flex;
  gap: 4px;
  font-size: 11px;
}
.sc-spacer {
  flex: 1;
}
.sc-select {
  height: 28px;
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 0 8px;
  font-size: 12px;
  background: var(--background);
  color: var(--foreground);
}
.sc-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 12px;
  background: color-mix(in srgb, var(--sys-orange) 12%, transparent);
  box-shadow: inset 0 -1px 0 0 var(--separator);
  flex-shrink: 0;
}
.sc-banner-text {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  font-size: 12px;
}
.sc-banner-msg {
  color: var(--muted-foreground);
}
.sc-banner-conflict {
  color: var(--destructive);
}
.sc-banner-ok {
  color: var(--sys-green);
}
.sc-banner-actions {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}
.sc-error {
  padding: 8px 12px;
  font-size: 12px;
  color: var(--destructive);
  background: color-mix(in oklab, var(--destructive) 8%, transparent);
  border-bottom: 1px solid var(--border);
}
.sc-body {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}
.sc-left {
  width: 360px;
  min-width: 260px;
  max-width: 48%;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border);
  overflow: hidden;
}
.sc-section {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px 4px;
  flex-shrink: 0;
}
.sc-section-title {
  font-size: 11px;
  font-weight: 600;
  color: var(--muted-foreground);
  text-transform: none;
}
.sc-section-title--conflict {
  color: var(--destructive);
}
.sc-file-list {
  flex: 1;
  min-height: 40px;
  overflow-y: auto;
  padding: 0 6px 6px;
}
.sc-file-list--conflicts {
  flex: 0 0 auto;
  max-height: 40%;
  overflow-y: auto;
}
.sc-file {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 12px;
}
.sc-file:hover {
  background: var(--accent);
}
.sc-file--active {
  background: color-mix(in oklab, var(--primary) 14%, transparent);
}
.sc-file--conflict {
  background: color-mix(in oklab, #dc2626 6%, transparent);
}
.sc-file-status {
  flex-shrink: 0;
  min-width: 22px;
  justify-content: center;
}
.sc-file-path {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.sc-file-actions {
  display: none;
  gap: 4px;
  flex-shrink: 0;
}
.sc-file:hover .sc-file-actions {
  display: flex;
}
.sc-commit {
  flex-shrink: 0;
  border-top: 1px solid var(--border);
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.sc-textarea {
  width: 100%;
  resize: vertical;
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 8px;
  font-size: 13px;
  font-family: inherit;
  background: var(--background);
  color: var(--foreground);
}
.sc-right {
  flex: 1;
  min-width: 0;
  padding: 8px;
  display: flex;
  flex-direction: column;
}
.sc-right > :deep(*) {
  flex: 1;
  min-height: 0;
}
</style>
