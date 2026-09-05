// Git 提交图的 lane 布局算法（移植自 terax 的 layoutGraph）
//
// 输入：commits 倒序（最新在前），每条含 parents（父提交 SHA 数组）。
// 维护 `lanes` 数组：`lanes[i]` = 第 i 条 lane 在下一次迭代时"期望出现"的 commit SHA
// （或 null）。这样从最新→最旧遍历即可正确分配 lane，处理 merge / branch。
//
// 同时提供渲染 rail 所需的尺寸常量与坐标函数。

import type { Commit } from '../types';

export const LANE_WIDTH = 14;
export const ROW_HEIGHT = 28;
export const MAX_VISIBLE_LANES = 6;
export const RAIL_PADDING_X = 6;

const LANE_COLORS = [
  '#26a69a', '#ef5350', '#ab47bc', '#ffa726', '#42a5f5',
  '#66bb6a', '#ec407a', '#8d6e63', '#789262', '#5c6bc0',
  '#ff7043', '#26c6da', '#d4e157', '#7e57c2',
];

export function laneColor(index: number): string {
  return LANE_COLORS[((index % LANE_COLORS.length) + LANE_COLORS.length) % LANE_COLORS.length];
}

export type GraphEdge =
  // 竖直穿过 / 第一父竖直
  | { kind: 'straight'; lane: number; color: string }
  // 合并：另一条 lane 汇入本提交（上 → 节点）
  | { kind: 'merge'; fromLane: number; toLane: number; color: string }
  // 分支：本提交分叉出新 lane（节点 → 下）
  | { kind: 'branch'; fromLane: number; toLane: number; color: string };

export interface GraphRow {
  id: string;
  lane: number;
  color: string;
  laneCount: number;
  // 在这一行上半部分绘制的边（来自上方行）
  topEdges: GraphEdge[];
  // 在这一行下半部分绘制的边（连向下方行）
  bottomEdges: GraphEdge[];
}

export interface GraphState {
  lanes: (string | null)[];
}

export const EMPTY_GRAPH_STATE: GraphState = { lanes: [] };

function trimTrailing(lanes: (string | null)[]): (string | null)[] {
  let end = lanes.length;
  while (end > 0 && lanes[end - 1] === null) end--;
  return end === lanes.length ? lanes : lanes.slice(0, end);
}

function firstFreeSlot(lanes: (string | null)[]): number {
  for (let i = 0; i < lanes.length; i++) {
    if (lanes[i] === null) return i;
  }
  return lanes.length;
}

export function layoutGraph(
  commits: readonly Commit[],
  previous: GraphState = EMPTY_GRAPH_STATE,
): { rows: GraphRow[]; state: GraphState } {
  const lanes: (string | null)[] = previous.lanes.slice();
  const rows: GraphRow[] = [];

  for (const commit of commits) {
    // 当前哪些 lane 正在等待这个 commit。
    const claiming: number[] = [];
    for (let i = 0; i < lanes.length; i++) {
      if (lanes[i] === commit.id) claiming.push(i);
    }

    let lane: number;
    if (claiming.length > 0) {
      lane = claiming[0];
    } else {
      lane = firstFreeSlot(lanes);
      if (lane === lanes.length) lanes.push(null);
    }

    const lanesBefore = lanes.slice();
    const topEdges: GraphEdge[] = [];

    // 上半部分穿透线：仍在等待某 commit 的 lane 继续向下画。
    for (let i = 0; i < lanesBefore.length; i++) {
      const v = lanesBefore[i];
      if (v === null) continue;
      if (v === commit.id && i !== lane) {
        // 这条 lane 正汇入本提交的 lane。
        topEdges.push({ kind: 'merge', fromLane: i, toLane: lane, color: laneColor(i) });
      } else {
        // 竖直穿过 / 同一 lane 竖直进入节点。
        topEdges.push({ kind: 'straight', lane: i, color: laneColor(i) });
      }
    }

    // 占用本行的 lane 已被消费。
    for (const idx of claiming) lanes[idx] = null;
    if (claiming.length === 0) {
      lanes[lane] = null;
    }

    // 放置父节点。
    const parents = commit.parents ?? [];
    const bottomEdges: GraphEdge[] = [];
    if (parents.length > 0) {
      // 第一父留在当前 lane。
      lanes[lane] = parents[0];

      // 其余父节点 → 复用已有 lane 或新分配。
      for (let p = 1; p < parents.length; p++) {
        const parentSha = parents[p];
        let parentLane = lanes.indexOf(parentSha);
        if (parentLane === -1) {
          parentLane = firstFreeSlot(lanes);
          if (parentLane === lanes.length) lanes.push(null);
          lanes[parentLane] = parentSha;
        }
        if (parentLane !== lane) {
          bottomEdges.push({ kind: 'branch', fromLane: lane, toLane: parentLane, color: laneColor(parentLane) });
        }
      }
    }

    // 下半部分穿透线：状态中仍活跃的 lane 继续向下画（分支目标已记录，跳过）。
    const branchTargets = new Set(
      bottomEdges
        .filter((e): e is Extract<GraphEdge, { kind: 'branch' }> => e.kind === 'branch')
        .map((e) => e.toLane),
    );
    for (let i = 0; i < lanes.length; i++) {
      const v = lanes[i];
      if (v === null) continue;
      if (branchTargets.has(i)) continue;
      bottomEdges.push({ kind: 'straight', lane: i, color: laneColor(i) });
    }

    const trimmed = trimTrailing(lanes);
    if (trimmed.length !== lanes.length) {
      lanes.length = trimmed.length;
    }

    const widestLane = Math.max(lanesBefore.length, lanes.length, lane + 1);

    rows.push({
      id: commit.id,
      lane,
      color: laneColor(lane),
      laneCount: widestLane,
      topEdges,
      bottomEdges,
    });
  }

  return { rows, state: { lanes: lanes.slice() } };
}

export function laneX(lane: number): number {
  return RAIL_PADDING_X + lane * LANE_WIDTH;
}

export function railWidth(maxLane: number): number {
  const visible = Math.min(maxLane, MAX_VISIBLE_LANES);
  return RAIL_PADDING_X * 2 + Math.max(0, visible - 1) * LANE_WIDTH + 6;
}
