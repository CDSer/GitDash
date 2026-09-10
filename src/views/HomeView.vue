<!--
  主页
  默认入口：全局统计 + 需要关注 + 活跃概览 + 最近动态 + 常用项目
-->
<template>
  <div class="home-page">
    <div class="home-scroll">
      <!-- 页头 -->
      <div class="home-header">
        <div>
          <h1 class="home-title">主页</h1>
          <p class="home-subtitle">今天有什么要处理</p>
        </div>
        <div class="home-actions">
          <Button
            variant="outline"
            :disabled="operationStore.isQueueRunning || projects.length === 0"
            @click="fetchAll"
          >
            <RefreshCw :size="14" :class="{ spinning: operationStore.isQueueRunning }" />
            立即获取全部
          </Button>
          <Button variant="primary" @click="showAddModal = true">
            <Plus :size="14" /> 添加项目
          </Button>
        </div>
      </div>

      <!-- 统计行 -->
      <div class="stats-row">
        <div class="stat-card">
          <div class="stat-label">项目</div>
          <div class="stat-value">{{ projects.length }}</div>
        </div>
        <div class="stat-card">
          <div class="stat-label">脏仓库</div>
          <div class="stat-value stat-value--warn">{{ dirtyCount }}</div>
        </div>
        <div class="stat-card">
          <div class="stat-label">冲突 / 进行中</div>
          <div class="stat-value" :class="attentionCount > 0 ? 'stat-value--danger' : ''">
            {{ attentionCount }}
          </div>
        </div>
        <div class="stat-card">
          <div class="stat-label">上次全量获取</div>
          <div class="stat-value stat-value--sm">{{ lastFetchLabel }}</div>
        </div>
      </div>

      <!-- 需要关注 + 活跃概览 -->
      <div class="mid-row">
        <section class="panel attention-panel">
          <div class="panel-head">
            <h2 class="panel-title">需要关注</h2>
            <span v-if="attentionItems.length" class="panel-count">{{ attentionItems.length }}</span>
          </div>

          <div v-if="loadingStatuses && !attentionItems.length" class="panel-empty">
            <Spinner />
          </div>
          <div v-else-if="!attentionItems.length" class="panel-empty panel-empty--ok">
            <CheckCircle2 :size="18" />
            <span>全部已处理，可打开 Workspace 继续工作</span>
          </div>

          <div v-else class="attention-list">
            <button
              v-for="item in attentionItems"
              :key="item.project.id"
              type="button"
              class="attention-row"
              :class="`attention-row--${item.kind}`"
              @click="onAttentionClick(item)"
            >
              <span class="attention-dot" />
              <div class="attention-body">
                <div class="attention-name">{{ item.project.name }}</div>
                <div class="attention-desc">{{ item.description }}</div>
              </div>
              <span class="attention-action" :class="`attention-action--${item.kind}`">
                {{ item.actionLabel }}
              </span>
            </button>
          </div>
        </section>

        <section class="panel activity-panel">
          <div class="panel-head">
            <h2 class="panel-title">活跃概览</h2>
            <Button
              variant="ghost"
              size="sm"
              class="ml-auto"
              :disabled="loadingActivity"
              @click="loadActivity"
            >
              <RefreshCw :size="13" :class="{ spinning: loadingActivity }" />
            </Button>
          </div>
          <p class="activity-summary">
            今日提交 <strong>{{ todayCommitCount }}</strong>
            <span class="activity-muted"> · 近 7 天趋势</span>
          </p>

          <div class="bars" aria-label="近 7 天提交趋势">
            <div v-for="bar in weekBars" :key="bar.label" class="bar-col">
              <div class="bar-track">
                <div
                  class="bar-fill"
                  :class="{ 'bar-fill--today': bar.isToday }"
                  :style="{ height: bar.height + '%' }"
                  :title="`${bar.label}：${bar.count} 次提交`"
                />
              </div>
              <span class="bar-label">{{ bar.label }}</span>
            </div>
          </div>

          <div v-if="aheadBehindChips.length" class="chips">
            <span v-for="chip in aheadBehindChips" :key="chip.id" class="chip">
              {{ chip.text }}
            </span>
          </div>
          <div v-else class="panel-empty panel-empty--inline">暂无 ahead / behind 差异</div>
        </section>
      </div>

      <!-- 最近动态 + 常用项目 -->
      <div class="bottom-row">
        <section class="panel">
          <div class="panel-head">
            <h2 class="panel-title">最近动态</h2>
          </div>
          <div v-if="loadingActivity && !recentCommits.length" class="panel-empty">
            <Spinner />
          </div>
          <div v-else-if="!recentCommits.length" class="panel-empty">
            暂无提交动态
          </div>
          <div v-else class="commit-list">
            <button
              v-for="item in recentCommits"
              :key="`${item.projectId}-${item.commit.id}`"
              type="button"
              class="commit-row"
              :title="`${item.projectName} · ${item.commit.message}`"
              @click="openHistoryById(item.projectId)"
            >
              <span class="commit-project">{{ item.projectName }}</span>
              <span class="commit-sha">{{ item.commit.short_id }}</span>
              <span class="commit-msg">{{ item.commit.message }}</span>
              <span class="commit-time">{{ formatRelative(item.commit.date) }}</span>
            </button>
            <button type="button" class="commit-more" @click="goProjects">
              查看全部项目…
            </button>
          </div>
        </section>

        <section class="panel">
          <div class="panel-head">
            <h2 class="panel-title">常用项目</h2>
          </div>
          <div v-if="!favoriteProjects.length" class="panel-empty">
            <span>暂无项目</span>
            <Button size="sm" variant="outline" class="mt-1" @click="showAddModal = true">
              <Plus :size="13" /> 添加项目
            </Button>
          </div>
          <div v-else class="fav-grid">
            <button
              v-for="p in favoriteProjects"
              :key="p.id"
              type="button"
              class="fav-card"
              @click="openHistoryById(p.id)"
            >
              <div class="fav-top">
                <span class="fav-name">{{ p.name }}</span>
                <span class="fav-dot" :class="`fav-dot--${favState(p).tone}`" />
              </div>
              <div class="fav-meta">{{ favState(p).text }}</div>
            </button>
            <button type="button" class="fav-card fav-card--add" @click="goProjects">
              <Plus :size="16" />
              <span>打开项目</span>
            </button>
          </div>
        </section>
      </div>
    </div>

    <AddProjectModal v-model="showAddModal" />
    <SourceControlModal v-model="showSourceControl" :project="sourceControlProject" />
  </div>
