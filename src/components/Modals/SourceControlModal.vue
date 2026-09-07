<!--
  源码控制面板
  - 左侧：逐文件改动列表（暂存/取消暂存/丢弃）+ 提交框
  - 右侧：选中文件的 diff 查看
  - 顶部：当前分支、ahead/behind、分支切换、远端跳转
-->
<template>
  <Dialog v-model="visible" :title="`源码控制 · ${project?.name ?? ''}`" width="1000px">
    <div v-if="project" class="sc-container">
      <!-- 顶部信息栏 -->
      <div class="sc-header">
        <div class="sc-branch">
          <GitBranch :size="14" />
          <span class="sc-branch-name" :title="status?.branch">{{ branchLabel }}</span>
          <Tag v-if="status?.is_detached" variant="warning">detached</Tag>
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
          :disabled="busy"
          @change="onCheckout"
        >
          <option v-for="b in localBranches" :key="b.name" :value="b.name">
            {{ b.display_name }}{{ b.is_current ? ' (当前)' : '' }}
          </option>
        </select>
        <Button v-if="remoteUrl" size="sm" variant="ghost" :disabled="busy" @click="openRemote">
          在远程查看
        </Button>
        <Button size="sm" variant="ghost" :disabled="busy" @click="refresh">
          <RefreshCw :size="14" /> 刷新
        </Button>
      </div>

      <div v-if="status?.error" class="sc-error">{{ status.error }}</div>

      <div class="sc-body">
        <!-- 左：文件列表 + 提交 -->
        <div class="sc-left">
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
            <Empty v-if="!stagedFiles.length && !unstagedFiles.length" description="工作区干净" />
          </div>

          <!-- 提交框 -->
          <div class="sc-commit">
            <textarea
              v-model="commitMessage"
              class="sc-textarea"
              placeholder="提交信息（第一行作为摘要）..."
              rows="3"
            />
            <Button variant="primary" size="sm" :disabled="busy || !canCommit" @click="commit">
              提交
            </Button>
          </div>
        </div>

        <!-- 右：diff -->
        <div class="sc-right">
          <DiffViewer :patch="diffPatch" :is-binary="diffIsBinary" :empty-text="diffEmpty" />
        </div>
      </div>
    </div>
    <div v-else class="sc-empty"><Spinner /></div>
  </Dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { open } from '@tauri-apps/plugin-shell';
import type { Branch, ChangedFile, Project, ProjectStatus } from '../../types';
import {
  getBranches,
  getProjectStatusForce,
  gitCheckoutBranch,
  gitCommit,
  gitDiff,
  gitDiscard,
  gitRemoteUrl,
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

const diffPatch = ref('');
const diffIsBinary = ref(false);
const diffEmpty = ref('选择左侧文件查看改动');

const localBranches = computed(() => branches.value.filter((b) => b.is_local && !b.is_detached));
const stagedFiles = computed(() => (status.value?.changed_files ?? []).filter((f) => f.staged));
const unstagedFiles = computed(() =>
  (status.value?.changed_files ?? []).filter((f) => !f.staged),
);
const branchLabel = computed(() => {
  if (!status.value) return '-';
  if (status.value.is_detached) return '分离 HEAD';
  return status.value.branch || '-';
});
const canCommit = computed(() => commitMessage.value.trim().length > 0);

watch(visible, async (open) => {
  if (open && props.project) {
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
    // 保持选中文件指向最新状态
    if (selectedFile.value) {
      selectedFile.value =
        st.changed_files.find((f) => f.path === selectedFile.value!.path) ?? null;
      if (selectedFile.value) await loadDiff(selectedFile.value);
    }
  } catch (e) {
    console.error('加载源码控制失败：', e);
    toast.error('加载源码控制失败');
  } finally {
    busy.value = false;
  }
}

function statusLabel(f: ChangedFile): string {
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
  height: 620px;
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
  width: 360px;
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
.sc-file-list {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
  padding: 0 6px;
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
}
.sc-empty {
  display: flex;
  height: 620px;
  align-items: center;
  justify-content: center;
}
</style>
