// 轻量 Toast（替代 Element Plus 的 ElMessage / ElNotification）
import { reactive } from 'vue';

export type ToastType = 'success' | 'error' | 'info';

export interface ToastItem {
  id: number;
  type: ToastType;
  message: string;
}

export const toasts = reactive<ToastItem[]>([]);

let seq = 0;

function show(type: ToastType, message: string, timeout = 2600): void {
  const id = ++seq;
  toasts.push({ id, type, message });
  setTimeout(() => {
    const idx = toasts.findIndex((t) => t.id === id);
    if (idx >= 0) toasts.splice(idx, 1);
  }, timeout);
}

export const toast = {
  success: (m: string) => show('success', m),
  error: (m: string) => show('error', m),
  info: (m: string) => show('info', m),
};