</template>

<script setup lang="ts">
import { computed, defineAsyncComponent, onMounted, ref } from 'vue';
import { Plus, RefreshCw, CheckCircle2 } from 'lucide-vue-next';
import { useRouter } from 'vue-router';
import type { Commit, Project, ProjectStatus } from '../types';
import { useAppStore } from '../stores/appStore';
import { useOperationStore } from '../stores/operationStore';
import { useTabStore } from '../stores/tabStore';
import { useProjectStatus } from '../composables/useProjectStatus';
import { getCommits } from '../lib/tauriApi';
import { toast } from '../lib/toast';
import Button from '../components/ui/Button.vue';
import Spinner from '../components/ui/Spinner.vue';

const AddProjectModal = defineAsyncComponent(
  () => import('../components/Modals/AddProjectModal.vue'),
);
const SourceControlModal = defineAsyncComponent(
  () => import('../components/Modals/SourceControlModal.vue'),
);

const appStore = useAppStore();
const operationStore = useOperationStore();
const tabStore = useTabStore();
const { getStatus } = useProjectStatus();
const router = useRouter();

const showAddModal = ref(false);
const showSourceControl = ref(false);
const sourceControlProject = ref<Project | null>(null);
const loadingActivity = ref(false);
const loadingStatuses = ref(false);

const STORAGE_LAST_FETCH = 'gitdash:last-batch-fetch';
const STORAGE_FAVORITES = 'gitdash:recent-projects';
const MAX_RECENT_COMMITS = 12;
const MAX_FAVORITES = 3;
/** 单次从每个仓库拉取的提交条数（用于动态聚合） */
const COMMITS_PER_REPO = 30;
/** 首页拉取动态时最多并行的仓库数 */
const ACTIVITY_REPO_LIMIT = 12;

const projects = computed(() => appStore.projects);
const statuses = computed(() => appStore.statuses);

const recentCommits = ref<Array<{ projectId: string; projectName: string; commit: Commit }>>([]);
const lastBatchFetchAt = ref<number | null>(null);

const dirtyCount = computed(() =>
  projects.value.filter((p) => {
    const s = statuses.value.get(p.id);
    return s && !s.is_clean;
  }).length,
);

