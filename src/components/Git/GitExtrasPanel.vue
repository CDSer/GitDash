<!--
  Git 增强操作：分支 CRUD / Stash / Tag / 两分支对比 / Rebase
-->
<template>
  <Dialog v-model="visible" :title="`Git 操作 · ${project?.name ?? ''}`" width="720px">
    <div v-if="project" class="ge">
      <div class="ge-tabs">
        <button
          v-for="t in tabs"
          :key="t.id"
          type="button"
          :class="['ge-tab', activeTab === t.id ? 'ge-tab--active' : '']"
          @click="activeTab = t.id"
        >
          {{ t.label }}
        </button>
      </div>

      <!-- 分支 -->
      <div v-if="activeTab === 'branch'" class="ge-pane">
        <div class="ge-section">
          <div class="ge-section-title">新建分支</div>
          <div class="ge-row">
            <Input v-model="newBranchName" placeholder="分支名" class="ge-grow" :disabled="busy" />
            <select v-model="newBranchStart" class="ge-select" :disabled="busy">
              <option value="">当前 HEAD</option>
              <option v-for="b in branches" :key="'s-' + b.name" :value="b.name">
                {{ b.display_name }}
              </option>
            </select>
            <label class="ge-check">
              <input v-model="newBranchCheckout" type="checkbox" :disabled="busy" /> 切换
            </label>
            <label v-if="newBranchStart.includes('/')" class="ge-check">
              <input v-model="newBranchTrack" type="checkbox" :disabled="busy" /> track
            </label>
            <Button size="sm" variant="primary" :disabled="busy || !newBranchName" @click="createBranch">
              创建
            </Button>
          </div>
        </div>

        <div class="ge-section">
          <div class="ge-section-title">删除分支</div>
          <div class="ge-row">
            <select v-model="deleteBranch" class="ge-select ge-grow" :disabled="busy">
              <option value="" disabled>选择分支</option>
              <option v-for="b in deletableBranches" :key="'d-' + b.name" :value="b.name">
                {{ b.display_name }}
              </option>
            </select>
            <label class="ge-check">
              <input v-model="deleteBranchForce" type="checkbox" :disabled="busy" /> 强制 (-D)
            </label>
            <label class="ge-check">
              <input v-model="deleteBranchRemote" type="checkbox" :disabled="busy" /> 远程
            </label>
            <Button
              size="sm"
              variant="ghost"
              :disabled="busy || !deleteBranch || deleteBranch === currentBranch"
              @click="deleteBranchAction"
            >
              删除
            </Button>
          </div>
        </div>

        <div class="ge-section">
          <div class="ge-section-title">设置上游</div>
          <div class="ge-row">
            <select v-model="upstreamBranch" class="ge-select" :disabled="busy">
              <option value="">当前分支</option>
              <option v-for="b in localBranches" :key="'u-' + b.name" :value="b.name">
                {{ b.display_name }}
              </option>
            </select>
            <select v-model="upstreamRemote" class="ge-select ge-grow" :disabled="busy">
              <option value="" disabled>远程分支</option>
              <option v-for="b in remoteBranches" :key="'r-' + b.name" :value="b.display_name">
                {{ b.display_name }}
              </option>
            </select>
            <Button
              size="sm"
              variant="ghost"
              :disabled="busy || !upstreamRemote"
              @click="setUpstream"
            >
              设置
            </Button>
          </div>
        </div>

        <div class="ge-section">
          <div class="ge-section-title">变基（当前分支 → onto）</div>
          <div class="ge-row">
            <select v-model="rebaseOnto" class="ge-select ge-grow" :disabled="busy">
              <option value="" disabled>选择目标分支</option>
              <option
                v-for="b in localBranches.filter((x) => !x.is_current)"
                :key="'rb-' + b.name"
                :value="b.name"
              >
                {{ b.display_name }}
              </option>
            </select>
            <Button
              size="sm"
              variant="ghost"
              :disabled="busy || !rebaseOnto || !!inProgress"
              @click="startRebase"
            >
              变基
            </Button>
          </div>
        </div>
      </div>

      <!-- Stash -->
      <div v-else-if="activeTab === 'stash'" class="ge-pane">
        <div class="ge-section">
          <div class="ge-section-title">新建 Stash</div>
          <div class="ge-row">
            <Input
              v-model="stashMessage"
              placeholder="说明（可选）"
              class="ge-grow"
              :disabled="busy"
            />
            <label class="ge-check">
              <input v-model="stashUntracked" type="checkbox" :disabled="busy" /> 含未跟踪
            </label>
            <Button size="sm" variant="primary" :disabled="busy" @click="pushStash">
              保存
            </Button>
          </div>
        </div>
        <div class="ge-section ge-section--fill">
          <div class="ge-section-title">Stash 列表（{{ stashes.length }}）</div>
          <div class="ge-list">
            <div v-for="s in stashes" :key="s.index" class="ge-item">
              <span class="ge-item-title" :title="s.message">
                stash@{{ s.index }} · {{ s.message }}
              </span>
              <span class="ge-item-sha font-mono">{{ s.sha.slice(0, 7) }}</span>
              <div class="ge-item-actions">
                <Button size="sm" variant="ghost" :disabled="busy" @click="applyStash(s.index)">
                  应用
                </Button>
                <Button size="sm" variant="ghost" :disabled="busy" @click="popStash(s.index)">
                  弹出
                </Button>
                <Button size="sm" variant="ghost" :disabled="busy" @click="dropStash(s.index)">
                  丢弃
                </Button>
              </div>
            </div>
            <Empty v-if="!stashes.length" description="暂无 Stash" />
          </div>
        </div>
      </div>

      <!-- Tag -->
      <div v-else-if="activeTab === 'tag'" class="ge-pane">
        <div class="ge-section">
          <div class="ge-section-title">创建 Tag</div>
          <div class="ge-row">
            <Input v-model="tagName" placeholder="tag 名" class="ge-grow" :disabled="busy" />
            <Input
              v-model="tagMessage"
              placeholder="注释（可选）"
              class="ge-grow"
              :disabled="busy"
            />
            <select v-model="tagCommit" class="ge-select" :disabled="busy">
              <option value="">HEAD</option>
              <option v-for="c in recentCommits" :key="'tc-' + c.id" :value="c.id">
                {{ c.short_id }}
              </option>
            </select>
            <Button size="sm" variant="primary" :disabled="busy || !tagName" @click="createTag">
              创建
            </Button>
          </div>
          <div class="ge-row">
            <Button size="sm" variant="ghost" :disabled="busy" @click="pushAllTags">
              推送全部 Tag
            </Button>
          </div>
        </div>
        <div class="ge-section ge-section--fill">
          <div class="ge-section-title">Tag 列表（{{ tags.length }}）</div>
          <div class="ge-list">
            <div v-for="t in tags" :key="t.name" class="ge-item">
              <span class="ge-item-title" :title="t.message ?? ''">{{ t.name }}</span>
              <span class="ge-item-sha font-mono">{{ t.sha.slice(0, 7) }}</span>
              <div class="ge-item-actions">
                <Button size="sm" variant="ghost" :disabled="busy" @click="pushTag(t.name)">
                  推送
                </Button>
                <Button size="sm" variant="ghost" :disabled="busy" @click="deleteTag(t.name)">
                  删除
                </Button>
              </div>
            </div>
            <Empty v-if="!tags.length" description="暂无 Tag" />
          </div>
        </div>
      </div>

      <!-- 对比 -->
      <div v-else class="ge-pane">
        <div class="ge-section">
          <div class="ge-section-title">两分支对比</div>
          <div class="ge-row">
            <select v-model="compareBase" class="ge-select" :disabled="busy">
              <option value="" disabled>base</option>
              <option v-for="b in branches" :key="'cb-' + b.name" :value="b.name">
                {{ b.display_name }}
              </option>
            </select>
            <select v-model="compareHead" class="ge-select" :disabled="busy">
              <option value="" disabled>head</option>
              <option v-for="b in branches" :key="'ch-' + b.name" :value="b.name">
                {{ b.display_name }}
              </option>
            </select>
            <Button
              size="sm"
              variant="primary"
              :disabled="busy || !compareBase || !compareHead || compareBase === compareHead"
              @click="runCompare"
            >
              对比
            </Button>
          </div>
          <p class="ge-hint">head 相对 base 领先 {{ compare?.ahead ?? 0 }} · 落后 {{ compare?.behind ?? 0 }}</p>
        </div>
        <div class="ge-section ge-section--fill">
          <div class="ge-list">
            <div v-for="c in compare?.commits ?? []" :key="c.id" class="ge-item">
              <span class="ge-item-title" :title="c.message">{{ c.message }}</span>
              <span class="ge-item-sha font-mono">{{ c.short_id }}</span>
            </div>
            <div v-if="compare" class="ge-subtitle">文件差异（{{ compare.files.length }}）</div>
            <div v-for="f in compare?.files ?? []" :key="'cf-' + f.path" class="ge-item">
              <Tag variant="info">{{ f.status }}</Tag>
              <span class="ge-item-title">{{ f.path }}</span>
              <span class="ge-item-count">+{{ f.added }} -{{ f.removed }}</span>
            </div>
            <Empty
              v-if="compare && !compare.commits.length && !compare.files.length"
              description="两分支无差异"
            />
            <Empty v-else-if="!compare" description="选择两个分支后点击对比" />
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <Button variant="ghost" :disabled="busy" @click="visible = false">关闭</Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import type {
  Branch,
  BranchCompareResult,
  Commit,
  InProgressOp,
  Project,
  StashEntry,
  TagInfo,
} from '../../types';
import {
  getBranches,
  getCommits,
  getProjectStatusForce,
  gitCherryPick,
  gitCreateBranch,
  gitCreateTag,
  gitDeleteBranch,
  gitDeleteTag,
  gitDiffBranches,
  gitListTags,
  gitPushTags,
  gitRebase,
  gitRevertCommit,
  gitReset,
  gitSearchCommits,
  gitSetUpstream,
  gitStashApply,
  gitStashDrop,
  gitStashList,
  gitStashPop,
  gitStashPush,
} from '../../lib/tauriApi';
import Dialog from '../ui/Dialog.vue';
import Button from '../ui/Button.vue';
import Input from '../ui/Input.vue';
import Tag from '../ui/Tag.vue';
import Empty from '../ui/Empty.vue';
import { toast } from '../../lib/toast';

