<!--
  统一格式 diff 查看器
  接收原始 patch 文本，按 +/-/@ 着色渲染
-->
<template>
  <div class="diff-viewer">
    <pre v-if="isBinary" class="diff-binary">二进制文件，无法显示文本差异</pre>
    <pre v-else-if="lines.length" class="diff-pre"><code
      v-for="(line, i) in lines"
      :key="i"
      :class="['diff-line', lineClass(line)]"
    >{{ line }}</code></pre>
    <Empty v-else :description="emptyText" />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import Empty from '../ui/Empty.vue';

const props = withDefaults(
  defineProps<{
    patch?: string;
    isBinary?: boolean;
    emptyText?: string;
  }>(),
  {
    patch: '',
    isBinary: false,
    emptyText: '暂无可显示的改动',
  },
);

const lines = computed(() => (props.patch ?? '').split('\n'));

function lineClass(line: string): string {
  if (line.startsWith('+++') || line.startsWith('---')) return 'diff-meta';
  if (line.startsWith('@@')) return 'diff-hunk';
  if (line.startsWith('+') && !line.startsWith('+++')) return 'diff-add';
  if (line.startsWith('-') && !line.startsWith('---')) return 'diff-del';
  return 'diff-context';
}
</script>

<style scoped>
.diff-viewer {
  height: 100%;
  overflow: auto;
  background-color: var(--muted);
  border-radius: 6px;
}
.diff-pre {
  margin: 0;
  padding: 8px 0;
  font-family: var(--font-mono, ui-monospace, monospace);
  font-size: 12px;
  line-height: 1.6;
  white-space: pre;
  min-width: 100%;
}
.diff-line {
  display: block;
  padding: 0 12px;
  white-space: pre-wrap;
  word-break: break-all;
}
.diff-context {
  color: var(--foreground);
}
.diff-meta {
  color: var(--muted-foreground);
}
.diff-hunk {
  color: #2563eb;
  background-color: color-mix(in oklab, #2563eb 12%, transparent);
}
.diff-add {
  color: #16a34a;
  background-color: color-mix(in oklab, #16a34a 16%, transparent);
}
.diff-del {
  color: #dc2626;
  background-color: color-mix(in oklab, #dc2626 16%, transparent);
}
.diff-binary {
  margin: 0;
  padding: 24px;
  color: var(--muted-foreground);
  font-size: 13px;
  text-align: center;
}
</style>
