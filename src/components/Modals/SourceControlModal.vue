<!--
  源码控制面板
  - 冲突区：三路对比、采纳我方/对方、标记已解决
  - 更改区：暂存/取消暂存/丢弃
  - 进行中操作横幅：merge/rebase/cherry-pick 中止与继续
-->
<template>
  <Dialog v-model="visible" :title="`源码控制 · ${project?.name ?? ''}`" width="1100px">
    <div v-if="project" class="sc-container">
      <!-- 顶部信息栏 -->
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
          title="合并选中分支到当前分支"
          @click="onMergeSelected"
        >
          合并…
        </Button>
        <Button v-if="remoteUrl" size="sm" variant="ghost" :disabled="busy" @click="openRemote">
          在远程查看
        </Button>
        <Button size="sm" variant="ghost" :disabled="busy" @click="refresh">
          <RefreshCw :size="14" /> 刷新
        </Button>
      </div>

      <!-- 进行中的 merge / rebase / cherry-pick 横幅 -->
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
        <!-- 左：文件列表 + 提交 -->
        <div class="sc-left">
          <!-- 冲突 -->
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
                  <Button size="sm" variant="ghost" title="采用当前分支版本" :disabled="busy" @click.stop="resolveSide(f, 'ours')">
                    我方
                  </Button>
                  <Button size="sm" variant="ghost" title="采用合入分支版本" :disabled="busy" @click.stop="resolveSide(f, 'theirs')">
                    对方
                  </Button>
                  <Button size="sm" variant="ghost" title="编辑后标记已解决" :disabled="busy" @click.stop="openInEditor(f)">
                    编辑
                  </Button>
                  <Button size="sm" variant="primary" title="标记为已解决" :disabled="busy" @click.stop="markResolved(f)">
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
                  <Button size="sm" variant="ghost" title="取消暂存" :disabled="busy" @click.stop="unstageFile(f)">
                    暂存↓
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
                <Button size="sm" variant="ghost" title="暂存" :disabled="busy" @click.stop="stageFile(f)">暂存</Button>
                <Button size="sm" variant="ghost" title="丢弃改动" :disabled="busy" @click.stop="discardFile(f)">丢弃</Button>
              </div>
            </div>
            <Empty
              v-if="!stagedFiles.length && !unstagedFiles.length && !conflictFiles.length"
              description="工作区干净"
            />
          </div>

          <!-- 提交框 -->
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

        <!-- 右：diff / 冲突三路 -->
        <div class="sc-right">
          <ConflictViewer
            v-if="selectedFile?.is_conflict"
            :content="conflictContent"
            :loading="conflictLoading"
          />
          <DiffViewer
            v-else
            :patch="diffPatch"
            :is-binary="diffIsBinary"
            :empty-text="diffEmpty"
          />
        </div>
      </div>
    </div>
    <div v-else class="sc-empty"><Spinner /></div>
  </Dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { open } from '@tauri-apps/plugin-shell';
import { router } from '../../router';
import type {
  Branch,
  ChangedFile,
  ConflictFileContent,
  ConflictSide,
  InProgressOp,
  Project,
  ProjectStatus,
} from '../../types';
import {
  getBranches,
  getProjectStatusForce,
  gitAbortOperation,
  gitCheckoutBranch,
  gitCommit,
  gitConflictFileContent,
  gitDiff,
  gitDiscard,
  gitMarkConflictResolved,
  gitMerge,
  gitMergeContinue,
  gitRemoteUrl,
  gitResolveConflict,
  gitStage,
  gitUnstage,
  readFile,
} from '../../lib/tauriApi';
import Dialog from '../ui/Dialog.vue';
import Button from '../ui/Button.vue';
import Tag from '../ui/Tag.vue';
import Empty from '../ui/Empty.vue';
import Spinner from '../ui/Spinner.vue';
import DiffViewer from '../Git/DiffViewer.vue';
import ConflictViewer from '../Git/ConflictViewer.vue';
import { GitBranch, RefreshCw } from 'lucide-vue-next';
import { toast } from '../../lib/toast';

const visible = defineModel<boolean>({ required: true });
const props = defineProps<{ project: Project | null }>();

const status = ref<ProjectStatus | null>(null);
const branches = ref<Branch[]>([]);
const remoteUrl = ref<string | null>(null);
const busy = ref(false);
const selectedFile = ref<ChangedFile | null>(null);
const commitMessage = ref('');
const mergeTarget = ref<string | null>(null);

