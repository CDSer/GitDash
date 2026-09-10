<!--
  Git 提交图的 SVG 绘制（左侧轨道）
  接收 commits，用 layoutGraph 计算 lane 布局，绘制圆点 + 连线。
  点击圆点 emit('select', commitId)
-->
<template>
  <svg
    class="git-rail-svg"
    :width="svgWidth"
    :height="svgHeight"
    :viewBox="`0 0 ${svgWidth} ${svgHeight}`"
    @click="onClick"
  >
    <g
      v-for="(r, i) in rendered"
      :key="r.row.id"
      :transform="`translate(0, ${i * ROW_HEIGHT})`"
    >
      <template v-for="e in r.topEdges" :key="e.key">
        <line
          v-if="e.kind === 'line'"
          :x1="e.x"
          y1="0"
          :x2="e.x"
          :y2="ROW_HEIGHT / 2"
          :stroke="e.color"
          stroke-width="1.5"
          stroke-linecap="round"
        />
        <path
          v-else
          :d="e.d"
          :stroke="e.color"
          fill="none"
          stroke-width="1.5"
          stroke-linecap="round"
        />
      </template>
      <template v-for="e in r.bottomEdges" :key="e.key">
        <line
          v-if="e.kind === 'line'"
          :x1="e.x"
          :y1="ROW_HEIGHT / 2"
          :x2="e.x"
          :y2="ROW_HEIGHT"
          :stroke="e.color"
          stroke-width="1.5"
          stroke-linecap="round"
        />
        <path
          v-else
          :d="e.d"
          :stroke="e.color"
          fill="none"
          stroke-width="1.5"
          stroke-linecap="round"
        />
      </template>
      <circle
        :cx="r.nodeX"
        :cy="ROW_HEIGHT / 2"
        :r="selectedId === r.row.id ? 5 : 4"
        :fill="selectedId === r.row.id ? '#fff' : r.row.color"
        :stroke="r.row.color"
        stroke-width="2"
        :data-id="r.row.id"
        class="git-dot"
        :class="{ 'git-dot--unpushed': isUnpushed(r.row.id) }"
      />
      <text
        v-if="r.overflow"
        :x="svgWidth - 4"
        :y="ROW_HEIGHT / 2 + 3"
        text-anchor="end"
        class="git-overflow"
      >
        +{{ r.row.laneCount - visibleLanes }}
      </text>
    </g>
  </svg>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { Commit } from '../../types';
import {
  layoutGraph,
  laneX,
  railWidth,
  ROW_HEIGHT,
  MAX_VISIBLE_LANES,
} from '../../lib/gitGraph';

const props = defineProps<{
  commits: Commit[];
  selectedId?: string | null;
}>();
const emit = defineEmits<{ (e: 'select', id: string): void }>();

interface RenderEdge {
  key: string;
  kind: 'line' | 'path';
  x?: number;
  d?: string;
  color: string;
}

const layout = computed(() => layoutGraph(props.commits));
const rows = computed(() => layout.value.rows);
const unpushedIds = computed(
  () => new Set(props.commits.filter((c) => c.is_pushed === false).map((c) => c.id)),
);

function isUnpushed(id: string): boolean {
  return unpushedIds.value.has(id);
}

const maxLaneCount = computed(() =>
  rows.value.reduce((m, r) => Math.max(m, r.laneCount), 1),
);
const visibleLanes = computed(() => Math.min(maxLaneCount.value, MAX_VISIBLE_LANES));
const svgWidth = computed(() => railWidth(maxLaneCount.value));
const svgHeight = computed(() => props.commits.length * ROW_HEIGHT);

const rendered = computed(() => {
  const midY = ROW_HEIGHT / 2;
  const bottom = ROW_HEIGHT;
  return rows.value.map((row) => {
    const nodeX = laneX(row.lane);

    const topEdges: RenderEdge[] = row.topEdges.map((e, idx) => {
      if (e.kind === 'straight') {
        const x = laneX(e.lane);
        return { key: `t-l-${e.lane}-${idx}`, kind: 'line', x, color: e.color };
      }
      const xFrom = laneX(e.fromLane);
      const xTo = laneX(e.toLane);
      const c1y = midY * 0.55;
      return {
        key: `t-m-${e.fromLane}-${e.toLane}`,
        kind: 'path',
        d: `M ${xFrom} 0 C ${xFrom} ${c1y}, ${xTo} ${c1y}, ${xTo} ${midY}`,
        color: e.color,
      };
    });

    const bottomEdges: RenderEdge[] = row.bottomEdges.map((e, idx) => {
      if (e.kind === 'straight') {
        const x = laneX(e.lane);
        return { key: `b-l-${e.lane}-${idx}`, kind: 'line', x, color: e.color };
      }
      const xFrom = laneX(e.fromLane);
      const xTo = laneX(e.toLane);
      const c1y = midY + (bottom - midY) * 0.45;
      return {
        key: `b-b-${e.fromLane}-${e.toLane}`,
        kind: 'path',
        d: `M ${xFrom} ${midY} C ${xFrom} ${c1y}, ${xTo} ${c1y}, ${xTo} ${bottom}`,
        color: e.color,
      };
    });

    return {
      row,
      nodeX,
      topEdges,
      bottomEdges,
      overflow: row.laneCount > visibleLanes.value,
    };
  });
});

function onClick(e: MouseEvent) {
  const target = e.target as Element | null;
  const id = target?.getAttribute('data-id');
  if (id) emit('select', id);
}
</script>

<style scoped>
.git-rail-svg {
  display: block;
  overflow: hidden;
}

.git-dot {
  cursor: pointer;
}

.git-dot:hover {
  stroke-width: 3;
}

.git-dot--unpushed {
  stroke-dasharray: 3 2;
}

.git-overflow {
  font-size: 8px;
  fill: var(--muted-foreground);
}
</style>