const visible = defineModel<boolean>({ required: true });
const props = defineProps<{
  project: Project | null;
}>();
const emit = defineEmits<{
  (e: 'changed'): void;
}>();

type TabId = 'branch' | 'stash' | 'tag' | 'compare';
const tabs: { id: TabId; label: string }[] = [
  { id: 'branch', label: '分支' },
  { id: 'stash', label: 'Stash' },
  { id: 'tag', label: 'Tag' },
  { id: 'compare', label: '对比' },
];

const activeTab = ref<TabId>('branch');
const busy = ref(false);
const branches = ref<Branch[]>([]);
const stashes = ref<StashEntry[]>([]);
const tags = ref<TagInfo[]>([]);
const recentCommits = ref<Commit[]>([]);
const inProgress = ref<InProgressOp | null>(null);

const newBranchName = ref('');
const newBranchStart = ref('');
const newBranchCheckout = ref(true);
const newBranchTrack = ref(false);
const deleteBranch = ref('');
const deleteBranchForce = ref(false);
const deleteBranchRemote = ref(false);
const upstreamBranch = ref('');
const upstreamRemote = ref('');
const rebaseOnto = ref('');

const stashMessage = ref('');
const stashUntracked = ref(true);

const tagName = ref('');
const tagMessage = ref('');
const tagCommit = ref('');

