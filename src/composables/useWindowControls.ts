// macOS 红绿灯对齐（移植自 dbx-main / DBX）
// 前端测量工具按钮中心 → invoke 后端把系统红绿灯挪到同一中心 → 用 reserved_inset 设置 paddingLeft

import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { IS_MAC, IS_TAURI } from '../lib/platform';

const MIN_UI_SCALE = 0.75;
const MAX_UI_SCALE = 2;
const MAC_TRAFFIC_LIGHT_X = 16;
const MAC_TRAFFIC_LIGHT_RESERVED_INSET = 70;

type MacosTrafficLightLayout = {
  x: number;
  y: number;
  center_y: number;
  previous_center_y: number;
  reserved_inset: number;
};

function normalizeTrafficLightUiScale(scale: number): number {
  return Number.isFinite(scale) ? Math.min(MAX_UI_SCALE, Math.max(MIN_UI_SCALE, scale)) : 1;
}

/** 未测到按钮时的左侧避让宽度（与 DBX fallback 一致） */
export function macTrafficLightInsetPaddingForScale(scale: number): string {
  const normalizedScale = normalizeTrafficLightUiScale(scale);
  return `${Math.ceil(MAC_TRAFFIC_LIGHT_RESERVED_INSET / normalizedScale)}px`;
}

export function shouldReserveMacTrafficLightInset(
  isMac: boolean,
  isFullscreen: boolean,
  isDesktop = true,
): boolean {
  return isDesktop && isMac && !isFullscreen;
}

export function shouldShowWindowControls(isMac: boolean, isDesktop = true): boolean {
  return isDesktop && !isMac;
}

/**
 * 将系统红绿灯移动到与工具栏按钮文字同一垂直中心，并返回实测 reserved_inset。
 * uiScale：应用级界面缩放（GitDash 暂固定 1）。
 */
export async function syncMacTrafficLightsToToolbar(
  toolbarEl: HTMLElement | null,
  targetEl: HTMLElement | null,
  uiScale = 1,
): Promise<number | null> {
  if (!IS_TAURI || !IS_MAC) return null;
  if (!toolbarEl || !targetEl) return null;

  const toolbarRect = toolbarEl.getBoundingClientRect();
  const targetRect = targetEl.getBoundingClientRect();
  const targetCenterY = targetRect.top - toolbarRect.top + targetRect.height / 2;

  try {
    const layout = await invoke<MacosTrafficLightLayout>('set_macos_traffic_light_position', {
      x: MAC_TRAFFIC_LIGHT_X,
      y: targetCenterY,
      scale: uiScale,
    });
    return Math.ceil(layout.reserved_inset / normalizeTrafficLightUiScale(uiScale));
  } catch (error) {
    console.warn('[GitDash] Failed to sync macOS traffic light position', {
      targetCenterY,
      error,
    });
    return null;
  }
}

export function useWindowControls() {
  const isFullscreen = ref(false);
  const isMac = IS_MAC;
  const isDesktop = IS_TAURI;
  const showControls = shouldShowWindowControls(isMac, isDesktop);

  let unlisten: (() => void) | null = null;

  async function updateWindowState() {
    if (!isDesktop) return;
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    isFullscreen.value = await getCurrentWindow().isFullscreen();
  }

  async function minimize() {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    await getCurrentWindow().minimize();
  }

  async function toggleMaximize() {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    await getCurrentWindow().toggleMaximize();
    setTimeout(updateWindowState, 50);
  }

  async function close() {
    if (!isDesktop) return;
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    await getCurrentWindow().close();
  }

  onMounted(async () => {
    if (!isDesktop) return;
    await updateWindowState();
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      unlisten = await getCurrentWindow().onResized(() => {
        void updateWindowState();
      });
    } catch {
      // 浏览器预览时忽略
    }
  });

  onUnmounted(() => {
    unlisten?.();
  });

  return {
    isMac,
    isDesktop,
    showControls,
    isFullscreen,
    minimize,
    toggleMaximize,
    close,
  };
}
