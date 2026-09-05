<!--
  Git 历史视图弹窗
  左侧分支列表 / 中间提交列表 / 右侧提交详情，类似 SourceTree 的提交历史
-->
<template>
  <el-dialog
    v-model="visible"
    title="Git 记录"
    width="900px"
    :close-on-click-modal="false"
    append-to-body
    destroy-on-close
    class="git-history-dialog"
  >
    <el-container class="history-container">
      <!-- 左侧分支 -->
      <el-aside width="200px" class="history-aside">
        <div class="aside-title">分支</div>
        <el-scrollbar class="branch-list">
          <el-menu
            :default-active="selectedBranch"
            class="branch-menu"
            @select="selectBranch"
          >
            <el-menu-item-group v-if="localBranches.length" title="本地">
              <el-menu-item
                v-for="branch in localBranches"
                :key="branch.name"
                :index="branch.display_name"
              >
                <span class="branch-name" :title="branch.display_name">{{ branch.display_name }}</span>
                <el-tag v-if="branch.is_current" size="small" class="current-tag">当前</el-tag>
              </el-menu-item>
            </el-menu-item-group>
            <el-menu-item-group v-if="remoteBranches.length" title="远程">
              <el-menu-item
                v-for="branch in remoteBranches"
                :key="branch.name"
                :index="branch.display_name"
              >
                <span class="branch-name" :title="branch.display_name">{{ branch.display_name }}</span>
              </el-menu-item>
            </el-menu-item-group>
            <el-empty v-if="!branches.length" description="暂无分支" :image-size="60" />
          </el-menu>
        </el-scrollbar>
      </el-aside>

      <!-- 中间提交列表 -->
      <el-main class="history-main">
        <el-table
          v-loading="loadingCommits"
          :data="commits"
          height="520"
          highlight-current-row
          @current-change="handleCommitChange"
        >
          <el-table-column prop="short_id" label="ID" width="70" />
          <el-table-column prop="message" label="提交信息" min-width="220" show-overflow-tooltip />
          <el-table-column prop="author" label="作者" width="100" show-overflow-tooltip />
          <el-table-column label="时间" width="150">
            <template #default="{ row }">
              {{ formatDate(row.date) }}
            </template>
          </el-table-column>
          <template #empty>
            <el-empty description="暂无提交记录" :image-size="60" />
          </template>
        </el-table>
      </el-main>

      <!-- 右侧详情 -->
      <el-aside width="260px" class="history-aside detail-aside">
        <div class="aside-title">提交详情</div>
        <el-scrollbar v-if="detail" class="detail-scroll">
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
              <el-tag :type="statusType(f.status)" size="small" class="file-status">{{ f.status }}</el-tag>
              <span class="file-path" :title="f.path">{{ f.path }}</span>
            </div>
          </div>
        </el-scrollbar>
        <el-empty v-else description="选择一条提交查看详情" :image-size="60" />
      </el-aside>
    </el-container>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { ElMessage } from 'element-plus';
import type { Branch, Commit, CommitDetail, Project } from '../../types';
import { getBranches, getCommits, getCommitDetail } from '../../lib/tauriApi';

const visible = defineModel<boolean>({ required: true });
const props = defineProps<{ project: Project | null }>();

const branches = ref<Branch[]>([]);
const selectedBranch = ref('');
const commits = ref<Commit[]>([]);
const selectedCommit = ref<Commit | null>(null);
const detail = ref<CommitDetail | null>(null);
const loadingCommits = ref(false);

const localBranches = computed(() => branches.value.filter(b => b.is_local));
const remoteBranches = computed(() => branches.value.filter(b => b.is_remote));

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
    const current = branches.value.find(b => b.is_current);
    selectedBranch.value = current?.display_name || branches.value[0]?.display_name || '';
  } catch (error) {
    console.error('加载分支失败：', error);
    ElMessage.error('加载分支失败');
  }
}

async function loadCommits(branch: string) {
  loadingCommits.value = true;
  try {
    commits.value = await getCommits(props.project!.id, branch, 100);
  } catch (error) {
    console.error('加载提交记录失败：', error);
    ElMessage.error('加载提交记录失败');
  } finally {
    loadingCommits.value = false;
  }
}

function selectBranch(branch: string) {
  selectedBranch.value = branch;
}

function handleCommitChange(commit: Commit | null, _oldCommit: Commit | null) {
  selectedCommit.value = commit;
  if (commit) {
    loadCommitDetail(commit.id);
  } else {
    detail.value = null;
  }
}

async function loadCommitDetail(commitId: string) {
  try {
    detail.value = await getCommitDetail(props.project!.id, commitId);
  } catch (error) {
    console.error('加载提交详情失败：', error);
    ElMessage.error('加载提交详情失败');
  }
}

function formatDate(ts: number) {
  return new Date(ts).toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function statusType(status: string): 'success' | 'danger' | 'warning' | 'info' {
  switch (status.charAt(0)) {
    case 'A': return 'success';
    case 'D': return 'danger';
    case 'M': return 'warning';
    case 'R':
    case 'C': return 'info';
    default: return 'info';
  }
}
</script>

<style scoped>
.history-container {
  height: 560px;
  border: 1px solid var(--el-border-color);
  border-radius: 4px;
  overflow: hidden;
}

.history-aside {
  display: flex;
  flex-direction: column;
  background-color: var(--el-fill-color-light);
  border-right: 1px solid var(--el-border-color);
}

.detail-aside {
  border-right: none;
  border-left: 1px solid var(--el-border-color);
}

.aside-title {
  padding: 10px 12px;
  font-size: 13px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  border-bottom: 1px solid var(--el-border-color);
  flex-shrink: 0;
}

.branch-list {
  flex: 1;
}

.branch-menu {
  border-right: none;
  background-color: transparent;
}

.branch-menu :deep(.el-menu-item) {
  height: 32px;
  line-height: 32px;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 12px !important;
}

.branch-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.current-tag {
  flex-shrink: 0;
}

.history-main {
  padding: 0;
  display: flex;
  flex-direction: column;
}

.detail-scroll {
  flex: 1;
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
  color: var(--el-text-color-secondary);
  margin-bottom: 6px;
}

.detail-message {
  font-size: 13px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  word-break: break-word;
}

.detail-value {
  font-size: 13px;
  color: var(--el-text-color-primary);
  word-break: break-word;
}

.detail-hash {
  font-size: 12px;
  font-family: var(--el-font-family-monospace, monospace);
  color: var(--el-text-color-regular);
  word-break: break-all;
}

.detail-body {
  font-size: 12px;
  line-height: 1.6;
  color: var(--el-text-color-regular);
  white-space: pre-wrap;
  word-break: break-word;
  margin: 0;
  font-family: inherit;
}

.detail-empty {
  font-size: 12px;
  color: var(--el-text-color-secondary);
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
  text-align: center;
}

.file-path {
  flex: 1;
  color: var(--el-text-color-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

:deep(.el-dialog__body) {
  padding-top: 10px;
  padding-bottom: 20px;
}
</style>