const compareBase = ref('');
const compareHead = ref('');
const compare = ref<BranchCompareResult | null>(null);

const currentBranch = computed(
  () => branches.value.find((b) => b.is_current)?.name ?? '',
);
const localBranches = computed(() => branches.value.filter((b) => b.is_local && !b.is_detached));
const remoteBranches = computed(() => branches.value.filter((b) => b.is_remote));
const deletableBranches = computed(() =>
  branches.value.filter((b) => !b.is_current && !b.is_detached),
);

watch(visible, (open) => {
  if (open) {
    newBranchName.value = '';
    newBranchStart.value = '';
    deleteBranch.value = '';
    upstreamBranch.value = '';
    upstreamRemote.value = '';
    rebaseOnto.value = '';
    stashMessage.value = '';
    tagName.value = '';
    tagMessage.value = '';
    tagCommit.value = '';
    compareBase.value = '';
    compareHead.value = '';
    compare.value = null;
    void reload();
  }
});

async function reload() {
  if (!props.project) return;
  busy.value = true;
  try {
    const [brs, status] = await Promise.all([
      getBranches(props.project.id),
      getProjectStatusForce(props.project.id),
    ]);
    branches.value = brs;
    inProgress.value = status.in_progress;
    const branch = status.branch && !status.is_detached ? status.branch : '';
    const [st, tg, cps] = await Promise.all([
      gitStashList(props.project.id).catch(() => [] as StashEntry[]),
      gitListTags(props.project.id).catch(() => [] as TagInfo[]),
      branch
        ? getCommits(props.project.id, branch, 30).catch(() => [] as Commit[])
        : Promise.resolve([] as Commit[]),
    ]);
    stashes.value = st;
    tags.value = tg;
    recentCommits.value = cps;
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '加载失败');
  } finally {
    busy.value = false;
  }
}

