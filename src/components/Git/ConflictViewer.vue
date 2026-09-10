<!--
  冲突三路对比查看器
  并排展示 Base / Ours / Theirs，可选查看工作区当前内容
-->
<template>
  <div class="conflict-viewer">
    <div class="cv-toolbar">
      <div class="cv-tabs">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          type="button"
          :class="['cv-tab', activeTab === tab.id ? 'cv-tab--active' : '']"
          @click="activeTab = tab.id"
        >
          {{ tab.label }}
        </button>
      </div>
      <span v-if="path" class="cv-path" :title="path">{{ path }}</span>
    </div>

    <div v-if="content?.is_binary" class="cv-empty">二进制文件，无法预览冲突内容</div>
    <div v-else-if="loading" class="cv-empty">加载中…</div>
    <div v-else-if="!content" class="cv-empty">选择左侧冲突文件查看三路内容</div>
    <template v-else>
      <div v-if="activeTab === 'compare'" class="cv-compare">
        <div class="cv-pane">
          <div class="cv-pane-title">我方（当前分支 / Ours）</div>
          <pre class="cv-pre"><code
            v-for="(line, i) in oursLines"
            :key="'o' + i"
            :class="['cv-line', lineClass(line, 'ours')]"
          >{{ line || ' ' }}</code></pre>
        </div>
        <div class="cv-pane">
          <div class="cv-pane-title">对方（合入分支 / Theirs）</div>
          <pre class="cv-pre"><code
            v-for="(line, i) in theirsLines"
            :key="'t' + i"
            :class="['cv-line', lineClass(line, 'theirs')]"
          >{{ line || ' ' }}</code></pre>
        </div>
      </div>
      <pre v-else class="cv-pre cv-pre--solo"><code
        v-for="(line, i) in activeLines"
        :key="'s' + i"
        :class="['cv-line', lineClass(line, activeTab)]"
      >{{ line || ' ' }}</code></pre>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import type { ConflictFileContent } from '../../types';

const props = withDefaults(
  defineProps<{
    content?: ConflictFileContent | null;
    loading?: boolean;
  }>(),
  {
    content: null,
    loading: false,
  },
);

type TabId = 'compare' | 'base' | 'ours' | 'theirs' | 'working';

const tabs = [
  { id: 'compare' as TabId, label: '对比' },
  { id: 'ours' as TabId, label: '我方' },
  { id: 'theirs' as TabId, label: '对方' },
  { id: 'base' as TabId, label: '祖先' },
];

const activeTab = ref<TabId>('compare');

watch(
  () => props.content?.path,
  () => {
    activeTab.value = 'compare';
  },
);

const path = computed(() => props.content?.path ?? '');

const oursLines = computed(() => splitLines(props.content?.ours ?? ''));
const theirsLines = computed(() => splitLines(props.content?.theirs ?? ''));
const baseLines = computed(() => splitLines(props.content?.base ?? ''));

const activeLines = computed(() => {
  switch (activeTab.value) {
    case 'base':
      return baseLines.value;
    case 'ours':
      return oursLines.value;
    case 'theirs':
      return theirsLines.value;
    default:
      return oursLines.value;
  }
});

function splitLines(s: string): string[] {
  return s.length ? s.split('\n') : [];
}

function lineClass(line: string, side: TabId | 'ours' | 'theirs'): string {
  if (line.startsWith('<<<<<<<') || line.startsWith('>>>>>>>') || line.startsWith('=======')) {
    return 'cv-mark';
  }
  if (line.startsWith('+++') || line.startsWith('---')) return 'cv-meta';
  if (line.startsWith('@@')) return 'cv-hunk';
  if (line.startsWith('+')) return 'cv-add';
  if (line.startsWith('-')) return 'cv-del';
  if (side === 'ours') return 'cv-ours';
  if (side === 'theirs') return 'cv-theirs';
  return 'cv-ctx';
}
</script>

<style scoped>
.conflict-viewer {
  height: 100%;
  display: flex;
  flex-direction: column;
  background-color: var(--muted);
  border-radius: 6px;
  overflow: hidden;
}
.cv-toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 8px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}
.cv-tabs {
  display: inline-flex;
  gap: 2px;
  padding: 2px;
  border-radius: 6px;
  background: color-mix(in oklab, var(--background) 70%, transparent);
}
.cv-tab {
  border: none;
  background: transparent;
  color: var(--muted-foreground);
  font-size: 12px;
  padding: 3px 8px;
  border-radius: 4px;
  cursor: pointer;
}
.cv-tab--active {
  background: var(--background);
  color: var(--foreground);
  font-weight: 600;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.08);
}
.cv-path {
  flex: 1;
  min-width: 0;
  font-size: 11px;
  color: var(--muted-foreground);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--font-mono, ui-monospace, monospace);
}
.cv-compare {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 1fr 1fr;
}
.cv-pane {
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border);
  overflow: auto;
}
.cv-pane:last-child {
  border-right: none;
}
.cv-pane-title {
  position: sticky;
  top: 0;
  z-index: 1;
  padding: 4px 10px;
  font-size: 11px;
  font-weight: 600;
  background: color-mix(in oklab, var(--muted) 92%, var(--background));
  border-bottom: 1px solid var(--border);
  color: var(--muted-foreground);
}
.cv-pre {
  margin: 0;
  padding: 6px 0;
  font-family: var(--font-mono, ui-monospace, monospace);
  font-size: 12px;
  line-height: 1.55;
  overflow: auto;
  flex: 1;
  min-height: 0;
}
.cv-pre--solo {
  padding: 8px 0;
}
.cv-line {
  display: block;
  padding: 0 10px;
  white-space: pre-wrap;
  word-break: break-all;
}
.cv-ctx {
  color: var(--foreground);
}
.cv-meta {
  color: var(--muted-foreground);
}
.cv-hunk {
  color: #2563eb;
  background-color: color-mix(in oklab, #2563eb 12%, transparent);
}
.cv-add {
  color: #16a34a;
  background-color: color-mix(in oklab, #16a34a 16%, transparent);
}
.cv-del {
  color: #dc2626;
  background-color: color-mix(in oklab, #dc2626 16%, transparent);
}
.cv-mark {
  color: #d97706;
  font-weight: 600;
  background-color: color-mix(in oklab, #d97706 18%, transparent);
}
.cv-ours {
  color: #2563eb;
  background-color: color-mix(in oklab, #2563eb 8%, transparent);
}
.cv-theirs {
  color: #9333ea;
  background-color: color-mix(in oklab, #9333ea 8%, transparent);
}
.cv-empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--muted-foreground);
  font-size: 13px;
  padding: 24px;
}
</style>