const attentionItems = computed(() => {
  const items: Array<{
    project: Project;
    kind: 'conflict' | 'in-progress' | 'error';
    description: string;
    actionLabel: string;
  }> = [];

  for (const p of projects.value) {
    const s = statuses.value.get(p.id);
    if (!s) continue;
    if (s.error) {
      items.push({
        project: p,
        kind: 'error',
        description: s.error,
        actionLabel: '重试',
      });
      continue;
    }
    if (s.conflict_count > 0) {
      items.push({
        project: p,
        kind: 'conflict',
        description: `merge 冲突未解决 · ${s.conflict_count} 个文件`,
        actionLabel: '处理',
      });
      continue;
    }
    if (s.in_progress) {
      const kind = s.in_progress.kind;
      const label =
        kind === 'merge'
          ? '合并进行中'
          : kind === 'rebase'
            ? 'rebase 进行中'
            : kind === 'cherry-pick'
              ? 'cherry-pick 进行中'
              : kind === 'revert'
                ? 'revert 进行中'
                : `${kind} 进行中`;
      items.push({
        project: p,
        kind: 'in-progress',
        description: `${label} · 待 continue`,
        actionLabel: '继续',
      });
    }
  }

  // 失败的批量任务（不在冲突/进行中列表里的补充）
  for (const t of operationStore.failedTasks) {
    if (t.message === '已取消') continue;
    if (items.some((i) => i.project.id === t.projectId)) continue;
    const p = projects.value.find((x) => x.id === t.projectId);
    if (!p) continue;
    items.push({
      project: p,
      kind: 'error',
      description: `${t.operation} 失败${t.message ? ` · ${truncate(t.message, 48)}` : ''}`,
      actionLabel: '重试',
    });
  }

  return items;
});

const attentionCount = computed(() => attentionItems.value.length);

const lastFetchLabel = computed(() => {
  if (lastBatchFetchAt.value) return formatRelative(lastBatchFetchAt.value);
  // 回退：各仓库 last_fetched 最大值
  let maxTs = 0;
  for (const s of statuses.value.values()) {
    if (s.last_fetched && s.last_fetched > maxTs) maxTs = s.last_fetched;
  }
  if (maxTs) return formatRelative(maxTs);
  return '—';
});

const todayCommitCount = computed(() => {
  const start = startOfDay(Date.now());
  return recentCommits.value.filter((r) => r.commit.date * 1000 >= start).length;
});

const weekBars = computed(() => {
  const days: Array<{ label: string; count: number; isToday: boolean }> = [];
  const now = new Date();
  const weekday = ['日', '一', '二', '三', '四', '五', '六'];
  for (let i = 6; i >= 0; i--) {
    const d = new Date(now);
    d.setDate(now.getDate() - i);
    const dayStart = startOfDay(d.getTime());
    const dayEnd = dayStart + 86_400_000;
    const count = recentCommits.value.filter((r) => {
      const t = r.commit.date * 1000;
      return t >= dayStart && t < dayEnd;
    }).length;
    days.push({
      label: weekday[d.getDay()],
      count,
      isToday: i === 0,
    });
  }
  const max = Math.max(1, ...days.map((d) => d.count));
  return days.map((d) => ({
    ...d,
    height: Math.max(d.count > 0 ? 12 : 4, Math.round((d.count / max) * 100)),
  }));
});

const aheadBehindChips = computed(() => {
  const chips: Array<{ id: string; text: string }> = [];
  for (const p of projects.value) {
    const s = statuses.value.get(p.id);
    if (!s) continue;
    const parts: string[] = [];
    if (s.ahead > 0) parts.push(`领先 ${s.ahead}`);
    if (s.behind > 0) parts.push(`落后 ${s.behind}`);
    if (parts.length) {
      chips.push({ id: p.id, text: `${p.name} ${parts.join(' · ')}` });
    }
  }
  return chips.slice(0, 6);
});

const favoriteProjects = computed(() => {
  const byId = new Map(projects.value.map((p) => [p.id, p]));
  const ordered: Project[] = [];
  const seen = new Set<string>();

  for (const id of loadFavoriteIds()) {
    const p = byId.get(id);
    if (p && !seen.has(p.id)) {
      ordered.push(p);
      seen.add(p.id);
    }
  }
  // 脏仓库优先补位
  for (const p of projects.value) {
    if (ordered.length >= MAX_FAVORITES) break;
    if (seen.has(p.id)) continue;
    const s = statuses.value.get(p.id);
    if (s && (!s.is_clean || s.conflict_count > 0 || s.in_progress)) {
      ordered.push(p);
      seen.add(p.id);
    }
  }
  // 仍不够则按创建时间
  for (const p of [...projects.value].sort((a, b) => b.created_at - a.created_at)) {
    if (ordered.length >= MAX_FAVORITES) break;
    if (seen.has(p.id)) continue;
    ordered.push(p);
    seen.add(p.id);
  }
  return ordered.slice(0, MAX_FAVORITES);
});