async function afterMutate() {
  emit('changed');
  await reload();
}

async function createBranch() {
  if (!props.project || !newBranchName.value) return;
  busy.value = true;
  try {
    await gitCreateBranch(
      props.project.id,
      newBranchName.value,
      newBranchCheckout.value,
      newBranchStart.value || undefined,
      newBranchTrack.value,
    );
    toast.success(`已创建分支 ${newBranchName.value}`);
    newBranchName.value = '';
    await afterMutate();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '创建分支失败');
  } finally {
    busy.value = false;
  }
}

async function deleteBranchAction() {
  if (!props.project || !deleteBranch.value) return;
  if (!confirm(`确定删除分支「${deleteBranch.value}」？`)) return;
  busy.value = true;
  try {
    await gitDeleteBranch(
      props.project.id,
      deleteBranch.value,
      deleteBranchForce.value,
      deleteBranchRemote.value,
    );
    toast.success('已删除分支');
    deleteBranch.value = '';
    await afterMutate();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '删除分支失败');
  } finally {
    busy.value = false;
  }
}

async function setUpstream() {
  if (!props.project || !upstreamRemote.value) return;
  busy.value = true;
  try {
    await gitSetUpstream(props.project.id, upstreamRemote.value, upstreamBranch.value || undefined);
    toast.success('已设置上游');
    await afterMutate();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '设置上游失败');
  } finally {
    busy.value = false;
  }
}

async function startRebase() {
  if (!props.project || !rebaseOnto.value) return;
  if (!confirm(`将当前分支变基到「${rebaseOnto.value}」？`)) return;
  busy.value = true;
  try {
    await gitRebase(props.project.id, rebaseOnto.value);
    toast.success('变基完成');
    await afterMutate();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '变基失败');
  } finally {
    busy.value = false;
  }
}

async function pushStash() {
  if (!props.project) return;
  busy.value = true;
  try {
    await gitStashPush(props.project.id, stashMessage.value || undefined, stashUntracked.value);
    toast.success('已保存 Stash');
    stashMessage.value = '';
    await afterMutate();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '保存 Stash 失败');
  } finally {
    busy.value = false;
  }
}

async function applyStash(index: number) {
  if (!props.project) return;
  busy.value = true;
  try {
    await gitStashApply(props.project.id, index);
    toast.success('已应用 Stash');
    await afterMutate();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '应用 Stash 失败');
  } finally {
    busy.value = false;
  }
}

async function popStash(index: number) {
  if (!props.project) return;
  busy.value = true;
  try {
    await gitStashPop(props.project.id, index);
    toast.success('已弹出 Stash');
    await afterMutate();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '弹出 Stash 失败');
  } finally {
    busy.value = false;
  }
}

async function dropStash(index: number) {
  if (!props.project) return;
  if (!confirm(`确定丢弃 stash@{${index}}？`)) return;
  busy.value = true;
  try {
    await gitStashDrop(props.project.id, index);
    toast.success('已丢弃 Stash');
    await afterMutate();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '丢弃 Stash 失败');
  } finally {
    busy.value = false;
  }
}

