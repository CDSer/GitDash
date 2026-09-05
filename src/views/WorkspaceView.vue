<!--
  工作区视图（整页，非弹窗）
  由项目列表点击「工作区」进入：左侧文件树 + 右侧多标签代码编辑器 + 底部可折叠 Git 图
-->
<template>
  <div class="flex h-screen flex-col overflow-hidden bg-background text-foreground">
    <header
      class="flex h-12 shrink-0 items-center justify-between border-b border-border bg-card px-4"
    >
      <div class="flex min-w-0 items-center gap-2">
        <Button variant="ghost" size="sm" @click="close">
          <ArrowLeft :size="14" /> 返回项目列表
        </Button>
        <Divider direction="vertical" />
        <span class="font-semibold">{{ project?.name }}</span>
        <span class="truncate text-xs text-muted-foreground">{{ project?.path }}</span>
      </div>
      <div class="flex items-center gap-2">
        <Button
          variant="ghost"
          size="sm"
          :class="showGitPanel ? 'text-primary' : ''"
          @click="showGitPanel = !showGitPanel"
        >
          <GitBranch :size="14" /> Git 图
        </Button>
      </div>
    </header>

    <div class="flex min-h-0 flex-1">
      <aside class="w-[260px] shrink-0 flex flex-col overflow-hidden border-r border-border bg-card">
        <FileTree v-if="project" :root-path="project.path" @open-file="onOpenFile" />
      </aside>

      <main class="flex min-w-0 flex-1 flex-col overflow-hidden">
        <div v-if="tabs.length" class="flex min-h-0 flex-1 flex-col">
          <div
            class="flex h-9 shrink-0 items-stretch overflow-x-auto border-b border-border bg-card"
          >
            <div
              v-for="tab in tabs"
              :key="tab.path"
              :class="[
                'flex cursor-pointer items-center gap-1.5 border-r border-border px-2.5 text-[13px]',
                tab.path === activePath
                  ? 'bg-background text-foreground'
                  : 'text-muted-foreground hover:bg-accent',
                isDirty(tab) ? 'font-medium' : '',
              ]"
              :title="tab.path"
              @click="activePath = tab.path"
            >
              <span class="whitespace-nowrap">{{ tab.name }}</span>
              <span
                v-if="isDirty(tab)"
                class="h-1.5 w-1.5 rounded-full bg-amber-500"
              />
              <X
                class="rounded p-0.5 hover:bg-muted"
                :size="12"
                @click.stop="closeTab(tab.path)"
              />
            </div>
          </div>
          <div class="relative min-h-0 flex-1 overflow-hidden">
            <CodeEditor
              v-if="activeTab"
              :model-value="activeTab.content"
              :language="langOf(activeTab.path)"
              @update:model-value="(v: string) => updateContent(v)"
              @save="saveActive"
            />
            <div
              v-if="activeTab?.loading"
              class="absolute inset-0 flex items-center justify-center text-[13px] text-muted-foreground"
            >
              加载中…
            </div>
          </div>
        </div>
        <Empty v-else description="从左侧选择文件查看或编辑" />

        <div v-if="showGitPanel && project" class="h-80 shrink-0 border-t border-border min-h-0">
          <GitGraphPanel :project-id="project.id" @close="showGitPanel = false" />
        </div>
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { ArrowLeft, GitBranch, X } from 'lucide-vue-next';
import { useAppStore } from '../stores/appStore';
import FileTree from '../components/Explorer/FileTree.vue';
import CodeEditor from '../components/Editor/CodeEditor.vue';
import GitGraphPanel from '../components/Git/GitGraphPanel.vue';
import Empty from '../components/ui/Empty.vue';
import Divider from '../components/ui/Divider.vue';
import Button from '../components/ui/Button.vue';
import { readFile, writeFile } from '../lib/tauriApi';
import { toast } from '../lib/toast';

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

const activeTab = computed(() => tabs.value.find((t) => t.path === activePath.value) ?? null);

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
  const existing = tabs.value.find((t) => t.path === path);
  if (existing) {
    activePath.value = path;
    return;
  }

  const tab: OpenTab = { path, name: basename(path), content: '', original: '', loading: true };
  const index = tabs.value.length;
  tabs.value.push(tab);
  activePath.value = path;

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
    toast.success(`已保存 ${tab.name}`);
  } catch (error) {
    toast.error(`保存失败：${error}`);
  }
}

function closeTab(path: string) {
  const idx = tabs.value.findIndex((t) => t.path === path);
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
