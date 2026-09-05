<!--
  Git 历史视图弹窗
  左侧分支列表 / 中间提交列表 / 右侧提交详情
-->
<template>
  <Dialog v-model="visible" title="Git 记录" width="900px">
    <div class="history-container">
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
      </aside>

      <!-- 中间提交列表 -->
      <main class="history-main">
        <div v-if="loadingCommits" class="flex h-full items-center justify-center">
          <Spinner size="lg" />
        </div>
        <template v-else>
          <div
            class="commit-head"
            style="grid-template-columns: 80px minmax(0, 1fr) 110px 150px"
          >
            <div>ID</div>
            <div>提交信息</div>
            <div>作者</div>
            <div>时间</div>
          </div>
          <div class="commit-list">
            <div
              v-for="commit in commits"
              :key="commit.id"
              :class="['commit-row', selectedCommit?.id === commit.id ? 'commit-row--active' : '']"
              style="grid-template-columns: 80px minmax(0, 1fr) 110px 150px"
              @click="handleCommitChange(commit)"
            >
              <div class="commit-id font-mono">{{ commit.short_id }}</div>
              <div class="commit-msg" :title="commit.message">{{ firstLine(commit.message) }}</div>
              <div class="commit-author truncate">{{ commit.author }}</div>
              <div class="commit-date">{{ formatDate(commit.date) }}</div>
            </div>
            <Empty v-if="!commits.length" description="暂无提交记录" />
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
            <div v-for="f in detail.files" :key="f.path" class="detail-file">
              <Tag :variant="statusVariant(f.status)" class="file-status">{{ f.status }}</Tag>
              <span class="file-path" :title="f.path">{{ f.path }}</span>
            </div>
          </div>
        </div>
        <Empty v-else description="选择一条提交查看详情" />
      </aside>
    </div>
  </Dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import type { Branch, Commit, CommitDetail, Project } from '../../types';
import { getBranches, getCommits, getCommitDetail } from '../../lib/tauriApi';
import Dialog from '../ui/Dialog.vue';
import Tag from '../ui/Tag.vue';
import Spinner from '../ui/Spinner.vue';
import Empty from '../ui/Empty.vue';
import { toast } from '../../lib/toast';

const visible = defineModel<boolean>({ required: true });
const props = defineProps<{ project: Project | null }>();

const branches = ref<Branch[]>([]);
const selectedBranch = ref('');
const commits = ref<Commit[]>([]);
const selectedCommit = ref<Commit | null>(null);
const detail = ref<CommitDetail | null>(null);
const loadingCommits = ref(false);

const localBranches = computed(() => branches.value.filter((b) => b.is_local));
const remoteBranches = computed(() => branches.value.filter((b) => b.is_remote));

watch(visible, async (open) => {
  if (open && props.project) {
    branches.value = [];
    commits.value = [];
    selectedCommit.value = null;
    detail.value = null;
    await loadBranches();
  }
});

watch(selectedBranch, async (branch) => {
  if (!branch) return;
  selectedCommit.value = null;
  detail.value = null;
  await loadCommits(branch);
});

async function loadBranches() {
  try {
    branches.value = await getBranches(props.project!.id);
    const current = branches.value.find((b) => b.is_current);
    selectedBranch.value = current?.display_name || branches.value[0]?.display_name || '';
  } catch (error) {
    console.error('加载分支失败：', error);
    toast.error('加载分支失败');
  }
}

async function loadCommits(branch: string) {
  loadingCommits.value = true;
  try {
    commits.value = await getCommits(props.project!.id, branch, 100);
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
  try {
    detail.value = await getCommitDetail(props.project!.id, commitId);
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
.history-container {
  display: flex;
  height: 560px;
  border: 1px solid var(--border);
  border-radius: 8px;
  overflow: hidden;
}
.history-aside {
  display: flex;
  flex-direction: column;
  width: 200px;
  flex-shrink: 0;
  background-color: var(--muted);
  border-right: 1px solid var(--border);
}
.detail-aside {
  border-right: none;
  border-left: 1px solid var(--border);
  width: 260px;
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
</style>
