<!--
  应用根组件
  挂载页面视图，并负责主题初始化与全局 Toast 宿主
-->
<template>
  <MainView v-if="!appStore.workspaceProjectId" />
  <WorkspaceView v-else />
  <ToastHost />
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import MainView from './views/MainView.vue';
import WorkspaceView from './views/WorkspaceView.vue';
import ToastHost from './components/ui/ToastHost.vue';
import { useAppStore } from './stores/appStore';
import { setupTheme } from './composables/useTheme';
import { invoke } from '@tauri-apps/api/core';

const appStore = useAppStore();

function onKeydown(e: KeyboardEvent) {
  // Cmd/Ctrl + Shift + I 切换开发者工具（仅 debug 构建有效）
  if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key.toLowerCase() === 'i') {
    e.preventDefault();
    invoke('toggle_devtools').catch(() => {});
  }
}

onMounted(() => {
  setupTheme();
  window.addEventListener('keydown', onKeydown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', onKeydown);
});
</script>
