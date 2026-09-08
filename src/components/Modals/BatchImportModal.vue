<!--
  批量导入项目模态框
  选择 Workspace 目录 -> 扫描 Git 仓库 -> 候选列表勾选 -> 批量导入
-->
<template>
  <Dialog v-model="visible" title="批量导入项目" width="560px">
    <div class="batch-import">
      <!-- 选择目录 -->
      <div v-if="step === 'select'" class="select-step">
        <div
          class="drop-zone"
          :class="{ 'is-dragover': isDragging }"
          @dragover.prevent="isDragging = true"
          @dragleave="isDragging = false"
          @drop.prevent="handleDrop"
          @click="selectFolder"
        >
          <FolderSearch :size="40" class="drop-icon" />
          <p class="drop-text">{{ isDragging ? '松开以选择' : '选择 Workspace 目录' }}</p>
          <p class="drop-hint">或拖拽文件夹到此处</p>
        </div>

        <div class="path-form">
          <Input
            v-model="path"
            placeholder="例如：C:\\Users\\name\\Workspace"
            @keyup.enter="startScan"
          />
        </div>

        <div class="options-row">
          <label class="option-label">
            扫描深度
            <select v-model="maxDepth" class="depth-select">
              <option v-for="d in depthOptions" :key="d" :value="d">{{ d }} 层</option>
            </select>
          </label>
          <span class="option-tip">默认 5 层，可覆盖更深目录</span>
        </div>
      </div>

      <!-- 扫描中 -->
      <div v-else-if="step === 'scanning'" class="progress-step">
        <div class="progress-header">
          <Spinner size="md" />
          <span>正在扫描目录...</span>
        </div>
        <div class="progress-stats">
          <span>已扫描 {{ progress.scanned }} 个目录</span>
          <span>发现 {{ progress.found }} 个仓库</span>
        </div>
        <div v-if="progress.current" class="progress-current" :title="progress.current">
          当前：{{ progress.current }}
        </div>
      </div>

      <!-- 候选列表 / 结果 -->
      <div v-else-if="step === 'preview' || step === 'done'" class="preview-step">
        <div v-if="result" class="result-bar">
          <span class="result-success">成功导入 {{ result.added.length }} 个</span>
          <span v-if="result.skipped.length" class="result-skipped">
            已存在 {{ result.skipped.length }} 个
          </span>
          <span v-if="result.failed.length" class="result-failed">
            失败 {{ result.failed.length }} 个
          </span>
        </div>

        <div v-if="step === 'preview'" class="group-form">
          <label class="form-label">导入到分组</label>
          <GroupPicker v-model="selectedGroupId" />
        </div>

        <div class="preview-toolbar">
          <label class="checkbox-label">
            <input
              type="checkbox"
              :checked="allSelected"
              :indeterminate="someSelected"
              @change="toggleAll"
            />
            <span>全选</span>
          </label>
          <span class="preview-count">共 {{ filteredCandidates.length }} 个候选仓库</span>
        </div>

        <div v-if="filteredCandidates.length === 0" class="empty-state">
          <p>未发现新的 Git 仓库</p>
          <p class="empty-tip">可检查扫描深度或设置中的黑名单配置</p>
        </div>

        <div v-else class="candidate-list">
          <label
            v-for="item in filteredCandidates"
            :key="item.path"
            class="candidate-item"
          >
            <input
              type="checkbox"
              :checked="selectedPaths.has(item.path)"
              @change="toggleItem(item.path)"
            />
            <FolderGit :size="16" class="candidate-icon" />
            <div class="candidate-info">
              <span class="candidate-name">{{ item.name }}</span>
              <span class="candidate-path" :title="item.path">{{ item.path }}</span>
            </div>
          </label>
        </div>
      </div>

      <Alert v-if="error" variant="error" :title="error" />
    </div>

    <template #footer>
      <Button variant="ghost" @click="closeModal">
        {{ step === 'done' ? '完成' : '取消' }}
      </Button>
      <Button
        v-if="step === 'select'"
        variant="primary"
        :disabled="!path.trim() || isScanning"
        @click="startScan"
      >
        开始扫描
      </Button>
      <Button
        v-else-if="step === 'preview'"
        variant="primary"
        :disabled="selectedPaths.size === 0 || isImporting"
        @click="importSelected"
      >
        {{ isImporting ? '导入中...' : `导入选中 (${selectedPaths.size})` }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch, onUnmounted } from 'vue';
