// 窗口控制 + macOS 红绿灯对齐
// Windows/Linux：自绘 min/max/close（无系统标题栏）
// macOS：系统红绿灯对齐到工具栏

import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
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
  const isMaximized = ref(false);
  const isMac = IS_MAC;
  const isDesktop = IS_TAURI;
  const showControls = shouldShowWindowControls(isMac, isDesktop);

  let unlisten: (() => void) | null = null;

  async function updateWindowState() {
    if (!isDesktop) return;
    try {
      const win = getCurrentWindow();
      isFullscreen.value = await win.isFullscreen();
      isMaximized.value = await win.isMaximized();
    } catch (error) {
      console.warn('[GitDash] updateWindowState failed', error);
    }
  }

  async function minimize() {
    if (!isDesktop) return;
    try {
      await getCurrentWindow().minimize();
    } catch (error) {
      console.error('[GitDash] minimize failed', error);
    }
  }

  async function toggleMaximize() {
    if (!isDesktop) return;
    try {
      await getCurrentWindow().toggleMaximize();
      await updateWindowState();
    } catch (error) {
      console.error('[GitDash] toggleMaximize failed', error);
    }
  }

  async function close() {
    if (!isDesktop) return;
    try {
      await getCurrentWindow().close();
    } catch (error) {
      console.error('[GitDash] close failed', error);
    }
  }

  onMounted(async () => {
    if (!isDesktop) return;
    await updateWindowState();
    try {
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
    isMaximized,
    minimize,
    toggleMaximize,
    close,
  };
}