async function createTag() {
  if (!props.project || !tagName.value) return;
  busy.value = true;
  try {
    await gitCreateTag(
      props.project.id,
      tagName.value,
      tagMessage.value || undefined,
      tagCommit.value || undefined,
    );
    toast.success(`已创建 Tag ${tagName.value}`);
    tagName.value = '';
    tagMessage.value = '';
    await afterMutate();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '创建 Tag 失败');
  } finally {
    busy.value = false;
  }
}

async function deleteTag(name: string) {
  if (!props.project) return;
  if (!confirm(`确定删除 Tag「${name}」？`)) return;
  busy.value = true;
  try {
    await gitDeleteTag(props.project.id, name, false);
    toast.success('已删除 Tag');
    await afterMutate();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '删除 Tag 失败');
  } finally {
    busy.value = false;
  }
}

async function pushTag(name: string) {
  if (!props.project) return;
  busy.value = true;
  try {
    await gitPushTags(props.project.id, name);
    toast.success(`已推送 Tag ${name}`);
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '推送 Tag 失败');
  } finally {
    busy.value = false;
  }
}

async function pushAllTags() {
  if (!props.project) return;
  busy.value = true;
  try {
    await gitPushTags(props.project.id);
    toast.success('已推送全部 Tag');
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '推送 Tag 失败');
  } finally {
    busy.value = false;
  }
}

async function runCompare() {
  if (!props.project || !compareBase.value || !compareHead.value) return;
  busy.value = true;
  compare.value = null;
  try {
    compare.value = await gitDiffBranches(props.project.id, compareBase.value, compareHead.value);
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '对比失败');
  } finally {
    busy.value = false;
  }
}

// 暴露给父组件：从历史页触发的单提交操作也可复用本面板（直接调用）
defineExpose({
  cherryPick: async (sha: string) => {
    if (!props.project) return;
    await gitCherryPick(props.project.id, sha);
    await afterMutate();
  },
  revert: async (sha: string) => {
    if (!props.project) return;
    await gitRevertCommit(props.project.id, sha);
    await afterMutate();
  },
  reset: async (target: string, mode: 'soft' | 'mixed' | 'hard') => {
    if (!props.project) return;
    await gitReset(props.project.id, target, mode);
    await afterMutate();
  },
  search: (q: string, branch?: string) =>
    gitSearchCommits(props.project!.id, q, branch, 80),
});
</script>

<style scoped>
.ge {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-height: 360px;
  max-height: 560px;
}
.ge-tabs {
  display: flex;
  gap: 4px;
  border-bottom: 1px solid var(--border);
  padding-bottom: 0;
}
.ge-tab {
  appearance: none;
  background: transparent;
  border: none;
  color: var(--muted-foreground);
  cursor: pointer;
  font-size: 13px;
  padding: 8px 12px;
  border-bottom: 2px solid transparent;
}
.ge-tab--active {
  color: var(--foreground);
  border-bottom-color: var(--primary);
  font-weight: 500;
}
.ge-pane {
  display: flex;
  flex-direction: column;
  gap: 14px;
  min-height: 0;
  flex: 1;
}
.ge-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.ge-section--fill {
  min-height: 0;
  flex: 1;
}
.ge-section-title {
  font-size: 12px;
  font-weight: 500;
  color: var(--muted-foreground);
  text-transform: none;
}
.ge-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}
.ge-grow {
  flex: 1;
  min-width: 140px;
}
.ge-select {
  height: 28px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--background);
  color: var(--foreground);
  font-size: 12px;
  padding: 0 8px;
  max-width: 220px;
}
.ge-check {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--muted-foreground);
  white-space: nowrap;
}
.ge-list {
  overflow: auto;
  max-height: 280px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.ge-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: 6px;
  border: 1px solid var(--border);
  font-size: 12px;
}
.ge-item-title {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.ge-item-sha {
  color: var(--muted-foreground);
  font-size: 11px;
}
.ge-item-count {
  color: var(--muted-foreground);
  font-size: 11px;
  white-space: nowrap;
}
.ge-item-actions {
  display: flex;
  gap: 2px;
}
.ge-subtitle {
  margin-top: 6px;
  font-size: 12px;
  font-weight: 500;
  color: var(--muted-foreground);
}
.ge-hint {
  font-size: 12px;
  color: var(--muted-foreground);
  margin: 0;
}
</style>