import { FolderSearch, FolderGit } from 'lucide-vue-next';
import { open } from '@tauri-apps/plugin-dialog';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useAppStore } from '../../stores/appStore';
import type { ScannedRepo, ImportProgressEvent, BatchImportResult } from '../../types';
import Dialog from '../ui/Dialog.vue';
import Input from '../ui/Input.vue';
import Button from '../ui/Button.vue';
import Alert from '../ui/Alert.vue';
import Spinner from '../ui/Spinner.vue';
import GroupPicker from '../ui/GroupPicker.vue';

const visible = defineModel<boolean>({ required: true });

const appStore = useAppStore();

const step = ref<'select' | 'scanning' | 'preview' | 'done'>('select');
const path = ref('');
const maxDepth = ref(5);
const selectedGroupId = ref<string | null>(null);
const isDragging = ref(false);
const isScanning = ref(false);
const isImporting = ref(false);
const error = ref('');
const candidates = ref<ScannedRepo[]>([]);
const selectedPaths = ref<Set<string>>(new Set());
const result = ref<BatchImportResult | null>(null);
const progress = ref<ImportProgressEvent>({ scanned: 0, found: 0, current: '' });

const depthOptions = [3, 5, 7, 10];

const existingPaths = computed(() => {
  return new Set(appStore.projects.map((p) => p.path));
});

const filteredCandidates = computed(() => {
  return candidates.value.filter((item) => !existingPaths.value.has(item.path));
});

const allSelected = computed(() => {
  return filteredCandidates.value.length > 0 && filteredCandidates.value.every((item) => selectedPaths.value.has(item.path));
});

const someSelected = computed(() => {
  const selected = filteredCandidates.value.filter((item) => selectedPaths.value.has(item.path)).length;
  return selected > 0 && selected < filteredCandidates.value.length;
});

let unlistenProgress: UnlistenFn | null = null;

watch(visible, (value) => {
  if (!value) {
    resetState();
  }
});

onUnmounted(() => {
  cleanupListener();
});

function cleanupListener() {
  if (unlistenProgress) {
    unlistenProgress();
    unlistenProgress = null;
  }
}

function resetState() {
  step.value = 'select';
  path.value = '';
  maxDepth.value = 5;
  isDragging.value = false;
  isScanning.value = false;
  isImporting.value = false;
  error.value = '';
  candidates.value = [];
  selectedPaths.value.clear();
  result.value = null;
  progress.value = { scanned: 0, found: 0, current: '' };
  cleanupListener();
}

async function selectFolder() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择 Workspace 目录',
    });
    if (selected) {
      path.value = selected as string;
      error.value = '';
    }
  } catch {
    error.value = '选择文件夹失败';
  }
}

function handleDrop(e: DragEvent) {
  isDragging.value = false;
  const file = e.dataTransfer?.files?.[0] as (File & { path?: string }) | undefined;
  const droppedPath = file?.path;

  if (droppedPath) {
    path.value = droppedPath;
    error.value = '';
  } else {
    error.value = '无法获取拖拽目录的完整路径，请点击上方区域浏览选择';
  }
}

async function startScan() {
  if (!path.value.trim()) return;

  isScanning.value = true;
  error.value = '';
  step.value = 'scanning';
  progress.value = { scanned: 0, found: 0, current: '' };

  try {
    cleanupListener();
    unlistenProgress = await listen<ImportProgressEvent>('import:progress', (event) => {
      progress.value = event.payload;
    });

    const repos = await appStore.scanProjects(path.value.trim(), { max_depth: maxDepth.value });
    candidates.value = repos;

    // 默认勾选所有新仓库
    selectedPaths.value = new Set(filteredCandidates.value.map((item) => item.path));

    step.value = 'preview';
  } catch (err) {
    error.value = err instanceof Error ? err.message : '扫描目录失败';
    step.value = 'select';
  } finally {
    isScanning.value = false;
    cleanupListener();
  }
}