const diffPatch = ref('');
const diffIsBinary = ref(false);
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

watch(visible, async (isOpen) => {
  if (isOpen && props.project) {
    await refresh();
  }
});

async function refresh() {
  if (!props.project) return;
  busy.value = true;
  try {
    const [st, brs, url] = await Promise.all([
      getProjectStatusForce(props.project.id),
      getBranches(props.project.id),
      gitRemoteUrl(props.project.id).catch(() => null),
    ]);
    status.value = st;
    branches.value = brs;
    remoteUrl.value = url;
    if (selectedFile.value) {
      const next =
        st.changed_files.find((f) => f.path === selectedFile.value!.path) ?? null;
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
  if (!props.project) return;
  if (f.is_conflict) {
    conflictContent.value = null;
    conflictLoading.value = true;
    diffPatch.value = '';
    try {
      conflictContent.value = await gitConflictFileContent(props.project.id, f.path);
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
  try {
    if (isUntracked(f)) {
      const content = await readFile([props.project.path, f.path].join('/'));
      diffPatch.value = content;
      diffEmpty.value = '未跟踪的新文件';
    } else if (f.staged) {
      diffPatch.value = await gitDiff(props.project.id, f.path, true);
    } else {
      diffPatch.value = await gitDiff(props.project.id, f.path, false);
    }
  } catch (e) {
    diffPatch.value = '';
    diffEmpty.value = '无法加载 diff';
    console.error('加载 diff 失败：', e);
  }
}

async function stageFile(f: ChangedFile) {
  if (!props.project) return;
  busy.value = true;
  try {
    await gitStage(props.project.id, [f.path]);
    await refresh();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '暂存失败');
  } finally {
    busy.value = false;
  }
}

async function unstageFile(f: ChangedFile) {
  if (!props.project) return;
  busy.value = true;
  try {
    await gitUnstage(props.project.id, [f.path]);
    await refresh();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '取消暂存失败');
  } finally {
    busy.value = false;
  }
}

async function discardFile(f: ChangedFile) {
  if (!props.project) return;
  if (!confirm(`确定丢弃「${f.path}」的改动？此操作不可撤销。`)) return;
  busy.value = true;
  try {
    await gitDiscard(props.project.id, [
      { path: f.path, untracked: isUntracked(f) },
    ]);
    await refresh();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '丢弃失败');
  } finally {
    busy.value = false;
  }
}

async function stageAll() {
  if (!props.project || !unstagedFiles.value.length) return;
  busy.value = true;
  try {
    await gitStage(
      props.project.id,
      unstagedFiles.value.map((f) => f.path),
    );
    await refresh();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '批量暂存失败');
  } finally {
    busy.value = false;
  }
}

async function unstageAll() {
  if (!props.project || !stagedFiles.value.length) return;
  busy.value = true;
  try {
    await gitUnstage(
      props.project.id,
      stagedFiles.value.map((f) => f.path),
    );
    await refresh();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '批量取消暂存失败');
  } finally {
    busy.value = false;
  }
}

async function resolveSide(f: ChangedFile, side: ConflictSide) {
  if (!props.project) return;
  busy.value = true;
  try {
    await gitResolveConflict(props.project.id, [f.path], side);
    toast.success(side === 'ours' ? '已采纳我方版本' : '已采纳对方版本');
    await refresh();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '解决冲突失败');
  } finally {
    busy.value = false;
  }
}

async function markResolved(f: ChangedFile) {
  if (!props.project) return;
  busy.value = true;
  try {
    await gitMarkConflictResolved(props.project.id, [f.path]);
    toast.success(`已标记解决 ${f.path}`);
    await refresh();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '标记失败');
  } finally {
    busy.value = false;
  }
}

async function openInEditor(f: ChangedFile) {
  if (!props.project) return;
  try {
    visible.value = false;
    await router.push({
      name: 'workspace',
      params: { projectId: props.project.id },
      query: { file: f.path },
    });
  } catch (e) {
    toast.error('打开编辑器失败');
    console.error(e);
  }
}

async function commit() {
  if (!props.project || !canCommit.value) return;
  busy.value = true;
  try {
    const res = await gitCommit(props.project.id, commitMessage.value.trim());
    commitMessage.value = '';
    toast.success(`已提交 ${res.commit_sha.slice(0, 7)}`);
    await refresh();
  } catch (e) {
    toast.error('提交失败：' + String(e));
  } finally {
    busy.value = false;
  }
}