function startOfDay(ts: number): number {
  const d = new Date(ts);
  d.setHours(0, 0, 0, 0);
  return d.getTime();
}

function truncate(s: string, n: number): string {
  return s.length > n ? `${s.slice(0, n)}…` : s;
}

function formatRelative(ts: number): string {
  // commit.date 是秒；last_fetched / 本地记录是毫秒
  const ms = ts > 1e12 ? ts : ts * 1000;
  const diff = Date.now() - ms;
  if (diff < 60_000) return '刚刚';
  if (diff < 3_600_000) return `${Math.floor(diff / 60_000)} 分钟前`;
  if (diff < 86_400_000) return `${Math.floor(diff / 3_600_000)} 小时前`;
  if (diff < 7 * 86_400_000) return `${Math.floor(diff / 86_400_000)} 天前`;
  const d = new Date(ms);
  return `${d.getMonth() + 1}/${d.getDate()}`;
}

function loadFavoriteIds(): string[] {
  try {
    const raw = localStorage.getItem(STORAGE_FAVORITES);
    if (!raw) return [];
    const arr = JSON.parse(raw);
    return Array.isArray(arr) ? arr.filter((x): x is string => typeof x === 'string') : [];
  } catch {
    return [];
  }
}

function rememberFavorite(projectId: string) {
  const ids = loadFavoriteIds().filter((id) => id !== projectId);
  ids.unshift(projectId);
  localStorage.setItem(STORAGE_FAVORITES, JSON.stringify(ids.slice(0, 8)));
}

function statusOf(projectId: string): ProjectStatus | null {
  return statuses.value.get(projectId) ?? null;
}

function favState(p: Project): { tone: 'danger' | 'warn' | 'ok' | 'info'; text: string } {
  const s = statusOf(p.id);
  if (!s) return { tone: 'info', text: '状态未知' };
  if (s.error) return { tone: 'danger', text: truncate(s.error, 28) };
  if (s.conflict_count > 0) return { tone: 'danger', text: `有冲突 · ${s.branch}` };
  if (s.in_progress) return { tone: 'warn', text: `${s.in_progress.kind} 中 · ${s.branch}` };
  if (!s.is_clean) return { tone: 'warn', text: `有变更 · ${s.branch}` };
  if (s.ahead > 0 || s.behind > 0) {
    const parts: string[] = [];
    if (s.ahead > 0) parts.push(`领先 ${s.ahead}`);
    if (s.behind > 0) parts.push(`落后 ${s.behind}`);
    return { tone: 'info', text: `${parts.join(' · ')} · ${s.branch}` };
  }
  return { tone: 'ok', text: `干净 · ${s.branch}` };
}

function onAttentionClick(item: { project: Project; kind: string }) {
  rememberFavorite(item.project.id);
  if (item.kind === 'conflict' || item.kind === 'in-progress') {
    sourceControlProject.value = item.project;
    showSourceControl.value = true;
    return;
  }
  // error：引导去项目列表重试批量操作
  goProjects();
}

function openHistoryById(projectId: string) {
  rememberFavorite(projectId);
  tabStore.openProject(projectId, 'history');
}

function goProjects() {
  tabStore.setActive(null);
  router.push({ name: 'projects' });
}

async function fetchAll() {
  if (!projects.value.length) return;
  const ids = projects.value.map((p) => p.id);
  try {
    await operationStore.batchFetch(ids);
    lastBatchFetchAt.value = Date.now();
    localStorage.setItem(STORAGE_LAST_FETCH, String(lastBatchFetchAt.value));
    toast.success(`已获取全部（${ids.length} 个仓库）`);
    await Promise.all(ids.map((id) => getStatus(id, true)));
    await loadActivity();
  } catch {
    toast.error('批量获取失败');
  }
}