function toggleAll() {
  if (allSelected.value) {
    selectedPaths.value.clear();
  } else {
    selectedPaths.value = new Set(filteredCandidates.value.map((item) => item.path));
  }
}

function toggleItem(itemPath: string) {
  const next = new Set(selectedPaths.value);
  if (next.has(itemPath)) {
    next.delete(itemPath);
  } else {
    next.add(itemPath);
  }
  selectedPaths.value = next;
}

async function importSelected() {
  if (selectedPaths.value.size === 0) return;

  isImporting.value = true;
  error.value = '';

  try {
    const res = await appStore.batchImportProjects(
      Array.from(selectedPaths.value),
      selectedGroupId.value
    );
    result.value = res;
    step.value = 'done';
  } catch (err) {
    error.value = err instanceof Error ? err.message : '批量导入失败';
  } finally {
    isImporting.value = false;
  }
}

function closeModal() {
  visible.value = false;
}
</script>

<style scoped>
.batch-import {
  display: flex;
  flex-direction: column;
  gap: 16px;
  min-height: 240px;
}

.drop-zone {
  border: 1px dashed var(--border);
  border-radius: 8px;
  padding: 24px;
  text-align: center;
  cursor: pointer;
  transition: border-color 0.2s, background-color 0.2s;
}
.drop-zone:hover,
.drop-zone.is-dragover {
  border-color: var(--primary);
  background-color: color-mix(in oklab, var(--primary) 10%, transparent);
}
.drop-icon {
  color: var(--muted-foreground);
}
.drop-text {
  margin: 12px 0 4px;
  font-size: 14px;
  color: var(--foreground);
}
.drop-hint {
  margin: 0;
  font-size: 12px;
  color: var(--muted-foreground);
}

.path-form {
  margin-top: 8px;
}

.options-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  font-size: 13px;
}
.option-label {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  color: var(--foreground);
}
.depth-select {
  height: 28px;
  padding: 0 8px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background-color: var(--background);
  color: var(--foreground);
  font-size: 13px;
}
.option-tip {
  color: var(--muted-foreground);
  font-size: 12px;
}

.progress-step {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 24px 0;
  text-align: center;
}
.progress-header {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 15px;
  color: var(--foreground);
}
.progress-stats {
  display: flex;
  gap: 16px;
  font-size: 13px;
  color: var(--muted-foreground);
}
.progress-current {
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 12px;
  color: var(--muted-foreground);
}

.preview-step {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.group-form {
  padding: 10px;
  border-radius: 8px;
  border: 1px solid var(--border);
  background-color: var(--background);
}
.form-label {
  display: block;
  margin-bottom: 6px;
  font-size: 13px;
  font-weight: 500;
  color: var(--foreground);
}
.result-bar {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  padding: 8px 10px;
  border-radius: 6px;
  background-color: var(--muted);
  font-size: 13px;
}
.result-success {
  color: oklch(0.6 0.16 150);
}
.result-skipped {
  color: oklch(0.7 0.15 75);
}
.result-failed {
  color: var(--destructive);
}

.preview-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 4px;
}
.checkbox-label {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--foreground);
  cursor: pointer;
  user-select: none;
}
.preview-count {
  font-size: 12px;
  color: var(--muted-foreground);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 32px 0;
  text-align: center;
  color: var(--muted-foreground);
  font-size: 14px;
}
.empty-tip {
  font-size: 12px;
  margin-top: 4px;
}

.candidate-list {
  max-height: 320px;
  overflow-y: auto;
  border: 1px solid var(--border);
  border-radius: 8px;
}
.candidate-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  cursor: pointer;
  transition: background-color 0.15s;
}
.candidate-item:hover {
  background-color: var(--accent);
}
.candidate-item + .candidate-item {
  border-top: 1px solid var(--border);
}
.candidate-icon {
  color: var(--primary);
  flex-shrink: 0;
}
.candidate-info {
  display: flex;
  flex-direction: column;
  min-width: 0;
  flex: 1;
}
.candidate-name {
  font-size: 13px;
  color: var(--foreground);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.candidate-path {
  font-size: 11px;
  color: var(--muted-foreground);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