async function continueOperation() {
  if (!props.project || !inProgress.value) return;
  if (conflictCount.value > 0) {
    toast.error('请先解决全部冲突');
    return;
  }
  busy.value = true;
  try {
    const msg = commitMessage.value.trim() || undefined;
    const res = await gitMergeContinue(props.project.id, msg);
    commitMessage.value = '';
    toast.success(
      res.commit_sha
        ? `已完成操作 ${res.commit_sha.slice(0, 7)}`
        : '操作已继续',
    );
    await refresh();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '继续操作失败');
  } finally {
    busy.value = false;
  }
}

async function abortOperation() {
  if (!props.project || !inProgress.value) return;
  if (!confirm(`确定中止${inProgressTitle.value}？工作区将恢复到操作前状态。`)) return;
  busy.value = true;
  try {
    await gitAbortOperation(props.project.id);
    toast.success('已中止操作');
    commitMessage.value = '';
    await refresh();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '中止失败');
  } finally {
    busy.value = false;
  }
}

function onMergeSelected() {
  const other = localBranches.value.filter((b) => !b.is_current);
  if (!other.length) {
    toast.error('没有可合并的本地分支');
    return;
  }
  const names = other.map((b, i) => `${i + 1}. ${b.display_name}`).join('\n');
  const pick = prompt(`输入要合并到当前分支的分支名：\n${names}`);
  if (!pick) return;
  mergeTarget.value = pick.trim();
  void runMerge();
}

async function runMerge() {
  if (!props.project || !mergeTarget.value) return;
  busy.value = true;
  try {
    const res = await gitMerge(props.project.id, mergeTarget.value);
    if (res.has_conflicts) {
      toast.error('合并存在冲突，请在左侧解决');
    } else {
      toast.success(res.message || '合并成功');
    }
    mergeTarget.value = null;
    await refresh();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '合并失败');
  } finally {
    busy.value = false;
  }
}

async function onCheckout(e: Event) {
  const target = e.target as HTMLSelectElement;
  const branch = target.value;
  if (!props.project || !branch) return;
  busy.value = true;
  try {
    await gitCheckoutBranch(props.project.id, branch);
    toast.success(`已切换到 ${branch}`);
    await refresh();
  } catch (err) {
    toast.error('切换分支失败');
    console.error(err);
  } finally {
    busy.value = false;
  }
}

async function openRemote() {
  if (remoteUrl.value) {
    await open(remoteUrl.value);
  }
}
</script>

<style scoped>
.sc-container {
  display: flex;
  flex-direction: column;
  height: 640px;
}
.sc-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}
.sc-branch {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 600;
}
.sc-branch-name {
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.sc-ahead-behind {
  display: flex;
  gap: 8px;
  font-size: 12px;
  font-variant-numeric: tabular-nums;
}
.sc-spacer {
  flex: 1;
}
.sc-select {
  height: 28px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--background);
  color: var(--foreground);
  font-size: 13px;
  padding: 0 6px;
  max-width: 200px;
}
.sc-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 8px 12px;
  background: color-mix(in oklab, #d97706 12%, transparent);
  border-bottom: 1px solid color-mix(in oklab, #d97706 30%, transparent);
  flex-shrink: 0;
}
.sc-banner-text {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 8px;
  font-size: 12px;
  min-width: 0;
}
.sc-banner-msg {
  color: var(--muted-foreground);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 360px;
}
.sc-banner-conflict {
  color: #dc2626;
  font-weight: 600;
}
.sc-banner-ok {
  color: #16a34a;
  font-weight: 500;
}
.sc-banner-actions {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}
.sc-error {
  padding: 8px 12px;
  font-size: 12px;
  color: #dc2626;
  background: color-mix(in oklab, #dc2626 10%, transparent);
}
.sc-body {
  flex: 1;
  display: flex;
  min-height: 0;
}
.sc-left {
  width: 380px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border);
  min-height: 0;
}
.sc-section {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px 4px;
  flex-shrink: 0;
}
.sc-section-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--muted-foreground);
}
.sc-section-title--conflict {
  color: #dc2626;
}
.sc-file-list {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
  padding: 0 6px;
}
.sc-file-list--conflicts {
  flex: 0 1 auto;
  max-height: 140px;
}
.sc-file {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 6px;
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
.sc-empty {
  display: flex;
  height: 640px;
  align-items: center;
  justify-content: center;
}
</style>
