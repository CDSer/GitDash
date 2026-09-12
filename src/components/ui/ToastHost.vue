<template>
  <Teleport to="body">
    <div class="toast-host">
      <!-- 进度日志提示框（可展开） -->
      <div
        v-for="t in progressToasts"
        :key="t.id"
        class="toast toast--progress"
        :class="`toast--progress-${t.progressStatus}`"
        @mouseenter="setToastHovered(t.id, true)"
        @mouseleave="setToastHovered(t.id, false)"
      >
        <div class="progress-head" @click="toggleToastExpanded(t.id)">
          <span class="toast-dot progress-dot" />
          <div class="progress-titles">
            <div class="progress-title">{{ t.title || t.message }}</div>
            <div class="progress-sub">{{ statusLabel(t.progressStatus) }} · {{ t.lines?.length || 0 }} 条日志</div>
          </div>
          <button
            type="button"
            class="progress-toggle"
            :title="t.expanded ? '收起日志' : '展开日志'"
            @click.stop="toggleToastExpanded(t.id)"
          >
            <ChevronUp v-if="t.expanded" :size="14" />
            <ChevronDown v-else :size="14" />
          </button>
          <button
            type="button"
            class="progress-close"
            title="关闭"
            @click.stop="dismissToastById(t.id)"
          >
            <X :size="14" />
          </button>
        </div>

        <div v-if="t.expanded" class="progress-body">
          <div ref="logBodyEl" class="progress-log">
            <div v-for="(line, i) in t.lines" :key="i" class="progress-line">{{ line }}</div>
            <div v-if="!t.lines?.length" class="progress-line progress-line--muted">等待输出…</div>
          </div>
        </div>
        <div v-if="t.progressStatus === 'running'" class="progress-bar">
          <div class="progress-bar-indeterminate" />
        </div>
      </div>

      <!-- 普通单行 toast -->
      <div
        v-for="t in plainToasts"
        :key="t.id"
        :class="['toast', `toast--${t.type}`]"
      >
        <span class="toast-dot" />
        <span class="toast-msg">{{ t.message }}</span>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { ChevronDown, ChevronUp, X } from 'lucide-vue-next';
import {
  toasts,
  toggleToastExpanded,
  dismissToastById,
  setToastHovered,
  type ToastItem,
} from '../../lib/toast';

const plainToasts = computed(() => toasts.filter((t) => t.type !== 'progress'));
const progressToasts = computed(() => toasts.filter((t) => t.type === 'progress'));

const logBodyEl = ref<HTMLElement | null>(null);

// 新日志滚动到底
watch(
  () => progressToasts.value.map((t) => t.lines?.length ?? 0).join(','),
  () => {
    nextTick(() => {
      const els = document.querySelectorAll('.progress-log');
      els.forEach((el) => {
        el.scrollTop = el.scrollHeight;
      });
    });
  },
);

function statusLabel(s: ToastItem['progressStatus']): string {
  if (s === 'success') return '完成';
  if (s === 'error') return '失败';
  return '进行中';
}
</script>

<style scoped>
.toast-host {
  position: fixed;
  top: 16px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 8px;
  align-items: center;
  pointer-events: none;
}

.toast {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 14px;
  border-radius: 12px;
  background-color: var(--popover-glass);
  backdrop-filter: saturate(var(--glass-saturate)) blur(var(--glass-popover-blur));
  -webkit-backdrop-filter: saturate(var(--glass-saturate)) blur(var(--glass-popover-blur));
  color: var(--popover-foreground);
  border: 0.5px solid transparent;
  box-shadow: var(--glass-float-edge), var(--shadow-md);
  font-size: 13px;
  letter-spacing: -0.01em;
  max-width: 80vw;
  animation: toast-in 180ms cubic-bezier(0.32, 0.72, 0, 1);
  pointer-events: auto;
}

.toast-dot {
  width: 8px;
  height: 8px;
  border-radius: 9999px;
  flex-shrink: 0;
}

.toast--success .toast-dot {
  background-color: var(--sys-green);
}
.toast--error .toast-dot {
  background-color: var(--destructive);
}
.toast--info .toast-dot {
  background-color: var(--primary);
}

.toast-msg {
  white-space: pre-wrap;
}

/* ===== 进度日志框 ===== */
.toast--progress {
  flex-direction: column;
  align-items: stretch;
  gap: 0;
  padding: 0;
  width: min(560px, 92vw);
  overflow: hidden;
}

.progress-head {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  cursor: pointer;
  user-select: none;
}

.progress-dot {
  width: 10px;
  height: 10px;
}

.toast--progress-running .progress-dot {
  background-color: var(--primary);
  box-shadow: 0 0 0 0 color-mix(in srgb, var(--primary) 50%, transparent);
  animation: pulse-dot 1.2s ease-out infinite;
}
.toast--progress-success .progress-dot {
  background-color: var(--sys-green);
}
.toast--progress-error .progress-dot {
  background-color: var(--destructive);
}

.progress-titles {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.progress-title {
  font-size: 13px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.progress-sub {
  font-size: 11px;
  color: var(--muted-foreground);
}

.progress-toggle,
.progress-close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--muted-foreground);
  cursor: default;
  flex-shrink: 0;
  transition: background-color 0.15s var(--ease-out), color 0.15s var(--ease-out);
}
.progress-toggle:hover,
.progress-close:hover {
  background: var(--accent);
  color: var(--foreground);
}

.progress-body {
  box-shadow: inset 0 1px 0 0 var(--separator);
  background: color-mix(in srgb, var(--muted) 40%, transparent);
}

.progress-log {
  max-height: 180px;
  overflow-y: auto;
  padding: 8px 12px;
  font-family: var(--font-mono);
  font-size: 11px;
  line-height: 1.55;
  color: var(--muted-foreground);
  white-space: pre-wrap;
  word-break: break-all;
}

.progress-line {
  padding: 1px 0;
}

.progress-line--muted {
  opacity: 0.5;
  font-style: italic;
}

.progress-bar {
  height: 2px;
  background: color-mix(in srgb, var(--primary) 12%, transparent);
  overflow: hidden;
}

.progress-bar-indeterminate {
  height: 100%;
  width: 40%;
  background: var(--primary);
  animation: progress-slide 1.1s ease-in-out infinite;
}

@keyframes pulse-dot {
  0% {
    box-shadow: 0 0 0 0 color-mix(in srgb, var(--primary) 45%, transparent);
  }
  70% {
    box-shadow: 0 0 0 8px transparent;
  }
  100% {
    box-shadow: 0 0 0 0 transparent;
  }
}

@keyframes progress-slide {
  0% {
    transform: translateX(-100%);
  }
  100% {
    transform: translateX(300%);
  }
}

@keyframes toast-in {
  from {
    opacity: 0;
    transform: translateY(-8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
