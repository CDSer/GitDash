<!--
  文件改动查看器
  默认左右并排对比；也可切换为统一 diff
-->
<template>
  <div class="diff-viewer">
    <div class="dv-toolbar">
      <div class="dv-modes">
        <button
          type="button"
          :class="['dv-mode', viewMode === 'split' ? 'dv-mode--active' : '']"
          @click="viewMode = 'split'"
        >
          并排
        </button>
        <button
          type="button"
          :class="['dv-mode', viewMode === 'unified' ? 'dv-mode--active' : '']"
          @click="viewMode = 'unified'"
        >
          统一
        </button>
      </div>
      <span v-if="rows.length || unifiedLines.length" class="dv-stat">
        <span class="dv-stat-add">+{{ addCount }}</span>
        <span class="dv-stat-del">-{{ delCount }}</span>
      </span>
    </div>

    <div v-if="isBinary" class="dv-empty">二进制文件，无法显示文本差异</div>
    <div v-else-if="loading" class="dv-empty">加载中…</div>
    <div v-else-if="showEmpty" class="dv-empty">{{ emptyText }}</div>

    <div v-else-if="viewMode === 'split'" ref="splitRoot" class="dv-split">
      <div class="dv-pane dv-pane--old">
        <div class="dv-pane-title">修改前</div>
        <div class="dv-rows" @scroll="onScroll('left', $event)">
          <div
            v-for="(row, i) in rows"
            :key="'l' + i"
            :class="['dv-row', rowClass(row, 'left')]"
          >
            <span class="dv-ln">{{ row.leftNo ?? '' }}</span>
            <span class="dv-code">{{ row.left ?? '' }}</span>
          </div>
        </div>
      </div>
      <div class="dv-pane dv-pane--new">
        <div class="dv-pane-title">修改后</div>
        <div class="dv-rows" @scroll="onScroll('right', $event)">
          <div
            v-for="(row, i) in rows"
            :key="'r' + i"
            :class="['dv-row', rowClass(row, 'right')]"
          >
            <span class="dv-ln">{{ row.rightNo ?? '' }}</span>
            <span class="dv-code">{{ row.right ?? '' }}</span>
          </div>
        </div>
      </div>
    </div>

    <div v-else class="dv-unified">
      <div
        v-for="(line, i) in unifiedLines"
        :key="'u' + i"
        :class="['dv-row', lineClass(line)]"
      >
        <span class="dv-code">{{ line }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';

type RowKind = 'ctx' | 'add' | 'del' | 'meta' | 'hunk';

interface SplitRow {
  left: string | null;
  right: string | null;
  leftNo: number | null;
  rightNo: number | null;
  kind: RowKind;
}

const props = withDefaults(
  defineProps<{
    /** 修改前完整内容 */
    original?: string;
    /** 修改后完整内容 */
    modified?: string;
    /** 统一 patch（回退 / 统一视图） */
    patch?: string;
    isBinary?: boolean;
    emptyText?: string;
    loading?: boolean;
  }>(),
  {
    original: '',
    modified: '',
    patch: '',
    isBinary: false,
    emptyText: '暂无可显示的改动',
    loading: false,
  },
);

const viewMode = ref<'split' | 'unified'>('split');
const splitRoot = ref<HTMLElement | null>(null);
let syncingScroll = false;

watch(
  () => [props.original, props.modified, props.patch],
  () => {
    viewMode.value = props.original || props.modified ? 'split' : 'unified';
    nextTick(() => {
      const panes = splitRoot.value?.querySelectorAll('.dv-rows');
      panes?.forEach((el) => {
        el.scrollTop = 0;
        el.scrollLeft = 0;
      });
    });
  },
  { immediate: true },
);

function splitLines(s: string): string[] {
  if (!s) return [];
  const lines = s.split('\n');
  if (lines.length && lines[lines.length - 1] === '') lines.pop();
  return lines;
}

/** 基于 LCS 的行对齐（限制规模，过大时退化为 zip） */
function alignLines(a: string[], b: string[]): SplitRow[] {
  const max = 2500;
  if (a.length > max || b.length > max) {
    const n = Math.max(a.length, b.length);
    const rows: SplitRow[] = [];
    for (let i = 0; i < n; i++) {
      const left = i < a.length ? a[i] : null;
      const right = i < b.length ? b[i] : null;
      const kind: RowKind =
        left !== null && right !== null && left === right
          ? 'ctx'
          : left !== null && right !== null
            ? 'del'
            : left !== null
              ? 'del'
              : 'add';
      rows.push({
        left,
        right,
        leftNo: left !== null ? i + 1 : null,
        rightNo: right !== null ? i + 1 : null,
        kind,
      });
    }
    return rows;
  }

  // DP LCS table
  const n = a.length;
  const m = b.length;
  const dp: number[][] = Array.from({ length: n + 1 }, () => new Array<number>(m + 1).fill(0));
  for (let i = n - 1; i >= 0; i--) {
    for (let j = m - 1; j >= 0; j--) {
      dp[i][j] = a[i] === b[j] ? dp[i + 1][j + 1] + 1 : Math.max(dp[i + 1][j], dp[i][j + 1]);
    }
  }

  const rows: SplitRow[] = [];
  let i = 0;
  let j = 0;
  let leftNo = 1;
  let rightNo = 1;
  while (i < n && j < m) {
    if (a[i] === b[j]) {
      rows.push({
        left: a[i],
        right: b[j],
        leftNo,
        rightNo,
        kind: 'ctx',
      });
      i++;
      j++;
      leftNo++;
      rightNo++;
    } else if (dp[i + 1][j] >= dp[i][j + 1]) {
      rows.push({ left: a[i], right: null, leftNo, rightNo: null, kind: 'del' });
      i++;
      leftNo++;
    } else {
      rows.push({ left: null, right: b[j], leftNo: null, rightNo, kind: 'add' });
      j++;
      rightNo++;
    }
  }
  while (i < n) {
    rows.push({ left: a[i], right: null, leftNo, rightNo: null, kind: 'del' });
    i++;
    leftNo++;
  }
  while (j < m) {
    rows.push({ left: null, right: b[j], leftNo: null, rightNo, kind: 'add' });
    j++;
    rightNo++;
  }
  return rows;
}

/** 相邻 del/add 合并为修改行（左右并排） */
function mergeModifies(rows: SplitRow[]): SplitRow[] {
  const out: SplitRow[] = [];
  for (let i = 0; i < rows.length; i++) {
    const cur = rows[i];
    if (cur.kind === 'del' && i + 1 < rows.length && rows[i + 1].kind === 'add') {
      const next = rows[i + 1];
      out.push({
        left: cur.left,
        right: next.right,
        leftNo: cur.leftNo,
        rightNo: next.rightNo,
        kind: cur.left === next.right ? 'ctx' : 'del',
      });
      i++;
      continue;
    }
    if (cur.kind === 'add' && i + 1 < rows.length && rows[i + 1].kind === 'del') {
      const next = rows[i + 1];
      out.push({
        left: next.left,
        right: cur.right,
        leftNo: next.leftNo,
        rightNo: cur.rightNo,
        kind: next.left === cur.right ? 'ctx' : 'del',
      });
      i++;
      continue;
    }
    out.push(cur);
  }
  return out;
}

const rows = computed(() => {
  const hasContent = props.original !== '' || props.modified !== '';
  if (!hasContent) return [];
  return mergeModifies(alignLines(splitLines(props.original), splitLines(props.modified)));
});

const unifiedLines = computed(() => {
  if (props.patch) return props.patch.split('\n');
  if (!props.original && !props.modified) return [];
  // 无 patch 时用内容拼一版统一视图
  const aligned = rows.value;
  const lines: string[] = [];
  for (const r of aligned) {
    if (r.kind === 'ctx') {
      lines.push(` ${r.left ?? r.right ?? ''}`);
    } else if (r.kind === 'del' || (r.left != null && r.right == null)) {
      lines.push(`-${r.left ?? ''}`);
    } else if (r.kind === 'add' || (r.right != null && r.left == null)) {
      lines.push(`+${r.right ?? ''}`);
    }
  }
  return lines;
});

const showEmpty = computed(() => {
  if (props.loading || props.isBinary) return false;
  if (props.original || props.modified) {
    if (!props.original && !props.modified) return true;
    // 内容相同且无 patch 改动
    if (props.original === props.modified && !props.patch.trim()) return true;
    return rows.value.length === 0;
  }
  return !props.patch.trim();
});

const addCount = computed(() =>
  rows.value.filter((r) => r.kind === 'add' || (r.kind === 'del' && r.right != null && r.left != null)).length,
);
const delCount = computed(() =>
  rows.value.filter((r) => r.kind === 'del').length,
);

function rowClass(row: SplitRow, side: 'left' | 'right'): string {
  if (row.kind === 'hunk') return 'dv-hunk';
  if (row.kind === 'meta') return 'dv-meta';
  if (row.kind === 'ctx') return 'dv-ctx';

  const text = side === 'left' ? row.left : row.right;
  if (text == null) return 'dv-empty-cell';
  if (side === 'left') return row.kind === 'add' ? 'dv-empty-cell' : 'dv-del';
  return row.kind === 'del' && row.left == null ? 'dv-empty-cell' : 'dv-add';
}

function lineClass(line: string): string {
  if (line.startsWith('+++') || line.startsWith('---')) return 'dv-meta';
  if (line.startsWith('@@')) return 'dv-hunk';
  if (line.startsWith('+') && !line.startsWith('+++')) return 'dv-add';
  if (line.startsWith('-') && !line.startsWith('---')) return 'dv-del';
  return 'dv-ctx';
}

function onScroll(side: 'left' | 'right', e: Event) {
  if (syncingScroll) return;
  syncingScroll = true;
  const el = e.currentTarget as HTMLElement;
  const panes = splitRoot.value?.querySelectorAll('.dv-rows');
  if (panes && panes.length === 2) {
    const other = panes[side === 'left' ? 1 : 0] as HTMLElement;
    other.scrollTop = el.scrollTop;
  }
  nextTick(() => {
    syncingScroll = false;
  });
}
</script>

<style scoped>
.diff-viewer {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background-color: var(--muted);
  border-radius: 6px;
}
.dv-toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 4px 8px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
  background: color-mix(in oklab, var(--muted) 92%, var(--background));
}
.dv-modes {
  display: inline-flex;
  gap: 2px;
  padding: 2px;
  border-radius: 6px;
  background: color-mix(in oklab, var(--background) 70%, transparent);
}
.dv-mode {
  border: none;
  background: transparent;
  color: var(--muted-foreground);
  font-size: 12px;
  padding: 3px 8px;
  border-radius: 4px;
  cursor: pointer;
}
.dv-mode--active {
  background: var(--background);
  color: var(--foreground);
  font-weight: 600;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.08);
}
.dv-stat {
  margin-left: auto;
  font-size: 11px;
  font-family: var(--font-mono, ui-monospace, monospace);
  font-variant-numeric: tabular-nums;
}
.dv-stat-add {
  color: #16a34a;
}
.dv-stat-del {
  color: #dc2626;
  margin-left: 6px;
}
.dv-split {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 1fr 1fr;
}
.dv-pane {
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border);
  overflow: hidden;
}
.dv-pane:last-child {
  border-right: none;
}
.dv-pane-title {
  position: sticky;
  top: 0;
  z-index: 1;
  padding: 4px 10px;
  font-size: 11px;
  font-weight: 600;
  background: color-mix(in oklab, var(--muted) 92%, var(--background));
  border-bottom: 1px solid var(--border);
  color: var(--muted-foreground);
  flex-shrink: 0;
}
.dv-rows {
  flex: 1;
  min-height: 0;
  overflow: auto;
  font-family: var(--font-mono, ui-monospace, monospace);
  font-size: 12px;
  line-height: 1.55;
}
.dv-row {
  display: flex;
  min-height: 1.55em;
}
.dv-ln {
  flex-shrink: 0;
  width: 44px;
  padding: 0 8px 0 4px;
  text-align: right;
  color: var(--muted-foreground);
  user-select: none;
  opacity: 0.7;
  border-right: 1px solid color-mix(in oklab, var(--border) 70%, transparent);
  background: color-mix(in oklab, var(--background) 40%, transparent);
}
.dv-code {
  flex: 1;
  min-width: 0;
  padding: 0 10px;
  white-space: pre-wrap;
  word-break: break-all;
}
.dv-ctx .dv-code {
  color: var(--foreground);
}
.dv-meta .dv-code {
  color: var(--muted-foreground);
}
.dv-hunk .dv-code {
  color: #2563eb;
  background-color: color-mix(in oklab, #2563eb 12%, transparent);
}
.dv-add {
  background-color: color-mix(in oklab, #16a34a 16%, transparent);
}
.dv-add .dv-code {
  color: #16a34a;
}
.dv-del {
  background-color: color-mix(in oklab, #dc2626 16%, transparent);
}
.dv-del .dv-code {
  color: #dc2626;
}
.dv-empty-cell {
  background-color: color-mix(in oklab, var(--border) 25%, transparent);
}
.dv-empty-cell .dv-code {
  color: transparent;
}
.dv-empty-cell .dv-ln {
  color: transparent;
}
.dv-unified {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 6px 0;
  font-family: var(--font-mono, ui-monospace, monospace);
  font-size: 12px;
  line-height: 1.6;
}
.dv-unified .dv-row {
  padding: 0 4px;
}
.dv-empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--muted-foreground);
  font-size: 13px;
  padding: 24px;
}
</style>
