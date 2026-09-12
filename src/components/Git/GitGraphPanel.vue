<!--
  Git 图面板（内嵌于工作区，非弹窗）
  顶部：分支选择 + 提交数 + 关闭
  主体：左侧 SVG 提交图 + 右侧提交列表（行高与图对齐，整体同步滚动）
  底部：选中提交的详情（提交说明 + 改动文件列表）
-->
<template>
  <div class="git-panel flex h-full min-h-0 flex-col">
    <div class="surface-toolbar flex h-11 shrink-0 items-center gap-3 hairline-b px-3">
      <Select
        :model-value="currentBranch"
        :options="branchOptions"
        class="w-52"
        @update:model-value="onBranchChange"
      />
      <span class="text-xs text-muted-foreground">{{ commits.length }} 次提交</span>
      <Tag v-if="unpushedCount > 0" variant="warning">
        {{ unpushedCount }} 条未推送
      </Tag>
      <Button variant="ghost" size="icon" class="ml-auto" title="关闭" @click="$emit('close')">
        <X :size="16" />
      </Button>
    </div>

    <div class="git-body relative flex min-h-0 flex-1 overflow-y-auto overflow-x-hidden" @scroll="onScroll">
      <div class="git-rail-col shrink-0 hairline-r">
        <div class="git-head-rail" />
        <GitGraphRail :commits="commits" :selected-id="selectedId" @select="onSelect" />
      </div>
      <div class="git-list min-w-0 flex-1">
        <div class="git-head" :style="{ gridTemplateColumns: gridTemplate }">
          <div class="col-th" data-col-id="msg" title="双击恢复默认宽度">
            <span class="truncate">提交信息</span>
            <span
              class="col-resize-handle"
              @pointerdown="beginResize('msg', $event)"
              @dblclick.stop="resetColumn('msg')"
            />
          </div>
          <div class="col-th" data-col-id="status" title="双击恢复默认宽度">
            <span>状态</span>
            <span
              class="col-resize-handle"
              @pointerdown="beginResize('status', $event)"
              @dblclick.stop="resetColumn('status')"
            />
          </div>
          <div class="col-th" data-col-id="author" title="双击恢复默认宽度">
            <span class="truncate">作者</span>
            <span
              class="col-resize-handle"
              @pointerdown="beginResize('author', $event)"
              @dblclick.stop="resetColumn('author')"
            />
          </div>
          <div class="col-th" data-col-id="date" title="双击恢复默认宽度">
            <span class="truncate">时间</span>
            <span
              class="col-resize-handle"
              @pointerdown="beginResize('date', $event)"
              @dblclick.stop="resetColumn('date')"
            />
          </div>
          <div class="col-th" data-col-id="id" title="双击恢复默认宽度">
            <span class="truncate">ID</span>
            <span
              class="col-resize-handle"
              @pointerdown="beginResize('id', $event)"
              @dblclick.stop="resetColumn('id')"
            />
          </div>
        </div>
        <div
          v-for="c in commits"
          :key="c.id"
          :class="['git-row', { active: c.id === selectedId }]"
          :style="{ gridTemplateColumns: gridTemplate }"
          @click="onSelect(c.id)"
        >
          <div class="git-row-msg" :title="c.message">{{ firstLine(c.message) }}</div>
          <div class="git-badge-cell">
            <Tag
              v-if="c.is_pushed === false"
              variant="warning"
              title="尚未推送到远程"
            >
              未推送
            </Tag>
          </div>
          <span class="git-author" :title="c.author">{{ c.author }}</span>
          <span class="git-date">{{ formatDate(c.date) }}</span>
          <span class="git-hash">{{ c.short_id }}</span>
        </div>
        <div v-if="!commits.length && !loading" class="git-empty">无提交记录</div>
        <div v-if="loadingMore" class="git-foot">加载更多…</div>
        <div v-else-if="endReached && commits.length" class="git-foot git-foot--muted">已到历史开头</div>
      </div>

      <div v-if="loading" class="git-loading">
        <Spinner size="lg" />
      </div>
    </div>

    <div v-if="detail" class="git-detail shrink-0 max-h-[40%] overflow-auto hairline-t p-3">
      <div class="detail-title">
        提交详情 · {{ detail.short_id }}
        <span class="detail-author">{{ detail.author }}</span>
      </div>
      <pre v-if="detail.body" class="detail-body">{{ detail.body }}</pre>
      <div class="detail-files">
        <div v-for="f in detail.files" :key="f.path" class="file-item">
          <Tag :variant="statusVariant(f.status)" class="file-status">{{ f.status }}</Tag>
          <span class="file-path" :title="f.path">{{ f.path }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { X } from 'lucide-vue-next';
import type { Branch, Commit, CommitDetail } from '../../types';
import { getBranches, getCommits, getCommitDetail } from '../../lib/tauriApi';
import GitGraphRail from './GitGraphRail.vue';
import Select from '../ui/Select.vue';
import Button from '../ui/Button.vue';
import Tag from '../ui/Tag.vue';
import Spinner from '../ui/Spinner.vue';
import { useResizableColumns } from '../../composables/useResizableColumns';

const props = defineProps<{ projectId: string }>();
defineEmits<{ (e: 'close'): void }>();

const branches = ref<Branch[]>([]);
const currentBranch = ref<string>('');
const commits = ref<Commit[]>([]);
const selectedId = ref<string | null>(null);
const detail = ref<CommitDetail | null>(null);
const loading = ref(false);
const loadingMore = ref(false);
const endReached = ref(false);

const PAGE_SIZE = 100;

const unpushedCount = computed(
  () => commits.value.filter((c) => c.is_pushed === false).length,
);

const { gridTemplate, beginResize, resetColumn } = useResizableColumns(
  'gitdash:git-graph-panel-cols',
  [
    { id: 'msg', defaultTrack: 'minmax(0, 1fr)', min: 120, max: 800 },
    { id: 'status', defaultTrack: 'auto', min: 56, max: 160 },
    { id: 'author', defaultTrack: 'minmax(90px, max-content)', min: 64, max: 240 },
    { id: 'date', defaultTrack: 'minmax(76px, max-content)', min: 64, max: 180 },
    { id: 'id', defaultTrack: '64px', min: 56, max: 140 },
  ],
);

const branchOptions = computed(() =>
  branches.value.map((b) => ({ label: b.display_name, value: b.name })),
);

async function loadBranches() {
  branches.value = await getBranches(props.projectId);
  const cur = branches.value.find((b) => b.is_current);
  currentBranch.value = cur ? cur.name : branches.value[0]?.name ?? '';
}

function onBranchChange(value: string) {
  currentBranch.value = value;
  void loadCommits();
}

async function loadCommits() {
  if (!currentBranch.value) return;
  loading.value = true;
  endReached.value = false;
  try {
    commits.value = await getCommits(props.projectId, currentBranch.value, PAGE_SIZE);
    selectedId.value = null;
    detail.value = null;
    if (commits.value.length < PAGE_SIZE) endReached.value = true;
  } finally {
    loading.value = false;
  }
}

async function loadMore() {
  if (loadingMore.value || endReached.value || loading.value) return;
  if (!commits.value.length) return;
  const last = commits.value[commits.value.length - 1];
  loadingMore.value = true;
  try {
    const more = await getCommits(props.projectId, currentBranch.value, PAGE_SIZE, last.id);
    const seen = new Set(commits.value.map((c) => c.id));
    for (const c of more) if (!seen.has(c.id)) commits.value.push(c);
    if (more.length < PAGE_SIZE) endReached.value = true;
  } finally {
    loadingMore.value = false;
  }
}

function onScroll(e: Event) {
  const el = e.target as HTMLElement;
  const remaining = el.scrollHeight - el.scrollTop - el.clientHeight;
  if (remaining < 240) void loadMore();
}

async function onSelect(id: string) {
  selectedId.value = id;
  try {
    detail.value = await getCommitDetail(props.projectId, id);
  } catch {
    detail.value = null;
  }
}

function statusVariant(s: string): 'success' | 'danger' | 'warning' | 'info' {
  if (s.startsWith('A')) return 'success';
  if (s.startsWith('D')) return 'danger';
  if (s.startsWith('M')) return 'warning';
  return 'info';
}

function firstLine(msg: string): string {
  return msg.split('\n')[0] || '(无说明)';
}

function pad(n: number): string {
  return n < 10 ? '0' + n : '' + n;
}

function formatDate(ts: number): string {
  // 后端返回 unix 秒
  const d = new Date(ts * 1000);
  const now = new Date();
  const sameYear = d.getFullYear() === now.getFullYear();
  if (sameYear) {
    return `${pad(d.getMonth() + 1)}/${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
  }
  return `${d.getFullYear()}/${pad(d.getMonth() + 1)}/${pad(d.getDate())}`;
}

onMounted(async () => {
  await loadBranches();
  await loadCommits();
});
</script>

<style scoped>
.git-body {
  background-color: var(--background);
}

.git-rail {
  padding: 0 2px;
}

.git-list {
  min-width: 0;
}

.git-rail-col {
  padding: 0 2px;
}
.git-head-rail {
  position: sticky;
  top: 0;
  z-index: 2;
  height: 30px;
  background-color: var(--background);
  border-bottom: 1px solid var(--border);
}
.git-head {
  position: sticky;
  top: 0;
  z-index: 2;
  height: 30px;
  display: grid;
  align-items: center;
  gap: 12px;
  padding: 0 12px;
  font-size: 12px;
  font-weight: 600;
  color: var(--muted-foreground);
  background-color: var(--background);
  border-bottom: 1px solid var(--border);
}
.git-row {
  height: 28px;
  display: grid;
  align-items: center;
  gap: 12px;
  padding: 0 12px;
  cursor: pointer;
  border-bottom: 1px solid var(--border);
}

.git-row:hover {
  background-color: var(--accent);
}

.git-row.active {
  background-color: color-mix(in oklab, var(--primary) 14%, transparent);
}

.git-row-msg {
  font-size: 13px;
  color: var(--foreground);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
}

.git-author {
  font-size: 11px;
  color: var(--muted-foreground);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
}

.git-date {
  font-size: 11px;
  color: var(--muted-foreground);
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
  text-align: left;
}

.git-hash {
  font-family: var(--font-mono, monospace);
  font-size: 11px;
  color: var(--muted-foreground);
  white-space: nowrap;
  text-align: left;
}

.git-badge-cell {
  display: flex;
  align-items: center;
  min-width: 0;
}

.git-empty {
  padding: 24px;
  text-align: center;
  color: var(--muted-foreground);
  font-size: 13px;
}

.git-foot {
  padding: 10px 12px;
  text-align: center;
  font-size: 12px;
  color: var(--muted-foreground);
}
.git-foot--muted {
  opacity: 0.7;
}

.git-loading {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: color-mix(in oklab, var(--background) 70%, transparent);
}

.git-detail {
  background-color: var(--card);
}

.detail-title {
  font-size: 13px;
  font-weight: 600;
  margin-bottom: 6px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.detail-author {
  font-weight: 400;
  color: var(--muted-foreground);
}

.detail-body {
  white-space: pre-wrap;
  font-size: 12px;
  color: var(--foreground);
  margin: 0 0 8px;
}

.detail-files {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
}

.file-status {
  flex-shrink: 0;
  min-width: 36px;
  justify-content: center;
}

.file-path {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
