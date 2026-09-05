<!--
  工作区视图（整页，非弹窗）
  由项目列表点击「工作区」进入：左侧文件树 + 右侧多标签代码编辑器 + 底部可折叠 Git 图
-->
<template>
  <div class="workspace">
    <el-container class="ws-container">
      <el-header class="ws-header" height="48px">
        <div class="ws-header-left">
          <el-button text :icon="ArrowLeft" @click="close">返回项目列表</el-button>
          <el-divider direction="vertical" />
          <span class="ws-project-name">{{ project?.name }}</span>
          <span class="ws-project-path">{{ project?.path }}</span>
        </div>
        <div class="ws-header-right">
          <el-button
            text
            size="small"
            :icon="Collection"
            :type="showGitPanel ? 'primary' : 'default'"
            @click="showGitPanel = !showGitPanel"
          >
            Git 图
          </el-button>
        </div>
      </el-header>

      <el-container class="ws-body">
        <el-aside width="260px" class="ws-explorer">
          <FileTree v-if="project" :root-path="project.path" @open-file="onOpenFile" />
        </el-aside>

        <el-main class="ws-content">
          <div v-if="tabs.length" class="ws-editor-wrap">
            <div class="ws-tabs">
              <div
                v-for="tab in tabs"
                :key="tab.path"
                :class="['ws-tab', { active: tab.path === activePath, dirty: isDirty(tab) }]"
                :title="tab.path"
                @click="activePath = tab.path"
              >
                <span class="ws-tab-name">{{ tab.name }}</span>
                <span v-if="isDirty(tab)" class="ws-tab-dot" />
                <el-icon class="ws-tab-close" :size="12" @click.stop="closeTab(tab.path)">
                  <Close />
                </el-icon>
              </div>
            </div>
            <div class="ws-editor">
              <CodeEditor
                v-if="activeTab"
                :model-value="activeTab.content"
                :language="langOf(activeTab.path)"
                @update:model-value="(v: string) => updateContent(v)"
                @save="saveActive"
              />
              <div v-if="activeTab?.loading" class="ws-loading">加载中…</div>
            </div>
          </div>
          <el-empty v-else description="从左侧选择文件查看或编辑" />

          <div v-if="showGitPanel && project" class="ws-git-panel">
            <GitGraphPanel :project-id="project.id" @close="showGitPanel = false" />
          </div>
        </el-main>
      </el-container>
    </el-container>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { ArrowLeft, Collection, Close } from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';
import { useAppStore } from '../stores/appStore';
import FileTree from '../components/Explorer/FileTree.vue';
import CodeEditor from '../components/Editor/CodeEditor.vue';
import GitGraphPanel from '../components/Git/GitGraphPanel.vue';
import { readFile, writeFile } from '../lib/tauriApi';

const appStore = useAppStore();
const project = computed(() => appStore.workspaceProject);

interface OpenTab {
  path: string;
  name: string;
  content: string;
  original: string;
  loading: boolean;
}

const tabs = ref<OpenTab[]>([]);
const activePath = ref<string | null>(null);
const showGitPanel = ref(false);

const activeTab = computed(() => tabs.value.find(t => t.path === activePath.value) ?? null);

function basename(p: string) {
  return p.split(/[\\/]/).pop() || p;
}

function langOf(p: string) {
  const parts = p.split('.');
  return parts.length > 1 ? parts.pop()!.toLowerCase() : '';
}

function isDirty(tab: OpenTab) {
  return tab.content !== tab.original;
}

async function onOpenFile(path: string) {
  const existing = tabs.value.find(t => t.path === path);
  if (existing) {
    activePath.value = path;
    return;
  }

  const tab: OpenTab = { path, name: basename(path), content: '', original: '', loading: true };
  const index = tabs.value.length;
  tabs.value.push(tab);
  activePath.value = path;

  // 通过响应式代理（tabs.value[index]）更新，否则直接改局部 tab 不会触发界面刷新
  const proxy = tabs.value[index];
  try {
    const content = await readFile(path);
    proxy.content = content;
    proxy.original = content;
  } catch (error) {
    proxy.content = `// 无法读取文件：${error}`;
  } finally {
    proxy.loading = false;
  }
}

function updateContent(value: string) {
  const tab = activeTab.value;
  if (tab) tab.content = value;
}

async function saveActive() {
  const tab = activeTab.value;
  if (!tab) return;
  try {
    await writeFile(tab.path, tab.content);
    tab.original = tab.content;
    ElMessage.success(`已保存 ${tab.name}`);
  } catch (error) {
    ElMessage.error(`保存失败：${error}`);
  }
}

function closeTab(path: string) {
  const idx = tabs.value.findIndex(t => t.path === path);
  if (idx === -1) return;
  tabs.value.splice(idx, 1);
  if (activePath.value === path) {
    activePath.value = tabs.value.length ? tabs.value[Math.max(0, idx - 1)].path : null;
  }
}

function close() {
  appStore.closeWorkspace();
}
</script>

<style scoped>
.workspace {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background-color: var(--el-bg-color);
  overflow: hidden;
}

.ws-container {
  flex: 1;
  overflow: hidden;
}

.ws-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  border-bottom: 1px solid var(--el-border-color);
  background-color: var(--el-bg-color-overlay);
}

.ws-header-left {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.ws-project-name {
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.ws-project-path {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ws-body {
  overflow: hidden;
}

.ws-explorer {
  overflow: hidden;
  border-right: 1px solid var(--el-border-color);
  background-color: var(--el-bg-color-overlay);
  display: flex;
  flex-direction: column;
}

.ws-explorer > * {
  flex: 1;
  min-height: 0;
}

.ws-content {
  padding: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.ws-editor-wrap {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.ws-tabs {
  display: flex;
  align-items: stretch;
  height: 36px;
  flex-shrink: 0;
  border-bottom: 1px solid var(--el-border-color);
  background-color: var(--el-bg-color-overlay);
  overflow-x: auto;
}

.ws-tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 10px;
  height: 36px;
  cursor: pointer;
  border-right: 1px solid var(--el-border-color-lighter);
  font-size: 13px;
  color: var(--el-text-color-secondary);
  white-space: nowrap;
}

.ws-tab.active {
  background-color: var(--el-bg-color);
  color: var(--el-text-color-primary);
}

.ws-tab-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background-color: var(--el-color-warning);
}

.ws-tab-close {
  border-radius: 4px;
  padding: 1px;
}

.ws-tab-close:hover {
  background-color: var(--el-fill-color-dark);
}

.ws-editor {
  flex: 1;
  min-height: 0;
  position: relative;
  overflow: hidden;
}

.ws-loading {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--el-text-color-secondary);
  font-size: 13px;
}

.ws-git-panel {
  flex-shrink: 0;
  height: 320px;
  border-top: 1px solid var(--el-border-color);
  min-height: 0;
}
</style>