async function loadActivity() {
  if (loadingActivity.value) return;
  loadingActivity.value = true;
  try {
    const candidates = pickActivityProjects();
    const settled = await Promise.allSettled(
      candidates.map(async (p) => {
        const s = statusOf(p.id);
        const branch = s && !s.is_detached && s.branch ? s.branch : 'HEAD';
        const commits = await getCommits(p.id, branch, COMMITS_PER_REPO);
        return { project: p, commits };
      }),
    );

    const merged: Array<{ projectId: string; projectName: string; commit: Commit }> = [];
    for (const r of settled) {
      if (r.status !== 'fulfilled') continue;
      for (const c of r.value.commits) {
        merged.push({
          projectId: r.value.project.id,
          projectName: r.value.project.name,
          commit: c,
        });
      }
    }
    merged.sort((a, b) => b.commit.date - a.commit.date);
    recentCommits.value = merged.slice(0, MAX_RECENT_COMMITS);
  } catch (error) {
    console.error('加载动态失败：', error);
  } finally {
    loadingActivity.value = false;
  }
}

function pickActivityProjects(): Project[] {
  const scored = projects.value.map((p) => {
    const s = statusOf(p.id);
    let score = 0;
    if (s) {
      if (s.conflict_count > 0) score += 100;
      if (s.in_progress) score += 80;
      if (!s.is_clean) score += 50;
      if (s.ahead || s.behind) score += 20;
      if (s.last_fetched) score += Math.min(15, (Date.now() - s.last_fetched) < 3_600_000 ? 15 : 5);
    }
    // 近期打开过
    if (loadFavoriteIds().includes(p.id)) score += 30;
    return { p, score };
  });
  scored.sort((a, b) => b.score - a.score);
  // 有状态优先；不够再补其余
  const preferred = scored.filter((x) => statusOf(x.p.id)).map((x) => x.p);
  const rest = scored.filter((x) => !statusOf(x.p.id)).map((x) => x.p);
  return [...preferred, ...rest].slice(0, ACTIVITY_REPO_LIMIT);
}

onMounted(async () => {
  const saved = localStorage.getItem(STORAGE_LAST_FETCH);
  if (saved) {
    const n = Number(saved);
    if (Number.isFinite(n) && n > 0) lastBatchFetchAt.value = n;
  }

  // AppLayout 已启动轮询；此处确保首页有初始状态再画「需要关注」
  loadingStatuses.value = projects.value.length > 0;
  try {
    await Promise.all(projects.value.map((p) => getStatus(p.id, false)));
  } finally {
    loadingStatuses.value = false;
  }

  await loadActivity();
});
</script>

<style scoped>
.home-page {
  display: flex;
  height: 100%;
  flex-direction: column;
  background-color: var(--background);
}
.home-scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 20px 24px 28px;
}
.home-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 16px;
}
.home-title {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
}
.home-subtitle {
  margin: 4px 0 0;
  font-size: 12px;
  color: var(--muted-foreground);
}
.home-actions {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  gap: 8px;
}
.spinning {
  animation: spin 1s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.stats-row {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
  margin-bottom: 12px;
}
.stat-card {
  border: 1px solid var(--border);
  border-radius: 10px;
  background-color: var(--card);
  padding: 12px 14px;
}
.stat-label {
  font-size: 11px;
  color: var(--muted-foreground);
}
.stat-value {
  margin-top: 6px;
  font-size: 24px;
  font-weight: 600;
  line-height: 1.15;
  color: var(--foreground);
}
.stat-value--sm {
  font-size: 18px;
}
.stat-value--warn {
  color: oklch(0.7 0.16 75);
}
.stat-value--danger {
  color: var(--destructive);
}

.mid-row,
.bottom-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  margin-bottom: 12px;
}
.bottom-row {
  margin-bottom: 0;
}

.panel {
  border: 1px solid var(--border);
  border-radius: 10px;
  background-color: var(--card);
  padding: 12px 14px 14px;
  min-height: 0;
}
.panel-head {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
}
.panel-title {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
}
.panel-count {
  font-size: 11px;
  color: var(--muted-foreground);
}
.panel-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  min-height: 100px;
  font-size: 12px;
  color: var(--muted-foreground);
}
.panel-empty--ok {
  flex-direction: row;
  min-height: 160px;
  color: oklch(0.6 0.14 150);
}
.panel-empty--inline {
  min-height: 40px;
  margin-top: 8px;
}

