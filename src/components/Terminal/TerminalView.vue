<!--
  内置终端面板
  基于 xterm.js + 后端 portable-pty：真实用户 shell，继承系统环境变量
-->
<template>
  <div class="terminal-view">
    <div ref="termEl" class="terminal-host" />
    <div v-if="exited" class="terminal-exited">
      <span>进程已退出</span>
      <Button size="sm" variant="outline" @click="restart">重新打开终端</Button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import '@xterm/xterm/css/xterm.css';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { terminalClose, terminalOpen, terminalResize, terminalWrite } from '../../lib/tauriApi';
import Button from '../ui/Button.vue';

const props = defineProps<{ projectId: string }>();

const termEl = ref<HTMLDivElement | null>(null);
const exited = ref(false);

let term: Terminal | null = null;
let fit: FitAddon | null = null;
let unlistenData: UnlistenFn | null = null;
let unlistenExit: UnlistenFn | null = null;
let resizeObserver: ResizeObserver | null = null;
let disposed = false;

function decodeBase64(b64: string): Uint8Array {
  const bin = atob(b64);
  const bytes = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
  return bytes;
}

function applyTheme(): { background: string; foreground: string } {
  const styles = getComputedStyle(document.body);
  const background = styles.getPropertyValue('--background').trim() || '#0f1115';
  const foreground = styles.getPropertyValue('--foreground').trim() || '#e6e6e6';
  return { background, foreground };
}

function fitAndResize() {
  if (!term || !fit || !termEl.value) return;
  try {
    fit.fit();
  } catch {
    return;
  }
  if (term.rows > 0 && term.cols > 0) {
    void terminalResize(props.projectId, term.cols, term.rows).catch(() => {});
  }
}

async function setup() {
  const el = termEl.value;
  if (!el) return;

  disposed = false;
  exited.value = false;

  term = new Terminal({
    convertEol: false,
    cursorBlink: true,
    fontSize: 13,
    fontFamily:
      "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace",
    scrollback: 5000,
    theme: applyTheme(),
    allowProposedApi: true,
  });
  fit = new FitAddon();
  term.loadAddon(fit);
  term.open(el);

  // 先监听再打开，避免丢掉启动输出
  unlistenData = await listen<{ projectId: string; data: string }>(
    'terminal:data',
    (event) => {
      if (event.payload.projectId !== props.projectId || !term) return;
      term.write(decodeBase64(event.payload.data));
    },
  );
  unlistenExit = await listen<{ projectId: string }>('terminal:exit', (event) => {
    if (event.payload.projectId !== props.projectId) return;
    exited.value = true;
  });

  fit.fit();
  const replay = await terminalOpen(props.projectId, term.cols, term.rows);
  if (disposed) return;
  if (replay) {
    term.write(decodeBase64(replay));
  }

  term.onData((data) => {
    void terminalWrite(props.projectId, data).catch(() => {});
  });

  resizeObserver = new ResizeObserver(() => fitAndResize());
  resizeObserver.observe(el);
}

async function teardown() {
  disposed = true;
  resizeObserver?.disconnect();
  resizeObserver = null;
  unlistenData?.();
  unlistenData = null;
  unlistenExit?.();
  unlistenExit = null;
  term?.dispose();
  term = null;
  fit = null;
}

async function restart() {
  await teardown();
  try {
    await terminalClose(props.projectId);
  } catch {
    // 会话可能已不存在
  }
  await setup();
}

onMounted(() => {
  void setup();
});

onBeforeUnmount(() => {
  void teardown();
  // 会话保留在后端：切换模式/标签后重挂载可回放；关闭标签时由上层调用 terminalClose
});
</script>

<style scoped>
.terminal-view {
  position: relative;
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background-color: var(--background);
}
.terminal-host {
  flex: 1;
  min-height: 0;
  padding: 8px 8px 4px;
}
.terminal-host :deep(.xterm) {
  height: 100%;
}
.terminal-host :deep(.xterm-viewport) {
  background-color: transparent !important;
}
.terminal-exited {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  font-size: 13px;
  color: var(--muted-foreground);
  background-color: color-mix(in oklab, var(--background) 92%, transparent);
  backdrop-filter: blur(2px);
}
</style>
