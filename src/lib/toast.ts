// 轻量 Toast（替代 Element Plus 的 ElMessage / ElNotification）
// progress 类型：操作进行中的大提示框，可展开查看实时日志
import { reactive } from 'vue';

export type ToastType = 'success' | 'error' | 'info' | 'progress';

export interface ToastItem {
  id: number;
  type: ToastType;
  message: string;
  /** progress 专用：标题 */
  title?: string;
  /** progress 专用：日志行 */
  lines?: string[];
  /** progress 专用：运行状态 */
  progressStatus?: 'running' | 'success' | 'error';
  /** progress 专用：是否展开日志 */
  expanded?: boolean;
  /** progress 专用：不自动消失（运行中） */
  sticky?: boolean;
  /** 鼠标是否悬停（悬停时不自动关闭） */
  hovered?: boolean;
}

export const toasts = reactive<ToastItem[]>([]);

let seq = 0;

function nowLabel(): string {
  const d = new Date();
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}

function dismissToast(id: number): void {
  const idx = toasts.findIndex((t) => t.id === id);
  if (idx >= 0) toasts.splice(idx, 1);
}

function show(type: ToastType, message: string, timeout = 2600): void {
  const id = ++seq;
  toasts.push({ id, type, message });
  setTimeout(() => dismissToast(id), timeout);
}

export interface ProgressToastHandle {
  log(line: string): void;
  setTitle(title: string): void;
  done(ok: boolean, summary?: string, opts?: { autoDismissMs?: number }): void;
  dismiss(): void;
}

/** 创建一个可展开的进度日志提示框；由调用方写入日志并在结束时 done/dismiss */
export function beginProgressToast(title: string, initialLog?: string): ProgressToastHandle {
  const id = ++seq;
  const item = reactive<ToastItem>({
    id,
    type: 'progress',
    message: title,
    title,
    lines: initialLog ? [`[${nowLabel()}] ${initialLog}`] : [],
    progressStatus: 'running',
    expanded: true,
    sticky: true,
    hovered: false,
  });
  toasts.push(item);

  /** 到时若仍在悬停则推迟关闭，直到移出 */
  function scheduleDismiss(delayMs: number) {
    setTimeout(() => {
      const current = toasts.find((t) => t.id === id);
      if (!current) return;
      if (current.hovered) {
        scheduleDismiss(400);
        return;
      }
      dismissToast(id);
    }, delayMs);
  }

  const api: ProgressToastHandle = {
    log(line: string) {
      item.lines!.push(`[${nowLabel()}] ${line}`);
      if (item.lines!.length > 300) {
        item.lines!.splice(0, item.lines!.length - 300);
      }
    },
    setTitle(next: string) {
      item.title = next;
      item.message = next;
    },
    done(ok, summary, opts) {
      item.progressStatus = ok ? 'success' : 'error';
      item.message = summary || (ok ? '操作完成' : '操作失败');
      item.sticky = false;
      const autoMs = opts?.autoDismissMs ?? (ok ? 4000 : 10000);
      if (autoMs > 0) {
        scheduleDismiss(autoMs);
      }
    },
    dismiss() {
      dismissToast(id);
    },
  };
  return api;
}

export function toggleToastExpanded(id: number): void {
  const t = toasts.find((x) => x.id === id);
  if (t && t.type === 'progress') {
    t.expanded = !t.expanded;
  }
}

export function setToastHovered(id: number, hovered: boolean): void {
  const t = toasts.find((x) => x.id === id);
  if (t) t.hovered = hovered;
}

export function dismissToastById(id: number): void {
  dismissToast(id);
}

export const toast = {
  success: (m: string) => show('success', m),
  error: (m: string) => show('error', m),
  info: (m: string) => show('info', m),
  progress: beginProgressToast,
};