.attention-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.attention-row {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  border: 1px solid var(--border);
  border-radius: 8px;
  background-color: var(--accent);
  padding: 10px 12px;
  text-align: left;
  cursor: pointer;
  transition: border-color 0.15s ease, background-color 0.15s ease;
}
.attention-row:hover {
  border-color: color-mix(in oklab, var(--primary) 40%, var(--border));
}
.attention-row--conflict {
  border-color: color-mix(in oklab, var(--destructive) 35%, var(--border));
}
.attention-row--in-progress {
  border-color: color-mix(in oklab, oklch(0.8 0.16 85) 40%, var(--border));
}
.attention-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
  background-color: var(--muted-foreground);
}
.attention-row--conflict .attention-dot {
  background-color: var(--destructive);
}
.attention-row--in-progress .attention-dot {
  background-color: oklch(0.75 0.16 85);
}
.attention-row--error .attention-dot {
  background-color: oklch(0.6 0.15 40);
}
.attention-body {
  flex: 1;
  min-width: 0;
}
.attention-name {
  font-size: 12px;
  font-weight: 500;
  color: var(--foreground);
}
.attention-desc {
  margin-top: 2px;
  font-size: 11px;
  color: var(--muted-foreground);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.attention-action {
  flex-shrink: 0;
  border-radius: 6px;
  padding: 4px 10px;
  font-size: 11px;
  font-weight: 500;
  color: var(--muted-foreground);
  background-color: var(--muted);
}
.attention-action--conflict {
  background-color: var(--primary);
  color: var(--primary-foreground);
}

.activity-summary {
  margin: 0 0 12px;
  font-size: 12px;
  color: var(--foreground);
}
.activity-summary strong {
  font-weight: 600;
}
.activity-muted {
  color: var(--muted-foreground);
}
.bars {
  display: flex;
  align-items: flex-end;
  gap: 12px;
  height: 110px;
  padding: 0 4px;
}
.bar-col {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  height: 100%;
}
.bar-track {
  flex: 1;
  width: 28px;
  display: flex;
  align-items: flex-end;
}
.bar-fill {
  width: 100%;
  border-radius: 4px;
  background-color: color-mix(in oklab, var(--primary) 35%, transparent);
  min-height: 4px;
  transition: height 0.25s ease;
}
.bar-fill--today {
  background-color: var(--primary);
}
.bar-label {
  font-size: 10px;
  color: var(--muted-foreground);
}
.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 12px;
}
.chip {
  border-radius: 999px;
  background-color: var(--muted);
  padding: 4px 10px;
  font-size: 10px;
  color: var(--foreground);
}

.commit-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.commit-row {
  display: grid;
  grid-template-columns: 88px 64px 1fr auto;
  gap: 8px;
  align-items: center;
  width: 100%;
  border-radius: 6px;
  padding: 7px 8px;
  text-align: left;
  cursor: pointer;
  color: var(--foreground);
}
.commit-row:hover {
  background-color: var(--accent);
}
.commit-project {
  font-size: 11px;
  color: var(--muted-foreground);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.commit-sha {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 11px;
  color: var(--primary);
}
.commit-msg {
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--foreground);
}
.commit-time {
  font-size: 11px;
  color: var(--muted-foreground);
  white-space: nowrap;
}
.commit-more {
  margin-top: 4px;
  padding: 6px 8px;
  border-radius: 6px;
  font-size: 11px;
  color: var(--muted-foreground);
  text-align: left;
}
.commit-more:hover {
  background-color: var(--accent);
  color: var(--foreground);
}

.fav-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 10px;
}
.fav-card {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-height: 72px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background-color: var(--accent);
  padding: 12px;
  text-align: left;
  cursor: pointer;
  transition: border-color 0.15s ease;
}
.fav-card:hover {
  border-color: color-mix(in oklab, var(--primary) 40%, var(--border));
}
.fav-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.fav-name {
  font-size: 12px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.fav-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
  background-color: var(--muted-foreground);
}
.fav-dot--danger {
  background-color: var(--destructive);
}
.fav-dot--warn {
  background-color: oklch(0.75 0.16 85);
}
.fav-dot--ok {
  background-color: oklch(0.65 0.16 150);
}
.fav-dot--info {
  background-color: var(--primary);
}
.fav-meta {
  font-size: 10px;
  color: var(--muted-foreground);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.fav-card--add {
  align-items: center;
  justify-content: center;
  flex-direction: row;
  gap: 6px;
  border-style: dashed;
  color: var(--muted-foreground);
  font-size: 12px;
  background-color: transparent;
}

@media (max-width: 960px) {
  .stats-row,
  .mid-row,
  .bottom-row {
    grid-template-columns: 1fr;
  }
  .commit-row {
    grid-template-columns: 72px 56px 1fr;
  }
  .commit-time {
    display: none;
  }
}
</style>
