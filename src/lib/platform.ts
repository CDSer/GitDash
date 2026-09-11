// 平台检测：判断当前运行平台，供布局和窗口控制使用
import { platform } from '@tauri-apps/plugin-os';

const PLATFORM = (() => {
  try {
    return platform();
  } catch {
    return '';
  }
})();

/** 是否运行在 Tauri 桌面壳（浏览器预览为 false） */
export const IS_TAURI =
  typeof window !== 'undefined' &&
  // @ts-expect-error 内部字段，运行时探测
  Boolean(window.__TAURI_INTERNALS__);

export const IS_MAC = PLATFORM === 'macos';
